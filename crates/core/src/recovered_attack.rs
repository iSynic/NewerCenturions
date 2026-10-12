use crate::{
    RuleError,
    recovered_economy::ArithmeticQualification,
    recovered_records::{self, ItemTables, RecordWrite, SPELL_BYTES},
};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES, span, word};

pub struct AttackTables<'a> {
    /// Native contiguous memory beginning at 0x448aa0, including reached aliases.
    pub warrior_memory: &'a [u8],
    pub items: ItemTables<'a>,
    pub spells: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttackWork {
    pub item: [u8; ITEM_BYTES],
    pub ammunition: [u8; ITEM_BYTES],
    pub weapon: [u8; ITEM_BYTES],
    pub packet: [u8; SPELL_BYTES],
    pub spell_coordinates: [i16; 3],
    pub attack_count: i16,
    pub damage_total: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttackEffect {
    Write(RecordWrite),
    LocationLog(i16),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalculatedDamage {
    pub work: AttackWork,
    pub damage: i16,
    pub native_return: i16,
    pub effects: Vec<AttackEffect>,
    pub qualification: ArithmeticQualification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageFrame {
    /// Incoming bytes at -4(A6), read if the 99-slot equipped search has no match.
    pub unmatched_multiplier_single_bits: u32,
}

pub fn calcdamage(
    current: &AttackWork,
    tables: &AttackTables<'_>,
    member: u16,
    mode: i16,
    return_total: i16,
    frame: DamageFrame,
) -> Result<CalculatedDamage, RuleError> {
    let mut result = CalculatedDamage {
        work: current.clone(),
        damage: 0,
        native_return: 0,
        effects: Vec::new(),
        qualification: ArithmeticQualification::SourceDerivedBinary32,
    };
    if mode == 0 || mode == 1 {
        let equipment = if mode == 0 { 0x12c } else { 0x12e };
        let id = member_word(tables, member, equipment)?;
        if id != 0 {
            let loaded = recovered_records::loaditem(&tables.items, &result.work.item, id)?;
            result.work.item = loaded.bytes;
            record_effects(&mut result.effects, loaded.writes);
            let base = if mode == 0 { 0x3e } else { 0x28 };
            let mut damage = word(&result.work.item, base)?;
            for offset in [0x8c, 0x40, 0x42, 0x44, 0x46, 0x48, 0x4a] {
                let component = word(&result.work.item, offset)?;
                if component > 0 {
                    damage = damage.wrapping_add(component);
                }
            }
            if mode == 0 {
                let ammunition = member_word(tables, member, 0x126)?;
                if ammunition != 0 {
                    let loaded = recovered_records::loadammo(
                        &tables.items,
                        &result.work.ammunition,
                        ammunition,
                    )?;
                    result.work.ammunition = loaded.bytes;
                    record_effects(&mut result.effects, loaded.writes);
                    damage = damage.wrapping_add(word(&result.work.ammunition, 0x3e)?);
                    for offset in [0x8c, 0x40, 0x42, 0x44, 0x46, 0x48, 0x4a] {
                        let component = word(&result.work.ammunition, offset)?;
                        if component > 0 {
                            damage = damage.wrapping_add(component);
                        }
                    }
                }
            }
            for offset in if mode == 0 {
                &[0x16][..]
            } else {
                &[0x16, 0x4c][..]
            } {
                let multiplier = word(&result.work.item, *offset)?;
                if multiplier > 1 {
                    damage = damage.wrapping_mul(multiplier);
                }
            }
            let mut multiplier = f32::from_bits(frame.unmatched_multiplier_single_bits);
            for slot in 0..99 {
                let offset = 0x132 + slot * 22;
                if member_word(tables, member, offset)? == id
                    && member_byte(tables, member, offset + 9)? != 0
                {
                    let instance = i16::from(member_byte(tables, member, offset + 18)? as i8);
                    let ammunition = if mode == 0 {
                        i16::from(result.work.ammunition[0x55] as i8)
                    } else {
                        0
                    };
                    multiplier = percentage_single(instance.wrapping_add(ammunition));
                    break;
                }
            }
            // This routine stores the product as Single before widening and truncating.
            let rounded_product = (f64::from(damage) * f64::from(multiplier)) as f32;
            result.damage = truncate_long(f64::from(rounded_product))? as i16;
            if mode == 0 {
                let total = (member_word(tables, member, 0x12)? / 2).wrapping_mul(result.damage);
                set_damage_total(&mut result, total);
                let multiplier = word(&result.work.item, 0x4c)?;
                if multiplier > 1 {
                    set_damage_total(&mut result, total.wrapping_mul(multiplier));
                }
            }
        }
        if mode == 1 {
            let bonus =
                member_word(tables, member, 0xc)?.wrapping_add(member_word(tables, member, 0xb7a)?);
            result.damage = result.damage.wrapping_add(bonus);
            let total = (member_word(tables, member, 0x12)? / 2).wrapping_mul(result.damage);
            set_damage_total(&mut result, total);
        }
    }
    result.native_return = if return_total != 0 {
        result.work.damage_total
    } else {
        result.damage
    };
    Ok(result)
}

fn set_damage_total(result: &mut CalculatedDamage, value: i16) {
    result.work.damage_total = value;
    effect_write(&mut result.effects, 0x41f714, &value.to_be_bytes());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreparationContext {
    pub active_member: u16,
    pub selected_item: i16,
    /// Low WORD of D3 on entry; forced-item and actor >=10 paths do not initialize it.
    pub entry_inventory_slot: u16,
    /// Incoming bytes at -6(A6); the successful suffix returns this unwritten WORD.
    pub frame_return_word: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedAttack {
    pub work: AttackWork,
    pub inventory_slot: u16,
    pub native_return: i16,
    pub effects: Vec<AttackEffect>,
    pub qualification: ArithmeticQualification,
}

pub fn prepattack(
    current: &AttackWork,
    tables: &AttackTables<'_>,
    actor: i16,
    mode: i16,
    context: PreparationContext,
) -> Result<PreparedAttack, RuleError> {
    let mut result = PreparedAttack {
        work: current.clone(),
        inventory_slot: context.entry_inventory_slot,
        native_return: context.frame_return_word,
        effects: Vec::new(),
        qualification: ArithmeticQualification::SourceDerivedBinary32,
    };
    if actor < 10 {
        if mode != 0 {
            load_weapon(&mut result, tables, context.selected_item)?;
            load_ammunition(&mut result, tables, 0)?;
        } else {
            let equipment = if member_byte(tables, actor as u16, 0x9dd)? == 0 {
                0xca
            } else {
                0xe4
            };
            load_weapon(
                &mut result,
                tables,
                member_word(tables, actor as u16, equipment)?,
            )?;
            let count = member_word(tables, context.active_member, 0x1a)?;
            let id = word(&result.work.weapon, 0x66)?;
            result.inventory_slot = 0;
            loop {
                if (result.inventory_slot as i16) >= count {
                    result.native_return = 0;
                    return Ok(result);
                }
                let offset = 0x132 + usize::from(result.inventory_slot) * 22;
                if member_word(tables, context.active_member, offset)? == id
                    && member_byte(tables, context.active_member, offset + 9)? == 1
                {
                    if member_word(tables, context.active_member, offset + 2)? == 0 {
                        result.native_return = 0;
                        return Ok(result);
                    }
                    load_ammunition(
                        &mut result,
                        tables,
                        member_word(tables, context.active_member, offset + 4)?,
                    )?;
                    break;
                }
                result.inventory_slot = result.inventory_slot.wrapping_add(1);
            }
        }
    }
    if word(&result.work.weapon, 0x2c)? > 1100 {
        let spell = recovered_records::loadspell2(tables.spells, word(&result.work.weapon, 0x2c)?)?;
        result.work.packet = spell.record.bytes;
        result.work.spell_coordinates = spell.coordinates;
        record_effects(&mut result.effects, spell.record.writes);
        build_weapon_packet(&mut result)?;
        result.effects.push(AttackEffect::LocationLog(1));
        if actor < 10 {
            result.effects.push(AttackEffect::LocationLog(2));
            let offset = 0x132 + usize::from(result.inventory_slot) * 22 + 20;
            let bonus = member_byte(tables, actor as u16, offset)?;
            let range = result.work.packet[2].wrapping_add(bonus);
            packet_write(&mut result, 2, &[range]);
        }
        result.effects.push(AttackEffect::LocationLog(3));
    } else {
        let source = span(tables.spells, 0x21c2, SPELL_BYTES)?;
        result.work.packet.copy_from_slice(source);
        for (index, bytes) in source[..56].as_chunks::<4>().0.iter().enumerate() {
            effect_write(&mut result.effects, 0x44374e + index as u32 * 4, bytes);
        }
        effect_write(&mut result.effects, 0x443786, &source[56..]);
    }
    result.effects.push(AttackEffect::LocationLog(4));
    let attacks = word(&result.work.weapon, 0x16)?;
    for offset in [32, 34] {
        let value = word(&result.work.packet, offset)?.wrapping_mul(attacks);
        packet_write(&mut result, offset, &value.to_be_bytes());
    }
    for offset in 23..30 {
        let value = (u16::from(result.work.packet[offset]).wrapping_mul(attacks as u16)) as u8;
        packet_write(&mut result, offset, &[value]);
    }
    let offset = 0x132 + usize::from(result.inventory_slot) * 22 + 18;
    let instance = i16::from(member_byte(tables, actor as u16, offset)? as i8);
    let ammunition = i16::from(result.work.ammunition[0x55] as i8);
    let multiplier = percentage_single(instance.wrapping_add(ammunition));
    for offset in [32, 34] {
        let value =
            truncate_long(f64::from(word(&result.work.packet, offset)?) * f64::from(multiplier))?
                as i16;
        packet_write(&mut result, offset, &value.to_be_bytes());
    }
    for offset in 23..30 {
        let value =
            truncate_long(f64::from(result.work.packet[offset]) * f64::from(multiplier))? as u8;
        packet_write(&mut result, offset, &[value]);
    }
    result.effects.push(AttackEffect::LocationLog(50));
    Ok(result)
}

fn build_weapon_packet(result: &mut PreparedAttack) -> Result<(), RuleError> {
    let count = result.work.weapon[0x4d];
    packet_write(result, 0, &[count]);
    result.work.attack_count = i16::from(count as i8);
    effect_write(
        &mut result.effects,
        0x4435cc,
        &result.work.attack_count.to_be_bytes(),
    );
    let filter = result.work.weapon[0x37];
    let category = result.work.weapon[0x35];
    packet_write(result, 17, &[filter]);
    packet_write(result, 18, &[category]);
    let range =
        word(&result.work.weapon, 0x3c)?.wrapping_add(word(&result.work.ammunition, 0x3c)?) as u8;
    let range = result.work.packet[2].wrapping_add(range);
    packet_write(result, 2, &[range]);
    let duration = result.work.ammunition[0x51].wrapping_add(result.work.weapon[0x51]);
    let duration = result.work.packet[5].wrapping_add(duration);
    packet_write(result, 5, &[duration]);
    let physical =
        word(&result.work.weapon, 0x3e)?.wrapping_add(word(&result.work.ammunition, 0x3e)?);
    packet_write(result, 34, &physical.to_be_bytes());
    if physical != 0 {
        packet_write(result, 32, &1_i16.to_be_bytes());
    }
    for (offset, source) in [
        (23, 0x8c),
        (24, 0x40),
        (25, 0x42),
        (26, 0x44),
        (27, 0x46),
        (28, 0x48),
        (29, 0x4a),
    ] {
        let value = word(&result.work.weapon, source)?
            .wrapping_add(word(&result.work.ammunition, source)?) as u8;
        packet_write(result, offset, &[value]);
    }
    for offset in [12, 14] {
        let target = 36 + offset - 12;
        let value = word(&result.work.weapon, offset)?;
        packet_write(result, target, &value.to_be_bytes());
    }
    for offset in [12, 14] {
        let value = word(&result.work.ammunition, offset)?;
        if value != 0 {
            weapon_write(result, offset, &value.to_be_bytes());
            packet_write(result, 36 + offset - 12, &value.to_be_bytes());
        }
    }
    let size =
        word(&result.work.weapon, 0x38)?.wrapping_add(word(&result.work.ammunition, 0x38)?) as u8;
    packet_write(result, 1, &[size]);
    if word(&result.work.ammunition, 0x38)? != 0 {
        let size = result.work.ammunition[0x39];
        packet_write(result, 1, &[size]);
        weapon_write(result, 0x38, &i16::from(size as i8).to_be_bytes());
    }
    let cost = result.work.weapon[0x3b];
    packet_write(result, 31, &[cost]);
    if word(&result.work.ammunition, 0x3a)? != 0 {
        let cost = result.work.ammunition[0x3b];
        packet_write(result, 31, &[cost]);
    }
    Ok(())
}

fn load_weapon(
    result: &mut PreparedAttack,
    tables: &AttackTables<'_>,
    id: i16,
) -> Result<(), RuleError> {
    let loaded = recovered_records::loadweapon(&tables.items, &result.work.weapon, id)?;
    result.work.weapon = loaded.bytes;
    record_effects(&mut result.effects, loaded.writes);
    Ok(())
}

fn load_ammunition(
    result: &mut PreparedAttack,
    tables: &AttackTables<'_>,
    id: i16,
) -> Result<(), RuleError> {
    let loaded = recovered_records::loadammo(&tables.items, &result.work.ammunition, id)?;
    result.work.ammunition = loaded.bytes;
    record_effects(&mut result.effects, loaded.writes);
    Ok(())
}

fn member_word(tables: &AttackTables<'_>, member: u16, offset: usize) -> Result<i16, RuleError> {
    Ok(word(
        tables.warrior_memory,
        usize::from(member) * WARRIOR_BYTES + offset,
    )?)
}

fn member_byte(tables: &AttackTables<'_>, member: u16, offset: usize) -> Result<u8, RuleError> {
    Ok(span(
        tables.warrior_memory,
        usize::from(member) * WARRIOR_BYTES + offset,
        1,
    )?[0])
}

fn percentage_single(bonus: i16) -> f32 {
    // LONG -> Single -> Extended, divide by Single 100, then round back to Single.
    ((f64::from(100.0_f32 + f32::from(bonus))) / 100.0) as f32
}

fn truncate_long(value: f64) -> Result<i32, RuleError> {
    let truncated = value.trunc();
    if !truncated.is_finite() || truncated < f64::from(i32::MIN) || truncated > f64::from(i32::MAX)
    {
        return Err(RuleError::Unsupported {
            action: "attack.sane",
            dependency: "nonfinite or overflowing SANE LONG conversion",
        });
    }
    Ok(truncated as i32)
}

fn packet_write(result: &mut PreparedAttack, offset: usize, bytes: &[u8]) {
    result.work.packet[offset..offset + bytes.len()].copy_from_slice(bytes);
    effect_write(&mut result.effects, 0x44374e + offset as u32, bytes);
}

fn weapon_write(result: &mut PreparedAttack, offset: usize, bytes: &[u8]) {
    result.work.weapon[offset..offset + bytes.len()].copy_from_slice(bytes);
    effect_write(&mut result.effects, 0x453768 + offset as u32, bytes);
}

fn record_effects(effects: &mut Vec<AttackEffect>, writes: Vec<RecordWrite>) {
    effects.extend(writes.into_iter().map(AttackEffect::Write));
}

fn effect_write(effects: &mut Vec<AttackEffect>, address: u32, bytes: &[u8]) {
    effects.push(AttackEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work() -> AttackWork {
        AttackWork {
            item: [0; 150],
            ammunition: [0; 150],
            weapon: [0; 150],
            packet: [0x5a; 58],
            spell_coordinates: [7, 8, 9],
            attack_count: 44,
            damage_total: -11,
        }
    }

    fn put(bytes: &mut [u8], offset: usize, value: i16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }

    #[test]
    fn damage_modes_keep_ordered_loaders_word_wrapping_and_the_distinct_multiplier_positions() {
        let mut memory = vec![0; WARRIOR_BYTES];
        put(&mut memory, 0x12, 7);
        put(&mut memory, 0xc, 4);
        put(&mut memory, 0xb7a, -1);
        put(&mut memory, 0x12c, 1);
        put(&mut memory, 0x12e, 1);
        put(&mut memory, 0x126, 2);
        put(&mut memory, 0x132, 1);
        memory[0x13b] = 2;
        memory[0x144] = 10;
        let mut items = vec![0; 30000];
        for (offset, value) in [
            (0x3e, 10),
            (0x28, 12),
            (0x8c, -2),
            (0x40, 3),
            (0x16, 2),
            (0x4c, 3),
        ] {
            put(&mut items, 150 + offset, value);
        }
        put(&mut items, 300 + 0x3e, -1);
        put(&mut items, 300 + 0x8c, 4);
        items[300 + 0x55] = 10;
        let tables = AttackTables {
            warrior_memory: &memory,
            items: ItemTables {
                families: [&items, &[], &[], &[], &[]],
            },
            spells: &[],
        };
        let state = work();
        let frame = DamageFrame {
            unmatched_multiplier_single_bits: 2.0_f32.to_bits(),
        };
        let ranged = calcdamage(&state, &tables, 0, 0, 1, frame).unwrap();
        // (10 + 3 - 1 + 4) * 2 = 32, Single 120% => trunc 38, then level/2 * 3.
        assert_eq!(ranged.damage, 38);
        assert_eq!(ranged.work.damage_total, 342);
        assert_eq!(ranged.native_return, 342);
        assert_eq!(ranged.effects.len(), 80);
        assert_eq!(ranged.work.item.as_slice(), &items[150..300]);
        assert_eq!(ranged.work.ammunition.as_slice(), &items[300..450]);
        let melee = calcdamage(&state, &tables, 0, 1, 0, frame).unwrap();
        // (12 + 3) * 2 * 3 = 90, Single 110% => 99, then +4 -1.
        assert_eq!(melee.damage, 102);
        assert_eq!(melee.work.damage_total, 306);
        assert_eq!(melee.work.ammunition, state.ammunition);
        assert_eq!(melee.effects.len(), 40);
        assert_eq!(melee.native_return, 102);
        assert_eq!(state.damage_total, -11);
    }

    #[test]
    fn damage_retains_the_native_no_equipment_and_unmatched_frame_paths() {
        let mut memory = vec![0; WARRIOR_BYTES];
        put(&mut memory, 0x12, -7);
        put(&mut memory, 0xc, 2);
        let mut items = vec![0; 30000];
        put(&mut items, 150 + 0x3e, 9);
        let tables = AttackTables {
            warrior_memory: &memory,
            items: ItemTables {
                families: [&items, &[], &[], &[], &[]],
            },
            spells: &[],
        };
        let state = work();
        let frame = DamageFrame {
            unmatched_multiplier_single_bits: 2.0_f32.to_bits(),
        };
        let absent = calcdamage(&state, &tables, 0, 0, 1, frame).unwrap();
        assert_eq!(absent.work, state);
        assert_eq!(absent.native_return, -11);
        assert!(absent.effects.is_empty());
        let melee = calcdamage(&state, &tables, 0, 1, 0, frame).unwrap();
        assert_eq!(melee.damage, 2);
        assert_eq!(melee.work.damage_total, -6);
        let other = calcdamage(&state, &tables, 0, -1, 0, frame).unwrap();
        assert_eq!(other.work, state);
        assert_eq!(other.native_return, 0);
        put(&mut memory, 0x12c, 1);
        let tables = AttackTables {
            warrior_memory: &memory,
            items: ItemTables {
                families: [&items, &[], &[], &[], &[]],
            },
            spells: &[],
        };
        let unmatched = calcdamage(&state, &tables, 0, 0, 0, frame).unwrap();
        assert_eq!(unmatched.damage, 18);
        assert_eq!(unmatched.work.damage_total, -54);
        let nonfinite = DamageFrame {
            unmatched_multiplier_single_bits: f32::NAN.to_bits(),
        };
        assert!(calcdamage(&state, &tables, 0, 0, 0, nonfinite).is_err());
        assert_eq!(state, work());
    }

    #[test]
    fn preparation_uses_active_member_for_admission_but_actor_for_enhancement_and_keeps_all_packet_bytes()
     {
        let mut memory = vec![0; 2 * WARRIOR_BYTES];
        put(&mut memory, 0xca, 1);
        put(&mut memory, WARRIOR_BYTES + 0x1a, 2);
        put(&mut memory, WARRIOR_BYTES + 0x132 + 22, 1);
        put(&mut memory, WARRIOR_BYTES + 0x134 + 22, 1);
        put(&mut memory, WARRIOR_BYTES + 0x136 + 22, 2);
        memory[WARRIOR_BYTES + 0x13b + 22] = 1;
        memory[0x144 + 22] = 10;
        memory[0x146 + 22] = 4;
        let mut items = vec![0; 30000];
        for (offset, value) in [
            (0x2c, 1101),
            (0x66, 1),
            (0x16, 2),
            (0x3c, 2),
            (0x3e, 10),
            (0x8c, 4),
            (0x40, -1),
            (12, 99),
            (14, 88),
            (0x38, 2),
        ] {
            put(&mut items, 150 + offset, value);
        }
        items[150 + 0x4d] = 3;
        items[150 + 0x37] = 7;
        items[150 + 0x35] = 9;
        items[150 + 0x51] = 5;
        items[150 + 0x3b] = 1;
        for (offset, value) in [
            (0x3c, 1),
            (0x3e, 5),
            (0x8c, 3),
            (0x40, -2),
            (12, 101),
            (0x38, -2),
            (0x3a, 0x102),
        ] {
            put(&mut items, 300 + offset, value);
        }
        items[300 + 0x51] = 253;
        items[300 + 0x55] = 251;
        let mut spells = vec![0; 11368];
        let base: [u8; 58] = std::array::from_fn(|i| (i + 30) as u8);
        spells[..58].copy_from_slice(&base);
        let tables = AttackTables {
            warrior_memory: &memory,
            items: ItemTables {
                families: [&items, &[], &[], &[], &[]],
            },
            spells: &spells,
        };
        let state = work();
        let context = PreparationContext {
            active_member: 1,
            selected_item: 1,
            entry_inventory_slot: 99,
            frame_return_word: -123,
        };
        let result = prepattack(&state, &tables, 0, 0, context).unwrap();
        let mut expected = base;
        expected[0] = 3;
        expected[1] = 254;
        expected[2] = base[2] + 3 + 4;
        expected[5] = base[5] + 2;
        expected[17] = 7;
        expected[18] = 9;
        expected[23..30].fill(0);
        expected[23] = 14;
        expected[24] = 6; // 253 * 2 wraps BYTE to250, times Single1.05 => low BYTE262.
        expected[31] = 2;
        put(&mut expected, 32, 2);
        put(&mut expected, 34, 31);
        put(&mut expected, 36, 101);
        put(&mut expected, 38, 88);
        assert_eq!(result.work.packet, expected);
        assert_eq!(result.work.spell_coordinates, [0, 0, 0]);
        assert_eq!(result.work.attack_count, 3);
        assert_eq!(result.inventory_slot, 1);
        assert_eq!(result.native_return, -123);
        let mut weapon: [u8; 150] = items[150..300].try_into().unwrap();
        put(&mut weapon, 12, 101);
        put(&mut weapon, 0x38, -2);
        assert_eq!(result.work.weapon, weapon);
        assert_eq!(result.work.ammunition.as_slice(), &items[300..450]);
        assert_eq!(result.work.item, state.item);
        let sound_stores: Vec<(u32, i16)> = result
            .effects
            .iter()
            .filter_map(|effect| {
                let AttackEffect::Write(write) = effect else {
                    return None;
                };
                if write.bytes.len() == 2
                    && [0x443772, 0x443774, 0x453774, 0x453776].contains(&write.address)
                {
                    Some((
                        write.address,
                        i16::from_be_bytes(write.bytes.as_slice().try_into().unwrap()),
                    ))
                } else {
                    None
                }
            })
            .collect();
        // Instructions 14c928/14c932 precede overrides 14c944/14c94e; the decompiler reorders them.
        assert_eq!(
            sound_stores,
            vec![
                (0x443772, 99),
                (0x443774, 88),
                (0x453774, 101),
                (0x443772, 101)
            ]
        );
        let logs: Vec<i16> = result
            .effects
            .iter()
            .filter_map(|e| {
                if let AttackEffect::LocationLog(value) = e {
                    Some(*value)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(logs, vec![1, 2, 3, 4, 50]);
        let mut empty = memory.clone();
        put(&mut empty, WARRIOR_BYTES + 0x134 + 22, 0);
        let empty_tables = AttackTables {
            warrior_memory: &empty,
            items: ItemTables {
                families: [&items, &[], &[], &[], &[]],
            },
            spells: &spells,
        };
        let unavailable = prepattack(&state, &empty_tables, 0, 0, context).unwrap();
        assert_eq!(unavailable.native_return, 0);
        assert_eq!(unavailable.work.packet, state.packet);
        assert_eq!(unavailable.work.ammunition, state.ammunition);
        assert_eq!(unavailable.effects.len(), 39);
    }

    #[test]
    fn forced_and_npc_paths_use_the_entry_slot_and_scale_the_default_packet_without_invented_loads()
    {
        let mut memory = vec![0; 11 * WARRIOR_BYTES];
        memory[0x144 + 4 * 22] = 50;
        memory[10 * WARRIOR_BYTES + 0x144 + 2 * 22] = 100;
        let mut items = vec![0; 30000];
        put(&mut items, 150 + 0x16, 2);
        let mut spells = vec![0; 11368];
        let mut packet = [0x31; 58];
        put(&mut packet, 32, 2);
        put(&mut packet, 34, 10);
        packet[23..30].fill(3);
        spells[0x21c2..0x21c2 + 58].copy_from_slice(&packet);
        let tables = AttackTables {
            warrior_memory: &memory,
            items: ItemTables {
                families: [&items, &[], &[], &[], &[]],
            },
            spells: &spells,
        };
        let context = PreparationContext {
            active_member: 99,
            selected_item: 1,
            entry_inventory_slot: 4,
            frame_return_word: 77,
        };
        let forced = prepattack(&work(), &tables, 0, 1, context).unwrap();
        let mut expected = packet;
        put(&mut expected, 32, 6);
        put(&mut expected, 34, 30);
        expected[23..30].fill(9);
        assert_eq!(forced.work.packet, expected);
        assert_eq!(forced.inventory_slot, 4);
        assert_eq!(forced.native_return, 77);
        assert_eq!(forced.work.attack_count, 44); // Default packet does not update global count.
        let mut retained = work();
        put(&mut retained.weapon, 0x16, 1);
        let npc_context = PreparationContext {
            entry_inventory_slot: 2,
            ..context
        };
        let npc = prepattack(&retained, &tables, 10, 0, npc_context).unwrap();
        put(&mut expected, 32, 4);
        put(&mut expected, 34, 20);
        expected[23..30].fill(6);
        assert_eq!(npc.work.packet, expected);
        assert_eq!(npc.work.weapon, retained.weapon);
        assert_eq!(npc.work.ammunition, retained.ammunition);
        assert_eq!(npc.inventory_slot, 2);
        assert_eq!(npc.effects.len(), 35);
        let short = AttackTables {
            warrior_memory: &memory[..WARRIOR_BYTES],
            items: ItemTables {
                families: [&items, &[], &[], &[], &[]],
            },
            spells: &spells,
        };
        assert!(prepattack(&retained, &short, 10, 0, npc_context).is_err());
        assert_eq!(retained.packet, [0x5a; 58]);
    }
}
