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
