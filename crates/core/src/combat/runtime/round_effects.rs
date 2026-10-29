//! Round effects in the original combat execution order.
use super::{
    EnemyRoundOutcome, FieldActor, collide_fields_impl, record_spell_deaths,
    reduce_enemy_round_impl,
};
use crate::combat::{Combat, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::{Warrior, records::Condition};

/// Without a mission no field collides, so WORD438786 is never read.
pub fn reduce_round(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
) -> Result<Vec<Event>, RuleError> {
    reduce_round_with_fields(
        c,
        w,
        random,
        None,
        &mut 0,
        &mut [0; 6],
        RoundContext {
            audio: false,
            selected_warrior: 0,
            energy_recovery_disabled: 0,
            item_work_alias: None,
            processing: None,
            movement_gate: &mut 0,
            items: None,
        },
    )
    .map(|outcome| outcome.events)
}

pub(super) struct RoundContext<'a> {
    pub(super) energy_recovery_disabled: u8,
    pub(super) audio: bool,
    pub(super) selected_warrior: usize,
    pub(super) item_work_alias: Option<i16>,
    pub(super) processing: Option<crate::combat::death_scripts::DeathProcessing>,
    /// WORD438786 for movement-field collisions.
    pub(super) movement_gate: &'a mut i16,
    /// combatupdate2's ITEM work for a full-roster summon refusal.
    pub(super) items: crate::combat::spell_summon::ItemRefresh<'a>,
}

pub(super) fn reduce_round_with_fields(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_mask: &mut [i16; 6],
    mut context: RoundContext<'_>,
) -> Result<EnemyRoundOutcome, RuleError> {
    let mut events = vec![];
    let mut last_defeated_body = None;
    for member in 0..w.len() {
        let warrior = &mut w[member];
        let health = warrior.record().stamina()?;
        let regeneration = warrior.record().condition(Condition::Regenerating)?;
        if regeneration != 0
            && health > 0
            && health < warrior.record().maximum_stamina()?
            && warrior.record().automatic_mode()? == 0
        {
            let healed = health
                .wrapping_add(regeneration.wrapping_abs())
                .min(warrior.record().maximum_stamina()?);
            warrior.record_mut().set_stamina(healed);
            if context.audio && healed != health && warrior.record().active_flag() != 0 {
                events.push(Event::Sound { resource_id: 624 });
            }
        }
        if let Some(tick) = crate::combat::natural_conditions::disease_tick(&w[member].bytes)? {
            w[member].record_mut().set_stamina(tick.health);
            if tick.killed {
                lifecycle::kill_body(c, w, member as i16)?;
                last_defeated_body = Some(member as i16);
            }
        }
        for (condition, amount, address) in [
            (Condition::SeventhLevelProtection, None, 0x401948),
            (Condition::Speedy, Some(2), 0x40193a),
        ] {
            let before = w[member].record().stamina()?;
            let condition = w[member].record().condition(condition)?;
            if condition != 0 && before > 0 {
                let amount = amount.unwrap_or(condition.wrapping_abs());
                let health = before.wrapping_sub(amount);
                w[member].record_mut().set_stamina(health);
                if health < 1 {
                    lifecycle::kill_body(c, w, member as i16)?;
                    last_defeated_body = Some(member as i16);
                    events.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
                        address,
                        x: 100,
                        y: 30,
                        wait: 100,
                        sound: 10122,
                    }));
                }
            }
        }
        if let Some(tick) = crate::combat::natural_conditions::poison_tick(&w[member].bytes)? {
            w[member].record_mut().set_stamina(tick.health);
            if tick.killed {
                lifecycle::kill_body(c, w, member as i16)?;
                last_defeated_body = Some(member as i16);
            }
        }
        let warrior = &mut w[member];
        crate::story_reduce::energy(&mut warrior.bytes, context.energy_recovery_disabled)?;
        if warrior.record().confusion_duration()? != 0 && warrior.record().faction()? != 0 {
            warrior.record_mut().set_faction(0);
            c.unavailable = c.unavailable.wrapping_sub(1);
            c.victory_work.add_total(-1);
        }
        warrior.record_mut().decrement_temporary_conditions()?;
    }
    for member in 0..w.len() {
        c.field_used = crate::combat::spells::reset_used(&c.field_used)?;
        if w[member].record().stamina()? > 0 {
            if let Some(mission) = mission {
                let collision_events = collide_fields_impl(
                    c,
                    w,
                    mission,
                    count,
                    random,
                    FieldActor {
                        body: member as i16,
                        selected_warrior: context.selected_warrior,
                        item_work_alias: context.item_work_alias,
                        movement_gate: &mut *context.movement_gate,
                        items: context
                            .items
                            .as_mut()
                            .map(|(items, buffers)| (*items, &mut **buffers)),
                    },
                    party_mask,
                )?;
                record_spell_deaths(&collision_events, &mut last_defeated_body);
                events.extend(collision_events);
            } else if c
                .spell_fields
                .as_chunks::<20>()
                .0
                .iter()
                .any(|field| field[15] != 0)
            {
                return Err(RuleError::Unsupported {
                    action: "combat.round",
                    dependency: "spell field content missing",
                });
            }
        }
        let warrior = &mut w[member];
        warrior.record_mut().clear_damage_marker();
        if warrior.record().bleeding_flag() != 0 && warrior.record().stamina()? > -10 {
            let health = warrior.record().stamina()?.wrapping_sub(1);
            warrior.record_mut().set_stamina(health);
            if health < -9 {
                warrior.record_mut().set_bleeding_flag(0);
                warrior.record_mut().record_bleeding_death();
                events.push(Event::Sound {
                    resource_id: random.bound(2).wrapping_add(506),
                });
            }
        }
    }
    let enemy = reduce_enemy_round_impl(
        c,
        w,
        random,
        mission,
        count,
        party_mask,
        (
            context.selected_warrior,
            context.item_work_alias,
            context.movement_gate,
            context.items,
        ),
        super::enemy_round::EnemyRoundCursor::default(),
        context.processing,
    )?;
    events.extend(enemy.events);
    Ok(EnemyRoundOutcome {
        events,
        last_defeated_body: enemy.last_defeated_body.or(last_defeated_body),
        suspended: enemy.suspended,
    })
}
