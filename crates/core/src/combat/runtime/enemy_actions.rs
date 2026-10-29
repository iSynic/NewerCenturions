//! Enemy actions in the original combat execution order.
use super::{
    FireOutcome, alive, check_neighbors_with_items, collide_fields_with_work, enemy_fire,
    enemy_spell, move_actor_with_reactions, position, select_enemy_target,
};
use crate::combat::Combat;
use crate::{Event, RandomState, RuleError};
use centurions_content::Warrior;

pub fn enemy_action(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
) -> Result<Vec<Event>, RuleError> {
    enemy_action_with_context(
        c,
        w,
        mission,
        random,
        items,
        &mut EnemyWork::default(),
        &[0; 20],
    )
}

pub struct EnemyWork {
    pub item_context: i16,
    pub attack_count: i16,
    pub packet: Vec<u8>,
    pub identity: [i16; 3],
    pub spell_attempt: i8,
    pub spell_disabled: u8,
    pub buffers: crate::ammunition::WorkBuffers,
    // Legacy helper fixtures omit combatupdate2. Session execution includes its writers.
    pub display_writers: bool,
    pub selected_warrior: usize,
    /// WORD438786, the movement-spell marker (zero at launch).
    pub movement_gate: i16,
    pub neighbor_mask: Option<[i16; 6]>,
    /// The party duration table at 442562 for an NPC global (mode7) cast.
    pub party_buffs: Option<Vec<u8>>,
    /// The animation-delay preference that lengthens a ray's step count.
    pub animation_delay: i16,
    /// The scenario-directory tail WORD44263e when it does not overlap the path text.
    pub scenario_tail: Option<i16>,
    /// The story flag BYTEs 44258c an indexed mode-7 duration can reach.
    pub flags: Option<Vec<u8>>,
    /// The current land's objects and events beside the map, for reads past its edges.
    pub land: Option<crate::combat::map_alias::Land>,
    /// Shop work 478288, written by an effect26/64 NPC alias.
    pub shop_work: Option<crate::shop::ShopWork>,
    /// The current map header 478e42, written by an effect26/64 NPC alias.
    pub event_headers: Option<Vec<u8>>,
    /// Warrior table records outside the squad; NPC bodies 10..13 index the backup.
    pub warrior_table: Option<crate::state::WarriorTable>,
}
impl Default for EnemyWork {
    fn default() -> Self {
        Self {
            item_context: 0,
            attack_count: 0,
            packet: crate::combat::attack_packet_startup(),
            identity: [0; 3],
            spell_attempt: 0,
            spell_disabled: 0,
            buffers: Default::default(),
            display_writers: false,
            selected_warrior: 0,
            movement_gate: 0,
            neighbor_mask: None,
            party_buffs: None,
            animation_delay: 0,
            scenario_tail: None,
            flags: None,
            land: None,
            shop_work: None,
            event_headers: None,
            warrior_table: None,
        }
    }
}

impl EnemyWork {
    /// Copies of the owners an effect26/64 NPC alias reads during one spell attempt.
    pub(crate) fn alias_snapshot(&self) -> crate::combat::warrior_alias::Snapshot {
        crate::combat::warrior_alias::Snapshot::new(crate::combat::warrior_alias::Owners {
            item_work: self.buffers.item.as_deref(),
            shop_work: self.shop_work.as_ref(),
            header: self.event_headers.as_deref(),
            warriors: None,
            table: self.warrior_table.as_ref(),
        })
    }

    /// Moves the attempt's pending writes into this work's owner copies.
    pub(crate) fn take_alias_writes(&mut self, c: &mut Combat) -> Result<(), RuleError> {
        let writes = std::mem::take(&mut c.alias_writes);
        crate::combat::warrior_alias::commit(
            self.buffers.item.as_mut(),
            self.shop_work.as_mut(),
            self.warrior_table.as_mut(),
            self.event_headers.as_mut(),
            &writes,
        )
    }
}

pub fn enemy_action_with_work(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    work: &mut EnemyWork,
) -> Result<Vec<Event>, RuleError> {
    enemy_action_with_work_and_buffs(c, w, mission, random, items, work, &[0; 20])
}

pub fn enemy_action_with_work_and_buffs(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    work: &mut EnemyWork,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    c.attack_packet = work.packet.clone();
    c.attack_identity = work.identity;
    let events = enemy_action_with_context(c, w, mission, random, items, work, buffs)?;
    work.packet = c.attack_packet.clone();
    work.identity = c.attack_identity;
    Ok(events)
}

pub(super) fn record_attack_context(work: &mut EnemyWork) -> Result<(), RuleError> {
    if let Some(item) = work.buffers.item.as_ref() {
        work.item_context = centurions_content::records::ItemDefinition(item).retained_field32()?;
    }
    Ok(())
}

pub(super) fn enemy_action_with_context(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    work: &mut EnemyWork,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    let body = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active enemy missing".into()))?;
    if body < 10 {
        return Err(RuleError::Invalid("active body is a warrior".into()));
    }
    let e = (body - 10) as usize;
    if c.enemies.records[e].record().faction()? == 0
        && c.enemies.records[e].record().primary_weapon()? == 1
    {
        return Err(RuleError::Unsupported {
            action: "combat.scientist",
            dependency: "work-item context requires a session",
        });
    }
    let mut events = vec![];
    if c.enemies.records[e].record().action_points() < 2 {
        return Ok(events);
    }
    let mut attempt = 0;
    let mut interrupted = false;
    let mut reactions_stopped = false;
    for _ in 0..32 {
        let monster = &mut c.enemies.records[e];
        if monster.record().action_points() < 2 {
            return Ok(events);
        }
        let reaction_ready = monster.record().action_points() >= 1;
        monster.record_mut().set_reaction_ready(reaction_ready);
        let fleeing = monster.record().fleeing_duration() != 0;
        let mut skip_neighbors = false;
        let firearm_chance = monster.record().firearm_chance()?;
        if !fleeing && random.bound(100) <= firearm_chance {
            match enemy_fire(c, w, mission, random, (items, work), body, &mut events)? {
                FireOutcome::Physical => continue,
                FireOutcome::Spell => {
                    attempt += 1;
                    if c.enemies.records[e].record().firearm_chance()? != 0
                        && c.enemies.records[e].record().action_points() > 1
                        && attempt < 2
                        && enemy_fire(c, w, mission, random, (items, work), body, &mut events)?
                            == FireOutcome::Physical
                    {
                        continue;
                    }
                    return Ok(events);
                }
                FireOutcome::Failed => {}
            }
            attempt += 1;
        } else if !fleeing {
            let spell_chance = monster.record().spell_chance()?;
            if random.bound(100) <= spell_chance {
                let blocked =
                    work.spell_disabled != 0 || monster.record().spell_blocked_before_movement();
                skip_neighbors = blocked;
                let cast = if blocked {
                    false
                } else {
                    attempt += 1;
                    enemy_spell(c, w, mission, random, (items, work), body, &mut events)?
                };
                if cast {
                    if c.enemies.records[e].record().firearm_chance()? != 0
                        && c.enemies.records[e].record().action_points() > 1
                        && attempt < 2
                        && enemy_fire(c, w, mission, random, (items, work), body, &mut events)?
                            == FireOutcome::Physical
                    {
                        continue;
                    }
                    return Ok(events);
                }
            }
        }
        let adjacent =
            if fleeing || skip_neighbors || c.enemies.records[e].record().turn_skip_duration() != 0
            {
                super::reaction_calls::ReactionResult::default()
            } else {
                check_neighbors_with_items(
                    c,
                    w,
                    random,
                    body,
                    Some(items),
                    Some(&mission.combat.as_ref().unwrap().attacks),
                    &mut reactions_stopped,
                    buffs,
                    &mut work.buffers,
                    work.display_writers,
                    work.neighbor_mask.as_mut().map(|mask| {
                        crate::combat::neighbor_work::NativeNeighbors {
                            selected_warrior: work.selected_warrior,
                            party_mask: mask,
                            aliases: crate::combat::map_alias::land_aliases(
                                mission,
                                work.land.as_ref(),
                            ),
                            table: work.warrior_table.as_ref(),
                        }
                    }),
                )?
            };
        interrupted |= adjacent
            .events
            .iter()
            .any(|event| matches!(event, Event::AttackResolved { .. }));
        record_attack_context(work)?;
        events.extend(adjacent.events);
        let mut movement_failed = false;
        for _ in 0..128 {
            if reactions_stopped {
                break;
            }
            if !alive(c, w, body) || !w.iter().any(|w| w.record().active_flag() != 0) {
                break;
            }
            let moving = if work.neighbor_mask.is_some() {
                c.victory_work.movement_admitted()
            } else {
                crate::combat::lifecycle::hostiles_remain(c)
            };
            if !moving {
                return Ok(events);
            }
            let monster = &c.enemies.records[e];
            if monster.record().movement_points() <= 0
                || monster.record().action_points() <= 1
                || monster.record().turn_skip_duration() != 0
            {
                break;
            }
            select_enemy_target(c, w, mission, random, body)?;
            let moved = move_actor_with_reactions(
                c,
                w,
                mission,
                random,
                body,
                items,
                buffs,
                &mut work.buffers,
                work.display_writers,
                work.neighbor_mask.as_mut().map(|mask| {
                    crate::combat::neighbor_work::NativeNeighbors {
                        selected_warrior: work.selected_warrior,
                        party_mask: mask,
                        aliases: crate::combat::map_alias::land_aliases(
                            mission,
                            work.land.as_ref(),
                        ),
                        table: work.warrior_table.as_ref(),
                    }
                }),
                &mut events,
            )?;
            events.extend(
                moved
                    .sounds
                    .into_iter()
                    .map(|resource_id| Event::Sound { resource_id }),
            );
            if moved.moved {
                let p = position(c, body);
                events.push(Event::TacticalMoved {
                    body,
                    x: p.0,
                    y: p.1,
                });
                events.extend(collide_fields_with_work(
                    c,
                    w,
                    mission,
                    &mut work.attack_count,
                    random,
                    (
                        body,
                        crate::combat::spells::retained_item_alias(&work.buffers)?,
                        work.selected_warrior,
                        &mut work.movement_gate,
                    ),
                    &mut [0; 6],
                    Some((items, &mut work.buffers)),
                )?);
            }
            let adjacent = check_neighbors_with_items(
                c,
                w,
                random,
                body,
                Some(items),
                Some(&mission.combat.as_ref().unwrap().attacks),
                &mut reactions_stopped,
                buffs,
                &mut work.buffers,
                work.display_writers,
                work.neighbor_mask.as_mut().map(|mask| {
                    crate::combat::neighbor_work::NativeNeighbors {
                        selected_warrior: work.selected_warrior,
                        party_mask: mask,
                        aliases: crate::combat::map_alias::land_aliases(
                            mission,
                            work.land.as_ref(),
                        ),
                        table: work.warrior_table.as_ref(),
                    }
                }),
            )?;
            interrupted |= adjacent
                .events
                .iter()
                .any(|event| matches!(event, Event::AttackResolved { .. }));
            record_attack_context(work)?;
            events.extend(adjacent.events);
            if !moved.moved {
                movement_failed = true;
                break;
            }
        }
        let monster = &c.enemies.records[e];
        if attempt < 2 && monster.record().spell_chance()? != 0 && !interrupted {
            c.enemies.records[e].record_mut().set_movement_points(0);
            let monster = &c.enemies.records[e];
            let blocked =
                work.spell_disabled != 0 || monster.record().spell_blocked_after_movement();
            if !blocked {
                attempt = if movement_failed { attempt + 2 } else { 2 };
                enemy_spell(c, w, mission, random, (items, work), body, &mut events)?;
                attempt += 1;
            }
        }
        let monster = &c.enemies.records[e];
        if monster.record().firearm_chance()? == 0
            || monster.record().action_points() < 2
            || attempt > 1
        {
            return Ok(events);
        }
        attempt += 1;
        if enemy_fire(c, w, mission, random, (items, work), body, &mut events)?
            == FireOutcome::Physical
        {
            continue;
        }
        return Ok(events);
    }
    Err(RuleError::Invalid(
        "enemy action exceeded point budget".into(),
    ))
}
