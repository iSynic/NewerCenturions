use super::*;
use centurions_content::word;

fn put(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

pub(crate) fn advance(
    state: &mut Session,
    content: &Content,
    ticks: u8,
    minutes_per_tick: i16,
) -> Result<Vec<Event>, RuleError> {
    let mut events = Vec::new();
    for _ in 0..ticks {
        let position = state.exploration.as_mut().unwrap();
        position.minute = position.minute.wrapping_add(minutes_per_tick);
        if position.minute < 60 {
            continue;
        }
        // CODE 1 timeclick: reduce precedes the hour, fatigue and energy updates.
        crate::combat::party_buffs::reduce(&mut state.party_buffs)?;
        let context = crate::story_health::context(
            &state.warriors,
            state.selected_warrior,
            state.health_context.as_ref(),
        );
        let reduced =
            crate::story_reduce::reduce_outdoors(&state.warriors, &context, &mut state.random)?;
        if reduced
            .events
            .iter()
            .any(|event| matches!(event, crate::story_health::HealthEvent::PartyLost))
        {
            return Err(RuleError::Unsupported {
                action: "time.conditions",
                dependency: "party defeat transition",
            });
        }
        state.warriors = reduced.warriors;
        state.health_context = Some(reduced.context);
        events.extend(reduced.events.into_iter().filter_map(|event| {
            if let crate::story_health::HealthEvent::Sound(resource_id) = event {
                Some(Event::Sound { resource_id })
            } else {
                None
            }
        }));
        position.minute -= 60;
        position.hour += 1;
        position.fatigue = position.fatigue.saturating_add(1).clamp(4, 135);
        for warrior in &mut state.warriors {
            let bytes = &mut warrior.bytes;
            let energy = word(bytes, 0x58)?;
            let maximum = word(bytes, 0x5a)?;
            if energy < maximum && word(bytes, 0x50)? > 0 && word(bytes, 0x90)? == 0 {
                let level = word(bytes, 0x3a)?;
                let amount = (level.wrapping_add((level as u16 >> 15) as i16) >> 1) as i8;
                put(
                    bytes,
                    0x58,
                    energy.wrapping_add(i16::from(amount.max(1))).min(maximum),
                );
            }
        }
        if position.hour == 24 {
            position.hour = 0;
            position.day = position.day.wrapping_add(1);
            let mut cursor = 0;
            loop {
                let selected = scan_schedule(state, &mut cursor, &mut events)?;
                recover(state, content)?;
                let Some(script_id) = selected else { break };
                let map = content
                    .map(
                        state
                            .exploration
                            .as_ref()
                            .map_or(0, |position| position.map),
                    )
                    .ok_or_else(|| RuleError::Invalid("schedule content unavailable".into()))?;
                let record = centurions_content::span(
                    &map.scheduled_scripts,
                    usize::from(script_id.unsigned_abs()) * 42,
                    42,
                )?
                .to_vec();
                state.exploration.as_mut().unwrap().script = Some(ScriptState {
                    record,
                    cursor: 0,
                    map_slot: 0,
                    scheduled: true,
                    menu: None,
                    persist: false,
                    origin: None,
                });
                events.extend(super::exploration::run_script(state, content)?);
                if state.exploration.as_ref().unwrap().presentation.is_some() {
                    return Err(RuleError::Unsupported {
                        action: "time.schedule",
                        dependency: "interactive scheduled-event continuation",
                    });
                }
            }
        } else if position.hour == 12 {
            recover(state, content)?;
        }
        let position = state.exploration.as_ref().unwrap();
        events.push(Event::TimeAdvanced {
            day: position.day,
            hour: position.hour,
            minute: position.minute,
        });
    }
    Ok(events)
}

fn recover(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    for index in 0..state.warriors.len() {
        let bytes = &state.warriors[index].bytes;
        let stamina = word(bytes, 0x50)?;
        let maximum = word(bytes, 0x52)?;
        if stamina <= -10 || stamina >= maximum {
            continue;
        }
        let level = word(bytes, 0x3a)?;
        let mut amount = if word(bytes, 0x32)? == 3 {
            level
        } else {
            level / 4
        } as i8;
        if !consume_food(state, content)? {
            amount /= 2;
        }
        let healing =
            i16::from(amount.max(1)).wrapping_add(word(&state.warriors[index].bytes, 0x66)?);
        let context = crate::story_health::context(
            &state.warriors,
            state.selected_warrior,
            state.health_context.as_ref(),
        );
        let healed = crate::story_health::heal_outdoors(
            &state.warriors,
            index,
            healing,
            false,
            &context,
            &mut state.random,
        )?;
        if !healed.events.is_empty() {
            return Err(RuleError::Unsupported {
                action: "time.recover",
                dependency: "terminal recovery presentation",
            });
        }
        state.warriors = healed.warriors;
        state.health_context = Some(healed.context);
    }
    Ok(())
}

fn consume_food(state: &mut Session, content: &Content) -> Result<bool, RuleError> {
    for warrior in &mut state.warriors {
        for (slot, item) in warrior.inventory()?.iter().enumerate() {
            if item.id() != 800 {
                continue;
            }
            let quantity = word(&item.0, 2)?;
            if quantity <= 0 {
                return Ok(false);
            }
            let definition = content
                .items
                .get(800)
                .ok_or_else(|| RuleError::Invalid("food definition unavailable".into()))?;
            let weight = word(definition, 0x24)?;
            let carried = word(&warrior.bytes, 0x9d6)?.wrapping_sub(weight);
            put(&mut warrior.bytes, 0x9d6, carried);
            put(&mut warrior.bytes, 0x132 + slot * 22 + 2, quantity - 1);
            if quantity == 1 && word(definition, 0x26)? != 0 {
                super::items::remove_inventory_slot(warrior, slot)?;
            }
            state.load_item_work(content, 800)?;
            return Ok(true);
        }
    }
    Ok(false)
}

fn scan_schedule(
    state: &mut Session,
    cursor: &mut usize,
    events: &mut Vec<Event>,
) -> Result<Option<i16>, RuleError> {
    let position = state.exploration.as_ref().unwrap();
    let day = position.day;
    let count = position.schedule.len() / 40;
    while *cursor < count {
        let slot = *cursor;
        *cursor += 1;
        let record =
            state.exploration.as_ref().unwrap().schedule[slot * 40..slot * 40 + 40].to_vec();
        let due = word(&record, 0)?;
        if due != day {
            if due == 0 {
                break;
            }
            continue;
        }
        let next_day = due.wrapping_add(word(&record, 2)?);
        let mut eligible = state.random.bound(100) <= word(&record, 4)?;
        if eligible && word(&record, 16)? > 0 {
            let required = word(&record, 16)?;
            eligible = state.warriors.iter().any(|w| {
                w.inventory()
                    .is_ok_and(|items| items.iter().any(|item| item.id() == required))
            });
        }
        if eligible && word(&record, 18)? >= 0 {
            return Err(RuleError::Unsupported {
                action: "time.schedule",
                dependency: "mission flag prerequisite",
            });
        }
        if eligible && word(&record, 20)? > 0 {
            let position = state.exploration.as_ref().unwrap();
            if word(&record, 20)? != 1 || i32::from(word(&record, 8)?) != position.map {
                continue;
            }
            let region = word(&record, 10)?;
            if region >= 0 {
                return Err(RuleError::Unsupported {
                    action: "time.schedule",
                    dependency: "schedule region prerequisite",
                });
            }
            let x = word(&record, 12)?;
            if x >= 0 && (position.x != i32::from(x) || position.y != i32::from(word(&record, 14)?))
            {
                continue;
            }
        }
        put(
            &mut state.exploration.as_mut().unwrap().schedule,
            slot * 40,
            next_day,
        );
        events.push(Event::ScheduleRescheduled {
            slot,
            next_day,
            dispatched: eligible,
        });
        if eligible {
            return Ok(Some(word(&record, 6)?));
        }
        *cursor = count;
        break;
    }
    Ok(None)
}

#[cfg(all(test, feature = "original-fixtures"))]
mod tests {
    use super::*;
    #[derive(Deserialize)]
    struct Fixture {
        cases: Vec<Case>,
    }
    #[derive(Deserialize)]
    struct Case {
        input: String,
        output: String,
        fatigue_before: i16,
        fatigue_after: i16,
        hour_after: i16,
        minute_after: i16,
    }
    fn decode(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }
    #[test]
    fn hourly_update_matches_original_instruction_execution() {
        let fixture: Fixture =
            serde_json::from_str(include_str!("../../../evidence/fixtures/time.json")).unwrap();
        for case in fixture.cases {
            let content = Content {
                identity: "hourly".into(),
                warriors: vec![],
                items: vec![],
                resources: vec![],
                original_services: None,
                scenario: None,
            };
            let mut state = Session::new(&content);
            state.warriors = decode(&case.input)
                .as_chunks::<3138>()
                .0
                .iter()
                .map(|record| Warrior::read(record).unwrap())
                .collect();
            state.exploration = Some(Exploration {
                map: 0,
                x: 2,
                y: 2,
                day: 1,
                hour: 12,
                minute: 55,
                fatigue: case.fatigue_before,
                facing: 0,
                last_move: [0; 2],
                script: None,
                presentation: None,
                event_headers: vec![0; 4200],
                schedule: vec![],
                region_state: vec![0; 644],
                visibility: vec![0; 8100],
                floor: vec![],
                objects: vec![],
                events: vec![],
            });
            advance(&mut state, &content, 1, 5).unwrap();
            let position = state.exploration.unwrap();
            assert_eq!(
                (position.hour, position.minute, position.fatigue),
                (case.hour_after, case.minute_after, case.fatigue_after)
            );
            assert_eq!(
                state
                    .warriors
                    .into_iter()
                    .flat_map(|warrior| warrior.bytes)
                    .collect::<Vec<_>>(),
                decode(&case.output)
            );
            assert_eq!(state.random.draws, 0);
        }
    }

    fn calendar_content(schedule: Vec<u8>) -> Content {
        let mut properties = vec![0; 12000];
        properties[2..4].copy_from_slice(&1i16.to_be_bytes());
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../evidence/fixtures/calendar.json")).unwrap();
        let mut items = vec![vec![0; 150]; 801];
        items[800] = decode(fixture["food_definition"].as_str().unwrap());
        Content {
            identity: "calendar".into(),
            warriors: vec![],
            items,
            resources: vec![],
            original_services: None,
            scenario: Some(centurions_content::mission::ScenarioContent::single(
                centurions_content::mission::OpeningMission {
                    name: "calendar".into(),
                    start: (2, 2),
                    header: vec![0; 4200],
                    regions: vec![0; 644],
                    floor: vec![0; 8100],
                    objects: vec![0; 8100],
                    events: vec![0; 8100],
                    tile_properties: properties,
                    terrain: 2,
                    default_tile: 0,
                    minutes_per_tick: 5,
                    shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
                        areas: vec![],
                        item_cache: vec![],
                        races: vec![],
                        profiles: vec![],
                        schedule,
                        door_scripts: vec![],
                        dialogue: vec![],
                        treasure: vec![],
                        prologue: String::new(),
                        item_descriptions: vec![],
                        combat: None,
                        scheduled_scripts: vec![0; 118 * 42],
                        menus: vec![],
                        typed_encounters: vec![],
                        movies: Default::default(),
                    }),
                },
            )),
        }
    }

    fn calendar_state(content: &Content, input: &str, hour: i16, minute: i16) -> Session {
        let mut state = Session::new(content);
        state.warriors = decode(input)
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|r| Warrior::read(r).unwrap())
            .collect();
        state.phase = Phase::Exploration;
        state.mission = Some("The Thunder of Guns".into());
        state.random = RandomState::new(1);
        state.exploration = Some(Exploration {
            map: 0,
            x: 2,
            y: 2,
            day: 1,
            hour,
            minute,
            fatigue: 4,
            facing: 0,
            last_move: [0; 2],
            script: None,
            presentation: None,
            event_headers: vec![0; 4200],
            schedule: content.map(0).unwrap().schedule.clone(),
            region_state: content.map(0).unwrap().regions.clone(),
            visibility: vec![0; 8100],
            floor: content.map(0).unwrap().floor.clone(),
            objects: content.map(0).unwrap().objects.clone(),
            events: content.map(0).unwrap().events.clone(),
        });
        state
    }

    #[test]
    fn noon_midnight_and_schedule_match_original_instruction_cases() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../evidence/fixtures/calendar.json")).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let content = calendar_content(decode(case["schedule_before"].as_str().unwrap()));
            let mut state = calendar_state(
                &content,
                case["input"].as_str().unwrap(),
                case["hour_before"].as_i64().unwrap() as i16,
                case["minute_before"].as_i64().unwrap() as i16,
            );
            let events = advance(
                &mut state,
                &content,
                case["ticks"].as_u64().unwrap() as u8,
                5,
            )
            .unwrap();
            let position = state.exploration.unwrap();
            assert_eq!(
                (
                    position.day,
                    position.hour,
                    position.minute,
                    position.fatigue
                ),
                (
                    case["day_after"].as_i64().unwrap() as i16,
                    case["hour_after"].as_i64().unwrap() as i16,
                    case["minute_after"].as_i64().unwrap() as i16,
                    case["fatigue_after"].as_i64().unwrap() as i16
                )
            );
            assert_eq!(
                position.schedule,
                decode(case["schedule_after"].as_str().unwrap())
            );
            assert_eq!(
                state
                    .warriors
                    .into_iter()
                    .flat_map(|w| w.bytes)
                    .collect::<Vec<_>>(),
                decode(case["output"].as_str().unwrap()),
                "variant {}",
                case["variant"]
            );
            assert_eq!(
                state.random.seed,
                case["seed_after"].as_u64().unwrap() as u32
            );
            assert_eq!(state.random.draws, case["draws"].as_u64().unwrap());
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(
                        e,
                        Event::ScheduleRescheduled {
                            dispatched: true,
                            ..
                        }
                    ))
                    .count(),
                case["dispatches"].as_u64().unwrap() as usize
            );
        }
    }

    #[test]
    fn calendar_save_resume_and_legacy_migration_preserve_trigger_history() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../evidence/fixtures/calendar.json")).unwrap();
        let case = &fixture["cases"][5];
        let content = calendar_content(decode(case["schedule_before"].as_str().unwrap()));
        let mut state = calendar_state(&content, case["input"].as_str().unwrap(), 23, 55);
        let path =
            std::env::temp_dir().join(format!("centurions-calendar-{}.json", std::process::id()));
        save(&path, &state, &content).unwrap();
        let mut resumed = load(&path, &content).unwrap();
        let command = Command::Move { dx: 1, dy: 0 };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert_eq!(state, resumed);
        save(&path, &state, &content).unwrap();
        assert_eq!(load(&path, &content).unwrap(), state);
        let mut legacy = state.clone();
        legacy.exploration.as_mut().unwrap().day = 1;
        legacy.exploration.as_mut().unwrap().schedule.clear();
        fs::write(
            &path,
            serde_json::to_vec(&Save {
                schema_version: 1,
                session: legacy,
            })
            .unwrap(),
        )
        .unwrap();
        let migrated = load(&path, &content).unwrap();
        assert_eq!(
            migrated.exploration.unwrap().schedule,
            content.map(0).unwrap().schedule
        );
        fs::write(
            &path,
            serde_json::to_vec(&Save {
                schema_version: 1,
                session: state.clone(),
            })
            .unwrap(),
        )
        .unwrap();
        assert!(
            load(&path, &content)
                .unwrap_err()
                .to_string()
                .contains("unknown schedule history")
        );
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn hourly_party_protection_ticks_without_changing_other_rules_and_resumes() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../evidence/fixtures/calendar.json")).unwrap();
        let case = &fixture["cases"][0];
        let content = calendar_content(decode(case["schedule_before"].as_str().unwrap()));
        for (duration, after) in [(20_i16, 19_i16), (1, 0), (-1, -1)] {
            let mut state = calendar_state(&content, case["input"].as_str().unwrap(), 13, 55);
            state.psi_memory = Some(crate::initial_psi_memory(&state.warriors).unwrap());
            let mut baseline = state.clone();
            state.party_buffs[4..6].copy_from_slice(&duration.to_be_bytes());
            let path = std::env::temp_dir().join(format!(
                "centurions-hourly-buff-{}-{duration}.json",
                std::process::id()
            ));
            save(&path, &state, &content).unwrap();
            let mut resumed = load(&path, &content).unwrap();
            let command = Command::Move { dx: 1, dy: 0 };
            let events = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(
                apply(command.clone(), &mut resumed, &content).unwrap(),
                events
            );
            assert_eq!(state, resumed);
            assert_eq!(apply(command, &mut baseline, &content).unwrap(), events);
            assert_eq!(word(&state.party_buffs, 4).unwrap(), after);
            baseline.party_buffs = state.party_buffs.clone();
            assert_eq!(state, baseline);
            fs::remove_file(path).unwrap();
        }
        let mut state = calendar_state(&content, case["input"].as_str().unwrap(), 13, 55);
        state.party_buffs[4..6].copy_from_slice(&20_i16.to_be_bytes());
        state.warriors[0].bytes[0x8a..0x8c].copy_from_slice(&1_i16.to_be_bytes());
        let before = state.clone();
        assert!(matches!(
            apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content),
            Err(RuleError::Unsupported { .. })
        ));
        assert_eq!(state, before);
    }
}
