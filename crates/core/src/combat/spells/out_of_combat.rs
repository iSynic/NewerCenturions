//! Spell resolution outside battle, in the same body order and with field usage.
use super::{
    DirectSpell, FieldUsage, RestrictionMemoryView, SupportSpellResult, effects, is_support,
    monster_race_is_excluded, race_is_excluded,
    support::{target::damage_and_conditions, train_party},
    support_rules::status_resisted_with_feedback,
};
use crate::{
    Event, RandomState, RuleError, combat::outdoor_health, feedback::GameFeedback, story_health,
};
use centurions_content::{Warrior, combat::MonsterRecord, records::SpellInfoRecord};

mod health;
use self::health::animation;

/// What an out-of-combat spell needs: the spell, its number, the current body, the restriction
/// memory, where the party stands, the animation delay, the outdoor health, and whether it is a
/// field collision.
pub(crate) struct Context<'a> {
    pub spell: DirectSpell<'a>,
    pub id: i16,
    pub current: Option<i16>,
    pub memory: RestrictionMemoryView<'a>,
    pub location: (i16, i16),
    pub animation_delay: i16,
    pub outdoor_health: outdoor_health::View<'a>,
    pub collision: bool,
    /// Set by getup when a movement spell may move the party; combat choices and
    /// result displays clear it.
    pub movement_gate: i16,
}

/// The spell arms that replace the body loop and write the tactical arrays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Arm {
    /// A movement spell while the movement marker is set.
    Movement,
    /// A summoning, which places a monster through spelllist.
    Summon,
}

/// Whether the spell returned normally or the party was lost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Control {
    Returned,
    PartyLost,
}

/// What a monster took from the spell, for the caster's statistics: the damage, whether it hit, and
/// whether it killed.
pub(crate) struct MonsterCredit {
    pub damage: i16,
    pub hit: bool,
    pub killed: bool,
}

/// What a spell cast out of combat did: the bodies, the health context, the events, any effect on
/// the party, who is credited, and any movement or summon still to run.
pub(crate) struct Outcome {
    /// Squad members first, then any held warrior table records the loop reached.
    pub bodies: SupportSpellResult,
    /// The squad size, which party defeat counts against.
    pub party: usize,
    pub health: story_health::HealthContext,
    pub events: Vec<Event>,
    pub party_effect: Option<(i16, i16)>,
    pub control: Control,
    pub monster_credit: Vec<MonsterCredit>,
    pub clear_departure: bool,
    pub monster_pick_written: bool,
    /// An arm the caller runs over its tactical arrays straight after this return.
    pub arm: Option<Arm>,
}

impl Outcome {
    /// Every body below 10 indexes the warrior table; bodies past the squad are the
    /// held records.
    fn hold(&mut self, body: usize, view: outdoor_health::View<'_>) -> Result<(), RuleError> {
        while self.bodies.warriors.len() <= body {
            let next = self.bodies.warriors.len();
            let record = view
                .scratch_character(next)
                .ok_or_else(|| RuleError::from("detached field body outside the warrior table"))?;
            self.bodies.warriors.push(record);
        }
        Ok(())
    }

    /// Shows what happened to a body, with a damage or effect number.
    fn show_result(&mut self, body: usize, code: i16, amount: i16) {
        self.clear_departure = true;
        self.events
            .push(Event::Feedback(GameFeedback::CombatResult {
                body: body as i16,
                code,
                amount,
                audible: false,
            }));
    }
}

/// The arm a packet takes in place of the body loop, before the movement marker is
/// tested: effects 55 and 56 move, and every other non-support packet that is
/// neither direct weapon damage nor a category-5 drain summons.
///
/// Original: resolvespell
pub(crate) fn arm_of(packet: &[u8]) -> Result<Option<Arm>, RuleError> {
    let attack = SpellInfoRecord(packet);
    let effect = attack.effect()?.wrapping_abs();
    if effects::is_movement(effect) {
        return Ok(Some(Arm::Movement));
    }
    let drain = effects::is_psi_transfer(effect) && attack.defense_category() == 5;
    let summon = !is_support(packet)? && !attack.is_direct_weapon_damage()? && !drain;
    Ok(summon.then_some(Arm::Summon))
}

/// Rolls a spell's duration before either global or body effects.
///
/// Original: resolvespell
pub(crate) fn duration(packet: &[u8], count: i16, random: &mut RandomState) -> i16 {
    let attack = SpellInfoRecord(packet);
    let (minimum, maximum) = attack.duration_range();
    let mut duration = random.range(minimum, maximum);
    let (minimum, maximum) = attack.power_duration_range();
    if minimum != 0 {
        for _ in 0..count {
            duration = duration.wrapping_add(random.range(minimum, maximum));
        }
    }
    duration
}

/// Resolves a spell cast out of combat on the bodies its mask picks, in the same order as in
/// battle, and tracks the party's losses.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16; 110],
    usage: &mut FieldUsage,
    context: &Context<'_>,
    previous: &story_health::HealthContext,
    random: &mut RandomState,
) -> Result<Outcome, RuleError> {
    let spell = &context.spell;
    let attack = SpellInfoRecord(spell.packet);
    let effect = attack.effect()?.wrapping_abs();
    let physical = attack.is_direct_weapon_damage()?;
    let movement = effects::is_movement(effect);
    // Category-5 drains move Psi through spelllist like the other drains.
    let drain = effects::is_psi_transfer(effect) && attack.defense_category() == 5;
    // Only effect 58 is left outside these; its arm follows the movement arm.
    let summon = !is_support(spell.packet)? && !physical && !movement && !drain;
    let duration = duration(spell.packet, spell.count, random);
    let mut outcome = Outcome {
        bodies: SupportSpellResult::new(warriors, enemies, vec![]),
        party: warriors.len(),
        health: previous.clone(),
        events: vec![],
        party_effect: None,
        control: Control::Returned,
        monster_credit: vec![],
        clear_departure: false,
        monster_pick_written: false,
        arm: None,
    };
    outcome.health.effect = attack.health_effect();
    if attack.is_party_duration() {
        outcome.party_effect = Some((effect, duration));
        return Ok(outcome);
    }
    if effect == 66 {
        train_party(&mut outcome.bodies, &attack, mask, random)?;
    }
    // The movement arm replaces the body loop. With the marker clear it only warns;
    // a set marker moves the selected warrior over the tactical arrays.
    if movement {
        if context.movement_gate != 0 {
            outcome.arm = Some(Arm::Movement);
        } else {
            outcome
                .events
                .push(Event::Feedback(GameFeedback::Warning { index: 87 }));
        }
        return Ok(outcome);
    }
    if summon {
        outcome.arm = Some(Arm::Summon);
        return Ok(outcome);
    }
    // The usage byte just past the sixty field records is reset too.
    usage.used[61] = 0;
    let mut amount = 0;
    for (body, selected) in mask.iter().enumerate() {
        if *selected <= 0 {
            continue;
        }
        // For the Upgrade records, bodies 6 and 7, the health mark would land on the
        // spell's duration and round counters (docs/source-notes/combat.md).
        if (6..8).contains(&body) {
            return Err(unavailable("detached field body in an Upgrade record"));
        }
        if (8..10).contains(&body) {
            // Bodies 8 and 9 index the editor backup.
            return Err(unavailable("detached field body in the editor backup"));
        }
        if body >= 10 + enemies.len() {
            return Err(unavailable(
                "detached field body past the loaded NPC records",
            ));
        }
        context.memory.require_monster(body)?;
        if body < 10 {
            outcome.hold(body, context.outdoor_health)?;
        }
        if body >= 10 {
            outcome.monster_pick_written = true;
        }
        let record = if body < 10 {
            &outcome.bodies.warriors[body].bytes
        } else {
            &outcome.bodies.enemies[body - 10].bytes
        };
        let excluded = if body < 10 {
            race_is_excluded(spell.packet, record, Some(context.memory))?
        } else {
            attack.race_restriction()? != 0
                && monster_race_is_excluded(spell.packet, body, Some(context.memory))?
        };
        if excluded {
            outcome.events.push(Event::Feedback(GameFeedback::Flash {
                address: 0x4018b4,
                x: 50,
                y: 50,
                wait: 40,
                sound: 651,
            }));
        } else {
            let mut mental_feedback = false;
            // Outside battle the missile test still runs, but its refusal is ignored.
            if attack.defense_category() == 9 {
                if !spell.armor_filter {
                    let (skip, speed) = if body < 10 {
                        let target = centurions_content::records::WarriorRecord::from_bytes(record);
                        (target.turn_skip_duration()? != 0, target.quickness()?)
                    } else {
                        let target = centurions_content::records::EnemyRecord::from_bytes(record);
                        (target.turn_skip_duration() != 0, target.agility()?)
                    };
                    if !skip {
                        random.bound(speed / 3);
                        random.bound(100);
                    }
                }
            } else {
                status_resisted_with_feedback(
                    record,
                    body >= 10,
                    spell,
                    Some(context.memory),
                    random,
                    &mut mental_feedback,
                )?;
            }
            if mental_feedback {
                outcome.show_result(body, 25, 0);
                outcome.events.push(Event::Sound { resource_id: 650 });
            }
        }
        let usage_index = if context.collision { usage.slot + 1 } else { 0 };
        if context.collision && usage.used[usage_index] != 0 {
            continue;
        }
        usage.used[usage_index] = 1;
        let marks = (
            outcome.bodies.sounds.len(),
            outcome.bodies.forced_kills.len(),
        );
        let (damage, sounds, reported, icon) = damage_and_conditions(
            &mut outcome.bodies,
            body,
            spell,
            effect,
            duration,
            &mut amount,
            (context.current, Some(context.memory)),
            random,
        )?;
        // The shield sound precedes spelllist; the impact sound follows it.
        let (shield, impact) = sounds.split_at(sounds.len() - usize::from(reported && damage == 0));
        outcome.events.extend(
            shield
                .iter()
                .map(|&resource_id| Event::Sound { resource_id }),
        );
        // Effect 90's conversion sounds, shows result -13 and animates.
        let converted: Vec<_> = outcome.bodies.sounds.drain(marks.0..).collect();
        if !converted.is_empty() {
            outcome.events.extend(
                converted
                    .into_iter()
                    .map(|resource_id| Event::Sound { resource_id }),
            );
            outcome.show_result(body, -13, 0);
            animation(
                &mut outcome.events,
                context,
                attack.effect_animation().wrapping_abs(),
            );
        }
        // Effect 90 kills inside spelllist; zero damage then skips the later killbody.
        let forced = outcome.bodies.forced_kills.len() != marks.1;
        if forced
            && health::defeat(&mut outcome, body, None, context, random)?
                == health::HealthReturn::PartyLost
        {
            outcome.control = Control::PartyLost;
            return Ok(outcome);
        }
        outcome.events.extend(
            impact
                .iter()
                .map(|&resource_id| Event::Sound { resource_id }),
        );
        if reported && damage == 0 {
            animation(
                &mut outcome.events,
                context,
                attack.effect_animation().wrapping_abs(),
            );
            outcome.show_result(body, effect, 0);
        }
        let defeated = match health::apply(&mut outcome, body, damage, context, random)? {
            health::HealthReturn::Applied { defeated } => defeated,
            health::HealthReturn::PartyLost => {
                outcome.control = Control::PartyLost;
                return Ok(outcome);
            }
        };
        outcome.events.push(Event::SpellResolved {
            caster: None,
            spell: context.id,
            target: body as i16,
            damage,
            defeated: defeated || forced,
            icon: crate::event::PresentationIcon(icon),
            flight: Default::default(),
        });
    }
    outcome.health.kill_party = outcome
        .health
        .kill_party
        .wrapping_sub(outcome.bodies.recovered as i8);
    Ok(outcome)
}

/// Refuses a case the reconstruction does not carry.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_field",
        dependency,
    }
}
