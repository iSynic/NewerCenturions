//! Effect58: resolvespell 151744 calls spelllist(0, 0x3a), which copies MONSTER
//! work 440dbc into the first dead NPC slot at the cursor cell (15be68..15c5a0).
use super::{Combat, TacticalMap, footprint, monster, retained::NpcSlot};
use crate::feedback::NativeFeedback;
use crate::{Event, RandomState, RuleError};
use centurions_content::combat::{BattleRecord, MonsterRecord};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{AttackPacket, Condition, EnemyRecordMut};
use centurions_content::{Warrior, word};

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.spell.summon",
        dependency,
    }
}

/// The arrays spelllist(0, 0x3a) reads and writes, live or after teardown.
pub(crate) struct SummonArena<'a> {
    /// WORD map4085b8 and its origin.
    pub map: &'a mut TacticalMap,
    /// Signed WORD43874a and the NPC slots past it.
    pub count: usize,
    pub slots: &'a [Option<NpcSlot>],
    /// Whether combatsetup left no slot gap for a skipped level-zero grid record.
    pub grid_exact: Option<bool>,
    pub order: &'a mut [i16],
    pub cursor: usize,
    pub difficulty: i16,
    /// MONSTER work 440dbc.
    pub template: Option<Vec<u8>>,
}

pub(crate) enum Summoned {
    Refused(Vec<Event>),
    Placed {
        events: Vec<Event>,
        record: MonsterRecord,
    },
}

/// The cell WORDs 443722/443724 select, as absolute tactical coordinates. spelllist
/// 15be68 indexes the map as an unsigned WORD column × 90 plus a signed row, so off-map
/// cells read the memory beside it.
fn tile(map: &TacticalMap, mission: &OpeningMission, x: i32, y: i32) -> Result<i16, RuleError> {
    super::map_alias::MapAliases::of_mission(mission)
        .read(map, i32::from(x as u16) * 90 + y)?
        .ok_or_else(|| {
            unavailable("summon cell check outside the map, its edge WORD and tutorial rectangles")
        })
}

/// combatsetup reads each nonzero grid record into the next slot before its
/// level test; the reconstruction keeps no slot for a skipped level-zero record.
pub(crate) fn grid_leaves_slots_exact(battle: i16, mission: &OpeningMission) -> Option<bool> {
    let tables = mission.combat.as_ref()?;
    let battle = BattleRecord::read(&tables.battles, battle).ok()?;
    for &id in battle.cells.iter().filter(|&&id| id != 0) {
        let level = MonsterRecord::read(&tables.monsters, id)
            .ok()?
            .record()
            .level()
            .ok()?;
        if level == 0 {
            return Some(false);
        }
    }
    Some(true)
}

/// combatupdate2's item-work owner for a live full-roster refusal.
pub(crate) type ItemRefresh<'a> = Option<(&'a [Vec<u8>], &'a mut crate::ammunition::WorkBuffers)>;

/// resolvespell 1516f0 compares the signed NPC count WORD43874a with 99.
pub(crate) fn roster_full(count: i16) -> bool {
    count > 99
}

/// resolvespell 1516fa..151724 for a full roster: cleartarget, WORD4435c0 0,
/// WORD442bd4 6 and flashmessage 0x401860. Ready 0 and combatupdate2 follow.
pub(crate) fn refuse(work: &mut super::target_work::TargetWork) -> Event {
    work.refuse_summon();
    Event::Feedback(NativeFeedback::Flash {
        address: 0x401860,
        x: 50,
        y: 100,
        wait: 0,
        sound: 6000,
    })
}

/// The first slot from the NPC count whose stamina WORD is below one.
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
    let attack = AttackPacket(packet);
    let target = target
        .filter(|_| {
            attack.targeting_mode() >= 2 && !matches!(attack.targeting_mode(), 6 | 7 | 9 | 10 | 12)
        })
        .ok_or_else(|| unavailable("summon cursor is not a tactical cell"));
    summon_at(
        combat, warriors, mission, random, selected, target, power, packet, refresh,
    )
}

/// The summon arm at its target cell, which spelllist reads only after the full-roster
/// branch; collide sets triple 0 from the field centre.
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
    // resolvespell15108c/1511ea draws the duration before the effect arms.
    super::spells::detached::duration(packet, power, random);
    if roster_full(combat.enemies.records.len() as i16) {
        // Every session caller lends its ITEM work; only record-level helpers omit it.
        let (items, buffers) = refresh.ok_or_else(|| {
            RuleError::Invalid(
                "full-roster summon refusal without combatupdate2's item work".into(),
            )
        })?;
        combat.clear_target_tail();
        let event = refuse(&mut combat.target_work);
        combat.ready = 0;
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
    let count = combat.enemies.records.len();
    let summoned = place(
        SummonArena {
            map: &mut combat.map,
            count,
            slots: &combat.retained_npcs.slots,
            grid_exact,
            order: &mut combat.order,
            cursor: combat.cursor,
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
            combat.enemies.records.push(record);
            combat.enemies.positions.push(target);
            Ok(events)
        }
    }
}

/// spelllist(0, 0x3a) after resolvespell's duration draw: copies MONSTER work into
/// the first dead slot at the target cell. The caller stores the placed record.
pub(crate) fn place(
    arena: SummonArena<'_>,
    warriors: &[Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    (faction, target): (i16, (i16, i16)),
    packet: &[u8],
) -> Result<Summoned, RuleError> {
    let attack = AttackPacket(packet);
    let template = arena
        .template
        .clone()
        .ok_or_else(|| unavailable("summon reads unrecorded MONSTER work"))?;
    // spelllist's reflection test always reads party body 0 here.
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
    let x = i32::from(target.0) + i32::from(arena.map.shift.0) + 5;
    let y = i32::from(target.1) + i32::from(arena.map.shift.1) + 5;
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
            Event::Feedback(NativeFeedback::CombatResult {
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
    record.record_mut().set_target_body(-1);
    if let Some(entry) = arena.order[1..].iter_mut().rev().find(|id| **id == -1) {
        *entry = body;
        events.push(Event::Feedback(NativeFeedback::CombatResult {
            body,
            code: -12,
            amount: 0,
            audible: false,
        }));
    }
    // The build matches buildmonster's except that the faction write is unconditional.
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
