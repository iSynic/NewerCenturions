use crate::{
    RandomState, RuleError,
    recovered_attack::AttackWork,
    recovered_random_item::{RandomItemEffect, RandomItemLimits, RandomizedItem, randomitem},
    recovered_records::{self, ItemTables, RecordWrite, SPELL_BYTES},
};
use centurions_content::ITEM_BYTES;

const ITEM: u32 = 0x45363c;
const BOOTY: u32 = 0x41fc74;
const TEMPORARY: u32 = 0x41fc48;
const ENHANCEMENT: u32 = 0x4046e6;
const MODIFIERS: [(u32, u32); 8] = [
    (14, 0x4e),
    (15, 0x51),
    (16, 0x53),
    (17, 0x54),
    (18, 0x55),
    (19, 0x56),
    (21, 0x50),
    (20, 0x4f),
];

/// An explicit native address binding, including any reached adjoining memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallMemory {
    pub base_address: u32,
    pub bytes: Vec<u8>,
}

impl InstallMemory {
    fn offset(&self, address: u32, length: usize) -> Result<usize, RuleError> {
        let offset = address.checked_sub(self.base_address).map(|n| n as usize);
        match offset {
            Some(offset)
                if offset
                    .checked_add(length)
                    .is_some_and(|end| end <= self.bytes.len()) =>
            {
                Ok(offset)
            }
            _ => Err(unavailable()),
        }
    }

    fn read(&self, address: u32, length: usize) -> Result<&[u8], RuleError> {
        let offset = self.offset(address, length)?;
        Ok(&self.bytes[offset..offset + length])
    }

    fn word(&self, address: u32) -> Result<i16, RuleError> {
        let bytes = self.read(address, 2)?;
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn byte(&self, address: u32) -> Result<u8, RuleError> {
        Ok(self.read(address, 1)?[0])
    }

    fn write(
        &mut self,
        address: u32,
        bytes: &[u8],
        writes: &mut Vec<RecordWrite>,
    ) -> Result<(), RuleError> {
        let offset = self.offset(address, bytes.len())?;
        self.bytes[offset..offset + bytes.len()].copy_from_slice(bytes);
        writes.push(RecordWrite {
            address,
            bytes: bytes.to_vec(),
        });
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallRequest {
    pub source_member: u16,
    pub destination_member: u16,
    pub source_slot: u16,
    pub destination_slot: u16,
    pub item_id: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledItem {
    pub memory: InstallMemory,
    pub writes: Vec<RecordWrite>,
    pub native_return: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RandomItemRequest {
    pub entry: u32,
    pub slot: i16,
    pub enhancement: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingInstallation {
    /// A speculative prefix. A command must not commit it before the child succeeds.
    pub memory: InstallMemory,
    pub writes: Vec<RecordWrite>,
    pub child: RandomItemRequest,
    destination: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstallOutcome {
    Complete(InstalledItem),
    PendingRandomItem(PendingInstallation),
}

pub fn installitem(
    current: &InstallMemory,
    tables: &ItemTables<'_>,
    request: InstallRequest,
) -> Result<InstallOutcome, RuleError> {
    let mut memory = current.clone();
    let mut writes = Vec::new();
    let destination = record_address(request.destination_member, request.destination_slot);
    match request.source_member {
        0xfffe => {
            load_item(&mut memory, tables, request.item_id, &mut writes)?;
            let source = BOOTY + u32::from(request.source_slot) * 22;
            for offset in (0..20).step_by(4) {
                transfer(
                    &mut memory,
                    source + offset,
                    TEMPORARY + offset,
                    4,
                    &mut writes,
                )?;
            }
            transfer(&mut memory, source + 20, TEMPORARY + 20, 2, &mut writes)?;
            install_booty(&mut memory, destination, &mut writes)?;
        }
        0xffff => {
            load_item(&mut memory, tables, request.item_id, &mut writes)?;
            if request.destination_member == 0xffff {
                let enhancement = memory.word(ENHANCEMENT)?;
                if enhancement != 0 {
                    memory.write(BOOTY, &request.item_id.to_be_bytes(), &mut writes)?;
                    let byte = memory.byte(ENHANCEMENT + 1)?;
                    memory.write(0x42df04, &[byte], &mut writes)?;
                    return Ok(InstallOutcome::PendingRandomItem(PendingInstallation {
                        memory,
                        writes,
                        child: RandomItemRequest {
                            entry: 0x12d558,
                            slot: 0,
                            enhancement,
                        },
                        destination,
                    }));
                }
            }
            install_fresh(&mut memory, destination, &mut writes)?;
        }
        member => {
            let source = record_address(member, request.source_slot);
            for offset in [0, 2, 6, 4] {
                transfer(
                    &mut memory,
                    source + offset,
                    destination + offset,
                    2,
                    &mut writes,
                )?;
            }
            for offset in [8, 9, 10, 13, 14, 15, 16, 17, 18, 19, 21, 20] {
                if offset == 9 {
                    memory.write(destination + offset, &[0], &mut writes)?;
                } else {
                    transfer(
                        &mut memory,
                        source + offset,
                        destination + offset,
                        1,
                        &mut writes,
                    )?;
                }
            }
        }
    }
    Ok(InstallOutcome::Complete(InstalledItem {
        memory,
        writes,
        native_return: 0,
    }))
}

/// Complete installitem control, including the recovered randomitem child.
/// Prefix, child writes/draws and caller suffix form one speculative result.
pub fn installitem_complete(
    current: &InstallMemory,
    random: &RandomState,
    tables: &ItemTables<'_>,
    request: InstallRequest,
    limits: RandomItemLimits,
) -> Result<RandomizedItem, RuleError> {
    match installitem(current, tables, request)? {
        InstallOutcome::Complete(installed) => Ok(RandomizedItem {
            memory: installed.memory,
            random: random.clone(),
            effects: installed
                .writes
                .into_iter()
                .map(RandomItemEffect::Write)
                .collect(),
            native_return: installed.native_return,
        }),
        InstallOutcome::PendingRandomItem(pending) => {
            let child = randomitem(
                &pending.memory,
                random,
                pending.child.slot as u16,
                pending.child.enhancement,
                limits,
            )?;
            let child_writes: Vec<_> = child
                .effects
                .iter()
                .filter_map(|effect| match effect {
                    RandomItemEffect::Write(write) => Some(write.clone()),
                    RandomItemEffect::Random(_) => None,
                })
                .collect();
            let installed =
                continue_installitem_after_randomitem(&pending, &child.memory, &child_writes)?;
            let prefix_write_count = pending.writes.len() + child_writes.len();
            let mut effects: Vec<_> = pending
                .writes
                .into_iter()
                .map(RandomItemEffect::Write)
                .collect();
            effects.extend(child.effects);
            effects.extend(
                installed
                    .writes
                    .into_iter()
                    .skip(prefix_write_count)
                    .map(RandomItemEffect::Write),
            );
            Ok(RandomizedItem {
                memory: installed.memory,
                random: child.random,
                effects,
                native_return: installed.native_return,
            })
        }
    }
}

/// Runs the caller suffix after an actual randomitem completion. This does not implement that child.
pub fn continue_installitem_after_randomitem(
    pending: &PendingInstallation,
    memory_after_child: &InstallMemory,
    child_writes: &[RecordWrite],
) -> Result<InstalledItem, RuleError> {
    let mut memory = memory_after_child.clone();
    let mut writes = pending.writes.clone();
    writes.extend_from_slice(child_writes);
    let modifier = i16::from(memory.byte(BOOTY + 13)? as i8);
    memory.write(ITEM + 0x8a, &modifier.to_be_bytes(), &mut writes)?;
    for (source, destination) in MODIFIERS {
        transfer(
            &mut memory,
            BOOTY + source,
            ITEM + destination,
            1,
            &mut writes,
        )?;
    }
    install_fresh(&mut memory, pending.destination, &mut writes)?;
    Ok(InstalledItem {
        memory,
        writes,
        native_return: 0,
    })
}

fn record_address(member: u16, slot: u16) -> u32 {
    let base = if member == 0xffff {
        0x44721c
    } else {
        0x448aa0 + u32::from(member) * 0xc42
    };
    base + 0x132 + u32::from(slot) * 22
}

fn load_item(
    memory: &mut InstallMemory,
    tables: &ItemTables<'_>,
    id: i16,
    writes: &mut Vec<RecordWrite>,
) -> Result<(), RuleError> {
    let current: [u8; ITEM_BYTES] = memory.read(ITEM, ITEM_BYTES)?.try_into().unwrap();
    let loaded = recovered_records::loaditem(tables, &current, id)?;
    for write in loaded.writes {
        memory.write(write.address, &write.bytes, writes)?;
    }
    Ok(())
}

fn transfer(
    memory: &mut InstallMemory,
    source: u32,
    destination: u32,
    length: usize,
    writes: &mut Vec<RecordWrite>,
) -> Result<(), RuleError> {
    let bytes = memory.read(source, length)?.to_vec();
    memory.write(destination, &bytes, writes)
}

fn item_low_byte(
    memory: &mut InstallMemory,
    source: u32,
    destination: u32,
    writes: &mut Vec<RecordWrite>,
) -> Result<(), RuleError> {
    transfer(memory, ITEM + source + 1, destination, 1, writes)
}

fn install_booty(
    memory: &mut InstallMemory,
    destination: u32,
    writes: &mut Vec<RecordWrite>,
) -> Result<(), RuleError> {
    transfer(memory, ITEM + 0x66, destination, 2, writes)?;
    for offset in [2, 6, 4] {
        transfer(memory, TEMPORARY + offset, destination + offset, 2, writes)?;
    }
    item_low_byte(memory, 0x22, destination + 8, writes)?;
    memory.write(destination + 9, &[0], writes)?;
    item_low_byte(memory, 0x1a, destination + 10, writes)?;
    for offset in [13, 14, 15, 16, 17, 18, 19, 21, 20] {
        transfer(memory, TEMPORARY + offset, destination + offset, 1, writes)?;
    }
    Ok(())
}

fn install_fresh(
    memory: &mut InstallMemory,
    destination: u32,
    writes: &mut Vec<RecordWrite>,
) -> Result<(), RuleError> {
    transfer(memory, ITEM + 0x66, destination, 2, writes)?;
    let quantity = memory
        .word(ITEM + 0x94)?
        .wrapping_add(memory.word(ITEM + 0x14)?);
    memory.write(destination + 6, &quantity.to_be_bytes(), writes)?;
    memory.write(destination + 2, &quantity.to_be_bytes(), writes)?;
    let capacity = memory.word(ITEM + 0x5a)?;
    if capacity != 0 {
        memory.write(destination + 6, &capacity.to_be_bytes(), writes)?;
    } else {
        let capacity = memory.word(ITEM + 0x18)?;
        if capacity != 0 {
            memory.write(destination + 6, &capacity.to_be_bytes(), writes)?;
        }
    }
    memory.write(destination + 4, &[0, 0], writes)?;
    item_low_byte(memory, 0x22, destination + 8, writes)?;
    memory.write(destination + 9, &[0], writes)?;
    item_low_byte(memory, 0x1a, destination + 10, writes)?;
    item_low_byte(memory, 0x8a, destination + 13, writes)?;
    for (destination_offset, source_offset) in MODIFIERS {
        transfer(
            memory,
            ITEM + source_offset,
            destination + destination_offset,
            1,
            writes,
        )?;
    }
    Ok(())
}

fn unavailable() -> RuleError {
    RuleError::Unsupported {
        action: "installitem",
        dependency: "a reached native memory span is unavailable",
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClearedAttackWork {
    pub work: AttackWork,
    pub writes: Vec<RecordWrite>,
    pub native_return: i32,
}

pub fn nobrassnoammo(
    current: &AttackWork,
    tables: &ItemTables<'_>,
    spells: &[u8],
) -> Result<ClearedAttackWork, RuleError> {
    let mut work = current.clone();
    let mut writes = Vec::new();
    let weapon = recovered_records::loadweapon(tables, &work.weapon, 0)?;
    work.weapon = weapon.bytes;
    writes.extend(weapon.writes);
    let ammunition = recovered_records::loadammo(tables, &work.ammunition, 0)?;
    work.ammunition = ammunition.bytes;
    writes.extend(ammunition.writes);
    let item = recovered_records::loaditem(tables, &work.item, 0)?;
    work.item = item.bytes;
    writes.extend(item.writes);
    let packet = spells
        .get(0x21c2..0x21c2 + SPELL_BYTES)
        .ok_or(RuleError::Unsupported {
            action: "nobrassnoammo",
            dependency: "the default spell record at native address 0x44594a is unavailable",
        })?;
    work.packet.copy_from_slice(packet);
    for offset in (0..56).step_by(4) {
        writes.push(RecordWrite {
            address: 0x44374e + offset as u32,
            bytes: packet[offset..offset + 4].to_vec(),
        });
    }
    writes.push(RecordWrite {
        address: 0x443786,
        bytes: packet[56..58].to_vec(),
    });
    Ok(ClearedAttackWork {
        work,
        writes,
        native_return: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory() -> InstallMemory {
        InstallMemory {
            base_address: ENHANCEMENT,
            bytes: vec![0xa5; (ITEM + ITEM_BYTES as u32 - ENHANCEMENT) as usize],
        }
    }

    fn put(memory: &mut InstallMemory, address: u32, bytes: &[u8]) {
        let offset = (address - memory.base_address) as usize;
        memory.bytes[offset..offset + bytes.len()].copy_from_slice(bytes);
    }

    fn definition() -> [u8; ITEM_BYTES] {
        let mut item = [0x6c; ITEM_BYTES];
        for (offset, value) in [
            (0x66, 900i16),
            (0x14, 32760),
            (0x94, 20),
            (0x5a, 777),
            (0x18, 222),
            (0x22, 0x1234),
            (0x1a, -7),
            (0x8a, -3),
        ] {
            item[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        for (index, (_, offset)) in MODIFIERS.iter().enumerate() {
            item[*offset as usize] = 60 + index as u8;
        }
        item
    }

    fn complete(outcome: InstallOutcome) -> InstalledItem {
        match outcome {
            InstallOutcome::Complete(output) => output,
            InstallOutcome::PendingRandomItem(_) => panic!("unexpected child request"),
        }
    }

    #[test]
    fn complete_install_runs_real_child_draw_and_commits_only_the_full_result() {
        let mut item = definition();
        item[0x1a..0x1c].copy_from_slice(&26i16.to_be_bytes());
        let mut table = vec![0; ITEM_BYTES * 2];
        table[ITEM_BYTES..].copy_from_slice(&item);
        let tables = ItemTables {
            families: [&table, &[], &[], &[], &[]],
        };
        let mut incoming = memory();
        let table_address = crate::recovered_records::ITEM_TABLE_ADDRESSES[0];
        incoming.bytes.resize(
            (table_address + table.len() as u32 - incoming.base_address) as usize,
            0xa5,
        );
        put(&mut incoming, table_address, &table);
        put(&mut incoming, ENHANCEMENT, &1i16.to_be_bytes());
        put(&mut incoming, 0x442576, &0i16.to_be_bytes());
        put(&mut incoming, BOOTY + 2, &0i16.to_be_bytes());
        let original = incoming.clone();
        let random = RandomState::new(12);
        let request = InstallRequest {
            source_member: 0xffff,
            destination_member: 0xffff,
            source_slot: 99,
            destination_slot: 7,
            item_id: 1,
        };
        let result = installitem_complete(
            &incoming,
            &random,
            &tables,
            request,
            RandomItemLimits { maximum_draws: 1 },
        )
        .unwrap();
        assert_eq!(result.random.draws, 1);
        assert_eq!(
            result
                .effects
                .iter()
                .filter(|e| matches!(e, RandomItemEffect::Random(_)))
                .count(),
            1
        );
        assert_eq!(result.memory.word(record_address(0xffff, 7)).unwrap(), 900);
        assert_eq!(
            result.memory.byte(record_address(0xffff, 7) + 9).unwrap(),
            0
        );
        assert_eq!(
            result
                .memory
                .read(record_address(0xffff, 7) + 11, 2)
                .unwrap(),
            [0xa5; 2]
        );
        assert_eq!(
            result.effects.last(),
            Some(&RandomItemEffect::Write(RecordWrite {
                address: record_address(0xffff, 7) + 20,
                bytes: vec![67],
            }))
        );
        assert!(
            installitem_complete(
                &incoming,
                &random,
                &tables,
                request,
                RandomItemLimits { maximum_draws: 0 }
            )
            .is_err()
        );
        assert_eq!(incoming, original);
        assert_eq!(random, RandomState::new(12));
    }

    #[test]
    fn six_transfers_preserve_gaps_and_use_ordered_word_widths_and_native_aliases() {
        let item = definition();
        let mut table = vec![0; ITEM_BYTES * 2];
        table[ITEM_BYTES..].copy_from_slice(&item);
        let tables = ItemTables {
            families: [&table, &[], &[], &[], &[]],
        };
        let record: [u8; 22] = std::array::from_fn(|n| 10 + n as u8);
        for source_member in [0u16, 0xfffe, 0xffff] {
            for destination_member in [1u16, 0xffff] {
                let mut incoming = memory();
                put(&mut incoming, ENHANCEMENT, &[0, 0]);
                put(&mut incoming, record_address(0, 3), &record);
                put(&mut incoming, BOOTY + 3 * 22, &record);
                let original = incoming.clone();
                let destination = record_address(destination_member, 4);
                let output = complete(
                    installitem(
                        &incoming,
                        &tables,
                        InstallRequest {
                            source_member,
                            destination_member,
                            source_slot: 3,
                            destination_slot: 4,
                            item_id: 1,
                        },
                    )
                    .unwrap(),
                );
                let mut expected = record;
                expected[9] = 0;
                expected[11..13].fill(0xa5);
                if source_member != 0 {
                    expected[0..2].copy_from_slice(&900i16.to_be_bytes());
                    expected[8] = 0x34;
                    expected[10] = 7;
                }
                if source_member == 0xffff {
                    expected[2..4].copy_from_slice(&(-32756i16).to_be_bytes());
                    expected[4..6].fill(0);
                    expected[6..8].copy_from_slice(&777i16.to_be_bytes());
                    expected[13] = 253;
                    expected[14..20].copy_from_slice(&[60, 61, 62, 63, 64, 65]);
                    expected[20] = 67;
                    expected[21] = 66;
                }
                assert_eq!(output.memory.read(destination, 22).unwrap(), expected);
                let destination_writes: Vec<_> = output
                    .writes
                    .iter()
                    .filter(|w| (destination..destination + 22).contains(&w.address))
                    .map(|w| (w.address - destination, w.bytes.len()))
                    .collect();
                let mut expected_order = if source_member == 0xffff {
                    vec![(0, 2), (6, 2), (2, 2), (6, 2), (4, 2)]
                } else {
                    vec![(0, 2), (2, 2), (6, 2), (4, 2)]
                };
                expected_order
                    .extend([8, 9, 10, 13, 14, 15, 16, 17, 18, 19, 21, 20].map(|n| (n, 1)));
                assert_eq!(destination_writes, expected_order);
                assert_eq!(incoming, original);
                assert_eq!(output.native_return, 0);
            }
        }
        for fallback_capacity in [222i16, 0] {
            let mut fallback_table = table.clone();
            fallback_table[ITEM_BYTES + 0x5a..ITEM_BYTES + 0x5c].fill(0);
            fallback_table[ITEM_BYTES + 0x18..ITEM_BYTES + 0x1a]
                .copy_from_slice(&fallback_capacity.to_be_bytes());
            let fallback_tables = ItemTables {
                families: [&fallback_table, &[], &[], &[], &[]],
            };
            let output = complete(
                installitem(
                    &memory(),
                    &fallback_tables,
                    InstallRequest {
                        source_member: 0xffff,
                        destination_member: 10,
                        source_slot: 0,
                        destination_slot: 0,
                        item_id: 1,
                    },
                )
                .unwrap(),
            );
            assert_eq!(
                output.memory.word(record_address(10, 0) + 6).unwrap(),
                if fallback_capacity == 0 { -32756 } else { 222 }
            );
        }
        let mut incoming = memory();
        let source = record_address(0, 142);
        let destination = record_address(1, 0);
        assert_eq!(destination - source, 14);
        put(&mut incoming, source, &record);
        let output = complete(
            installitem(
                &incoming,
                &tables,
                InstallRequest {
                    source_member: 0,
                    destination_member: 1,
                    source_slot: 142,
                    destination_slot: 0,
                    item_id: 1,
                },
            )
            .unwrap(),
        );
        assert_eq!(
            output.memory.read(destination + 14, 6).unwrap(),
            &[10, 11, 12, 13, 14, 15]
        );
        let original = incoming.clone();
        assert!(
            installitem(
                &incoming,
                &tables,
                InstallRequest {
                    source_member: 0,
                    destination_member: 65534,
                    source_slot: 142,
                    destination_slot: 65535,
                    item_id: 1,
                }
            )
            .is_err()
        );
        assert_eq!(incoming, original);
    }

    #[test]
    fn enhanced_loot_is_pending_and_only_actual_child_completion_enables_the_suffix() {
        let item = definition();
        let mut table = vec![0; ITEM_BYTES * 2];
        table[ITEM_BYTES..].copy_from_slice(&item);
        let tables = ItemTables {
            families: [&table, &[], &[], &[], &[]],
        };
        let mut incoming = memory();
        put(&mut incoming, ENHANCEMENT, &(-258i16).to_be_bytes());
        let original = incoming.clone();
        let request = InstallRequest {
            source_member: 0xffff,
            destination_member: 0xffff,
            source_slot: 99,
            destination_slot: 7,
            item_id: 1,
        };
        let pending = match installitem(&incoming, &tables, request).unwrap() {
            InstallOutcome::PendingRandomItem(pending) => pending,
            InstallOutcome::Complete(_) => panic!("unfinished randomitem was bypassed"),
        };
        assert_eq!(
            pending.child,
            RandomItemRequest {
                entry: 0x12d558,
                slot: 0,
                enhancement: -258
            }
        );
        assert_eq!(pending.memory.word(BOOTY).unwrap(), 1);
        assert_eq!(pending.memory.byte(0x42df04).unwrap(), 254);
        assert_eq!(
            pending.memory.read(record_address(0xffff, 7), 22).unwrap(),
            [0xa5; 22]
        );
        assert_eq!(pending.writes.len(), 41);
        assert_eq!(incoming, original);

        // This is a supplied child-result example, not a randomitem implementation test.
        let mut after_child = pending.memory.clone();
        let mut child_writes = Vec::new();
        after_child
            .write(
                BOOTY + 13,
                &[128, 2, 3, 4, 5, 6, 7, 8, 9],
                &mut child_writes,
            )
            .unwrap();
        let output =
            continue_installitem_after_randomitem(&pending, &after_child, &child_writes).unwrap();
        let destination = record_address(0xffff, 7);
        assert_eq!(output.memory.word(ITEM + 0x8a).unwrap(), -128);
        assert_eq!(
            output.memory.read(destination + 13, 9).unwrap(),
            &[128, 2, 3, 4, 5, 6, 7, 8, 9]
        );
        assert_eq!(&output.writes[41..42], child_writes);
        assert_eq!(output.writes[42].address, ITEM + 0x8a);
        assert_eq!(output.writes[42].bytes, (-128i16).to_be_bytes());
        assert_eq!(output.native_return, 0);
        let party = complete(
            installitem(
                &incoming,
                &tables,
                InstallRequest {
                    destination_member: 1,
                    ..request
                },
            )
            .unwrap(),
        );
        assert_eq!(party.memory.word(BOOTY).unwrap(), -23131);
    }

    #[test]
    fn clearing_attack_loads_resource_zero_and_the_entire_default_packet_in_call_order() {
        let item = definition();
        let tables = ItemTables {
            families: [&item, &[], &[], &[], &[]],
        };
        let packet: [u8; SPELL_BYTES] = std::array::from_fn(|n| 200u8.wrapping_add(n as u8));
        let mut spells = vec![0; 0x21c2 + SPELL_BYTES];
        spells[0x21c2..].copy_from_slice(&packet);
        let incoming = AttackWork {
            item: [1; ITEM_BYTES],
            ammunition: [2; ITEM_BYTES],
            weapon: [3; ITEM_BYTES],
            packet: [4; SPELL_BYTES],
            spell_coordinates: [-1, 8, 9],
            attack_count: -5,
            damage_total: 456,
        };
        let output = nobrassnoammo(&incoming, &tables, &spells).unwrap();
        let mut expected_item = item;
        expected_item[0x1a..0x1c].copy_from_slice(&7i16.to_be_bytes());
        assert_eq!(output.work.item, expected_item);
        assert_eq!(output.work.weapon, expected_item);
        assert_eq!(output.work.ammunition, expected_item);
        assert_eq!(output.work.packet, packet);
        assert_eq!(output.work.spell_coordinates, incoming.spell_coordinates);
        assert_eq!(output.work.attack_count, -5);
        assert_eq!(output.work.damage_total, 456);
        assert_eq!(output.writes.len(), 132);
        assert_eq!(output.writes[0].address, 0x453768);
        assert_eq!(output.writes[39].address, 0x4536d2);
        assert_eq!(output.writes[78].address, 0x45363c);
        assert_eq!(output.writes[117].address, 0x44374e);
        assert_eq!(output.writes[131].address, 0x443786);
        assert_eq!(output.writes[131].bytes, packet[56..]);
        assert_eq!(output.native_return, 0);
        assert!(nobrassnoammo(&incoming, &tables, &spells[..spells.len() - 1]).is_err());
        assert_eq!(incoming.packet, [4; SPELL_BYTES]);
    }
}
