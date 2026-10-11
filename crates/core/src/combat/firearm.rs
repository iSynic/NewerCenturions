use super::*;
use attack::AttackResult;
use damage::{DamageFlags, DamagePacket, resolve_damage};

pub struct PreparedFirearm {
    pub packet: Vec<u8>,
    pub slot: usize,
    pub shots: i16,
    pub rounds: i16,
    pub weight: i16,
    pub sound: i16,
    pub armor_filter: bool,
}

pub struct FitAmmunitionResult {
    pub warrior: Warrior,
    pub ammunition: Vec<i32>,
    pub loaded_item_field32: i16,
    pub sounds: Vec<i16>,
    pub loaded: bool,
}

pub struct EarnedRefitResult {
    pub warrior: Warrior,
    pub reserve: i32,
    pub loaded_item_field32: i16,
    pub weapon_work: weapon_work::WeaponWork,
}

pub fn refit_earned_secondary(
    warrior: &Warrior,
    content: &centurions_content::Content,
    reserve: i32,
) -> Result<EarnedRefitResult, RuleError> {
    let unsupported = || RuleError::Unsupported {
        action: "combat.refit",
        dependency: "other earned secondary refill instance or stock",
    };
    let w = &warrior.bytes;
    if w.len() != 3138
        || w[0x9dd] != 1
        || word(w, 0xe4)? != 165
        || word(w, 0x126)? != 280
        || !(1..=3).contains(&reserve)
    {
        return Err(unsupported());
    }
    let inventory = warrior.inventory()?;
    let item = inventory.first().ok_or_else(unsupported)?;
    if item.id() != 165
        || word(&item.0, 2)? != 0
        || word(&item.0, 4)? != 280
        || word(&item.0, 6)? != 1
        || item.0[8] != 0
        || item.0[9] != 1
        || item.0[10] != 15
        || item.0[11..13].iter().any(|value| *value != 0)
        || !matches!((item.0[13], item.0[18]), (2, 0) | (3, 10))
        || item.0[14..18].iter().any(|value| *value != 0)
        || item.0[19..].iter().any(|value| *value != 0)
    {
        return Err(unsupported());
    }
    let weapon = content.items.get(165).ok_or_else(unsupported)?;
    let ammunition = content.items.get(280).ok_or_else(unsupported)?;
    if weapon.len() != 150
        || ammunition.len() != 150
        || word(weapon, 26)? != 15
        || word(weapon, 32)? != 16
        || word(weapon, 22)? != 1
        || word(weapon, 90)? != 0
        || word(weapon, 102)? != 165
        || word(ammunition, 26)? != 26
        || word(ammunition, 34)? != 16
        || word(ammunition, 102)? != 280
    {
        return Err(unsupported());
    }
    let mut result = EarnedRefitResult {
        warrior: warrior.clone(),
        reserve,
        loaded_item_field32: word(weapon, 32)?,
        weapon_work: weapon_work::WeaponWork::zero(),
    };
    result.weapon_work.load_pair(weapon, ammunition)?;
    let needed = word(&item.0, 6)?.wrapping_sub(word(&item.0, 2)?);
    let loaded = word(&item.0, 2)?.wrapping_add(needed);
    put_word(&mut result.warrior.bytes, 0x134, loaded);
    result.reserve = reserve.wrapping_sub(i32::from(needed));
    Ok(result)
}

fn fit_battery(
    warrior: &Warrior,
    slot: usize,
    content: &centurions_content::Content,
    reserves: &[i32],
    in_combat: bool,
    random: &mut RandomState,
) -> Result<FitAmmunitionResult, RuleError> {
    let inventory = warrior.inventory()?;
    let item = &inventory[slot];
    let loaded = word(&item.0, 2)?;
    let ammunition = word(&item.0, 4)?;
    let battery = content
        .items
        .get(637)
        .ok_or_else(|| RuleError::Invalid("battery definition missing".into()))?;
    let supply = content
        .items
        .get(307)
        .ok_or_else(|| RuleError::Invalid("battery supply definition missing".into()))?;
    if in_combat
        || item.0[9] != 1
        || item.0[10] != 20
        || item.0[8] != 0
        || item.0[11..13].iter().any(|value| *value != 0)
        || item.0[13] != 2
        || item.0[14..].iter().any(|value| *value != 0)
        || word(&item.0, 6)? != 250
        || !((loaded == 0 && ammunition == 0) || ((0..=250).contains(&loaded) && ammunition == 307))
        || word(&warrior.bytes, 0xee)? != 637
        || word(&warrior.bytes, 0x26)? != loaded
        || word(&warrior.bytes, 0x28)? != 250
        || word(battery, 26)? != 20
        || word(battery, 32)? != 24
        || word(battery, 90)? != 250
        || word(battery, 148)? != 0
        || word(supply, 26)? != 26
        || word(supply, 34)? != 24
        || (loaded != 0 && reserves[107] > 0)
    {
        return Err(RuleError::Unsupported {
            action: "equipment.fit_ammunition",
            dependency: "other battery instance, repeated fill or combat fitting",
        });
    }
    if reserves[107] < 0 {
        return Err(RuleError::Invalid("negative battery supply reserve".into()));
    }
    let mut result = FitAmmunitionResult {
        warrior: warrior.clone(),
        ammunition: reserves.to_vec(),
        loaded_item_field32: 24,
        sounds: vec![],
        loaded: false,
    };
    if reserves[107] == 0 {
        if loaded != 0 {
            result.sounds.push(random.bound(2).wrapping_add(403));
        }
        return Ok(result);
    }
    let amount = reserves[107].min(250) as i16;
    let offset = 0x132 + slot * 22;
    put_word(&mut result.warrior.bytes, offset + 2, amount);
    result.ammunition[107] = reserves[107].wrapping_sub(i32::from(amount));
    for at in [0x126, offset + 4] {
        put_word(&mut result.warrior.bytes, at, 307);
    }
    put_word(&mut result.warrior.bytes, 0x26, loaded.wrapping_add(amount));
    result.sounds.push(random.bound(2).wrapping_add(403));
    result.loaded = true;
    Ok(result)
}

pub fn fit_ammunition(
    warrior: &Warrior,
    slot: usize,
    content: &centurions_content::Content,
    reserves: &[i32],
    in_combat: bool,
    random: &mut RandomState,
) -> Result<FitAmmunitionResult, RuleError> {
    centurions_content::span(&warrior.bytes, 0, 3138)?;
    if reserves.len() != 200 {
        return Err(RuleError::Invalid("ammunition reserve size".into()));
    }
    let inventory = warrior.inventory()?;
    let item = inventory
        .get(slot)
        .ok_or_else(|| RuleError::Invalid("ammunition fitting slot missing".into()))?;
    if item.id() == 637 {
        return fit_battery(warrior, slot, content, reserves, in_combat, random);
    }
    let loaded = word(&item.0, 2)?;
    let ammunition = word(&item.0, 4)?;
    let (ammo_id, reserve_index, capacity, caliber, level, bonus) = match item.id() {
        2 => (201, 1, 12, 2, 1, 0),
        38 => (203, 3, 3, 4, 4, 35),
        _ => {
            return Err(RuleError::Unsupported {
                action: "equipment.fit_ammunition",
                dependency: "other firearm",
            });
        }
    };
    let loaded_supported = if item.id() == 2 {
        (ammunition == 0 && loaded == 0)
            || (ammunition == ammo_id && (0..=capacity).contains(&loaded))
    } else {
        matches!((loaded, ammunition), (0, 0) | (2, 203))
    };
    if item.0[9] != 1
        || item.0[10] != 15
        || item.0[13] != level
        || item.0[14] != 0
        || item.0[15] != bonus
        || item.0[16..].iter().any(|value| *value != 0)
        || word(&item.0, 6)? != capacity
        || !loaded_supported
        || word(&warrior.bytes, 0xe4)? != item.id()
        || warrior.bytes[0x9dd] != 1
        || (item.id() != 2 && word(&warrior.bytes, 0x126)? != ammunition)
    {
        return Err(RuleError::Unsupported {
            action: "equipment.fit_ammunition",
            dependency: "other firearm, ammunition or loaded instance",
        });
    }
    let weapon = content
        .items
        .get(item.id() as usize)
        .ok_or_else(|| RuleError::Invalid("firearm definition missing".into()))?;
    let ammo = content
        .items
        .get(ammo_id as usize)
        .ok_or_else(|| RuleError::Invalid("ammunition definition missing".into()))?;
    if word(weapon, 32)? != caliber || word(ammo, 34)? != caliber || word(ammo, 26)? != 26 {
        return Err(RuleError::Unsupported {
            action: "equipment.fit_ammunition",
            dependency: "other ammunition definitions",
        });
    }
    if reserves[reserve_index] < 0 {
        return Err(RuleError::Invalid("negative ammunition reserve".into()));
    }
    let mut result = FitAmmunitionResult {
        warrior: warrior.clone(),
        ammunition: reserves.to_vec(),
        loaded_item_field32: caliber,
        sounds: if in_combat { vec![141] } else { vec![] },
        loaded: false,
    };
    if reserves[reserve_index] == 0 {
        return Ok(result);
    }
    let mut next_random = random.clone();
    let offset = 0x132 + slot * 22;
    // Stored ammunition IDs differ from popup reserve indices, so unload first.
    if ammunition != 0 {
        let returned = reserves[reserve_index]
            .checked_add(i32::from(loaded))
            .ok_or(RuleError::Unsupported {
                action: "equipment.fit_ammunition",
                dependency: "ammunition stock overflow",
            })?;
        result.ammunition[reserve_index] = returned;
        for at in [offset + 2, offset + 4, 0x126] {
            put_word(&mut result.warrior.bytes, at, 0);
        }
    }
    let amount = result.ammunition[reserve_index].min(i32::from(capacity)) as i16;
    if in_combat && amount > 0 {
        let actions = word(&result.warrior.bytes, 0x40)?.wrapping_sub(1);
        put_word(&mut result.warrior.bytes, 0x40, actions);
    }
    put_word(&mut result.warrior.bytes, offset + 2, amount);
    result.ammunition[reserve_index] =
        result.ammunition[reserve_index].wrapping_sub(i32::from(amount));
    for at in [0x126, offset + 4] {
        put_word(&mut result.warrior.bytes, at, ammo_id);
    }
    result.sounds.push(next_random.bound(2).wrapping_add(403));
    result.loaded = true;
    *random = next_random;
    Ok(result)
}

pub fn prepare(
    warrior: &Warrior,
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
) -> Result<PreparedFirearm, RuleError> {
    let w = &warrior.bytes;
    let id = word(w, 0xe4)?;
    if w[0x9dd] != 1 || !matches!(id, 1 | 2 | 38 | 165) || weapon.len() != 150 || ammo.len() != 150
    {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.prepare",
            dependency: "other secondary firearm",
        });
    }
    let slot = warrior
        .inventory()?
        .iter()
        .position(|i| i.id() == id && i.0[9] == 1)
        .ok_or_else(|| RuleError::Invalid("equipped firearm absent from inventory".into()))?;
    let item = 0x132 + slot * 22;
    let ammunition = match id {
        2 => 201,
        38 => 203,
        165 => 280,
        _ => 200,
    };
    if word(w, item + 4)? != ammunition
        || (id != 165 && w[item + 18] != 0)
        || w[item + 20] != 0
        || (id == 38 && (w[item + 15] != 35 || word(w, item + 6)? != 3))
        || (id == 2
            && (w[item + 10] != 15
                || w[item + 13] != 1
                || w[item + 14..item + 22].iter().any(|value| *value != 0)
                || word(w, item + 6)? != 12
                || !(1..=12).contains(&word(w, item + 2)?)))
        || (id == 165
            && (w[item + 8] != 0
                || w[item + 10] != 15
                || w[item + 11..item + 13].iter().any(|value| *value != 0)
                || !matches!((w[item + 13], w[item + 18]), (2, 0) | (3, 10))
                || w[item + 14..item + 18].iter().any(|value| *value != 0)
                || w[item + 19..item + 22].iter().any(|value| *value != 0)
                || word(w, item + 2)? != 1
                || word(w, item + 6)? != 1
                || word(weapon, 102)? != 165
                || word(weapon, 32)? != 16
                || word(ammo, 102)? != 280
                || word(ammo, 34)? != 16))
    {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.prepare",
            dependency: "other ammunition or inventory weapon enhancements",
        });
    }
    let descriptor = word(weapon, 44)?;
    if descriptor != 4101 {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.prepare",
            dependency: "other attack descriptors",
        });
    }
    let p = prepare_packet_with_enhancement(weapon, ammo, attacks, w[item + 18] as i8)?;
    let rounds = word(weapon, 22)?;
    let impact = word(&p, 38)?;
    if p[19] != 9 || p[20] != 9 || word(&p, 40)? != 0 || p[23..30].iter().any(|v| *v != 0) {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.prepare",
            dependency: "nonphysical firearm effects",
        });
    }
    Ok(PreparedFirearm {
        packet: p,
        slot,
        shots: i16::from(weapon[77] as i8),
        rounds,
        weight: word(weapon, 36)?,
        sound: impact,
        armor_filter: word(weapon, 136)?.wrapping_add(word(ammo, 136)?) != 0,
    })
}

pub(crate) fn prepare_packet(
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
) -> Result<Vec<u8>, RuleError> {
    prepare_packet_with_enhancement(weapon, ammo, attacks, 0)
}

fn prepare_packet_with_enhancement(
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
    enhancement: i8,
) -> Result<Vec<u8>, RuleError> {
    let descriptor = word(weapon, 44)?;
    if !matches!(descriptor, 4101 | 4102) {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.prepare",
            dependency: "other attack descriptors",
        });
    }
    let offset = 3 * 2842 + (descriptor - 4101) as usize * 58;
    let mut p = centurions_content::span(attacks, offset, 58)?.to_vec();
    p[0] = weapon[77];
    p[17] = weapon[55];
    p[18] = weapon[53];
    p[2] = p[2].wrapping_add(word(weapon, 60)?.wrapping_add(word(ammo, 60)?) as u8);
    p[5] = p[5].wrapping_add(weapon[81]).wrapping_add(ammo[81]);
    put_word(&mut p, 34, word(weapon, 62)?.wrapping_add(word(ammo, 62)?));
    if word(&p, 34)? != 0 {
        put_word(&mut p, 32, 1);
    }
    for (dest, offset) in [
        (23, 140),
        (24, 64),
        (25, 66),
        (26, 68),
        (27, 70),
        (28, 72),
        (29, 74),
    ] {
        p[dest] = word(weapon, offset)?.wrapping_add(word(ammo, offset)?) as u8;
    }
    let launch = if word(ammo, 12)? != 0 {
        word(ammo, 12)?
    } else {
        word(weapon, 12)?
    };
    let impact = if word(ammo, 14)? != 0 {
        word(ammo, 14)?
    } else {
        word(weapon, 14)?
    };
    put_word(&mut p, 36, launch);
    put_word(&mut p, 38, impact);
    p[1] = word(weapon, 56)?.wrapping_add(word(ammo, 56)?) as u8;
    if word(ammo, 56)? != 0 {
        p[1] = word(ammo, 56)? as u8;
    }
    p[31] = weapon[59];
    if word(ammo, 58)? != 0 {
        p[31] = word(ammo, 58)? as u8;
    }
    let rounds = word(weapon, 22)?;
    for offset in [32, 34] {
        let value = word(&p, offset)?.wrapping_mul(rounds);
        put_word(&mut p, offset, value);
    }
    for value in &mut p[23..30] {
        *value = value.wrapping_mul(rounds as u8);
    }
    let adjustment = i16::from(enhancement).wrapping_add(i16::from(ammo[85] as i8));
    let factor = (100_i32 + i32::from(adjustment)) as f32 / 100.0;
    for offset in [32, 34] {
        let value = (f32::from(word(&p, offset)?) * factor).trunc() as i16;
        put_word(&mut p, offset, value);
    }
    for value in &mut p[23..30] {
        *value = (f32::from(*value) * factor).trunc() as u8;
    }
    Ok(p)
}

pub fn prepare_enemy(
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    attacks: &[u8],
) -> Result<PreparedFirearm, RuleError> {
    if !matches!(word(&enemy.bytes, 0x26)?, 680 | 681)
        || word(&enemy.bytes, 0x4c)? != 0
        || word(weapon, 44)? != 4101
    {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "other enemy weapons or ammunition",
        });
    }
    let id = word(&enemy.bytes, 0x26)?;
    put_word(&mut enemy.bytes, 0x4e, id);
    let p = prepare_packet(weapon, &[0; 150], attacks)?;
    if p[19] != 9
        || p[20] != 9
        || word(&p, 40)? != 0
        || p[23] != 0
        || p[25..30].iter().any(|v| *v != 0)
    {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "other enemy firearm effects",
        });
    }
    Ok(PreparedFirearm {
        sound: word(&p, 38)?,
        packet: p,
        slot: 0,
        shots: i16::from(weapon[77] as i8),
        rounds: word(weapon, 22)?,
        weight: 0,
        armor_filter: word(weapon, 136)? != 0,
    })
}

pub fn resolve_enemy(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    random.range(
        prepared.packet[13] as i8 as i16,
        prepared.packet[14] as i8 as i16,
    );
    resolve_enemy_after_duration(warrior, enemy, prepared, random, difficulty)
}

pub fn resolve_enemy_after_duration(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let w = &mut warrior.bytes;
    let m = &enemy.bytes;
    let p = &prepared.packet;
    if !prepared.armor_filter && word(w, 0x60)? == 0 {
        let hit = 33_i16
            .wrapping_add(word(m, 0)?.wrapping_mul(5))
            .wrapping_add(word(m, 0x66)?)
            .wrapping_sub(word(w, 8)?.wrapping_add(word(w, 0xb5c)?))
            .wrapping_sub(5)
            .wrapping_sub(random.bound(word(w, 0x9e8)? / 3))
            .wrapping_sub(word(w, 0x16)?.wrapping_mul(word(w, 0x3a)?));
        if random.bound(100) > hit.clamp(15 - difficulty * 5, 90 - difficulty * 5) {
            return Ok(AttackResult {
                damage: 0,
                killed: false,
                sounds: vec![],
            });
        }
    }
    let result = resolve_damage(
        w,
        false,
        &DamagePacket {
            bytes: p.clone(),
            attack_count: prepared.shots,
        },
        DamageFlags {
            shield: true,
            armor: true,
            saves: true,
            armor_filter: prepared.armor_filter,
        },
        random,
    )?;
    let mut sounds = vec![];
    if result.shield_sound {
        sounds.push(206);
    }
    if result.total != 0 {
        if p[19] != 9 {
            sounds.push(prepared.sound);
        }
        let health = word(w, 0x50)?.wrapping_sub(result.total);
        put_word(w, 0x50, health);
        if result.total > 0 {
            put_word(w, 0x14, 1);
        }
    }
    Ok(AttackResult {
        damage: result.total,
        killed: word(w, 0x50)? <= 0,
        sounds,
    })
}

fn increment(bytes: &mut [u8], offset: usize, value: i32) {
    let old = i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
    bytes[offset..offset + 4].copy_from_slice(&old.wrapping_add(value).to_be_bytes());
}

pub fn resolve(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    if word(&warrior.bytes, 0xe4)? == 165 {
        validate_earned_shot(warrior, enemy, prepared, difficulty)?;
        let mut next_warrior = warrior.clone();
        let mut next_enemy = enemy.clone();
        let mut next_random = random.clone();
        let result = resolve_inner(
            &mut next_warrior,
            &mut next_enemy,
            prepared,
            &mut next_random,
            difficulty,
        )?;
        *warrior = next_warrior;
        *enemy = next_enemy;
        *random = next_random;
        return Ok(result);
    }
    resolve_inner(warrior, enemy, prepared, random, difficulty)
}

fn validate_earned_shot(
    warrior: &Warrior,
    enemy: &MonsterRecord,
    prepared: &PreparedFirearm,
    difficulty: i16,
) -> Result<(), RuleError> {
    let unsupported = || RuleError::Unsupported {
        action: "combat.firearm.resolve",
        dependency: "other earned firearm shot instance",
    };
    let w = &warrior.bytes;
    if w.len() != 3138
        || enemy.bytes.len() != 288
        || w[0x9dd] != 1
        || word(w, 0x126)? != 280
        || word(&enemy.bytes, 0x110)? <= 0
        || difficulty != 0
        || prepared.slot != 0
        || prepared.shots != 1
        || prepared.rounds != 1
        || prepared.weight != 0
        || prepared.sound != 406
        || prepared.armor_filter
    {
        return Err(unsupported());
    }
    let inventory = warrior.inventory()?;
    let item = inventory.first().ok_or_else(unsupported)?;
    if item.id() != 165
        || word(&item.0, 2)? != 0
        || word(&item.0, 4)? != 280
        || word(&item.0, 6)? != 1
        || item.0[8] != 0
        || item.0[9] != 1
        || item.0[10] != 15
        || item.0[11..13].iter().any(|value| *value != 0)
        || item.0[14..18].iter().any(|value| *value != 0)
        || item.0[19..].iter().any(|value| *value != 0)
    {
        return Err(unsupported());
    }
    let maximum = match (item.0[13], item.0[18]) {
        (2, 0) => 20,
        (3, 10) => 22,
        _ => return Err(unsupported()),
    };
    const PACKET: [u8; 58] = [
        1, 1, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 9, 9, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1, 0, 1, 0, 20, 1, 166, 1, 150, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut expected = PACKET;
    put_word(&mut expected, 34, maximum);
    if prepared.packet != expected {
        return Err(unsupported());
    }
    Ok(())
}

fn resolve_inner(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    let w = &mut warrior.bytes;
    let m = &mut enemy.bytes;
    let p = &prepared.packet;
    // resolvespell draws the effect duration even when the duration range is zero.
    random.range(p[13] as i8 as i16, p[14] as i8 as i16);
    let mut hit = 33_i16
        .wrapping_add(word(w, 4)?)
        .wrapping_add(word(w, 0xb58)?);
    if m[0xb3] == 0 {
        hit = hit
            .wrapping_sub(word(m, 0x62)?)
            .wrapping_sub(random.bound(word(m, 4)? / 3));
        hit = hit.wrapping_sub(word(m, 0)?.wrapping_mul(3));
        hit = hit.wrapping_sub(i16::from(m[0x104] as i8).wrapping_mul(word(m, 0)?));
        hit = hit.wrapping_add(match word(m, 0x10)?.wrapping_abs() {
            0 => -5,
            1 | 2 => 5,
            3 => 10,
            _ => 0,
        });
        let minimum = 15_i16.wrapping_sub(difficulty.wrapping_mul(5));
        let maximum = 90_i16.wrapping_sub(difficulty.wrapping_mul(5));
        if random.bound(100) > hit.clamp(minimum, maximum) {
            return Ok(AttackResult {
                damage: 0,
                killed: false,
                sounds: vec![],
            });
        }
    }
    let result = resolve_damage(
        m,
        true,
        &DamagePacket {
            bytes: p.clone(),
            attack_count: prepared.shots,
        },
        DamageFlags {
            shield: true,
            armor: true,
            saves: true,
            ..Default::default()
        },
        random,
    )?;
    let damage = result.total;
    let mut sounds = vec![];
    if result.shield_sound {
        sounds.push(206);
    }
    if damage != 0 {
        let health = word(m, 0x110)?.wrapping_sub(damage);
        put_word(m, 0x110, health);
        increment(w, 0xa46, i32::from(damage));
        increment(w, 0xa4a, 1);
        if health <= 0 {
            increment(w, 0xa5a, 1);
        } else {
            sounds.push(prepared.sound);
        }
    }
    Ok(AttackResult {
        damage,
        killed: word(m, 0x110)? <= 0,
        sounds,
    })
}

pub fn reload(
    warrior: &mut Warrior,
    reserve: &mut i32,
    silent: bool,
    random: &mut RandomState,
) -> Result<(bool, Vec<i16>), RuleError> {
    if word(&warrior.bytes, 0xe4)? == 165 {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.reload",
            dependency: "earned firearm reload",
        });
    }
    let inventory = warrior.inventory()?;
    let mut sounds = vec![];
    for (slot, item) in inventory.iter().enumerate() {
        if item.0[9] == 0 {
            continue;
        }
        let loaded = word(&item.0, 2)?;
        if item.id() == 2
            && (item.0[9] != 1
                || item.0[10] != 15
                || item.0[13] != 1
                || item.0[14..22].iter().any(|value| *value != 0)
                || word(&item.0, 6)? != 12
                || word(&item.0, 4)? != 201
                || loaded != 0
                || word(&warrior.bytes, 0xe4)? != 2
                || word(&warrior.bytes, 0x126)? != 201
                || warrior.bytes[0x9dd] != 1
                || !(0..=i32::from(i16::MAX)).contains(reserve)
                || !silent)
        {
            return Err(RuleError::Unsupported {
                action: "combat.reload",
                dependency: "other ordinary firearm reload instance",
            });
        }
        let needed = word(&item.0, 6)?.wrapping_sub(loaded);
        if needed <= 0 || word(&item.0, 4)? == 0 {
            continue;
        }
        let expected_ammo = match item.id() {
            2 => 201,
            38 => 203,
            _ => 200,
        };
        if !matches!(item.id(), 1 | 2 | 38)
            || word(&item.0, 4)? != expected_ammo
            || item.0[10] != 15
            || (item.id() == 38 && (item.0[15] != 35 || word(&item.0, 6)? != 3))
        {
            return Err(RuleError::Unsupported {
                action: "combat.reload",
                dependency: "other active equipment",
            });
        }
        if needed <= 0 || *reserve == 0 {
            continue;
        }
        let amount = (*reserve as i16).min(needed);
        if item.id() == 38 && (*reserve as i16) < needed {
            return Err(RuleError::Unsupported {
                action: "combat.reload",
                dependency: "partial refill with ammunition 203",
            });
        }
        put_word(
            &mut warrior.bytes,
            0x132 + 22 * slot + 2,
            loaded.wrapping_add(amount),
        );
        let complete = (*reserve as i16) >= needed;
        // The original partial-refill branch leaves the stockpile unchanged.
        if complete {
            *reserve = reserve.wrapping_sub(i32::from(needed));
        }
        if !silent {
            sounds.push(random.bound(2).wrapping_add(403));
        }
        if complete {
            return Ok((true, sounds));
        }
    }
    Ok((false, sounds))
}

pub fn reload_command(
    state: &mut Session,
    content: &Content,
    index: usize,
) -> Result<Vec<Event>, RuleError> {
    if state.phase == Phase::SquadSelection
        || state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_some())
    {
        return Err(RuleError::Invalid(
            "reload requires an unobstructed mission".into(),
        ));
    }
    if state.phase == Phase::Combat
        && state.combat.as_ref().and_then(Combat::active_body) != Some(index as i16)
    {
        return Err(RuleError::Invalid(
            "reload requires the active warrior".into(),
        ));
    }
    if state.phase == Phase::Exploration {
        let warrior = state
            .warriors
            .get(index)
            .ok_or_else(|| RuleError::Invalid("warrior outside squad".into()))?;
        let result = crate::ammunition::refit(
            warrior,
            &state.ammunition,
            content,
            crate::ammunition::RefitFlags {
                in_combat: false,
                force: false,
                silent: false,
                warnings: true,
            },
            &state.ammunition_work,
            &mut state.random,
        )?;
        return crate::ammunition::commit_result(state, index, result);
    }
    let warrior = state
        .warriors
        .get_mut(index)
        .ok_or_else(|| RuleError::Invalid("warrior outside squad".into()))?;
    let ammo = word(&warrior.bytes, 0x126)?;
    let reserve_index = match ammo {
        200 => 0,
        201 => 1,
        203 => 3,
        _ => {
            return Err(RuleError::Unsupported {
                action: "combat.reload",
                dependency: "other ammunition",
            });
        }
    };
    let inventory = warrior.inventory()?;
    let ammunition_loads: Vec<_> = inventory
        .iter()
        .filter(|item| {
            item.0[9] != 0
                && word(&item.0, 2).unwrap_or(0) < word(&item.0, 6).unwrap_or(0)
                && word(&item.0, 4).unwrap_or(0) != 0
        })
        .map(|item| word(&item.0, 4))
        .collect::<Result<_, _>>()?;
    let owned_firearm = word(&warrior.bytes, 0xe4)? == 2;
    let (complete, sounds) = reload(
        warrior,
        &mut state.ammunition[reserve_index],
        owned_firearm,
        &mut state.random,
    )?;
    let work = if let Some(combat) = state.combat.as_mut() {
        &mut combat.weapon_work
    } else {
        &mut state.weapon_work
    };
    if owned_firearm {
        // The reload preflight loads the equipped weapon before the empty-clip return.
        work.load_weapon(&content.items[2])?;
    }
    for ammunition in ammunition_loads {
        work.load_ammunition(
            content
                .items
                .get(ammunition as usize)
                .ok_or_else(|| RuleError::Invalid("reload ammunition definition missing".into()))?,
        )?;
    }
    let mut events: Vec<_> = sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect();
    if owned_firearm && !complete {
        events.push(Event::CollectionRejected { warning: 25 });
    }
    Ok(events)
}
