//! Party script operations, preserving original dispatch order.
use super::super::{present_treasure, present_treasure_record, unsupported};
use super::{Operation, ScriptStep};
use crate::{
    Event, Presentation, RuleError, Session, story, story_allies, story_inventory, story_party,
};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{
    ClinicExtra, EventHeaderTable, ExtraTable, FatigueChange, LimitedTreasureExtra,
    RecoveryRestrictions, StoryRecordMut,
};
use centurions_content::records::{TreasureRecord, WarriorRecord};
use centurions_content::{Content, span, word};

pub(super) fn execute(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
    cursor: usize,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_mut().unwrap();
    match operation {
        Operation::AdjustStatistics => {
            let extra = ExtraTable(
                &map.combat
                    .as_ref()
                    .ok_or("story combat tables missing")?
                    .encounters,
            )
            .record(parameter.unsigned_abs() as usize)?;
            state.native_work.extra_record = extra.to_vec();
            state.warriors = story_party::adjust_statistics(
                &state.warriors,
                &state.native_work.party_mask,
                extra,
            )?;
        }
        Operation::Confiscate => {
            // The restore loop counts its items in newland's encounter-caller local.
            let restored = state
                .confiscated
                .as_ref()
                .filter(|saved| parameter == 0 && saved.active)
                .map(|saved| {
                    let slots = &saved.items[..state.warriors.len() * 100];
                    slots.iter().filter(|item| item.id() != 0).count() as i16
                });
            effects.extend(crate::confiscation::apply(state, content, parameter)?);
            if restored.is_some() {
                let position = state.exploration.as_mut().unwrap();
                position.script.as_mut().unwrap().caller_action = restored;
            }
            if state.exploration.as_ref().unwrap().presentation.is_some() {
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::Shop => {
            if crate::shop::visit(state, content, parameter)? {
                effects.push(Event::StoryPresented);
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::Clinic => {
            let extra = ExtraTable(
                &map.combat
                    .as_ref()
                    .ok_or("clinic extra records missing")?
                    .encounters,
            )
            .record(parameter.unsigned_abs() as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let clinic = ClinicExtra(extra);
            let percent = clinic.percent()?;
            let sound = clinic.sound()?;
            let text = clinic.text()?;
            state.field_controls.get_or_insert_default().clinic = Some(percent);
            if sound != 0 {
                effects.push(Event::Sound { resource_id: sound });
            }
            if text != 0 {
                let text_body = super::super::dialogue(map, &state.warriors, text)?;
                state.exploration.as_mut().unwrap().presentation =
                    Some(Presentation::ClinicEntry { percent });
                effects.push(Event::Message {
                    id: text,
                    text: text_body,
                });
                return Ok(ScriptStep::Pause);
            }
            effects.extend(crate::clinic::visit(state, percent)?);
            return Ok(ScriptStep::Pause);
        }
        Operation::RevealNote => {
            let index = usize::from(parameter.unsigned_abs());
            let flag = state
                .notes
                .get_mut(index)
                .ok_or("map or note index unavailable")?;
            *flag = 1;
            if parameter > -1 {
                effects.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
                    address: 0x401c28,
                    x: 30,
                    y: 355,
                    wait: 0,
                    sound: 30005,
                }));
            }
        }
        Operation::SelectedHealth | Operation::PartyHealth => {
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters)
                .record(parameter.unsigned_abs() as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let context = crate::story_health::retained_context(
                state.selected_warrior,
                &state.native_work.health_context,
            );
            let outcome = crate::story_health::party_health(
                &state.warriors,
                extra,
                operation == Operation::PartyHealth,
                &state.native_work.party_mask,
                &context,
                &mut state.random,
            )?;
            state.warriors = outcome.warriors;
            state.native_work.health_context = outcome.context;
            let mut party_lost = false;
            for event in outcome.events {
                match event {
                    crate::story_health::HealthEvent::Flash { address, sound } => {
                        effects.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
                            address,
                            x: 100,
                            y: 30,
                            wait: 100,
                            sound,
                        }))
                    }
                    crate::story_health::HealthEvent::Animation { effect, .. } => {
                        effects.push(Event::OutdoorEffect {
                            effect: i16::from(effect),
                            x: position.x as i16,
                            y: position.y as i16,
                            size: 32,
                            delay: 0,
                        })
                    }
                    crate::story_health::HealthEvent::Sound(resource_id) => {
                        effects.push(Event::Sound { resource_id })
                    }
                    crate::story_health::HealthEvent::Dialogue(id) => {
                        position.presentation = Some(Presentation::Dialogue {
                            id,
                            text: map.dialogue(id)?,
                        });
                        effects.push(Event::StoryPresented);
                    }
                    crate::story_health::HealthEvent::Defeated { warrior, .. } => {
                        if let Some(combat) = state.combat.as_mut() {
                            crate::combat::lifecycle::kill_body(
                                combat,
                                &mut state.warriors,
                                warrior as i16,
                            )?;
                        }
                    }
                    crate::story_health::HealthEvent::PartyLost => {
                        party_lost = true;
                    }
                }
            }
            if party_lost {
                effects.extend(crate::mission::defeat(state, content)?);
                return Ok(ScriptStep::Pause);
            }
            if position.presentation.is_some() {
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::FilterParty => {
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters)
                .record(parameter.unsigned_abs() as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let memory = crate::combat::spells::RestrictionMemoryView::ordinary_startup(
                &map.item_cache,
                crate::combat::spells::retained_item_alias(&state.native_work.ammunition_work)?,
            )
            .with_alias_owners(crate::combat::warrior_alias::session_owners!(state))
            .with_definitions(&content.items);
            state.native_work.party_mask = crate::story_party::filter_party(
                &state.warriors,
                state.native_work.party_mask,
                cursor,
                extra,
                &mut state.random,
                memory,
            )?;
        }
        Operation::SetRecoveryRestrictions => {
            let extra = ExtraTable(
                &map.combat
                    .as_ref()
                    .ok_or("script extra records unavailable")?
                    .encounters,
            )
            .record(parameter.unsigned_abs() as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let restrictions = RecoveryRestrictions::decode(extra);
            state.psi_disabled = restrictions.psi;
            state.enemy_spell_disabled = restrictions.enemy_spells;
            state.energy_recovery_disabled = restrictions.energy;
        }
        Operation::ChangeFatigue => {
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters)
                .record(parameter.unsigned_abs() as usize)?;
            state.native_work.extra_record = extra.to_vec();
            position.fatigue = match FatigueChange::decode(extra)? {
                FatigueChange::Set(value) => value,
                // Native order divides first, then multiplies with WORD wrapping.
                FatigueChange::Percent(percent) => (percent / 100).wrapping_mul(position.fatigue),
                FatigueChange::Keep => position.fatigue,
            };
        }
        Operation::SetMissionFlag => {
            state.flags = story::set_mission_flag(&state.flags, parameter)?;
        }
        Operation::AddMissionScore => {
            state.mission_score = story::add_mission_score(state.mission_score, parameter);
        }
        Operation::AddAlly => {
            let definition = span(
                &map.combat.as_ref().unwrap().monsters,
                parameter as usize * story_allies::ALLY_BYTES,
                story_allies::ALLY_BYTES,
            )?;
            let addition = story_allies::add_ally(
                &state.allies,
                state.ally_count,
                definition,
                state.difficulty,
                &mut state.random,
            )?;
            state.allies = addition.records;
            state.ally_count = addition.count;
            if let Some(warning) = addition.warning {
                effects.push(Event::CollectionRejected { warning });
            } else {
                effects.push(Event::AllyAdded {
                    definition: parameter,
                });
            }
        }
        Operation::RemoveAllies => {
            let removed = crate::story_remove_allies::remove_class(
                &state.allies,
                state.ally_count,
                parameter,
            )?;
            state.allies = removed.records;
            state.ally_count = removed.count;
        }
        Operation::CopyMapEvent => {
            // newland1434d4 copies the current map's mutable header, not Data ED3.
            let slot = usize::try_from(parameter)
                .map_err(|_| RuleError::Invalid("negative event header slot".into()))?;
            let replacement = EventHeaderTable(&position.event_headers).record(slot)?;
            StoryRecordMut(&mut script.record).copy_actions_and_byte40(replacement);
            script.cursor = 0;
        }
        Operation::ReplaceScript => {
            let replacement = map.script(parameter)?;
            script.restart_from(&replacement);
        }
        Operation::LimitedShopOrTreasure => {
            let extra = ExtraTable(&map.combat.as_ref().unwrap().encounters)
                .record(parameter.unsigned_abs() as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let limited = LimitedTreasureExtra(extra);
            let id = limited.treasure()?;
            if id <= 0 {
                crate::shop::visit_restricted(state, content, extra)?;
                effects.push(Event::StoryPresented);
            } else {
                state.loot_limit = if limited.limit_mode()?.wrapping_add(1) != 0 {
                    Some(limited.limit()?)
                } else {
                    None
                };
                effects.extend(present_treasure(state, content, id)?);
            }
            return Ok(ScriptStep::Pause);
        }
        // newland43 first clears a positive duration in slot e2 for every member, then
        // adds e3 for members chosen by e1 (all, picked, stamina above -10) with sound
        // e4, the portrait effect and showresults code e2 + 1.
        Operation::GiveCondition => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let slot = usize::try_from(word(extra, 2)?)
                .ok()
                .filter(|&slot| slot < 40)
                .ok_or_else(|| unsupported("story.opcode", "condition outside the record"))?;
            let (who, amount, sound) = (word(extra, 0)?, word(extra, 4)?, word(extra, 6)?);
            for member in 0..state.warriors.len() {
                let warrior = &mut state.warriors[member];
                let current = WarriorRecord::from_bytes(&warrior.bytes).condition_slot(slot)?;
                if current > 0 {
                    warrior.record_mut().set_condition_duration(slot, 0);
                }
                let chosen = match who {
                    0 => true,
                    1 => state.native_work.party_mask[member] != 0,
                    2 => warrior.record().stamina()? > -10,
                    _ => false,
                };
                if chosen {
                    let current = WarriorRecord::from_bytes(&warrior.bytes).condition_slot(slot)?;
                    warrior
                        .record_mut()
                        .set_condition_duration(slot, current.wrapping_add(amount));
                    if sound != 0 {
                        effects.push(Event::Sound { resource_id: sound });
                    }
                    effects.push(Event::Feedback(
                        crate::feedback::NativeFeedback::CombatResult {
                            body: member as i16,
                            code: slot as i16 + 1,
                            amount: 0,
                            audible: false,
                        },
                    ));
                }
            }
        }
        // newland50 clears the pick mask, then picks members (living ones when e5 is
        // set) by race e3 (0), gender e2 (1), caste e3 (2), race class e3 (3) or caste
        // class e3 (4).
        Operation::PickByTrait => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let (mode, gender, code) = (word(extra, 0)?, word(extra, 2)?, word(extra, 4)?);
            let mut mask = [0; 6];
            for (member, warrior) in state.warriors.iter().enumerate() {
                let record = warrior.record();
                if word(extra, 8)? != 0 && record.stamina()? <= 0 {
                    continue;
                }
                let picked = match mode {
                    0 => record.race()? == code,
                    1 => record.gender()? == gender,
                    2 => record.profession()? == code,
                    3 => super::branches::race_class(map, record.race()?, code)?,
                    4 => super::branches::caste_class(map, record.profession()?)? == code,
                    _ => false,
                };
                mask[member] = i16::from(picked);
            }
            state.native_work.party_mask = mask;
        }
        // newland53 clears the pick mask first, so its picked-only check (e3 = 2)
        // admits nobody. Caste e1 picks, as do the caste families of e2.
        Operation::PickByCaste => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let (caste, family, check) = (word(extra, 0)?, word(extra, 2)?, word(extra, 4)?);
            let families: &[i16] = match family {
                1 => &[1, 3, 4],
                2 => &[3, 6, 7, 8],
                3 => &[2, 5],
                _ => &[],
            };
            let mut mask = [0; 6];
            for (member, warrior) in state.warriors.iter().enumerate() {
                let record = warrior.record();
                let admitted = check == 0
                    || (check == 1 && record.stamina()? > 0)
                    || (check == 2 && mask[member] != 0);
                let profession = record.profession()?;
                if admitted && (profession == caste || families.contains(&profession)) {
                    mask[member] = 1;
                }
            }
            state.native_work.party_mask = mask;
        }
        // newland60 clears the carried credits of every member, or of picked members
        // when e2 is set; the wealth type e1 is not read.
        Operation::DropMemberCredits => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let picked_only = word(extra, 2)? != 0;
            for (member, warrior) in state.warriors.iter_mut().enumerate() {
                if !picked_only || state.native_work.party_mask[member] != 0 {
                    warrior.record_mut().set_carried_credits(0);
                }
            }
        }
        // newland90 moves e1 from upgrade points into the remaining experience. Its
        // picked mode (e2 = 1) tests the pick word at the action's own index for every
        // member; mode 2 spreads the signed quotient over the party.
        Operation::TakeVictoryPoints => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let amount = word(extra, 0)?;
            let share = match word(extra, 2)? {
                1 => {
                    let picked =
                        *state.native_work.party_mask.get(cursor).ok_or_else(|| {
                            unsupported("story.opcode", "pick word past the party")
                        })?;
                    if picked == 0 {
                        return Ok(ScriptStep::Continue);
                    }
                    amount
                }
                2 => amount / state.warriors.len() as i16,
                _ => amount,
            };
            for warrior in &mut state.warriors {
                let record = warrior.record();
                let remaining = record
                    .remaining_experience()?
                    .wrapping_add(i32::from(share));
                let points = record.upgrade_points().wrapping_sub(i32::from(share));
                warrior.record_mut().set_remaining_experience(remaining);
                warrior.record_mut().set_upgrade_points(points);
            }
        }
        // newland91 empties every member's inventory, then redraws.
        Operation::DropAllEquipment => {
            let mut warriors = state.warriors.clone();
            let mut loaded = None;
            let mut globals = vec![];
            for warrior in &mut warriors {
                let (emptied, last, writes) =
                    crate::equipment::drop_all(warrior, content, map, effects)?;
                *warrior = emptied;
                loaded = last.or(loaded);
                globals.extend(writes);
            }
            state.apply_equipment_globals(&globals)?;
            state.warriors = warriors;
            if let Some(item) = loaded {
                state.load_item_work(content, item)?;
            }
        }
        Operation::AlterShop => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            state.native_work.extra_record = extra.to_vec();
            crate::shop::alter(state, content, extra)?;
        }
        // newland65 fills the booty buffer that opcode 10 reads from Data TD with |e1|
        // item words from randrange(e2, e3) (Rand(|e1|) of them for negative e1) and
        // keeps the count in the encounter-caller local; later words overflow into the
        // record's enhancement and award fields.
        Operation::RandomTreasure => {
            let extra =
                ExtraTable(&map.combat.as_ref().unwrap().encounters).record(parameter as usize)?;
            state.native_work.extra_record = extra.to_vec();
            let requested = word(extra, 0)?;
            if !(-44..=44).contains(&requested) {
                return Err(unsupported("story.opcode", "random items past the record"));
            }
            let count = if requested < 0 {
                state.random.bound(requested.wrapping_abs())
            } else {
                requested
            };
            let mut record = [0; TreasureRecord::BYTES];
            for item in 0..count.max(0) as usize {
                let id = state.random.range(word(extra, 2)?, word(extra, 4)?);
                record[item * 2..item * 2 + 2].copy_from_slice(&id.to_be_bytes());
            }
            let position = state.exploration.as_mut().unwrap();
            position.script.as_mut().unwrap().caller_action = Some(count);
            effects.extend(present_treasure_record(state, content, &record)?);
            return Ok(ScriptStep::Pause);
        }
        Operation::Treasure => {
            effects.extend(present_treasure(state, content, parameter)?);
            return Ok(ScriptStep::Pause);
        }
        Operation::Experience => {
            let awarded =
                story_inventory::award_story_experience(&state.warriors, i32::from(parameter))?;
            state.warriors = awarded.warriors;
            // newland11 stores the amount in LONG 0x443392 before booty(12, 0).
            state.booty_tally = crate::BootyTally {
                credits: 0,
                share: awarded.share,
                experience: i32::from(parameter),
            };
            let position = state.exploration.as_mut().unwrap();
            position.presentation = Some(Presentation::Loot);
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        _ => unreachable!("opcode routed to wrong story adapter"),
    }
    Ok(ScriptStep::Continue)
}
