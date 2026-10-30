//! A monster's turn: its spells, fire, movement and attacks.
use super::neighbor_scan::NeighborScan;
use super::{
    FireOutcome, check_neighbors_with_items, collide_fields_with_refresh, enemy_fire, enemy_spell,
    move_actor_with_reactions, select_enemy_target,
};
use crate::combat::Combat;
use crate::combat::map_alias;
use crate::combat::map_alias::Land;
use crate::combat::warrior_alias::Owners;
use crate::combat::warrior_alias::Snapshot;
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

/// Plays the active monster's action with startup globals and an empty buff table.
pub fn enemy_action(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
) -> Result<Vec<Event>, RuleError> {
    enemy_action_with_context(
        combat,
        warriors,
        mission,
        random,
        items,
        &mut EnemyGlobals::default(),
        &[0; 20],
    )
}

/// What a monster's turn carries from step to step: the item context, the spell packet and attempt
/// count, the loaded item buffers, and the map and story data its effects can reach.
pub struct EnemyGlobals {
    pub item_context: i16,
    pub attack_count: i16,
    pub packet: Vec<u8>,
    pub identity: [i16; 3],
    pub spell_attempt: i8,
    pub spell_disabled: u8,
    pub buffers: crate::ammunition::LoadedItems,
    // Record-only fixtures skip the weapon refresh; a session always runs it.
    pub display_writers: bool,
    pub char_select_new: usize,
    /// The movement-spell marker, zero at launch.
    pub movement_gate: i16,
    pub neighbor_mask: Option<[i16; 6]>,
    /// The party duration table for a monster's global (mode 7) cast.
    pub party_condition: Option<Vec<u8>>,
    /// The animation-delay preference that lengthens a ray's step count.
    pub animation_delay: i16,
    /// The duration that follows the scenario directory, when it does not overlap
    /// the path text.
    pub scenario_tail: Option<i16>,
    /// The story flags an indexed mode-7 duration can reach.
    pub flags: Option<Vec<u8>>,
    /// The current land's objects and events beside the map, for reads past its edges.
    pub land: Option<Land>,
    /// Shop work, written by a monster's effect 26 or 64 alias.
    pub the_shop: Option<crate::shop::TheShop>,
    /// The current map header, written by a monster's effect 26 or 64 alias.
    pub event_headers: Option<Vec<u8>>,
    /// Warrior table records outside the squad; monster bodies 10..13 index the backup.
    pub warrior_table: Option<crate::state::WarriorTable>,
}
impl Default for EnemyGlobals {
    /// The startup values of the monster-turn globals: no item loaded, no spell attempted.
    fn default() -> Self {
        Self {
            item_context: 0,
            attack_count: 0,
            packet: crate::combat::spell_info_startup(),
            identity: [0; 3],
            spell_attempt: 0,
            spell_disabled: 0,
            buffers: Default::default(),
            display_writers: false,
            char_select_new: 0,
            movement_gate: 0,
            neighbor_mask: None,
            party_condition: None,
            animation_delay: 0,
            scenario_tail: None,
            flags: None,
            land: None,
            the_shop: None,
            event_headers: None,
            warrior_table: None,
        }
    }
}

impl EnemyGlobals {
    /// Copies of the owners a monster's effect 26 or 64 alias reads during one spell.
    pub(crate) fn alias_snapshot(&self) -> Snapshot {
        Snapshot::new(Owners {
            loaded_item: self.buffers.item.as_deref(),
            the_shop: self.the_shop.as_ref(),
            header: self.event_headers.as_deref(),
            warriors: None,
            table: self.warrior_table.as_ref(),
        })
    }

    /// Moves the attempt's pending writes into this work's owner copies.
    pub(crate) fn take_alias_writes(&mut self, combat: &mut Combat) -> Result<(), RuleError> {
        let writes = std::mem::take(&mut combat.alias_writes);
        crate::combat::warrior_alias::commit(
            self.buffers.item.as_mut(),
            self.the_shop.as_mut(),
            self.warrior_table.as_mut(),
            self.event_headers.as_mut(),
            &writes,
        )
    }
}

/// Plays the active monster's action on the given globals with an empty buff table.
pub fn enemy_action_with_globals(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    globals: &mut EnemyGlobals,
) -> Result<Vec<Event>, RuleError> {
    enemy_action_with_globals_and_buffs(combat, warriors, mission, random, items, globals, &[0; 20])
}

/// Plays the active monster's action on the given globals; the spell packet and cast identity are
/// carried in from the globals and written back after the action.
pub fn enemy_action_with_globals_and_buffs(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    globals: &mut EnemyGlobals,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    combat.spell_info = globals.packet.clone();
    combat.cast_identity = globals.identity;
    let events =
        enemy_action_with_context(combat, warriors, mission, random, items, globals, buffs)?;
    globals.packet = combat.spell_info.clone();
    globals.identity = combat.cast_identity;
    Ok(events)
}

/// Takes the caliber of the loaded weapon as the item context of the next attack.
pub(super) fn record_attack_context(globals: &mut EnemyGlobals) -> Result<(), RuleError> {
    if let Some(item) = globals.buffers.item.as_ref() {
        globals.item_context =
            centurions_content::records::ItemDefinition(item).ammunition_caliber()?;
    }
    Ok(())
}

/// Plays the active monster's action. Each pass needs two action points. It first fires its
/// firearm by its missile chance or else casts a psi power by its cast chance, neither while
/// fleeing; then the warriors beside it react and it approaches its target; then it may cast a
/// power and fire once more. A physical shot starts another pass. The action ends when the points
/// run out, a power is cast or no more can be done.
///
/// Original: combat
pub(super) fn enemy_action_with_context(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    globals: &mut EnemyGlobals,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    let body = combat
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active enemy missing".into()))?;
    if body < 10 {
        return Err(RuleError::Invalid("active body is a warrior".into()));
    }
    let monster_index = (body - 10) as usize;
    if combat.roster.records[monster_index].record().faction()? == 0
        && combat.roster.records[monster_index]
            .record()
            .primary_weapon()?
            == 1
    {
        return Err(RuleError::Unsupported {
            action: "combat.scientist",
            dependency: "work-item context requires a session",
        });
    }
    let mut events = vec![];
    if combat.roster.records[monster_index]
        .record()
        .action_points()
        < 2
    {
        return Ok(events);
    }
    let turn = EnemyTurn {
        mission,
        items,
        buffs,
        body,
    };
    let mut attempt = 0;
    let mut reactions = Reactions::default();
    for _ in 0..32 {
        let monster = &mut combat.roster.records[monster_index];
        if monster.record().action_points() < 2 {
            return Ok(events);
        }
        let reaction_ready = monster.record().action_points() >= 1;
        monster.record_mut().set_reaction_ready(reaction_ready);
        let fleeing = monster.record().fleeing_duration() != 0;
        let mut skip_neighbors = false;
        let firearm_chance = monster.record().missile_percent()?;
        if !fleeing && random.bound(100) <= firearm_chance {
            match enemy_fire(
                combat,
                warriors,
                mission,
                random,
                (items, globals),
                body,
                &mut events,
            )? {
                FireOutcome::Physical => continue,
                FireOutcome::Spell => {
                    attempt += 1;
                    if combat.roster.records[monster_index]
                        .record()
                        .missile_percent()?
                        != 0
                        && combat.roster.records[monster_index]
                            .record()
                            .action_points()
                            > 1
                        && attempt < 2
                        && enemy_fire(
                            combat,
                            warriors,
                            mission,
                            random,
                            (items, globals),
                            body,
                            &mut events,
                        )? == FireOutcome::Physical
                    {
                        continue;
                    }
                    return Ok(events);
                }
                FireOutcome::Failed => {}
            }
            attempt += 1;
        } else if !fleeing {
            let spell_chance = monster.record().cast_percent()?;
            if random.bound(100) <= spell_chance {
                let blocked =
                    globals.spell_disabled != 0 || monster.record().spell_blocked_before_movement();
                skip_neighbors = blocked;
                let cast = if blocked {
                    false
                } else {
                    attempt += 1;
                    enemy_spell(
                        combat,
                        warriors,
                        mission,
                        random,
                        (items, globals),
                        body,
                        &mut events,
                    )?
                };
                if cast {
                    if combat.roster.records[monster_index]
                        .record()
                        .missile_percent()?
                        != 0
                        && combat.roster.records[monster_index]
                            .record()
                            .action_points()
                            > 1
                        && attempt < 2
                        && enemy_fire(
                            combat,
                            warriors,
                            mission,
                            random,
                            (items, globals),
                            body,
                            &mut events,
                        )? == FireOutcome::Physical
                    {
                        continue;
                    }
                    return Ok(events);
                }
            }
        }
        let adjacent = if fleeing
            || skip_neighbors
            || combat.roster.records[monster_index]
                .record()
                .turn_skip_duration()
                != 0
        {
            super::reaction_calls::ReactionResult::default()
        } else {
            check_adjacent(
                combat,
                warriors,
                random,
                &turn,
                globals,
                &mut reactions.stopped,
            )?
        };
        reactions.take(adjacent, globals, &mut events)?;
        let movement_failed = match approach(
            combat,
            warriors,
            random,
            &turn,
            globals,
            &mut reactions,
            &mut events,
        )? {
            Some(failed) => failed,
            None => return Ok(events),
        };
        let monster = &combat.roster.records[monster_index];
        if attempt < 2 && monster.record().cast_percent()? != 0 && !reactions.interrupted {
            combat.roster.records[monster_index]
                .record_mut()
                .set_movement(0);
            let monster = &combat.roster.records[monster_index];
            let blocked =
                globals.spell_disabled != 0 || monster.record().spell_blocked_after_movement();
            if !blocked {
                attempt = if movement_failed { attempt + 2 } else { 2 };
                enemy_spell(
                    combat,
                    warriors,
                    mission,
                    random,
                    (items, globals),
                    body,
                    &mut events,
                )?;
                attempt += 1;
            }
        }
        let monster = &combat.roster.records[monster_index];
        if monster.record().missile_percent()? == 0
            || monster.record().action_points() < 2
            || attempt > 1
        {
            return Ok(events);
        }
        attempt += 1;
        if enemy_fire(
            combat,
            warriors,
            mission,
            random,
            (items, globals),
            body,
            &mut events,
        )? == FireOutcome::Physical
        {
            continue;
        }
        return Ok(events);
    }
    Err(RuleError::Invalid(
        "enemy action exceeded point budget".into(),
    ))
}

/// The acting monster and the read-only owners its turn passes to every service.
struct EnemyTurn<'a> {
    mission: &'a centurions_content::mission::OpeningMission,
    items: &'a [Vec<u8>],
    buffs: &'a [u8],
    body: i16,
}

/// The reaction locals one monster action carries between its neighbor checks.
#[derive(Default)]
struct Reactions {
    /// A reaction stopped the monster's movement.
    stopped: bool,
    /// A neighbor's reaction resolved an attack.
    interrupted: bool,
}

impl Reactions {
    /// Takes one neighbor check's events, recording whether it struck and the attack
    /// context it left.
    fn take(
        &mut self,
        adjacent: super::reaction_calls::ReactionResult,
        globals: &mut EnemyGlobals,
        events: &mut Vec<Event>,
    ) -> Result<(), RuleError> {
        self.interrupted |= adjacent
            .events
            .iter()
            .any(|event| matches!(event, Event::AttackResolved { .. }));
        record_attack_context(globals)?;
        events.extend(adjacent.events);
        Ok(())
    }
}

/// The adjacent warriors' reactions to the monster where it stands.
fn check_adjacent(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    turn: &EnemyTurn,
    globals: &mut EnemyGlobals,
    stopped: &mut bool,
) -> Result<super::reaction_calls::ReactionResult, RuleError> {
    check_neighbors_with_items(
        combat,
        warriors,
        random,
        turn.body,
        &mut super::reaction_calls::ReactionContext {
            items: Some(turn.items),
            attacks: Some(&turn.mission.combat.as_ref().unwrap().attacks),
            buffs: turn.buffs,
            buffers: &mut globals.buffers,
            display_writers: globals.display_writers,
            scan: globals.neighbor_mask.as_mut().map(|mask| NeighborScan {
                char_select_new: globals.char_select_new,
                party_track: mask,
                aliases: map_alias::land_aliases(turn.mission, globals.land.as_ref()),
                table: globals.warrior_table.as_ref(),
            }),
        },
        stopped,
    )
}

/// The monster's steps toward its target, each followed by field collisions and the
/// neighbors' reactions. `Some(failed)` says whether a step failed; `None` means
/// movement is no longer admitted and the action ends.
fn approach(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    turn: &EnemyTurn,
    globals: &mut EnemyGlobals,
    reactions: &mut Reactions,
    events: &mut Vec<Event>,
) -> Result<Option<bool>, RuleError> {
    let (mission, items, body) = (turn.mission, turn.items, turn.body);
    for _ in 0..128 {
        if reactions.stopped {
            break;
        }
        if !combat.body_alive(warriors, body)
            || !warriors
                .iter()
                .any(|warriors| warriors.record().in_battle() != 0)
        {
            break;
        }
        let moving = if globals.neighbor_mask.is_some() {
            combat.kill_counts.movement_admitted()
        } else {
            crate::combat::lifecycle::hostiles_remain(combat)
        };
        if !moving {
            return Ok(None);
        }
        let monster = &combat.roster.records[(body - 10) as usize];
        if monster.record().movement() <= 0
            || monster.record().action_points() <= 1
            || monster.record().turn_skip_duration() != 0
        {
            break;
        }
        select_enemy_target(combat, warriors, mission, random, body)?;
        let moved = move_actor_with_reactions(
            combat,
            warriors,
            mission,
            random,
            body,
            items,
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
            events.extend(collide_fields_with_refresh(
                combat,
                warriors,
                mission,
                &mut globals.attack_count,
                random,
                (
                    body,
                    crate::combat::spells::loaded_item_restriction(&globals.buffers)?,
                    globals.char_select_new,
                    &mut globals.movement_gate,
                ),
                &mut [0; 6],
                Some((items, &mut globals.buffers)),
            )?);
        }
        let adjacent = check_adjacent(
            combat,
            warriors,
            random,
            turn,
            globals,
            &mut reactions.stopped,
        )?;
        reactions.take(adjacent, globals, events)?;
        if !moved.moved {
            return Ok(Some(true));
        }
    }
    Ok(Some(false))
}
