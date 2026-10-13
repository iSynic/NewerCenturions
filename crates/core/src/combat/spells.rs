use super::*;
use attack::AttackResult;
use damage::{DamageFlags, DamagePacket, resolve_damage};

pub fn load_packet(attacks: &[u8], id: i16) -> Result<Vec<u8>, RuleError> {
    if id < 1101 {
        return Err(RuleError::Invalid("spell descriptor".into()));
    }
    let school = id / 1000 - 1;
    let level = id % 1000 / 100 - 1;
    let index = id % 100 - 1;
    if school < 0 || level < 0 || index < 0 {
        return Err(RuleError::Invalid("spell descriptor fields".into()));
    }
    Ok(centurions_content::span(
        attacks,
        school as usize * 2842 + level as usize * 406 + index as usize * 58,
        58,
    )?
    .to_vec())
}

pub fn select_packet(
    monster: &MonsterRecord,
    attacks: &[u8],
    random: &mut RandomState,
) -> Result<Option<(i16, Vec<u8>)>, RuleError> {
    let mut next_random = random.clone();
    for _ in 0..30 {
        let index = next_random.range(0, 9) as usize;
        let id = word(&monster.bytes, 0x12 + index * 2)?;
        if id > 1100 {
            let packet = load_packet(attacks, id)?;
            *random = next_random;
            return Ok(Some((id, packet)));
        }
    }
    *random = next_random;
    Ok(None)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellTarget {
    pub first: i16,
    pub second: i16,
    pub kind: i16,
}

pub struct SpellSelection {
    pub cast: bool,
    pub caster: MonsterRecord,
    pub count: i16,
    pub range: i16,
    pub nearest: i16,
    pub distances: [i16; 112],
    pub targets: Vec<SpellTarget>,
}

fn position(combat: &Combat, body: usize) -> Option<(i16, i16)> {
    if body < 10 {
        combat.squad.positions.get(body).copied()
    } else {
        combat.enemies.positions.get(body - 10).copied()
    }
}

fn faction(combat: &Combat, warriors: &[Warrior], body: usize) -> Result<i16, RuleError> {
    if body < 10 {
        Ok(word(
            &warriors
                .get(body)
                .ok_or_else(|| RuleError::Invalid("spell warrior body".into()))?
                .bytes,
            0x10,
        )?)
    } else {
        Ok(word(
            &combat
                .enemies
                .records
                .get(body - 10)
                .ok_or_else(|| RuleError::Invalid("spell monster body".into()))?
                .bytes,
            0xe,
        )?)
    }
}

fn best_choice(
    combat: &Combat,
    warriors: &[Warrior],
    areas: &[u8],
    shape: usize,
    side: i16,
    distances: &[i16; 112],
    range: i16,
) -> Result<Option<usize>, RuleError> {
    if shape == 0 || side == 0 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.target",
            dependency: "zero-area or allied spell targeting",
        });
    }
    let pattern = centurions_content::span(areas, (shape - 1) * 49, 49)?;
    let (mut best, mut best_friendly, mut best_hostile) = (None, 0, 0);
    for (candidate, distance) in distances.iter().enumerate().take(110) {
        if *distance > range {
            continue;
        }
        let eligible = if candidate < warriors.len() {
            warriors[candidate].bytes[0x9df] != 0
        } else if candidate >= 10 {
            combat
                .enemies
                .records
                .get(candidate - 10)
                .is_some_and(|m| word(&m.bytes, 0x110).unwrap_or(0) > 0)
        } else {
            false
        };
        if !eligible {
            continue;
        }
        let center = position(combat, candidate)
            .ok_or_else(|| RuleError::Invalid("spell candidate position".into()))?;
        let (mut friendly, mut hostile) = (0, 0);
        let mut seen = [false; 110];
        for y in 0..7 {
            for x in 0..7 {
                if pattern[y * 7 + x] == 0 {
                    continue;
                }
                let ax = combat.map.shift.0 + 5 + center.0 - 3 + x as i16;
                let ay = combat.map.shift.1 + 5 + center.1 - 3 + y as i16;
                if !(0..90).contains(&ax) || !(0..90).contains(&ay) {
                    return Err(RuleError::Unsupported {
                        action: "combat.spell.target",
                        dependency: "spell area outside tactical map",
                    });
                }
                let body = combat.map.tiles[ax as usize * 90 + ay as usize].wrapping_abs();
                if !(0..110).contains(&body) {
                    continue;
                }
                let body = body as usize;
                if body >= 10 && seen[body] {
                    continue;
                }
                seen[body] = true;
                let same = faction(combat, warriors, body)? == side
                    || (body >= 10 && combat.enemies.records[body - 10].bytes[0xb0] == 0xff);
                if same {
                    friendly += 1;
                } else {
                    hostile += 1;
                }
            }
        }
        if hostile >= best_hostile && (friendly <= best_friendly || hostile > best_hostile) {
            best = Some(candidate);
            best_friendly = friendly;
            best_hostile = hostile;
        }
    }
    Ok(best)
}

pub fn select_targets(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    packet: &[u8],
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    if packet.len() == 58 && word(packet, 40)? == 36 {
        if packet[9] != 3 || packet[19] != 8 || packet[20] != 8 || word(packet, 50)? != 0 {
            return Err(RuleError::Unsupported {
                action: "combat.spell.target",
                dependency: "other spell condition packet",
            });
        }
    } else {
        validate_direct(packet)?;
    }
    select_targets_impl(
        combat,
        warriors,
        mission,
        areas,
        caster_index,
        (packet, None),
        random,
    )
}

pub fn select_firearm_targets(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    prepared: &firearm::PreparedFirearm,
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    if prepared.packet.len() != 58 || prepared.packet[19] != 9 {
        return Err(RuleError::Invalid("firearm target packet".into()));
    }
    select_targets_impl(
        combat,
        warriors,
        mission,
        areas,
        caster_index,
        (&prepared.packet, Some(prepared.shots)),
        random,
    )
}

pub fn select_retained_targets(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    prepared: &firearm::PreparedFirearm,
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    if prepared.packet.len() != 58 {
        return Err(RuleError::Invalid("retained target packet".into()));
    }
    let count = (prepared.packet[19] == 9).then_some(prepared.shots);
    select_targets_impl(
        combat,
        warriors,
        mission,
        areas,
        caster_index,
        (&prepared.packet, count),
        random,
    )
}

fn select_targets_impl(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    targeting: (&[u8], Option<i16>),
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    let (packet, firearm_count) = targeting;
    if !matches!(packet[1], 0 | 1 | 3 | 6 | 9 | 10 | 12) || packet[7] != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.target",
            dependency: "other spell targeting or rotated shapes",
        });
    }
    let caster = combat
        .enemies
        .records
        .get(caster_index)
        .ok_or_else(|| RuleError::Invalid("spell caster index".into()))?;
    let start = combat.enemies.positions[caster_index];
    let side = word(&caster.bytes, 0xe)?;
    let mut next_random = random.clone();
    let mut count = next_random.bound(7);
    if let Some(firearm_count) = firearm_count {
        count = firearm_count;
    }
    let mut range = i16::from(packet[3] as i8)
        .wrapping_mul(count)
        .wrapping_add(i16::from(packet[2] as i8))
        .wrapping_abs();
    let size = word(&caster.bytes, 0x10)?.wrapping_abs();
    range = range.wrapping_add(i16::from(size != 0) + i16::from(size == 3));
    let mut result = SpellSelection {
        cast: false,
        caster: caster.clone(),
        count,
        range,
        nearest: 127,
        distances: [127; 112],
        targets: vec![],
    };
    let mut available = 0;
    for body in (0..warriors.len()).chain(10..10 + combat.enemies.records.len()) {
        let eligible = if body < 10 {
            warriors[body].bytes[0x9df] != 0 && word(&warriors[body].bytes, 0x50)? > 0
        } else {
            let m = &combat.enemies.records[body - 10].bytes;
            word(m, 0x110)? > 0 && m[0xb0] != 0xff
        };
        if !eligible || ((faction(combat, warriors, body)? == side) != (packet[9] == 4)) {
            continue;
        }
        let end = position(combat, body)
            .ok_or_else(|| RuleError::Invalid("spell body position".into()))?;
        let dx = f32::from(end.0.wrapping_sub(start.0));
        let dy = f32::from(end.1.wrapping_sub(start.1));
        let distance = (dx * dx + dy * dy).sqrt();
        if distance < f32::from(result.nearest) {
            result.nearest = distance.trunc() as i16;
            result.distances[111] = result.nearest;
        }
        let in_range = if body < 10 {
            distance.trunc() as i16 <= range
        } else {
            distance <= f32::from(range)
        };
        if in_range
            && (i16::from(packet[0] as i8) + i16::from(packet[2] as i8) <= 0
                || lifecycle::can_see(combat, mission, start, end, 0)?)
        {
            result.distances[body] = distance.trunc() as i16;
            available += 1;
        }
    }
    if available == 0 {
        *random = next_random;
        return Ok(result);
    }
    if packet[1] == 0 {
        count = next_random.bound(available);
    }
    count = count.min(7);
    while i16::from(packet[30]).wrapping_mul(count) > word(&caster.bytes, 0x10e)? {
        count -= 1;
        if count == 0 {
            result.count = 0;
            *random = next_random;
            return Ok(result);
        }
    }
    result.count = count;
    let desired = if packet[1] > 0 && packet[19] != 9 {
        1
    } else {
        count
    };
    if packet[1] > 1 && result.nearest < range {
        if let Some(body) = best_choice(
            combat,
            warriors,
            areas,
            usize::from(packet[31]),
            side,
            &result.distances,
            range,
        )? {
            let target = position(combat, body).unwrap();
            result.targets.push(SpellTarget {
                first: target.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
                second: target.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
                kind: 3,
            });
        }
    } else if packet[1] < 2 && result.nearest <= range {
        for _ in 0..desired {
            let mut selected = None;
            for _ in 0..100 {
                let body =
                    (next_random.bound(combat.enemies.records.len() as i16 + 10) - 1) as usize;
                if result.distances[body] <= range {
                    selected = Some(body);
                    break;
                }
            }
            let Some(body) = selected else { break };
            result.targets.push(SpellTarget {
                first: body as i16,
                second: -1,
                kind: if body < 9 { 1 } else { 2 },
            });
        }
    }
    if !result.targets.is_empty()
        || (firearm_count == Some(0) && packet[1] < 2 && result.nearest <= range)
    {
        result.cast = true;
        result.caster.bytes[0x109] = result.caster.bytes[0x109].wrapping_sub(2);
        result.caster.bytes[0x106] = result.caster.bytes[0x106].wrapping_sub(3);
        result.caster.bytes[0x104] = u8::from((result.caster.bytes[0x109] as i8) >= 1);
        let psi = word(&result.caster.bytes, 0x10e)?
            .wrapping_sub(i16::from(packet[30]).wrapping_mul(count));
        put_word(&mut result.caster.bytes, 0x10e, psi);
        if firearm_count.is_some() {
            result.count = 1;
        }
    }
    *random = next_random;
    Ok(result)
}

fn validate_direct(packet: &[u8]) -> Result<(), RuleError> {
    if packet.len() != 58 {
        return Err(RuleError::Invalid("spell packet size".into()));
    }
    let compress = packet[1] == 0
        && packet[4] == 8
        && packet[9] == 3
        && packet[11..17] == [0, 0, 1, 3, 0, 0]
        && packet[19] == 8
        && packet[20] == 8
        && word(packet, 32)? == 1
        && word(packet, 34)? == 8
        && packet[42] == 1
        && word(packet, 50)? == 0;
    let poison = packet[..11] == [0, 1, 6, 0, 0, 0, 0, 0, 0, 0, 0]
        && packet[11..23] == [0, 0, 0, 0, 1, 3, 5, 10, 1, 1, 1, 0]
        && packet[30..36] == [5, 0, 0, 0, 0, 0]
        && word(packet, 36)? == 431
        && word(packet, 38)? == 430
        && word(packet, 40)? == 4
        && packet[42..50].iter().all(|value| *value == 0)
        && word(packet, 50)? == 0x2000
        && packet[52..].iter().all(|value| *value == 0);
    if (word(packet, 40)? != 0 && !poison)
        || (!(0..=6).contains(&packet[19]) && !compress)
        || packet[20] as i8 <= 0
        || packet[23..30].iter().any(|value| *value != 0)
        || !matches!(word(packet, 50)?, 0 | 0x2000)
    {
        return Err(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "spell effects, other race restrictions or elemental damage",
        });
    }
    Ok(())
}

fn race_is_excluded(
    packet: &[u8],
    warrior: &[u8],
    action: &'static str,
) -> Result<bool, RuleError> {
    let race_mask = word(packet, 50)? as u16;
    if race_mask == 0 {
        return Ok(false);
    }
    let race = word(warrior, 0x32)?;
    if !(1..=16).contains(&race) {
        return Err(RuleError::Unsupported {
            action,
            dependency: "racial mask beyond recovered party races",
        });
    }
    Ok(race_mask & (0x8000_u16 >> (race - 1)) != 0)
}

fn restriction_bit(packet: &[u8], index: i16) -> Result<bool, RuleError> {
    let offset = 50 + (i32::from(index) >> 3);
    let byte = usize::try_from(offset)
        .ok()
        .and_then(|offset| packet.get(offset))
        .ok_or(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "racial bit test outside the loaded spell packet",
        })?;
    Ok(byte & (0x80 >> (index as u16 & 7)) != 0)
}

#[derive(Clone, Copy)]
pub struct RestrictionMemoryView<'a> {
    item_cache: &'a [u8],
    ordinary_backup: bool,
    editor_backup: Option<&'a [u8]>,
    item_work_alias: Option<i16>,
}

impl<'a> RestrictionMemoryView<'a> {
    pub fn item_cache_only(item_cache: &'a [u8]) -> Self {
        Self {
            item_cache,
            ordinary_backup: false,
            editor_backup: None,
            item_work_alias: None,
        }
    }

    /// Supported ordinary history preserves the original startup editor backup.
    /// item_work_alias is WORD at offset 0x32, distinct from field32 at 0x20.
    pub fn ordinary_startup(item_cache: &'a [u8], item_work_alias: Option<i16>) -> Self {
        Self {
            item_cache,
            ordinary_backup: true,
            editor_backup: None,
            item_work_alias,
        }
    }

    pub fn with_editor_backup(
        item_cache: &'a [u8],
        item_work_alias: Option<i16>,
        backup: &'a [u8],
    ) -> Result<Self, RuleError> {
        if backup.len() != 18_828 {
            return Err(RuleError::Invalid("editor backup record size".into()));
        }
        Ok(Self {
            item_cache,
            ordinary_backup: false,
            editor_backup: Some(backup),
            item_work_alias,
        })
    }

    fn race_word(self, body: usize) -> Result<i16, RuleError> {
        let address = body
            .checked_mul(3138)
            .and_then(|stride| 0x448aa0_usize.checked_add(stride))
            .and_then(|address| address.checked_add(0x32))
            .ok_or_else(|| RuleError::Invalid("restriction alias address overflow".into()))?;
        if self.ordinary_backup && (0x44ecb0..0x45363b).contains(&address) {
            return Ok(0);
        }
        if let Some(backup) = self.editor_backup
            && let Some(offset) = address
                .checked_sub(0x44ecb0)
                .filter(|offset| *offset <= backup.len() - 2)
        {
            return Ok(word(backup, offset)?);
        }
        if address == 0x45366e
            && let Some(alias) = self.item_work_alias
        {
            return Ok(alias);
        }
        if self.item_cache.len() == 30_000
            && let Some(offset) = address
                .checked_sub(0x4537fe)
                .filter(|offset| *offset <= self.item_cache.len() - 2)
        {
            return Ok(word(self.item_cache, offset)?);
        }
        Err(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "restricted monster race alias outside the represented memory owners",
        })
    }
}

fn monster_race_is_excluded(
    packet: &[u8],
    body: usize,
    memory: Option<RestrictionMemoryView<'_>>,
) -> Result<bool, RuleError> {
    let memory = memory.ok_or(RuleError::Unsupported {
        action: "combat.spell",
        dependency: "restricted monster target uses unrepresented party memory",
    })?;
    restriction_bit(packet, memory.race_word(body)?.wrapping_sub(1))
}

pub struct WarriorSpellResult {
    pub warriors: Vec<Warrior>,
    pub targets: Vec<(usize, AttackResult)>,
    pub race_excluded: Vec<usize>,
}

pub struct BodySpellResult {
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub sounds: Vec<i16>,
    pub targets: Vec<(usize, AttackResult)>,
    pub race_excluded: Vec<usize>,
}

pub struct SupportSpellResult {
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub sounds: Vec<i16>,
    pub targets: Vec<(usize, AttackResult)>,
    pub healed: bool,
    pub recovered: u8,
}

pub fn resolve_player_support(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    packet: &[u8],
    id: i16,
    count: i16,
    random: &mut RandomState,
) -> Result<SupportSpellResult, RuleError> {
    if !(1..=6).contains(&warriors.len())
        || enemies.len() > 100
        || mask.len() < 10 + enemies.len()
        || packet.len() != 58
        || !matches!(count, 1..=4)
    {
        return Err(RuleError::Invalid("support spell inputs".into()));
    }
    let effect = word(packet, 40)?;
    if !matches!((id, effect), (1101, 57) | (1305, 13) | (1401, 57))
        || packet[9] != 3
        || packet[19] != 8
        || packet[20] != 8
        || word(packet, 34)? != 0
        || packet[23..30].iter().any(|v| *v != 0)
    {
        return Err(RuleError::Unsupported {
            action: "combat.spell.support",
            dependency: "other support packet",
        });
    }
    if count > 2 {
        let target = mask[..warriors.len()].iter().position(|value| *value > 0);
        if id != 1101
            || mask.iter().filter(|value| **value > 0).count() != 1
            || target.is_none()
            || !(-9..=0).contains(&word(&warriors[target.unwrap()].bytes, 0x50)?)
        {
            return Err(RuleError::Unsupported {
                action: "combat.spell.support",
                dependency: "other support power or target",
            });
        }
    }
    if id == 1401
        && (count != 1
            || packet[..11] != [1, 1, 4, 0, 0, 0, 0, 0, 0, 3, 0]
            || packet[11..19] != [8, 16, 0, 0, 0, 0, 15, 6]
            || packet[21..23] != [1, 1]
            || packet[30..36] != [10, 1, 0, 0, 0, 0]
            || word(packet, 36)? != 431
            || word(packet, 38)? != 430
            || packet[42..50].iter().any(|value| *value != 0)
            || word(packet, 50)? != 0x2000
            || packet[52..].iter().any(|value| *value != 0))
    {
        return Err(RuleError::Unsupported {
            action: "combat.spell.support",
            dependency: "other medical support packet or count",
        });
    }
    if mask[warriors.len()..].iter().any(|value| *value > 0) {
        return Err(RuleError::Unsupported {
            action: "combat.spell.support",
            dependency: "other support body",
        });
    }
    for (member, warrior) in warriors.iter().enumerate() {
        centurions_content::span(&warrior.bytes, 0, 3138)?;
        if mask[member] > 0 && word(&warrior.bytes, 0x9a)? != 0 {
            return Err(RuleError::Unsupported {
                action: "combat.spell.support",
                dependency: "support spell reflection",
            });
        }
    }
    let mut next_random = random.clone();
    let duration = next_random.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
    let mut result = SupportSpellResult {
        warriors: warriors.to_vec(),
        enemies: enemies.to_vec(),
        sounds: vec![],
        targets: vec![],
        healed: false,
        recovered: 0,
    };
    for (member, warrior) in result.warriors.iter_mut().enumerate() {
        if mask[member] <= 0 {
            continue;
        }
        let w = &mut warrior.bytes;
        if race_is_excluded(packet, w, "combat.spell.support")? {
            result.sounds.push(651);
            continue;
        }
        let damage = resolve_damage(
            w,
            false,
            &DamagePacket {
                bytes: packet.to_vec(),
                attack_count: count,
            },
            DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter: false,
            },
            &mut next_random,
        )?
        .total;
        let mut sounds = vec![];
        if id == 1305 {
            spell_conditions::apply(w, false, 13, duration, &mut next_random)?;
            sounds.push(word(packet, 38)?);
        } else {
            let previous = word(w, 0x50)?;
            if damage < 0 && previous > -10 {
                let after = previous
                    .wrapping_add(damage.wrapping_abs())
                    .max(-10)
                    .min(word(w, 0x52)?);
                if previous > 0 && after < 1 {
                    return Err(RuleError::Unsupported {
                        action: "combat.spell.support",
                        dependency: "healing overflow death",
                    });
                }
                if previous <= 0 && after > 0 {
                    result.recovered = result.recovered.wrapping_add(1);
                }
                if previous <= 0 && after < -9 {
                    sounds.push(next_random.bound(2).wrapping_add(506));
                }
                put_word(w, 0x50, after);
                sounds.insert(0, word(packet, 38)?);
                result.healed = true;
            }
        }
        result.targets.push((
            member,
            AttackResult {
                damage,
                killed: false,
                sounds,
            },
        ));
    }
    *random = next_random;
    Ok(result)
}

pub struct DirectSpell<'a> {
    pub packet: &'a [u8],
    pub level: i16,
    pub count: i16,
    pub armor_filter: bool,
    pub weapon_sound: i16,
}

pub struct FieldUsage {
    pub slot: usize,
    pub used: Vec<u8>,
}

pub struct SpellContext<'a> {
    pub packet: &'a [u8],
    pub id: i16,
    pub count: i16,
    pub caster: i16,
    pub initiative: i16,
    pub collision: bool,
}

pub struct SpellMask {
    pub mask: [i16; 110],
    pub fields: Vec<u8>,
}

pub fn tick_fields(fields: &[u8], cursor: i16) -> Result<Vec<u8>, RuleError> {
    if fields.len() != 1200 {
        return Err(RuleError::Invalid("spell field table size".into()));
    }
    let mut result = fields.to_vec();
    for field in result.as_chunks_mut::<20>().0 {
        if field[15] != 0 && i16::from(field[18] as i8) == cursor {
            field[15] = field[15].wrapping_sub(1);
        }
    }
    Ok(result)
}

pub fn reset_used(used: &[u8]) -> Result<Vec<u8>, RuleError> {
    if used.len() != 128 {
        return Err(RuleError::Invalid("spell field usage size".into()));
    }
    let mut result = used.to_vec();
    result[..60].fill(0);
    Ok(result)
}

pub struct FieldCollision {
    pub packet: Vec<u8>,
    pub id: i16,
    pub count: i16,
    pub level: i16,
    pub target: SpellTarget,
}

pub fn collision_field(
    combat: &Combat,
    warriors: &[Warrior],
    fields: &[u8],
    usage: &FieldUsage,
    body: usize,
    attacks: &[u8],
) -> Result<Option<FieldCollision>, RuleError> {
    if fields.len() != 1200 || usage.used.len() != 128 || usage.slot >= 60 || body >= 110 {
        return Err(RuleError::Invalid("spell collision inputs".into()));
    }
    if body < 10
        && warriors
            .get(body)
            .ok_or_else(|| RuleError::Invalid("spell collision warrior".into()))?
            .bytes[0x9df]
            == 0
    {
        return Ok(None);
    }
    let field = &fields[usage.slot * 20..usage.slot * 20 + 20];
    if field[15] == 0 || usage.used[usage.slot] != 0 {
        return Ok(None);
    }
    let point = position(combat, body)
        .ok_or_else(|| RuleError::Invalid("spell collision body position".into()))?;
    let point = (
        point.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
        point.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
    );
    let size = if body < 10 {
        0
    } else {
        word(
            &combat
                .enemies
                .records
                .get(body - 10)
                .ok_or_else(|| RuleError::Invalid("spell collision monster".into()))?
                .bytes,
            0x10,
        )?
        .wrapping_abs()
    };
    if !(0..=3).contains(&size) {
        return Err(RuleError::Unsupported {
            action: "combat.spell.collision",
            dependency: "other monster footprint",
        });
    }
    let inside = |p: (i16, i16)| -> Result<bool, RuleError> {
        Ok(p.1 >= word(field, 0)?
            && p.1 < word(field, 4)?
            && p.0 >= word(field, 2)?
            && p.0 < word(field, 6)?)
    };
    if !inside(point)?
        && !((size == 1 || size == 3) && inside((point.0, point.1.wrapping_sub(1)))?)
        && !((size == 2 || size == 3) && inside((point.0.wrapping_sub(1), point.1))?)
        && !(size == 3 && inside((point.0.wrapping_sub(1), point.1.wrapping_sub(1)))?)
    {
        return Ok(None);
    }
    if field[17] != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.collision",
            dependency: "rotated spell fields",
        });
    }
    let id = word(field, 8)?;
    let mut packet = load_packet(attacks, id)?;
    if packet[19] == 9 {
        packet[19] = 1;
        packet[20] = 1;
    }
    packet[4] = 0;
    Ok(Some(FieldCollision {
        packet,
        id,
        count: i16::from(field[16] as i8),
        level: i16::from(field[19] as i8),
        target: SpellTarget {
            first: i16::from(field[12] as i8).wrapping_add(3),
            second: i16::from(field[13] as i8).wrapping_add(3),
            kind: 3,
        },
    }))
}

pub struct GlobalSpellTargets {
    pub caster: MonsterRecord,
    pub mask: [i16; 110],
}

pub fn global_targets(
    combat: &Combat,
    warriors: &[Warrior],
    caster_index: usize,
    packet: &[u8],
) -> Result<GlobalSpellTargets, RuleError> {
    if packet.len() != 58
        || !matches!(packet[1], 9 | 10 | 12)
        || warriors.len() > 6
        || combat.enemies.records.len() > 100
    {
        return Err(RuleError::Invalid("global spell targets".into()));
    }
    let caster = combat
        .enemies
        .records
        .get(caster_index)
        .ok_or_else(|| RuleError::Invalid("global spell caster".into()))?;
    let side = word(&caster.bytes, 0xe)?;
    let mut result = GlobalSpellTargets {
        caster: caster.clone(),
        mask: [0; 110],
    };
    for (member, warrior) in warriors.iter().enumerate() {
        if warrior.bytes[0x9df] != 0
            && (packet[1] == 12 || (word(&warrior.bytes, 0x10)? == side) == (packet[1] == 9))
        {
            result.mask[member] = 1;
        }
    }
    for (index, enemy) in combat.enemies.records.iter().enumerate() {
        if word(&enemy.bytes, 0x110)? > 0
            && (packet[1] == 12 || (word(&enemy.bytes, 0xe)? == side) == (packet[1] == 9))
        {
            result.mask[index + 10] = 1;
        }
    }
    if packet[1] != 12 {
        result.caster.bytes[0x104] = 0;
    }
    Ok(result)
}

pub fn target_mask(
    combat: &Combat,
    warriors: &[Warrior],
    areas: &[u8],
    target: &SpellTarget,
    context: &SpellContext<'_>,
    fields: &[u8],
    random: &mut RandomState,
) -> Result<SpellMask, RuleError> {
    target_mask_with_origin(
        combat,
        warriors,
        areas,
        (target, target),
        context,
        fields,
        random,
    )
}

pub fn target_mask_with_origin(
    combat: &Combat,
    warriors: &[Warrior],
    areas: &[u8],
    targets: (&SpellTarget, &SpellTarget),
    context: &SpellContext<'_>,
    fields: &[u8],
    random: &mut RandomState,
) -> Result<SpellMask, RuleError> {
    let (target, origin) = targets;
    let p = context.packet;
    if p.len() != 58 || fields.len() != 1200 || !(0..110).contains(&context.caster) {
        return Err(RuleError::Invalid("spell mask inputs".into()));
    }
    if p[7] != 0 || word(p, 40)? == 58 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.mask",
            dependency: "rotated shapes or summon targeting",
        });
    }
    let shape = match p[1] {
        3 => usize::from(p[31]),
        4 => context.count as usize,
        _ => 1,
    };
    if shape == 0 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.mask",
            dependency: "zero-area spell pattern",
        });
    }
    let pattern = centurions_content::span(areas, (shape - 1) * 49, 49)?;
    let mut next_random = random.clone();
    let mut result = SpellMask {
        mask: [0; 110],
        fields: fields.to_vec(),
    };
    if p[4] != 0
        && !context.collision
        && let Some(slot) = fields.as_chunks::<20>().0.iter().position(|f| f[15] == 0)
    {
        let f = &mut result.fields[slot * 20..slot * 20 + 20];
        f[18] = context.initiative as u8;
        f[16] = context.count as u8;
        f[19] = (context.id % 1000 / 100 - 1) as u8;
        f[17] = 0;
        f[14] = shape as u8;
        f[12] = origin.first.wrapping_sub(3) as u8;
        f[13] = origin.second.wrapping_sub(3) as u8;
        let left = i16::from(f[12] as i8);
        let top = i16::from(f[13] as i8);
        put_word(f, 0, top);
        put_word(f, 2, left);
        put_word(f, 4, top.wrapping_add(7));
        put_word(f, 6, left.wrapping_add(7));
        put_word(f, 8, context.id);
        let mut duration = next_random.range(i16::from(p[13] as i8), i16::from(p[14] as i8)) as u8;
        for _ in 0..context.count {
            duration = duration.wrapping_add(
                next_random.range(i16::from(p[15] as i8), i16::from(p[16] as i8)) as u8,
            );
        }
        f[15] = duration;
        put_word(f, 10, i16::from(p[4] as i8));
    }
    let center = if p[1] >= 2 && target.kind == 3 {
        (target.first, target.second)
    } else {
        let body = usize::try_from(target.first)
            .map_err(|_| RuleError::Invalid("negative spell target".into()))?;
        let point = position(combat, body)
            .ok_or_else(|| RuleError::Invalid("spell mask target body".into()))?;
        (
            point.0 + combat.map.shift.0 + 5,
            point.1 + combat.map.shift.1 + 5,
        )
    };
    for y in 0..7 {
        for x in 0..7 {
            if pattern[y * 7 + x] == 0 {
                continue;
            }
            let ax = center.0 - 3 + x as i16;
            let ay = center.1 - 3 + y as i16;
            if !(0..90).contains(&ax) || !(0..90).contains(&ay) {
                return Err(RuleError::Unsupported {
                    action: "combat.spell.mask",
                    dependency: "spell area outside tactical map",
                });
            }
            let body = combat.map.tiles[ax as usize * 90 + ay as usize].wrapping_abs();
            if !(0..110).contains(&body) {
                continue;
            }
            let body = body as usize;
            if context.collision && body != context.caster as usize {
                continue;
            }
            let reflected = if p[19] != 9 {
                let reflective = if body < 9 {
                    word(
                        &warriors
                            .get(body)
                            .ok_or_else(|| RuleError::Invalid("spell reflection warrior".into()))?
                            .bytes,
                        0x9a,
                    )? != 0
                } else {
                    combat
                        .enemies
                        .records
                        .get(body.wrapping_sub(10))
                        .ok_or_else(|| RuleError::Invalid("spell reflection monster".into()))?
                        .bytes[0xd0]
                        != 0
                };
                if word(p, 40)? == 4 && reflective {
                    return Err(RuleError::Unsupported {
                        action: "combat.spell",
                        dependency: "poison ward reflection",
                    });
                }
                reflective && next_random.bound(100) < 34 && !context.collision
            } else {
                false
            };
            if reflected {
                result.mask[context.caster as usize] = 1;
            } else if body < 10
                || word(
                    &combat
                        .enemies
                        .records
                        .get(body - 10)
                        .ok_or_else(|| RuleError::Invalid("spell mask monster".into()))?
                        .bytes,
                    0xa,
                )? < 101
            {
                result.mask[body] = 1;
            } else {
                result.mask[body] = 0;
            }
        }
    }
    *random = next_random;
    Ok(result)
}

pub fn resolve_warriors(
    warriors: &[Warrior],
    packet: &[u8],
    level: i16,
    count: i16,
    mask: &[i16],
    armor_filter: bool,
    random: &mut RandomState,
) -> Result<WarriorSpellResult, RuleError> {
    resolve_party(
        warriors,
        mask,
        &DirectSpell {
            packet,
            level,
            count,
            armor_filter,
            weapon_sound: 0,
        },
        None,
        random,
    )
    .map(|(party, _)| party)
}

fn resolve_party(
    warriors: &[Warrior],
    mask: &[i16],
    context: &DirectSpell<'_>,
    mut usage: Option<&mut FieldUsage>,
    random: &mut RandomState,
) -> Result<(WarriorSpellResult, i16), RuleError> {
    let DirectSpell {
        packet,
        level,
        count,
        armor_filter,
        ..
    } = *context;
    validate_direct(packet)?;
    if mask.len() < warriors.len() || !(0..7).contains(&level) || count < 0 {
        return Err(RuleError::Invalid("spell resolution inputs".into()));
    }
    let poison = word(packet, 40)? == 4;
    if poison
        && (level != 1
            || !(1..=7).contains(&count)
            || armor_filter
            || usage.is_some()
            || warriors.iter().enumerate().any(|(member, warrior)| {
                mask[member] > 0 && word(&warrior.bytes, 0x9a).unwrap_or(1) != 0
            }))
    {
        return Err(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "other poison count, defense or field collision",
        });
    }
    let mut next_random = random.clone();
    let mut result = WarriorSpellResult {
        warriors: warriors.to_vec(),
        targets: vec![],
        race_excluded: vec![],
    };
    let mut duration = next_random.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
    if packet[15] != 0 {
        for _ in 0..count {
            duration = duration.wrapping_add(
                next_random.range(i16::from(packet[15] as i8), i16::from(packet[16] as i8)),
            );
        }
    }
    for (member, warrior) in result.warriors.iter_mut().enumerate() {
        if mask[member] <= 0 {
            continue;
        }
        let w = &mut warrior.bytes;
        if race_is_excluded(packet, w, "combat.spell")? {
            result.race_excluded.push(member);
            continue;
        }
        if !armor_filter && matches!(packet[9], 0 | 2) {
            if packet[19] == 0 {
                let threshold = word(w, 0x112)?
                    .wrapping_add(if word(w, 0x32)? == 3 { 1000 } else { 0 })
                    .wrapping_add(i16::from(packet[8] as i8).wrapping_mul(count));
                if next_random.bound(100) <= threshold {
                    continue;
                }
            }
            if matches!(packet[19], 0 | 5) && word(w, 0x90)? != 0 {
                continue;
            }
            let immune = (level..7).any(|school_level| {
                word(w, 0x5e + (school_level as usize + 16) * 2).unwrap_or(0) != 0
            });
            let threshold =
                word(w, 0x22)?.wrapping_add(i16::from(packet[10] as i8).wrapping_mul(count));
            if next_random.bound(100) > threshold && immune {
                continue;
            }
        }
        if let Some(usage) = usage.as_deref_mut() {
            if usage.used[usage.slot + 1] != 0 {
                continue;
            }
            usage.used[usage.slot + 1] = 1;
        }
        let damage = resolve_damage(
            w,
            false,
            &DamagePacket {
                bytes: packet.to_vec(),
                attack_count: count,
            },
            DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter,
            },
            &mut next_random,
        )?;
        let mut sounds = vec![];
        if damage.shield_sound {
            sounds.push(206);
        }
        if poison {
            spell_conditions::apply(w, false, 4, duration, &mut next_random)?;
            sounds.push(word(packet, 38)?);
        }
        let previous = word(w, 0x50)?;
        if damage.total != 0 && previous > -10 {
            sounds.push(word(packet, 38)?);
            if damage.total < 0 {
                return Err(RuleError::Unsupported {
                    action: "combat.spell",
                    dependency: "spell healing",
                });
            }
            if previous >= 0 {
                put_word(w, 0x50, previous.wrapping_sub(damage.total));
                if damage.total > 0 {
                    put_word(w, 0x14, 1);
                }
            }
        }
        result.targets.push((
            member,
            AttackResult {
                damage: damage.total,
                killed: previous > 0 && word(w, 0x50)? < 1,
                sounds,
            },
        ));
    }
    *random = next_random;
    Ok((result, duration))
}

pub fn resolve_bodies(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_bodies_inner(warriors, enemies, mask, context, None, None, random)
}

pub fn resolve_bodies_with_item_cache(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    item_cache: &[u8],
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_bodies_inner(
        warriors,
        enemies,
        mask,
        context,
        None,
        Some(RestrictionMemoryView::item_cache_only(item_cache)),
        random,
    )
}

pub fn resolve_bodies_with_memory(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    memory: RestrictionMemoryView<'_>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_bodies_inner(warriors, enemies, mask, context, None, Some(memory), random)
}

pub fn resolve_collision(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: &mut FieldUsage,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_collision_inner(warriors, enemies, mask, context, usage, None, random)
}

pub fn resolve_collision_with_item_cache(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: &mut FieldUsage,
    item_cache: &[u8],
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_collision_inner(
        warriors,
        enemies,
        mask,
        context,
        usage,
        Some(RestrictionMemoryView::item_cache_only(item_cache)),
        random,
    )
}

fn resolve_collision_inner(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: &mut FieldUsage,
    memory: Option<RestrictionMemoryView<'_>>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    if usage.slot >= 60 || usage.used.len() != 128 {
        return Err(RuleError::Invalid("spell field collision usage".into()));
    }
    let mut next_usage = FieldUsage {
        slot: usage.slot,
        used: usage.used.clone(),
    };
    let result = resolve_bodies_inner(
        warriors,
        enemies,
        mask,
        context,
        Some(&mut next_usage),
        memory,
        random,
    )?;
    *usage = next_usage;
    Ok(result)
}

fn resolve_bodies_inner(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    mut usage: Option<&mut FieldUsage>,
    memory: Option<RestrictionMemoryView<'_>>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    let DirectSpell {
        packet,
        level,
        count,
        armor_filter,
        weapon_sound,
    } = *context;
    if warriors.len() > 6 || enemies.len() > 100 || mask.len() < 10 + enemies.len() {
        return Err(RuleError::Invalid("spell body inputs".into()));
    }
    if word(packet, 40)? == 36 {
        if usage.is_some() {
            return Err(RuleError::Unsupported {
                action: "combat.spell.collision",
                dependency: "lingering condition effects",
            });
        }
        let outcome =
            spell_conditions::resolve_1703(warriors, enemies, packet, count, mask, random)?;
        let targets = (0..warriors.len())
            .chain(10..10 + enemies.len())
            .filter(|body| mask[*body] > 0)
            .map(|body| {
                (
                    body,
                    AttackResult {
                        damage: 0,
                        killed: false,
                        sounds: vec![word(packet, 38).unwrap_or(0)],
                    },
                )
            })
            .collect();
        return Ok(BodySpellResult {
            warriors: outcome.warriors,
            enemies: outcome.enemies,
            sounds: vec![],
            targets,
            race_excluded: vec![],
        });
    }
    validate_direct(packet)?;
    let poison = word(packet, 40)? == 4;
    if poison && mask[6..10].iter().any(|flag| *flag > 0) {
        return Err(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "poison target outside the represented body records",
        });
    }
    let mut excluded_monsters = vec![false; enemies.len()];
    if word(packet, 50)? != 0 {
        for (body, flag) in mask.iter().enumerate().skip(10) {
            if *flag <= 0 {
                continue;
            }
            let excluded = excluded_monsters
                .get_mut(body - 10)
                .ok_or(RuleError::Unsupported {
                    action: "combat.spell",
                    dependency: "restricted monster target outside the current records",
                })?;
            *excluded = monster_race_is_excluded(packet, body, memory)?;
        }
    }
    let mut next_random = random.clone();
    let (party, duration) = resolve_party(
        warriors,
        mask,
        context,
        usage.as_deref_mut(),
        &mut next_random,
    )?;
    let mut result = BodySpellResult {
        warriors: party.warriors,
        enemies: enemies.to_vec(),
        sounds: if matches!(packet[1], 9 | 10) {
            vec![6001]
        } else {
            vec![]
        },
        targets: party.targets,
        race_excluded: party.race_excluded,
    };
    for (index, enemy) in result.enemies.iter_mut().enumerate() {
        if mask[index + 10] <= 0 {
            continue;
        }
        let m = &mut enemy.bytes;
        centurions_content::span(m, 0, 288)?;
        if excluded_monsters[index] {
            result.race_excluded.push(index + 10);
            continue;
        }
        if !armor_filter {
            if packet[19] == 0 {
                let threshold = word(m, 0)?
                    .wrapping_mul(4)
                    .wrapping_add(if m[0xab] != 0 { 1035 } else { 35 })
                    .wrapping_add(i16::from(packet[8] as i8).wrapping_mul(count));
                if next_random.bound(100) <= threshold {
                    continue;
                }
            }
            // Categories above six skip both indexed and general NPC immunity checks.
            if packet[19] <= 6
                && (m[0x8b + usize::from(packet[19])] as i8 >= 101 || word(m, 0xa)? >= 101)
            {
                continue;
            }
            if matches!(packet[9], 0 | 2) {
                if matches!(packet[19], 0 | 5) && m[0xcb] != 0 {
                    continue;
                }
                let immune =
                    (level..7).any(|school_level| m[0xb2 + school_level as usize + 16] != 0);
                let threshold =
                    word(m, 0xa)?.wrapping_add(i16::from(packet[10] as i8).wrapping_mul(count));
                if next_random.bound(100) <= threshold || immune {
                    continue;
                }
            }
        }
        if let Some(usage) = usage.as_deref_mut() {
            if usage.used[usage.slot + 1] != 0 {
                continue;
            }
            usage.used[usage.slot + 1] = 1;
        }
        let damage = resolve_damage(
            m,
            true,
            &DamagePacket {
                bytes: packet.to_vec(),
                attack_count: count,
            },
            DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter,
            },
            &mut next_random,
        )?;
        let mut sounds = vec![];
        if damage.shield_sound {
            sounds.push(206);
        }
        if poison {
            spell_conditions::apply(m, true, 4, duration, &mut next_random)?;
            sounds.push(word(packet, 38)?);
        }
        let previous = word(m, 0x110)?;
        if damage.total != 0 {
            if damage.total < 0 {
                return Err(RuleError::Unsupported {
                    action: "combat.spell",
                    dependency: "spell healing",
                });
            }
            sounds.push(word(packet, 38)?);
            m[0x105] = 1;
            put_word(m, 0x110, previous.wrapping_sub(damage.total));
            if word(m, 0x110)? > 0 {
                sounds.push(weapon_sound);
            }
        }
        result.targets.push((
            index + 10,
            AttackResult {
                damage: damage.total,
                killed: word(m, 0x110)? < 1 && damage.total != 0,
                sounds,
            },
        ));
    }
    *random = next_random;
    Ok(result)
}

pub fn resolve_enemy_firearm_body(
    target: &mut MonsterRecord,
    caster: &MonsterRecord,
    prepared: &firearm::PreparedFirearm,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<AttackResult, RuleError> {
    let p = &prepared.packet;
    centurions_content::span(p, 0, 58)?;
    let mut next_random = random.clone();
    next_random.range(i16::from(p[13] as i8), i16::from(p[14] as i8));
    let result = resolve_enemy_firearm_body_after_duration(
        target,
        caster,
        prepared,
        difficulty,
        &mut next_random,
    )?;
    *random = next_random;
    Ok(result)
}

pub fn resolve_enemy_firearm_body_after_duration(
    target: &mut MonsterRecord,
    caster: &MonsterRecord,
    prepared: &firearm::PreparedFirearm,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<AttackResult, RuleError> {
    let p = &prepared.packet;
    centurions_content::span(&target.bytes, 0, 288)?;
    centurions_content::span(&caster.bytes, 0, 288)?;
    if p.len() != 58 || p[19] != 9 || word(p, 40)? != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "other ranged body effects",
        });
    }
    let mut next_random = random.clone();
    let mut next = target.clone();
    let m = &mut next.bytes;
    if !prepared.armor_filter && m[0xb3] == 0 {
        let mut hit = 33_i16
            .wrapping_add(word(&caster.bytes, 0)?.wrapping_mul(5))
            .wrapping_add(word(&caster.bytes, 0x66)?)
            .wrapping_sub(word(m, 0x62)?)
            .wrapping_sub(next_random.bound(word(m, 4)? / 3))
            .wrapping_sub(word(m, 0)?.wrapping_mul(3))
            .wrapping_sub(i16::from(m[0x104] as i8).wrapping_mul(word(m, 0)?));
        hit = hit.wrapping_add(match word(m, 0x10)?.wrapping_abs() {
            0 => -5,
            1 | 2 => 5,
            3 => 10,
            _ => 0,
        });
        let minimum = 15_i16.wrapping_sub(difficulty.wrapping_mul(5));
        let maximum = 90_i16.wrapping_sub(difficulty.wrapping_mul(5));
        if next_random.bound(100) > hit.clamp(minimum, maximum) {
            *random = next_random;
            return Ok(AttackResult {
                damage: 0,
                killed: false,
                sounds: vec![],
            });
        }
    }
    let damage = resolve_damage(
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
            armor_filter: prepared.armor_filter,
        },
        &mut next_random,
    )?;
    let mut sounds = vec![];
    if damage.shield_sound {
        sounds.push(206);
    }
    if damage.total != 0 {
        let health = word(m, 0x110)?.wrapping_sub(damage.total);
        put_word(m, 0x110, health);
        m[0x105] = 1;
        if word(m, 0x110)? > 0 {
            sounds.push(prepared.sound);
        }
    }
    let result = AttackResult {
        damage: damage.total,
        killed: damage.total != 0 && word(m, 0x110)? < 1,
        sounds,
    };
    *target = next;
    *random = next_random;
    Ok(result)
}
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../../tests/unit/spells_restriction_tests.rs"]
mod restriction_tests;
