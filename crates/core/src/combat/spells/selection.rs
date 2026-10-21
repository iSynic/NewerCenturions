//! Selection for combat/spells.
use super::{Combat, RandomState, RuleError, firearm, lifecycle, validate_direct};
use centurions_content::combat::MonsterRecord;
use centurions_content::{
    Warrior,
    records::{AttackPacket, EnemyRecord},
};

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
    pub rotation: u8,
}

pub(super) fn position(combat: &Combat, body: usize) -> Option<(i16, i16)> {
    if body < 10 {
        combat.squad.positions.get(body).copied()
    } else {
        combat.enemies.positions.get(body - 10).copied()
    }
}

pub(super) fn faction(
    combat: &Combat,
    warriors: &[Warrior],
    body: usize,
) -> Result<i16, RuleError> {
    if body < 10 {
        Ok(warriors
            .get(body)
            .ok_or_else(|| RuleError::Invalid("spell warrior body".into()))?
            .record()
            .faction()?)
    } else {
        Ok(combat
            .enemies
            .records
            .get(body - 10)
            .ok_or_else(|| RuleError::Invalid("spell monster body".into()))?
            .record()
            .faction()?)
    }
}

pub(super) fn best_choice(
    combat: &Combat,
    warriors: &[Warrior],
    areas: &[u8],
    shape: usize,
    side: i16,
    distances: &[i16; 112],
    range: i16,
) -> Result<Option<usize>, RuleError> {
    // Compatibility policy: the original WORD underflow reads outside Mac RAM.
    // Basilisk can read unrelated host bytes there; only unreadable-page results
    // support this empty mask. See evidence/hive-memory-verification.md.
    let empty = [0; 49];
    let pattern = if shape == 0 {
        &empty[..]
    } else {
        centurions_content::span(areas, (shape - 1) * 49, 49)?
    };
    let (mut best, mut best_friendly, mut best_hostile) = (None, 0, 0);
    for (candidate, distance) in distances.iter().enumerate().take(110) {
        if *distance > range {
            continue;
        }
        let eligible = if candidate < warriors.len() {
            warriors[candidate].record().active_flag() != 0
        } else if candidate >= 10 {
            combat
                .enemies
                .records
                .get(candidate - 10)
                .is_some_and(|m| m.record().stamina().unwrap_or(0) > 0)
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
                    || (body >= 10
                        && combat.enemies.records[body - 10]
                            .record()
                            .mechanical_state()
                            == 0xff);
                if same {
                    friendly += 1;
                } else {
                    hostile += 1;
                }
            }
        }
        let preferred = if side == 0 {
            hostile >= best_hostile - best_friendly
                && hostile > friendly
                && ((hostile >= best_hostile && friendly <= best_friendly)
                    || hostile - friendly > best_hostile)
        } else {
            hostile >= best_hostile && (friendly <= best_friendly || hostile > best_hostile)
        };
        if preferred {
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
    if packet.len() == 58 && AttackPacket(packet).effect()? == 36 {
        if AttackPacket(packet).saving_throw_mode() != 3
            || AttackPacket(packet).defense_category() != 8
            || packet[20] != 8
            || AttackPacket(packet).race_restriction()? != 0
        {
            return Err(RuleError::Unsupported {
                action: "combat.spell.target",
                dependency: "other spell condition packet",
            });
        }
    } else if !(AttackPacket(packet).effect()? == 57
        && AttackPacket(packet).saving_throw_mode() == 3
        && AttackPacket(packet).defense_category() == 8
        && packet[20] == 8
        && packet[4] == 0)
    {
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
    if prepared.packet.len() != 58 || AttackPacket(&prepared.packet).defense_category() != 9 {
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
    let count = (AttackPacket(&prepared.packet).defense_category() == 9).then_some(prepared.shots);
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

pub(super) fn select_targets_impl(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    targeting: (&[u8], Option<i16>),
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    let (packet, firearm_count) = targeting;
    if !matches!(
        AttackPacket(packet).targeting_mode(),
        0 | 1 | 3 | 6 | 9 | 10 | 12
    ) {
        return Err(RuleError::Unsupported {
            action: "combat.spell.target",
            dependency: "other spell targeting",
        });
    }
    let caster = combat
        .enemies
        .records
        .get(caster_index)
        .ok_or_else(|| RuleError::Invalid("spell caster index".into()))?;
    let start = combat.enemies.positions[caster_index];
    let side = caster.record().faction()?;
    let mut next_random = random.clone();
    let mut count = next_random.bound(7);
    if let Some(firearm_count) = firearm_count {
        count = firearm_count;
    }
    let mut range = AttackPacket(packet)
        .range_per_power()
        .wrapping_mul(count)
        .wrapping_add(AttackPacket(packet).base_range())
        .wrapping_abs();
    let size = caster.record().size()?.wrapping_abs();
    range = range.wrapping_add(i16::from(size != 0) + i16::from(size == 3));
    let mut result = SpellSelection {
        cast: false,
        caster: caster.clone(),
        count,
        range,
        nearest: 127,
        distances: [127; 112],
        targets: vec![],
        rotation: 0,
    };
    let mut available = 0;
    for body in (0..warriors.len()).chain(10..10 + combat.enemies.records.len()) {
        let eligible = if body < 10 {
            warriors[body].record().active_flag() != 0 && warriors[body].record().stamina()? > 0
        } else {
            let m = &combat.enemies.records[body - 10].bytes;
            EnemyRecord::from_bytes(m).stamina()? > 0
                && EnemyRecord::from_bytes(m).mechanical_state() != 0xff
        };
        if !eligible
            || ((faction(combat, warriors, body)? == side)
                != (AttackPacket(packet).saving_throw_mode() == 4))
        {
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
            && (AttackPacket(packet).attack_count() + AttackPacket(packet).base_range() <= 0
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
    if AttackPacket(packet).targeting_mode() == 0 {
        count = next_random.bound(available);
    }
    count = count.min(7);
    while AttackPacket(packet).psi_cost().wrapping_mul(count) > caster.record().psi_points()? {
        count -= 1;
        if count == 0 {
            result.count = 0;
            *random = next_random;
            return Ok(result);
        }
    }
    result.count = count;
    let desired = if AttackPacket(packet).targeting_mode() > 0
        && AttackPacket(packet).defense_category() != 9
    {
        1
    } else {
        count
    };
    let random_point =
        AttackPacket(packet).targeting_mode() == 3 && AttackPacket(packet).area_shape() == 10;
    if !random_point && AttackPacket(packet).targeting_mode() > 1 && result.nearest < range {
        if let Some(body) = best_choice(
            combat,
            warriors,
            areas,
            usize::from(AttackPacket(packet).area_shape()),
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
    } else if (random_point && result.nearest < range)
        || (AttackPacket(packet).targeting_mode() < 2 && result.nearest <= range)
    {
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
            result.targets.push(if random_point {
                let target = position(combat, body)
                    .ok_or_else(|| RuleError::Invalid("spell body position".into()))?;
                SpellTarget {
                    first: target.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
                    second: target.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
                    kind: 3,
                }
            } else {
                SpellTarget {
                    first: body as i16,
                    second: -1,
                    kind: if body < 9 { 1 } else { 2 },
                }
            });
        }
    }
    if !result.targets.is_empty()
        || (firearm_count == Some(0)
            && AttackPacket(packet).targeting_mode() < 2
            && result.nearest <= range)
    {
        result.cast = true;
        let actions = result.caster.record().action_points().wrapping_sub(2);
        let movement = result.caster.record().movement_points().wrapping_sub(3);
        result.caster.record_mut().set_action_points(actions);
        result.caster.record_mut().set_movement_points(movement);
        result.caster.record_mut().set_reaction_ready(actions >= 1);
        let psi = result
            .caster
            .record()
            .psi_points()?
            .wrapping_sub(AttackPacket(packet).psi_cost().wrapping_mul(count));
        result.caster.record_mut().set_psi_points(psi);
        if firearm_count.is_some() {
            result.count = 1;
        }
        if AttackPacket(packet).has_randomized_animation() {
            result.rotation = (next_random.bound(4) - 1) as u8;
        }
    }
    *random = next_random;
    Ok(result)
}
