//! Summoning (effect 58): the monster template comes to life in the first dead
//! monster slot, at the target cell.
use crate::combat::map_alias::MapAliases;
use crate::combat::spells::out_of_combat;
use crate::combat::targeting::target_picks::TargetPicks;
use crate::combat::{Combat, CombatField, footprint, globals::MonsterSlot, monster};
use crate::feedback::GameFeedback;
use crate::{Event, RandomState, RuleError};
use centurions_content::combat::{BattleRecord, MonsterRecord};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{Condition, EnemyRecordMut, SpellInfoRecord};
use centurions_content::{Warrior, word};

/// Refuses a summon whose case the reconstruction does not carry.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.spell.summon",
        dependency,
    }
}

/// What a summon reads and writes, during or after a battle.
pub(crate) struct SummonArena<'a> {
    /// The battle map and its origin.
    pub map: &'a mut CombatField,
    /// The monster count, and the slots past it.
    pub count: usize,
    pub slots: &'a [Option<MonsterSlot>],
    /// Whether battle setup left no slot for a skipped level-zero grid record.
    pub grid_exact: Option<bool>,
    pub order: &'a mut [i16],
    pub cursor: usize,
    pub difficulty: i16,
    /// The monster template.
    pub template: Option<Vec<u8>>,
}

/// How a summon ended: refused, with the events that show it, or the monster placed.
pub(crate) enum Summoned {
    Refused(Vec<Event>),
    Placed {
        events: Vec<Event>,
        record: MonsterRecord,
    },
}

/// The tile at the summon cell. The map is read as an unsigned column × 90 plus a
/// signed row, so off-map cells read what lies beside the map.
fn tile(map: &CombatField, mission: &OpeningMission, x: i32, y: i32) -> Result<i16, RuleError> {
    MapAliases::of_mission(mission)
        .read(map, i32::from(x as u16) * 90 + y)?
        .ok_or_else(|| {
            unavailable("summon cell check outside the map, its edge WORD and tutorial rectangles")
        })
}

/// Battle setup reads each nonzero grid record into the next slot before its level
/// test, so a skipped level-zero record may leave a slot behind.
pub(crate) fn grid_leaves_slots_exact(battle: i16, mission: &OpeningMission) -> Option<bool> {
    let tables = mission.combat.as_ref()?;
    let battle = BattleRecord::read(&tables.battles, battle).ok()?;
    for &id in battle.battle_grid.iter().filter(|&&id| id != 0) {
        let level = MonsterRecord::read(&tables.monsters, id)
            .ok()?
            .record()
            .hit_dice()
            .ok()?;
        if level == 0 {
            return Some(false);
        }
    }
    Some(true)
}

/// The item records combatupdate2 reloads when a full roster refuses a summon.
pub(crate) type ItemRefresh<'a> = Option<(&'a [Vec<u8>], &'a mut crate::ammunition::LoadedItems)>;

/// A roster of more than 99 monsters refuses a summon.
///
/// Original: resolvespell
pub(crate) fn roster_full(count: i16) -> bool {
    count > 99
}

/// A full roster refuses the summon: the targets clear, the cast loop ends and a
/// message flashes. The spell then ends and the panel redraws.
///
/// Original: resolvespell
pub(crate) fn refuse(picks: &mut TargetPicks) -> Event {
    picks.refuse_summon();
    Event::Feedback(GameFeedback::Flash {
        address: 0x401860,
        x: 50,
        y: 100,
        wait: 0,
        sound: 6000,
    })
}

/// The first dead slot, counting from the monster count.
fn dead_slot(arena: &SummonArena<'_>) -> Result<Option<usize>, RuleError> {
    let count = arena.count;
    match arena.grid_exact {
        Some(true) => {}
        Some(false) => {
            return Err(unavailable(
                "summon slot after a skipped level-zero grid record",
            ));
        }
        None => {
            return Err(unavailable(
                "summon slot scan without the battle grid record",
            ));
        }
    }
    for index in count..100 {
        let slot = arena.slots[index]
            .as_ref()
            .ok_or_else(|| unavailable("summon reads an unrecorded inactive NPC slot"))?;
        if slot.record.stamina()? < 1 {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

/// Summons the monster template into the battle at the spell's target cell. The spell must target a
/// cell on the battlefield.
#[allow(clippy::too_many_arguments)]
pub(crate) fn summon(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    selected: usize,
    target: Option<(i16, i16)>,
    power: i16,
    packet: &[u8],
    refresh: ItemRefresh<'_>,
) -> Result<Vec<Event>, RuleError> {
    let attack = SpellInfoRecord(packet);
    let target = target
        .filter(|_| {
            attack.target_type() >= 2 && !matches!(attack.target_type(), 6 | 7 | 9 | 10 | 12)
        })
        .ok_or_else(|| unavailable("summon cursor is not a tactical cell"));
    summon_at(
        combat, warriors, mission, random, selected, target, power, packet, refresh,
    )
}

/// A summon at its target cell, read only after the full-roster test; a field
/// collision targets the field's centre.
///
/// Original: resolvespell, spelllist
#[allow(clippy::too_many_arguments)]
pub(crate) fn summon_at(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    selected: usize,
    target: Result<(i16, i16), RuleError>,
    power: i16,
    packet: &[u8],
    refresh: ItemRefresh<'_>,
) -> Result<Vec<Event>, RuleError> {
    // The duration is drawn before the effect.
    out_of_combat::duration(packet, power, random);
    if roster_full(combat.roster.records.len() as i16) {
        let (items, buffers) = refresh.ok_or_else(|| {
            RuleError::Invalid(
                "full-roster summon refusal without combatupdate2's item work".into(),
            )
        })?;
        combat.clear_track_tail();
        let event = refuse(&mut combat.target_picks);
        combat.in_spell = 0;
        crate::combat::runtime::refresh_body_item(
            combat,
            warriors,
            items,
            selected as i16,
            buffers,
        )?;
        return Ok(vec![event]);
    }
    let target = target?;
    let template = combat
        .monster_template()
        .ok_or_else(|| unavailable("summon reads unrecorded MONSTER work"))?
        .to_vec();
    let faction = warriors
        .get(selected)
        .ok_or("summon selected warrior missing")?
        .record()
        .faction()?;
    let grid_exact = grid_leaves_slots_exact(combat.battle, mission);
    let count = combat.roster.records.len();
    let summoned = place(
        SummonArena {
            map: &mut combat.field,
            count,
            slots: &combat.monsters.slots,
            grid_exact,
            order: &mut combat.battle_order,
            cursor: combat.battle_order_index,
            difficulty: combat.difficulty,
            template: Some(template),
        },
        warriors,
        mission,
        random,
        (faction, target),
        packet,
    )?;
    match summoned {
        Summoned::Refused(events) => Ok(events),
        Summoned::Placed { events, record } => {
            combat.roster.records.push(record);
            combat.roster.mon_pos.push(target);
            Ok(events)
        }
    }
}

/// Brings the monster template to life in the first dead slot at the target cell.
/// The caller stores the placed record.
///
/// Original: spelllist
pub(crate) fn place(
    arena: SummonArena<'_>,
    warriors: &[Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    (faction, target): (i16, (i16, i16)),
    packet: &[u8],
) -> Result<Summoned, RuleError> {
    let attack = SpellInfoRecord(packet);
    let template = arena
        .template
        .clone()
        .ok_or_else(|| unavailable("summon reads unrecorded MONSTER work"))?;
    // The reflection test always reads the first warrior (see
    // docs/source-notes/combat.md).
    let reflective = warriors
        .first()
        .ok_or("summon party missing")?
        .record()
        .condition(Condition::ReflectingScans)?
        != 0;
    if reflective {
        random.bound(100);
    }
    let slot = dead_slot(&arena)?;
    let size = word(&template, 0x10)?;
    let x = i32::from(target.0) + i32::from(arena.map.map_shift.0) + 5;
    let y = i32::from(target.1) + i32::from(arena.map.map_shift.1) + 5;
    let blocked = [
        (tile(arena.map, mission, x, y)?, true),
        (
            tile(arena.map, mission, x, y - 1)?,
            size == 1 || size.wrapping_abs() == 3,
        ),
        (tile(arena.map, mission, x - 1, y)?, size.wrapping_abs() > 1),
        (
            tile(arena.map, mission, x - 1, y - 1)?,
            size.wrapping_abs() == 3,
        ),
    ]
    .iter()
    .any(|&(tile, occupied)| tile < 0x6f && occupied);
    let Some(slot) = slot.filter(|_| !blocked) else {
        let current = arena.order[arena.cursor];
        return Ok(Summoned::Refused(vec![
            Event::Sound { resource_id: 0x85 },
            Event::Feedback(GameFeedback::CombatResult {
                body: current,
                code: -9,
                amount: 0,
                audible: false,
            }),
        ]));
    };
    if slot != arena.count {
        return Err(unavailable("summon slot past the NPC count"));
    }
    let mut events = vec![Event::Sound {
        resource_id: attack.impact_sound()?,
    }];
    let body = slot as i16 + 10;
    let mut record = MonsterRecord { bytes: template };
    record.record_mut().set_target(-1);
    if let Some(entry) = arena.order[1..].iter_mut().rev().find(|id| **id == -1) {
        *entry = body;
        events.push(Event::Feedback(GameFeedback::CombatResult {
            body,
            code: -12,
            amount: 0,
            audible: false,
        }));
    }
    // Built as buildmonster builds, except the faction is always written.
    let mut record = monster::summon(record, -1, arena.difficulty, random)?;
    EnemyRecordMut::from_bytes(&mut record.bytes).set_faction(faction);
    let placed = footprint::place(arena.map, &record, target, body)?;
    *arena.map = placed.map;
    events.push(Event::CombatantSummoned { body });
    Ok(Summoned::Placed {
        events,
        record: placed.monster,
    })
}
