//! Conditions and fields that act at the end of each round.
use super::{
    EnemyRoundOutcome, FieldActor, collide_fields_impl, record_spell_deaths,
    reduce_enemy_round_impl,
};
use crate::combat::death_scripts::DeathProcessing;
use crate::combat::natural_conditions;
use crate::combat::spells::summon::ItemRefresh;
use crate::combat::{Combat, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::{Warrior, records::Condition};

/// Without a mission no field acts, so the movement-spell marker is never read.
pub fn reduce_round(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
) -> Result<Vec<Event>, RuleError> {
    reduce_round_with_fields(
        combat,
        warriors,
        random,
        None,
        &mut 0,
        &mut [0; 6],
        RoundContext {
            audio: false,
            char_select_new: 0,
            energy_recovery_disabled: 0,
            item_restriction: None,
            processing: None,
            movement_gate: &mut 0,
            items: None,
        },
    )
    .map(|outcome| outcome.events)
}

/// What the end of a round needs besides the battle: whether energy recovery is disabled, whether
/// sounds play, the selected warrior, the item restriction, death processing and the
/// movement-spell marker.
pub(super) struct RoundContext<'a> {
    pub(super) energy_recovery_disabled: u8,
    pub(super) audio: bool,
    pub(super) char_select_new: usize,
    pub(super) item_restriction: Option<i16>,
    pub(super) processing: Option<DeathProcessing>,
    /// The movement-spell marker that movement fields test.
    pub(super) movement_gate: &'a mut i16,
    /// The item work refreshed when a full roster refuses a summon.
    pub(super) items: ItemRefresh<'a>,
}

/// Applies the end of a round to each warrior and then to the monsters. A warrior regenerates;
/// disease, seventh-level protection, speed and poison cost stamina and kill a warrior who runs
/// out; energy recovers, a confused warrior of the other faction returns and temporary conditions
/// tick down. Spell fields then act on each warrior still standing, and bleeding costs one
/// stamina point until the warrior bleeds to death.
///
/// Original: getup
pub(super) fn reduce_round_with_fields(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    mission: Option<&centurions_content::mission::OpeningMission>,
    count: &mut i16,
    party_track: &mut [i16; 6],
    mut context: RoundContext<'_>,
) -> Result<EnemyRoundOutcome, RuleError> {
    let mut events = vec![];
    let mut last_defeated_body = None;
    for member in 0..warriors.len() {
        let warrior = &mut warriors[member];
        let health = warrior.record().stamina()?;
        let regeneration = warrior.record().condition(Condition::Regenerating)?;
        if regeneration != 0
            && health > 0
            && health < warrior.record().stamina_max()?
            && warrior.record().computer_control()? == 0
        {
            let healed = health
                .wrapping_add(regeneration.wrapping_abs())
                .min(warrior.record().stamina_max()?);
            warrior.record_mut().set_stamina(healed);
            if context.audio && healed != health && warrior.record().in_battle() != 0 {
                events.push(Event::Sound { resource_id: 624 });
            }
        }
        if let Some(tick) = natural_conditions::disease_tick(&warriors[member].bytes)? {
            warriors[member].record_mut().set_stamina(tick.health);
            if tick.killed {
                lifecycle::kill_body(combat, warriors, member as i16)?;
                last_defeated_body = Some(member as i16);
            }
        }
        for (condition, amount, address) in [
            (Condition::SeventhLevelProtection, None, 0x401948),
            (Condition::Speedy, Some(2), 0x40193a),
        ] {
            let before = warriors[member].record().stamina()?;
            let condition = warriors[member].record().condition(condition)?;
            if condition != 0 && before > 0 {
                let amount = amount.unwrap_or(condition.wrapping_abs());
                let health = before.wrapping_sub(amount);
                warriors[member].record_mut().set_stamina(health);
                if health < 1 {
                    lifecycle::kill_body(combat, warriors, member as i16)?;
                    last_defeated_body = Some(member as i16);
                    events.push(Event::Feedback(crate::feedback::GameFeedback::Flash {
                        address,
                        x: 100,
                        y: 30,
                        wait: 100,
                        sound: 10122,
                    }));
                }
            }
        }
        if let Some(tick) = natural_conditions::poison_tick(&warriors[member].bytes)? {
            warriors[member].record_mut().set_stamina(tick.health);
            if tick.killed {
                lifecycle::kill_body(combat, warriors, member as i16)?;
                last_defeated_body = Some(member as i16);
            }
        }
        let warrior = &mut warriors[member];
        crate::story::reduce::energy(&mut warrior.bytes, context.energy_recovery_disabled)?;
        if warrior.record().confusion_duration()? != 0 && warrior.record().faction()? != 0 {
            warrior.record_mut().set_faction(0);
            combat.kill_party = combat.kill_party.wrapping_sub(1);
            combat.kill_counts.add_num_enemy(-1);
        }
        warrior.record_mut().decrement_temporary_conditions()?;
    }
    for member in 0..warriors.len() {
        combat.collide_check = crate::combat::spells::reset_used(&combat.collide_check)?;
        if warriors[member].record().stamina()? > 0 {
            if let Some(mission) = mission {
                let collision_events = collide_fields_impl(
                    combat,
                    warriors,
                    mission,
                    count,
                    random,
                    FieldActor {
                        body: member as i16,
                        char_select_new: context.char_select_new,
                        item_restriction: context.item_restriction,
                        movement_gate: &mut *context.movement_gate,
                        items: context
                            .items
                            .as_mut()
                            .map(|(items, buffers)| (*items, &mut **buffers)),
                    },
                    party_track,
                )?;
                record_spell_deaths(&collision_events, &mut last_defeated_body);
                events.extend(collision_events);
            } else if combat
                .spell_queue
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
        let warrior = &mut warriors[member];
        warrior.record_mut().clear_been_attacked();
        if warrior.record().bleeding() != 0 && warrior.record().stamina()? > -10 {
            let health = warrior.record().stamina()?.wrapping_sub(1);
            warrior.record_mut().set_stamina(health);
            if health < -9 {
                warrior.record_mut().set_bleeding(0);
                warrior.record_mut().record_bleeding_death();
                events.push(Event::Sound {
                    resource_id: random.bound(2).wrapping_add(506),
                });
            }
        }
    }
    let monster = reduce_enemy_round_impl(
        combat,
        warriors,
        random,
        mission,
        count,
        party_track,
        (
            context.char_select_new,
            context.item_restriction,
            context.movement_gate,
            context.items,
        ),
        super::enemy_round::EnemyRoundCursor::default(),
        context.processing,
    )?;
    events.extend(monster.events);
    Ok(EnemyRoundOutcome {
        events,
        last_defeated_body: monster.last_defeated_body.or(last_defeated_body),
        suspended: monster.suspended,
    })
}
