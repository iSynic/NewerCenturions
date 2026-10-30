//! Spell target selection: bestchoice's aim and the target triples it fills.
use super::{Combat, RandomState, RuleError, firearm, validate_direct};
use centurions_content::combat::MonsterRecord;
use centurions_content::{
    Warrior,
    records::{EnemyRecord, SpellInfoRecord},
};

mod view;
use crate::combat::map_alias::{Land, MapAliases, land_aliases};
pub(crate) use view::SelectionView;

/// One target triple (`target[n]`): its first and second values and its kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellTarget {
    pub first: i16,
    pub second: i16,
    pub kind: i16,
}

impl SpellTarget {
    /// A body triple: kind 1 for a warrior (body below 10), kind 2 for a monster.
    pub fn body(body: i16) -> Self {
        Self {
            first: body,
            second: 0,
            kind: if body < 10 { 1 } else { 2 },
        }
    }

    /// A location triple (kind 3) at map coordinates.
    pub fn cell(x: i16, y: i16) -> Self {
        Self {
            first: x,
            second: y,
            kind: 3,
        }
    }
}

/// A monster's chosen cast: whether it can cast, the caster, the power count, the reach, the
/// distance to the nearest foe and to each body in reach, the targets, and the rotation.
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

/// Which side a warrior or monster body is on.
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
            .roster
            .records
            .get(body - 10)
            .ok_or_else(|| RuleError::Invalid("spell monster body".into()))?
            .record()
            .faction()?)
    }
}

/// Picks the body at whose cell the spell's area would do the most good: the most hostile bodies
/// and the fewest friendly ones among those within range, judged by a rule that differs for the
/// party's side (side 0) and the monsters' side.
///
/// Original: bestchoice
pub(super) fn best_choice(
    (combat, aliases): (&Combat, MapAliases<'_>),
    warriors: &[Warrior],
    areas: &[u8],
    shape: usize,
    side: i16,
    distances: &[i16; 112],
    range: i16,
) -> Result<Option<usize>, RuleError> {
    // A pattern index below the table reads outside memory and is treated as an
    // empty pattern (evidence/hive-memory-verification.md).
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
            warriors[candidate].record().in_battle() != 0
        } else if candidate >= 10 {
            combat
                .roster
                .records
                .get(candidate - 10)
                .is_some_and(|m| m.record().stamina().unwrap_or(0) > 0)
        } else {
            false
        };
        if !eligible {
            continue;
        }
        let center = combat
            .get_body_position(candidate)
            .ok_or_else(|| RuleError::Invalid("spell candidate position".into()))?;
        let (mut friendly, mut hostile) = (0, 0);
        let mut seen = [false; 110];
        for y in 0..7 {
            for x in 0..7 {
                if pattern[y * 7 + x] == 0 {
                    continue;
                }
                let ax = combat.field.map_shift.0 + 5 + center.0 - 3 + x as i16;
                let ay = combat.field.map_shift.1 + 5 + center.1 - 3 + y as i16;
                // The column is unsigned and the row signed, so a row past either edge
                // reads the neighbouring column or the memory beside the map.
                let index = i32::from(ax as u16) * 90 + i32::from(ay);
                let body = aliases
                    .read(&combat.field, index)?
                    .ok_or(RuleError::Unsupported {
                        action: "combat.spell.target",
                        dependency: "spell area outside the map and the represented memory beside it",
                    })?
                    .wrapping_abs();
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
                        && combat.roster.records[body - 10].record().mechanical_state() == 0xff);
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

/// A monster's spell targets: the bodies in reach and the cells the spell is aimed at.
pub fn select_targets(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    packet: &[u8],
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    if !super::is_support(packet)? {
        validate_direct(packet)?;
    }
    select_targets_impl(
        (combat, MapAliases::of_mission(mission)),
        warriors,
        mission,
        areas,
        caster_index,
        (packet, None),
        random,
    )
}

/// select_targets with the current land arrays beside the map.
pub(crate) fn select_targets_beside(
    combat: &Combat,
    warriors: &[Warrior],
    (mission, land): (&centurions_content::mission::OpeningMission, Option<&Land>),
    caster_index: usize,
    packet: &[u8],
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    if !super::is_support(packet)? {
        validate_direct(packet)?;
    }
    select_targets_impl(
        (combat, land_aliases(mission, land)),
        warriors,
        mission,
        &mission.areas,
        caster_index,
        (packet, None),
        random,
    )
}

/// select_firearm_targets with the current land arrays beside the map.
pub(crate) fn select_firearm_targets_beside(
    combat: &Combat,
    warriors: &[Warrior],
    (mission, land): (&centurions_content::mission::OpeningMission, Option<&Land>),
    caster_index: usize,
    prepared: &firearm::PreparedFirearm,
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    if prepared.packet.len() != 58 || SpellInfoRecord(&prepared.packet).defense_category() != 9 {
        return Err(RuleError::Invalid("firearm target packet".into()));
    }
    select_targets_impl(
        (combat, land_aliases(mission, land)),
        warriors,
        mission,
        &mission.areas,
        caster_index,
        (&prepared.packet, Some(prepared.shots)),
        random,
    )
}

/// A monster's targets for a firearm shot, with the shot count of the prepared firearm.
pub fn select_firearm_targets(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    prepared: &firearm::PreparedFirearm,
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    if prepared.packet.len() != 58 || SpellInfoRecord(&prepared.packet).defense_category() != 9 {
        return Err(RuleError::Invalid("firearm target packet".into()));
    }
    select_targets_impl(
        (combat, MapAliases::of_mission(mission)),
        warriors,
        mission,
        areas,
        caster_index,
        (&prepared.packet, Some(prepared.shots)),
        random,
    )
}

/// A monster's targets for the prepared attack: the shot count is used only for a defense category
/// 9 shot.
pub fn select_spell_info_targets(
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
    let count =
        (SpellInfoRecord(&prepared.packet).defense_category() == 9).then_some(prepared.shots);
    select_targets_impl(
        (combat, MapAliases::of_mission(mission)),
        warriors,
        mission,
        areas,
        caster_index,
        (&prepared.packet, count),
        random,
    )
}

/// Chooses a monster's targets over the live battle.
pub(super) fn select_targets_impl(
    (combat, aliases): (&Combat, MapAliases<'_>),
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    targeting: (&[u8], Option<i16>),
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    select_from_view(
        &view::LiveSelection(combat, aliases),
        warriors,
        mission,
        areas,
        caster_index,
        targeting,
        random,
    )
}

/// A monster's targets for a firearm shot over held records, with the shot count of the prepared
/// firearm.
pub(crate) fn select_firearm_from_view(
    view: &impl SelectionView,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    prepared: &firearm::PreparedFirearm,
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    if prepared.packet.len() != 58 || SpellInfoRecord(&prepared.packet).defense_category() != 9 {
        return Err(RuleError::Invalid("firearm target packet".into()));
    }
    select_from_view(
        view,
        warriors,
        mission,
        areas,
        caster_index,
        (&prepared.packet, Some(prepared.shots)),
        random,
    )
}

/// An enemy shot's held-packet selection: the shot count applies only to category 9.
pub(crate) fn select_spell_info_from_view(
    view: &impl SelectionView,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    prepared: &firearm::PreparedFirearm,
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    let count =
        (SpellInfoRecord(&prepared.packet).defense_category() == 9).then_some(prepared.shots);
    select_from_view(
        view,
        warriors,
        mission,
        areas,
        caster_index,
        (&prepared.packet, count),
        random,
    )
}

/// The enemy decision's spell targeting over held records.
pub(crate) fn select_spell_from_view(
    view: &impl SelectionView,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    packet: &[u8],
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    select_from_view(
        view,
        warriors,
        mission,
        areas,
        caster_index,
        (packet, None),
        random,
    )
}

/// Chooses what a monster casts at. It draws the power count (1 to 7, or the shot count), works out
/// the reach from the packet and the caster's size, and measures every eligible body on the
/// aimed-at side (the caster's own side when the packet's cannot byte is 4, the other side
/// otherwise). A body counts when it is within reach and, if the spell has range, in sight. With
/// nothing in reach it does not cast. The count is cut until the caster has the psi for it. A mode
/// above 1 aims at the best cell; mode 0 and 1 pick random bodies in reach. A cast spends the
/// caster's 2 action points, 3 movement and the psi.
///
/// Original: getrange, bestchoice
fn select_from_view(
    view: &impl SelectionView,
    warriors: &[Warrior],
    mission: &centurions_content::mission::OpeningMission,
    areas: &[u8],
    caster_index: usize,
    targeting: (&[u8], Option<i16>),
    random: &mut RandomState,
) -> Result<SpellSelection, RuleError> {
    let (packet, firearm_count) = targeting;
    // The mode is a signed byte. Every mode above 1 aims at bestchoice's body;
    // spelltargets shapes only modes 3 and 4, and the mode-7 arm ignores the mask.
    // Modes 128 and up are negative: they fail every signed test like mode 0, but
    // not its equality test.
    let mode = SpellInfoRecord(packet).target_type() as i8;
    let caster = view.monster(caster_index)?;
    let start = view.position(caster_index + 10)?;
    let side = caster.record().faction()?;
    let mut next_random = random.clone();
    let mut count = next_random.bound(7);
    if let Some(firearm_count) = firearm_count {
        count = firearm_count;
    }
    let mut range = SpellInfoRecord(packet)
        .range_per_power()
        .wrapping_mul(count)
        .wrapping_add(SpellInfoRecord(packet).range_base())
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
    for body in (0..warriors.len()).chain(10..10 + view.monster_count()) {
        let eligible = if body < 10 {
            warriors[body].record().in_battle() != 0 && warriors[body].record().stamina()? > 0
        } else {
            let record = &view.monster(body - 10)?.bytes;
            EnemyRecord::from_bytes(record).stamina()? > 0
                && EnemyRecord::from_bytes(record).mechanical_state() != 0xff
        };
        if !eligible
            || ((view.faction(warriors, body)? == side) != (SpellInfoRecord(packet).cannot() == 4))
        {
            continue;
        }
        let end = view.position(body)?;
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
        // Sight matters only when the range bytes sum above 0, as getrange tests.
        if in_range
            && (SpellInfoRecord(packet).range_per_power() + SpellInfoRecord(packet).range_base()
                <= 0
                || view.visible(mission, start, end)?)
        {
            result.distances[body] = distance.trunc() as i16;
            available += 1;
        }
    }
    if available == 0 {
        *random = next_random;
        return Ok(result);
    }
    if mode == 0 {
        count = next_random.bound(available);
    }
    count = count.min(7);
    while SpellInfoRecord(packet).cost().wrapping_mul(count) > caster.record().psi_points()? {
        count -= 1;
        if count == 0 {
            result.count = 0;
            *random = next_random;
            return Ok(result);
        }
    }
    result.count = count;
    let desired = if mode > 0 && SpellInfoRecord(packet).defense_category() != 9 {
        1
    } else {
        count
    };
    let random_point = mode == 3 && SpellInfoRecord(packet).area_size() == 10;
    if !random_point && mode > 1 && result.nearest < range {
        if let Some(body) = view.best_choice(
            warriors,
            areas,
            usize::from(SpellInfoRecord(packet).area_size()),
            side,
            &result.distances,
            range,
        )? {
            let target = view.position(body)?;
            result.targets.push(SpellTarget::cell(
                target.0.wrapping_add(view.map_shift().0).wrapping_add(5),
                target.1.wrapping_add(view.map_shift().1).wrapping_add(5),
            ));
        }
    } else if (random_point && result.nearest < range) || (mode < 2 && result.nearest <= range) {
        for _ in 0..desired {
            let mut selected = None;
            for _ in 0..100 {
                let body = next_random
                    .bound(view.monster_target_bound())
                    .wrapping_sub(1);
                let body = usize::try_from(body)
                    .ok()
                    .filter(|body| *body < result.distances.len())
                    .ok_or(RuleError::Unsupported {
                        action: "combat.spell.target",
                        dependency: "NPC target draw outside represented distance row",
                    })?;
                if result.distances[body] <= range {
                    selected = Some(body);
                    break;
                }
            }
            let Some(body) = selected else { break };
            result.targets.push(if random_point {
                let target = view.position(body)?;
                SpellTarget::cell(
                    target.0.wrapping_add(view.map_shift().0).wrapping_add(5),
                    target.1.wrapping_add(view.map_shift().1).wrapping_add(5),
                )
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
        || (firearm_count == Some(0) && mode < 2 && result.nearest <= range)
    {
        result.cast = true;
        let actions = result.caster.record().action_points().wrapping_sub(2);
        let movement = result.caster.record().movement().wrapping_sub(3);
        result.caster.record_mut().set_action_points(actions);
        result.caster.record_mut().set_movement(movement);
        result.caster.record_mut().set_reaction_ready(actions >= 1);
        let psi = result
            .caster
            .record()
            .psi_points()?
            .wrapping_sub(SpellInfoRecord(packet).cost().wrapping_mul(count));
        result.caster.record_mut().set_psi_points(psi);
        if firearm_count.is_some() {
            result.count = 1;
        }
        if SpellInfoRecord(packet).can_rotate() {
            result.rotation = (next_random.bound(4) - 1) as u8;
        }
    }
    *random = next_random;
    Ok(result)
}
