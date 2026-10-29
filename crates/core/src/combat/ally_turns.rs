use crate::combat::{Combat, runtime};
use crate::{Event, RandomState, RuleError};
use centurions_content::{Warrior, records::ItemDefinition};

fn position(c: &Combat, body: i16) -> (i16, i16) {
    if body < 10 {
        c.squad.positions[body as usize]
    } else {
        c.enemies.positions[(body - 10) as usize]
    }
}
fn alive(c: &Combat, w: &[Warrior], body: i16) -> bool {
    if (0..6).contains(&body) {
        w[body as usize].record().active_flag() != 0
    } else if body >= 10 {
        c.enemies
            .records
            .get((body - 10) as usize)
            .is_some_and(|m| m.stamina().unwrap_or(0) > 0 && !m.record().removed())
    } else {
        false
    }
}
fn faction(c: &Combat, w: &[Warrior], body: i16) -> Result<i16, RuleError> {
    if body < 10 {
        Ok(w[body as usize].record().faction()?)
    } else {
        Ok(c.enemies.records[(body - 10) as usize].record().faction()?)
    }
}

pub fn item_context_after_load(items: &[Vec<u8>], id: i16) -> Result<i16, RuleError> {
    let index = usize::try_from(id.wrapping_abs())
        .map_err(|_| RuleError::Invalid("invalid item context".into()))?;
    let item = items
        .get(index)
        .ok_or_else(|| RuleError::Invalid("item context missing".into()))?;
    Ok(ItemDefinition(item).retained_field32()?)
}

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
pub(crate) use natural::{NpcMelee, Struck, natural_attack_enemy_with_processing, natural_core};
pub use natural::{
    natural_attack_enemy, natural_attack_enemy_with_actor, natural_attack_enemy_with_packet,
    result_audio,
};

mod firearm;
use firearm::fire;
pub(crate) use firearm::{
    Duel, ScientistWorld, Shot, fire as fire_scientist, fire_in as fire_scientist_in,
};

pub fn scientist_turn_with_item_context(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    item_context: i16,
) -> Result<Vec<Event>, RuleError> {
    scientist_turn_with_item_context_and_buffs(c, w, mission, random, items, item_context, &[0; 20])
}

pub fn scientist_turn_with_item_context_and_buffs(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    item_context: i16,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    scientist_turn_with_work(
        c,
        w,
        mission,
        random,
        items,
        &mut runtime::EnemyWork {
            item_context,
            display_writers: false,
            ..Default::default()
        },
        buffs,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn scientist_turn_with_work(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    work: &mut runtime::EnemyWork,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    let mut item_context = work.item_context;
    let body = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active scientist missing".into()))?;
    if body < 10 {
        return Err(RuleError::Invalid("active scientist is warrior".into()));
    }
    let e = (body - 10) as usize;
    if c.enemies.records[e].record().faction()? != 0
        || c.enemies.records[e].record().primary_weapon()? != 1
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
    let mut events = vec![];
    let mut attempt = 0;
    for _ in 0..32 {
        let actor = &mut c.enemies.records[e];
        let action_points = actor.record().action_points();
        if action_points < 2 {
            return Ok(events);
        }
        actor.record_mut().set_reaction_ready(action_points >= 1);
        let m = actor.record();
        let fleeing = m.fleeing_duration() != 0;
        let mut skip_neighbors = false;
        if !fleeing && random.bound(100) <= m.firearm_chance()? {
            if fire(
                c,
                w,
                mission,
                random,
                items,
                &mut events,
                (item_context, body),
                &mut work.buffers,
                work.display_writers,
                &mut work.attack_count,
            )? {
                if let Some(item) = work.buffers.item.as_ref() {
                    item_context = ItemDefinition(item).retained_field32()?;
                    work.item_context = item_context;
                }
                continue;
            }
            attempt += 1;
        } else if !fleeing && random.bound(100) <= m.spell_chance()? {
            if work.spell_disabled != 0 || m.spell_blocked_before_movement() {
                skip_neighbors = true;
            } else {
                attempt += 1;
                if runtime::enemy_spell(c, w, mission, random, (items, work), body, &mut events)? {
                    if let Some(item) = work.buffers.item.as_ref() {
                        item_context = ItemDefinition(item).retained_field32()?;
                        work.item_context = item_context;
                    }
                    let m = c.enemies.records[e].record();
                    if m.firearm_chance()? != 0
                        && m.action_points() >= 2
                        && attempt < 2
                        && fire(
                            c,
                            w,
                            mission,
                            random,
                            items,
                            &mut events,
                            (item_context, body),
                            &mut work.buffers,
                            work.display_writers,
                            &mut work.attack_count,
                        )?
                    {
                        if let Some(item) = work.buffers.item.as_ref() {
                            item_context = ItemDefinition(item).retained_field32()?;
                            work.item_context = item_context;
                        }
                        continue;
                    }
                    return Ok(events);
                }
            }
        }
        events.extend(if fleeing || skip_neighbors {
            vec![]
        } else if work.display_writers {
            runtime::check_neighbors_with_work(
                c,
                w,
                random,
                body,
                items,
                attacks,
                buffs,
                &mut work.buffers,
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
            .events
        } else {
            runtime::check_neighbors_with_buffs(c, w, random, body, items, attacks, buffs)?
        });
        if let Some(item) = work.buffers.item.as_ref() {
            item_context = ItemDefinition(item).retained_field32()?;
            work.item_context = item_context;
        }
        for _ in 0..128 {
            if !alive(c, w, body) || !w.iter().any(|w| w.record().active_flag() != 0) {
                break;
            }
            let m = c.enemies.records[e].record();
            if m.movement_points() <= 0 || m.action_points() <= 1 || m.turn_skip_duration() != 0 {
                break;
            }
            runtime::select_enemy_target(c, w, mission, random, body)?;
            let moved = runtime::move_actor_with_reactions(
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
            }
            events.extend(if work.display_writers {
                runtime::check_neighbors_with_work(
                    c,
                    w,
                    random,
                    body,
                    items,
                    attacks,
                    buffs,
                    &mut work.buffers,
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
                .events
            } else {
                runtime::check_neighbors_with_buffs(c, w, random, body, items, attacks, buffs)?
            });
            if let Some(item) = work.buffers.item.as_ref() {
                item_context = ItemDefinition(item).retained_field32()?;
                work.item_context = item_context;
            }
            if !moved.moved {
                break;
            }
        }
        let m = c.enemies.records[e].record();
        // combat115548..1155ee retries casting after movement, then reaches
        // the distinct outer firearm retry at 0x115732.
        if m.spell_chance()? != 0 && m.action_points() >= 2 && attempt < 2 {
            c.enemies.records[e].record_mut().set_movement_points(0);
            let m = c.enemies.records[e].record();
            if work.spell_disabled == 0 && !m.spell_blocked_after_movement() {
                attempt = 2;
                runtime::enemy_spell(c, w, mission, random, (items, work), body, &mut events)?;
                if let Some(item) = work.buffers.item.as_ref() {
                    item_context = ItemDefinition(item).retained_field32()?;
                    work.item_context = item_context;
                }
            }
        }
        work.item_context = item_context;
        let m = c.enemies.records[e].record();
        if work.display_writers && m.firearm_chance()? != 0 && m.action_points() >= 2 && attempt < 2
        {
            attempt += 1;
            if fire(
                c,
                w,
                mission,
                random,
                items,
                &mut events,
                (item_context, body),
                &mut work.buffers,
                work.display_writers,
                &mut work.attack_count,
            )? {
                if let Some(item) = work.buffers.item.as_ref() {
                    item_context = ItemDefinition(item).retained_field32()?;
                    work.item_context = item_context;
                }
                continue;
            }
        }
        return Ok(events);
    }
    Err(RuleError::Invalid("scientist action point budget".into()))
}

/// Execute the scientist with a zero loaditem work-record ammo selector.
pub fn scientist_turn(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
) -> Result<Vec<Event>, RuleError> {
    scientist_turn_with_item_context(c, w, mission, random, items, 0)
}
