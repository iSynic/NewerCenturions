//! resolvespell body order while combat mode is clear, including field usage.
use super::{
    DirectSpell, FieldUsage, RestrictionMemoryView, SupportSpellResult, is_support,
    monster_race_is_excluded, race_is_excluded,
    support::{target::damage_and_conditions, train_party},
    support_rules::status_resisted_with_feedback,
};
use crate::{
    Event, RandomState, RuleError, combat::outdoor_health, feedback::NativeFeedback, story_health,
};
use centurions_content::{Warrior, combat::MonsterRecord, records::AttackPacket};

mod health;
use self::health::animation;

pub(crate) struct Context<'a> {
    pub spell: DirectSpell<'a>,
    pub id: i16,
    pub current: Option<i16>,
    pub memory: RestrictionMemoryView<'a>,
    pub location: (i16, i16),
    pub animation_delay: i16,
    pub outdoor_health: outdoor_health::View<'a>,
    pub collision: bool,
    /// WORD438786, Getup's movement-spell marker; cleared by combatchoice and showresults.
    pub movement_gate: i16,
}

/// resolvespell's arms that replace the body loop and write the tactical arrays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Arm {
    /// 151418 with WORD438786 set.
    Movement,
    /// 1516ea, which calls spelllist at 151744.
    Summon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Control {
    Returned,
    PartyLost,
}

pub(crate) struct NpcCredit {
    pub damage: i16,
    pub hit: bool,
    pub killed: bool,
}

pub(crate) struct Outcome {
    /// Squad members first, then any held warrior table records the loop reached.
    pub bodies: SupportSpellResult,
    /// The squad size, which party defeat counts against.
    pub party: usize,
    pub health: story_health::HealthContext,
    pub events: Vec<Event>,
    pub party_effect: Option<(i16, i16)>,
    pub control: Control,
    pub npc_credit: Vec<NpcCredit>,
    pub clear_departure: bool,
    pub monster_work_written: bool,
    /// An arm the caller runs over its tactical arrays straight after this return.
    pub arm: Option<Arm>,
}

impl Outcome {
    /// resolvespell indexes the warrior table 0x448aa0 + body * 0xc42 for every body
    /// below 10; bodies past the squad are the held records.
    fn hold(&mut self, body: usize, view: outdoor_health::View<'_>) -> Result<(), RuleError> {
        while self.bodies.warriors.len() <= body {
            let next = self.bodies.warriors.len();
            let record = view
                .held_record(next)
                .ok_or_else(|| RuleError::from("detached field body outside the warrior table"))?;
            self.bodies.warriors.push(record);
        }
        Ok(())
    }

    fn show_result(&mut self, body: usize, code: i16, amount: i16) {
        self.clear_departure = true;
        self.events
            .push(Event::Feedback(NativeFeedback::CombatResult {
                body: body as i16,
                code,
                amount,
                audible: false,
            }));
    }
}

/// The arm a packet takes in place of resolvespell's body loop, before the WORD438786
/// test: effects 55 and 56 move, and every other non-support packet that is neither
/// direct weapon damage nor a category-5 drain summons.
pub(crate) fn arm_of(packet: &[u8]) -> Result<Option<Arm>, RuleError> {
    let attack = AttackPacket(packet);
    let effect = attack.effect()?.wrapping_abs();
    if matches!(effect, 55 | 56) {
        return Ok(Some(Arm::Movement));
    }
    let drain = matches!(effect, 60 | 61) && attack.defense_category() == 5;
    let summon = !is_support(packet)? && !attack.is_direct_weapon_damage()? && !drain;
    Ok(summon.then_some(Arm::Summon))
}

/// resolvespell15108c/1511ea rolls duration before either global or body effects.
pub(crate) fn duration(packet: &[u8], count: i16, random: &mut RandomState) -> i16 {
    let attack = AttackPacket(packet);
    let (minimum, maximum) = attack.duration_bounds();
    let mut duration = random.range(minimum, maximum);
    let (minimum, maximum) = attack.additional_duration_bounds();
    if minimum != 0 {
        for _ in 0..count {
            duration = duration.wrapping_add(random.range(minimum, maximum));
        }
    }
    duration
}

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
    let attack = AttackPacket(spell.packet);
    let effect = attack.effect()?.wrapping_abs();
    let physical = attack.is_direct_weapon_damage()?;
    let movement = matches!(effect, 55 | 56);
    // resolvespell has no defense-category test of its own; category-5 drains move Psi
    // through spelllist15ba5a like the others.
    let drain = matches!(effect, 60 | 61) && attack.defense_category() == 5;
    // Only effect 58 is left outside these; its arm follows the movement arm.
    let summon = !is_support(spell.packet)? && !physical && !movement && !drain;
    let duration = duration(spell.packet, spell.count, random);
    let mut outcome = Outcome {
        bodies: SupportSpellResult {
            total_delta: 0,
            warriors: warriors.to_vec(),
            enemies: enemies.to_vec(),
            sounds: vec![],
            targets: vec![],
            healed: false,
            recovered: 0,
            race_excluded: vec![],
            fatigue_reset: false,
            area_shape: None,
            forced_kills: vec![],
            alias_writes: vec![],
        },
        party: warriors.len(),
        health: previous.clone(),
        events: vec![],
        party_effect: None,
        control: Control::Returned,
        npc_credit: vec![],
        clear_departure: false,
        monster_work_written: false,
        arm: None,
    };
    outcome.health.effect = attack.health_effect();
    if attack.targeting_mode() == 7 {
        outcome.party_effect = Some((effect, duration));
        return Ok(outcome);
    }
    if effect == 66 {
        train_party(&mut outcome.bodies, &attack, mask, random)?;
    }
    // resolvespell151418: the movement arm replaces the body loop. With the marker
    // clear it only warns; a set marker moves WORD4435d2 over the tactical arrays.
    if movement {
        if context.movement_gate != 0 {
            outcome.arm = Some(Arm::Movement);
        } else {
            outcome
                .events
                .push(Event::Feedback(NativeFeedback::Warning { index: 87 }));
        }
        return Ok(outcome);
    }
    if summon {
        outcome.arm = Some(Arm::Summon);
        return Ok(outcome);
    }
    // BYTE4358c1, adjacent to the sixty reset usage bytes.
    usage.used[61] = 0;
    let mut amount = 0;
    for (body, selected) in mask.iter().enumerate() {
        if *selected <= 0 {
            continue;
        }
        // The health arm's affected WORD 0x442798 + 2 * body (0x151b2e) reaches
        // resolvespell's duration WORD 0x4427a4 and round WORD 0x4427a6 for these.
        if (6..8).contains(&body) {
            return Err(unavailable("detached field body in an Upgrade record"));
        }
        if (8..10).contains(&body) {
            // 0x448aa0 + body * 0xc42 is the editor backup 0x44ecb0.
            return Err(unavailable("detached field body in the editor backup"));
        }
        if body >= 10 + enemies.len() {
            return Err(unavailable(
                "detached field body past the loaded NPC records",
            ));
        }
        context.memory.require_npc(body)?;
        if body < 10 {
            outcome.hold(body, context.outdoor_health)?;
        }
        if body >= 10 {
            outcome.monster_work_written = true;
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
            outcome.events.push(Event::Feedback(NativeFeedback::Flash {
                address: 0x4018b4,
                x: 50,
                y: 50,
                wait: 40,
                sound: 651,
            }));
        } else {
            let mut mental_feedback = false;
            // The native call still runs, but combat mode0 ignores its refusal.
            if attack.defense_category() == 9 {
                if !spell.armor_filter {
                    let (skip, speed) = if body < 10 {
                        let target = centurions_content::records::WarriorRecord::from_bytes(record);
                        (target.turn_skip_duration()? != 0, target.quickness()?)
                    } else {
                        let target = centurions_content::records::EnemyRecord::from_bytes(record);
                        (target.turn_skip_duration() != 0, target.speed()?)
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
        // spelllist15c6de: effect90's conversion sounds, shows result -13 and animates.
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
        // spelllist15c6d4 kills inside the call; zero damage skips resolvespell's killbody.
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
    outcome.health.unavailable = outcome
        .health
        .unavailable
        .wrapping_sub(outcome.bodies.recovered as i8);
    Ok(outcome)
}

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_field",
        dependency,
    }
}
