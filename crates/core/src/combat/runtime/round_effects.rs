//! Round effects in the original combat execution order.
use super::{
    EnemyRoundOutcome, FieldActor, collide_fields_impl, record_spell_deaths,
    reduce_enemy_round_impl,
};
use crate::combat::{Combat, lifecycle, put_word};
use crate::{Event, RandomState, RuleError};
use centurions_content::{Warrior, word};

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
        },
    )
    .map(|outcome| outcome.events)
}

pub(super) struct RoundContext {
    pub(super) energy_recovery_disabled: u8,
    pub(super) audio: bool,
    pub(super) selected_warrior: usize,
}

pub(super) fn reduce_round_with_fields(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_mask: &mut [i16; 6],
    context: RoundContext,
) -> Result<EnemyRoundOutcome, RuleError> {
    let mut events = vec![];
    let mut last_defeated_body = None;
    for member in 0..w.len() {
        let b = &mut w[member].bytes;
        let health = word(b, 0x50)?;
        if word(b, 0x66)? != 0 && health > 0 && health < word(b, 0x52)? && word(b, 0x90)? == 0 {
            let healed = health
                .wrapping_add(word(b, 0x66)?.wrapping_abs())
                .min(word(b, 0x52)?);
            put_word(b, 0x50, healed);
            if context.audio && healed != health && b[0x9df] != 0 {
                events.push(Event::Sound { resource_id: 624 });
            }
        }
        if let Some(tick) = crate::combat::natural_conditions::disease_tick(&w[member].bytes)? {
            put_word(&mut w[member].bytes, 0x50, tick.health);
            if tick.killed {
                lifecycle::kill_body(c, w, member as i16)?;
                last_defeated_body = Some(member as i16);
            }
        }
        for (offset, amount, address) in [(0x8a, None, 0x401948), (0x8c, Some(2), 0x40193a)] {
            let before = w[member].record().stamina()?;
            let condition = word(&w[member].bytes, offset)?;
            if condition != 0 && before > 0 {
                let amount = amount.unwrap_or(condition.wrapping_abs());
                let health = before.wrapping_sub(amount);
                put_word(&mut w[member].bytes, 0x50, health);
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
            put_word(&mut w[member].bytes, 0x50, tick.health);
            if tick.killed {
                lifecycle::kill_body(c, w, member as i16)?;
                last_defeated_body = Some(member as i16);
            }
        }
        let b = &mut w[member].bytes;
        crate::story_reduce::energy(b, context.energy_recovery_disabled)?;
        if word(b, 0x98)? != 0 && word(b, 0x10)? != 0 {
            put_word(b, 0x10, 0);
            c.unavailable = c.unavailable.wrapping_sub(1);
        }
        for offset in (0x5e..0xae).step_by(2) {
            let value = word(b, offset)?;
            if value > 0 {
                put_word(b, offset, value - 1);
            }
        }
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
                        selected_warrior: Some(context.selected_warrior),
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
        let b = &mut w[member].bytes;
        put_word(b, 0x14, 0);
        if b[0x9de] != 0 && word(b, 0x50)? > -10 {
            let health = word(b, 0x50)?.wrapping_sub(1);
            put_word(b, 0x50, health);
            if health < -9 {
                b[0x9de] = 0;
                for (offset, delta) in [(0xa62, -1_i32), (0xa5e, 1)] {
                    let value = i32::from_be_bytes(b[offset..offset + 4].try_into().unwrap())
                        .wrapping_add(delta);
                    b[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
                }
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
        context.selected_warrior,
    )?;
    events.extend(enemy.events);
    Ok(EnemyRoundOutcome {
        events,
        last_defeated_body: enemy.last_defeated_body.or(last_defeated_body),
    })
}
