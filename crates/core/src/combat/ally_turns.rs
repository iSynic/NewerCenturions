use crate::combat::map_alias;
use crate::combat::runtime::neighbor_scan::NeighborScan;
use crate::combat::{Combat, runtime};
use crate::{Event, RandomState, RuleError};
use centurions_content::{Warrior, records::ItemDefinition};

/// Whether a body is alive. A warrior slot past a short squad holds no one.
fn alive(combat: &Combat, warriors: &[Warrior], body: i16) -> bool {
    if (0..6).contains(&body) {
        warriors
            .get(body as usize)
            .is_some_and(|warrior| warrior.record().in_battle() != 0)
    } else {
        combat.enemy_body_alive(body)
    }
}

/// The item context after loading item `id`: the ammunition caliber of that item.
pub fn item_context_after_load(items: &[Vec<u8>], id: i16) -> Result<i16, RuleError> {
    let index = usize::try_from(id.wrapping_abs())
        .map_err(|_| RuleError::Invalid("invalid item context".into()))?;
    let item = items
        .get(index)
        .ok_or_else(|| RuleError::Invalid("item context missing".into()))?;
    Ok(ItemDefinition(item).ammunition_caliber()?)
}

/// The item context after a warrior's load is recalculated: the ammunition caliber of the last
/// inventory item, or the previous context when the pack is empty.
pub fn item_context_after_weight_calc(
    warrior: &Warrior,
    items: &[Vec<u8>],
    previous: i16,
) -> Result<i16, RuleError> {
    let inventory = warrior.inventory()?;
    if let Some(last) = inventory.last() {
        item_context_after_load(items, last.id())
    } else {
        Ok(previous)
    }
}

mod natural;
pub(crate) use natural::{
    MonsterMelee, Struck, natural_attack_enemy_with_processing, natural_core,
};
pub use natural::{
    natural_attack_enemy, natural_attack_enemy_with_actor, natural_attack_enemy_with_packet,
    result_audio,
};

mod firearm;
use firearm::fire;
pub(crate) use firearm::{
    Duel, ScientistWorld, Shot, fire as fire_scientist, fire_in as fire_scientist_in,
};

/// A friendly firearm monster's turn with the given item in hand and no party buffs.
pub fn scientist_turn_with_item_context(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    item_context: i16,
) -> Result<Vec<Event>, RuleError> {
    scientist_turn_with_item_context_and_buffs(
        combat,
        warriors,
        mission,
        random,
        items,
        item_context,
        &[0; 20],
    )
}

/// A friendly firearm monster's turn with the given item in hand and the party buffs in force.
pub fn scientist_turn_with_item_context_and_buffs(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    item_context: i16,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    scientist_turn_with_globals(
        combat,
        warriors,
        mission,
        random,
        items,
        &mut runtime::EnemyGlobals {
            item_context,
            display_writers: false,
            ..Default::default()
        },
        buffs,
    )
}

/// A friendly firearm monster takes its turn: while it has at least two action points it fires its
/// rifle, casts, or walks toward its target, with the adjacent warriors reacting after each step.
/// Refused unless the monster is faction zero and carries firearm one.
pub(crate) fn scientist_turn_with_globals(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    globals: &mut runtime::EnemyGlobals,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    let mut item_context = globals.item_context;
    let body = combat
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active scientist missing".into()))?;
    if body < 10 {
        return Err(RuleError::Invalid("active scientist is warrior".into()));
    }
    let monster_index = (body - 10) as usize;
    if combat.roster.records[monster_index].record().faction()? != 0
        || combat.roster.records[monster_index]
            .record()
            .primary_weapon()?
            != 1
    {
        return Err(RuleError::Unsupported {
            action: "combat.scientist",
            dependency: "ally rifle caller requires friendly firearm one",
        });
    }
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?
        .attacks;
    let turn = ScientistTurn {
        mission,
        items,
        attacks,
        buffs,
        body,
    };
    let mut events = vec![];
    let mut attempt = 0;
    for _ in 0..32 {
        let actor = &mut combat.roster.records[monster_index];
        let action_points = actor.record().action_points();
        if action_points < 2 {
            return Ok(events);
        }
        actor.record_mut().set_reaction_ready(action_points >= 1);
        let record = actor.record();
        let fleeing = record.fleeing_duration() != 0;
        let mut skip_neighbors = false;
        if !fleeing && random.bound(100) <= record.missile_percent()? {
            if fire_rifle(
                combat,
                warriors,
                random,
                &turn,
                globals,
                item_context,
                &mut events,
            )? {
                reload_item_context(globals, &mut item_context)?;
                continue;
            }
            attempt += 1;
        } else if !fleeing && random.bound(100) <= record.cast_percent()? {
            if globals.spell_disabled != 0 || record.spell_blocked_before_movement() {
                skip_neighbors = true;
            } else {
                attempt += 1;
                if runtime::enemy_spell(
                    combat,
                    warriors,
                    mission,
                    random,
                    (items, globals),
                    body,
                    &mut events,
                )? {
                    reload_item_context(globals, &mut item_context)?;
                    let record = combat.roster.records[monster_index].record();
                    if record.missile_percent()? != 0
                        && record.action_points() >= 2
                        && attempt < 2
                        && fire_rifle(
                            combat,
                            warriors,
                            random,
                            &turn,
                            globals,
                            item_context,
                            &mut events,
                        )?
                    {
                        reload_item_context(globals, &mut item_context)?;
                        continue;
                    }
                    return Ok(events);
                }
            }
        }
        events.extend(if fleeing || skip_neighbors {
            vec![]
        } else {
            neighbor_reactions(combat, warriors, random, &turn, globals)?
        });
        reload_item_context(globals, &mut item_context)?;
        approach(
            combat,
            warriors,
            random,
            &turn,
            globals,
            &mut item_context,
            &mut events,
        )?;
        let record = combat.roster.records[monster_index].record();
        // After moving, the scientist tries to cast again before its separate
        // firearm retry.
        if record.cast_percent()? != 0 && record.action_points() >= 2 && attempt < 2 {
            combat.roster.records[monster_index]
                .record_mut()
                .set_movement(0);
            let record = combat.roster.records[monster_index].record();
            if globals.spell_disabled == 0 && !record.spell_blocked_after_movement() {
                attempt = 2;
                runtime::enemy_spell(
                    combat,
                    warriors,
                    mission,
                    random,
                    (items, globals),
                    body,
                    &mut events,
                )?;
                reload_item_context(globals, &mut item_context)?;
            }
        }
        globals.item_context = item_context;
        let record = combat.roster.records[monster_index].record();
        if globals.display_writers
            && record.missile_percent()? != 0
            && record.action_points() >= 2
            && attempt < 2
        {
            attempt += 1;
            if fire_rifle(
                combat,
                warriors,
                random,
                &turn,
                globals,
                item_context,
                &mut events,
            )? {
                reload_item_context(globals, &mut item_context)?;
                continue;
            }
        }
        return Ok(events);
    }
    Err(RuleError::Invalid("scientist action point budget".into()))
}

/// The acting scientist and the read-only owners its turn passes to every service.
struct ScientistTurn<'a> {
    mission: &'a centurions_content::mission::OpeningMission,
    items: &'a [Vec<u8>],
    attacks: &'a [u8],
    buffs: &'a [u8],
    body: i16,
}

/// The item in hand after a service reloaded it.
fn reload_item_context(
    globals: &mut runtime::EnemyGlobals,
    item_context: &mut i16,
) -> Result<(), RuleError> {
    if let Some(item) = globals.buffers.item.as_ref() {
        *item_context = ItemDefinition(item).ammunition_caliber()?;
        globals.item_context = *item_context;
    }
    Ok(())
}

/// The scientist's rifle shot; true when it fired.
fn fire_rifle(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    turn: &ScientistTurn,
    globals: &mut runtime::EnemyGlobals,
    item_context: i16,
    events: &mut Vec<Event>,
) -> Result<bool, RuleError> {
    fire(
        combat,
        warriors,
        turn.mission,
        random,
        turn.items,
        events,
        (item_context, turn.body),
        &mut globals.buffers,
        globals.display_writers,
        &mut globals.attack_count,
    )
}

/// The adjacent warriors' reactions to the scientist where it stands.
fn neighbor_reactions(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    turn: &ScientistTurn,
    globals: &mut runtime::EnemyGlobals,
) -> Result<Vec<Event>, RuleError> {
    let (body, items, attacks, buffs) = (turn.body, turn.items, turn.attacks, turn.buffs);
    Ok(if globals.display_writers {
        runtime::check_neighbors_with_loaded(
            combat,
            warriors,
            random,
            body,
            items,
            attacks,
            buffs,
            &mut globals.buffers,
            globals.neighbor_mask.as_mut().map(|mask| NeighborScan {
                char_select_new: globals.char_select_new,
                party_track: mask,
                aliases: map_alias::land_aliases(turn.mission, globals.land.as_ref()),
                table: globals.warrior_table.as_ref(),
            }),
        )?
        .events
    } else {
        runtime::check_neighbors_with_buffs(combat, warriors, random, body, items, attacks, buffs)?
    })
}

/// The scientist's steps toward its target, each followed by the neighbors'
/// reactions, until it stops, fails a step or runs out of points.
fn approach(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    turn: &ScientistTurn,
    globals: &mut runtime::EnemyGlobals,
    item_context: &mut i16,
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    let (mission, body) = (turn.mission, turn.body);
    for _ in 0..128 {
        if !alive(combat, warriors, body)
            || !warriors
                .iter()
                .any(|warriors| warriors.record().in_battle() != 0)
        {
            break;
        }
        let record = combat.roster.records[(body - 10) as usize].record();
        if record.movement() <= 0 || record.action_points() <= 1 || record.turn_skip_duration() != 0
        {
            break;
        }
        runtime::select_enemy_target(combat, warriors, mission, random, body)?;
        let moved = runtime::move_actor_with_reactions(
            combat,
            warriors,
            mission,
            random,
            body,
            turn.items,
            turn.buffs,
            &mut globals.buffers,
            globals.display_writers,
            globals.neighbor_mask.as_mut().map(|mask| NeighborScan {
                char_select_new: globals.char_select_new,
                party_track: mask,
                aliases: map_alias::land_aliases(mission, globals.land.as_ref()),
                table: globals.warrior_table.as_ref(),
            }),
            events,
        )?;
        events.extend(
            moved
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        if moved.moved {
            let position = combat.body_position(body);
            events.push(Event::TacticalMoved {
                body,
                x: position.0,
                y: position.1,
            });
        }
        events.extend(neighbor_reactions(combat, warriors, random, turn, globals)?);
        reload_item_context(globals, item_context)?;
        if !moved.moved {
            break;
        }
    }
    Ok(())
}

/// A scientist's turn, taken with no ammunition selected.
///
/// Original: combat
pub fn scientist_turn(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
) -> Result<Vec<Event>, RuleError> {
    scientist_turn_with_item_context(combat, warriors, mission, random, items, 0)
}
