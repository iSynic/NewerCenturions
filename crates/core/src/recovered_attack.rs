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
