use crate::{prelude::*, support};

use centurions_core::combat::spells::resolve_warriors;

#[test]
fn enemy_spell_damage_matches_original_complete_party_and_random_consumption() {
    let fixture: serde_json::Value = support::fixture("enemy-spell-damage.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let warriors = support::warriors(&case.bytes("before"));
        let packet = case.bytes("packet");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = resolve_warriors(
            &warriors,
            &packet,
            (case.n::<i64>("spell_id") % 1000 / 100 - 1) as i16,
            case.n::<i16>("count"),
            &[1; 6],
            false,
            &mut random,
        )
        .unwrap();
        let actual = support::record_bytes(&result.warriors);
        support::native_expect! {
            bytes actual => &case["after"], &format!("case {index}");
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_enemy_spells";
            scalar (result
                        .targets
                        .iter()
                        .flat_map(|(_, target)| target.sounds.iter().copied())
                        .collect::<Vec<_>>()) => &case["sounds"], "binary_enemy_spells";
        }
    }
}

#[test]
fn enemy_spell_selection_matches_original_targets_caster_and_random_consumption() {
    use centurions_content::combat::MonsterRecord;
    use centurions_core::combat::spells::select_targets;
    let hive = support::fixture("hive-native-memory.json");
    let mut hive_compared = 0;
    let mut additional: serde_json::Value = support::fixture("additional-spells.json");
    additional["cases"] = additional["selection_cases"].take();
    let fixtures: [serde_json::Value; 6] = [
        support::fixture("enemy-spell-selection.json"),
        support::fixture("npc-area-selection.json"),
        support::fixture("firearm-targets.json"),
        additional,
        support::fixture("condition-targets.json"),
        support::fixture("zero-area-selection.json"),
    ];
    for fixture in fixtures {
        let areas = fixture.bytes("areas");
        let mission = support::terrain_mission(&fixture);
        for (index, case) in fixture.array("cases").iter().enumerate() {
            let warriors = support::warriors(&case.bytes("party"));
            let distance = case.n::<i16>("distance");
            let tiles = support::words(&case.bytes("map"));
            let mut combat = support::battle();
            combat.battle = 28;
            combat.field.tiles = tiles;
            combat.squad.pos = (0..6)
                .map(|i| {
                    (
                        20 + distance + i * if case["weapon_id"].is_null() { 1 } else { 4 },
                        20,
                    )
                })
                .collect();
            combat.roster.records = vec![MonsterRecord {
                bytes: case.bytes("caster_before"),
            }];
            combat.roster.mon_pos = vec![(20, 20)];
            combat.battle_order = vec![-1, 10];
            let mut random = RandomState::new(case.n::<u32>("seed_before"));
            let packet = case.bytes("packet");
            let rotated = packet[7] != 0;
            let result = if case["weapon_id"].is_null() {
                select_targets(
                    &combat,
                    &warriors,
                    &mission,
                    &areas,
                    0,
                    &packet,
                    &mut random,
                )
            } else {
                centurions_core::combat::spells::select_firearm_targets(
                    &combat,
                    &warriors,
                    &mission,
                    &areas,
                    0,
                    &centurions_core::combat::firearm::PreparedFirearm {
                        packet,
                        slot: 0,
                        shots: case.n::<i16>("shots"),
                        rounds: 0,
                        weight: 0,
                        armor_filter: false,
                        sound: 0,
                    },
                    &mut random,
                )
            };
            let result = result.unwrap();
            if let Some(native) = hive
                .array("fault_controls")
                .iter()
                .find(|native| native["input_case"] == case["case"])
            {
                assert_eq!(result.cast, native.n::<i16>("selected_word") >= 0);
                hive_compared += 1;
            }
            if result.cast && rotated {
                assert_eq!(
                    result.rotation,
                    case.array("draws").last().unwrap().n::<u8>("result") - 1,
                );
            }
            support::native_expect! {
                scalar result.range => &case["range"], "binary_enemy_spells";
                scalar result.nearest => &case["nearest"], "binary_enemy_spells";
                scalar result.count => &case["count"], "binary_enemy_spells";
                bytes support::word_bytes(&result.distances) => &case["distances"], "distances case {index}";
            }
            if case["invalid_shape"].as_bool().unwrap_or(false) {
                // These original receipts stop before bestchoice: retain their measured prefix.
                support::assert_rng(&random, &case["seed_after"], &case["rng_draws"]);
                continue;
            }
            support::native_expect! {
                bytes &result.caster.bytes => &case.bytes("caster_after"), "caster";
            };
            let mut targets = [255; 42];
            for (slot, target) in result.targets.iter().enumerate() {
                targets[slot * 6..slot * 6 + 6].copy_from_slice(
                    &[target.first, target.second, target.kind]
                        .iter()
                        .flat_map(|v| v.to_be_bytes())
                        .collect::<Vec<_>>(),
                );
            }
            support::native_expect! {
                bytes targets => &case["targets"], &format!("targets case {index}");
                scalar !result.targets.is_empty() => &case["cast"], "binary_enemy_spells";
                rng random => (&case["seed_after"], &case["rng_draws"]), "binary_enemy_spells";
            }
        }
    }
    assert_eq!(hive_compared, hive.array("fault_controls").len());
}

#[test]
fn spell_target_masks_reflection_and_fields_match_original_complete_buffers() {
    use centurions_core::combat::spells::{SpellContext, SpellTarget, target_mask};
    let fixture: serde_json::Value = support::fixture("spell-targets.json");
    let areas = fixture.bytes("areas");
    let tiles: Vec<i16> = support::words(&fixture.bytes("map"));
    let party = fixture.bytes("party");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let variant = case.n::<i64>("variant");
        let mut warriors = support::warriors(&party);
        if variant == 1 {
            for warrior in &mut warriors {
                warrior.bytes[0x9a..0x9c].copy_from_slice(&1_i16.to_be_bytes());
            }
        }
        let monsters = support::monsters(&case["records"]);
        let mut combat = support::battle();
        combat.battle = 28;
        combat.field.tiles = tiles.clone();
        combat.squad.pos = (0..6).map(|i| (20 + i, 20)).collect();
        combat.roster.records = monsters;
        combat.roster.mon_pos = vec![(20, 22), (22, 22)];
        combat.battle_order = vec![-1, 10];
        let mut fields = vec![0; 1200];
        if variant == 2 {
            for slot in fields.as_chunks_mut::<20>().0 {
                slot[15] = 1;
            }
        }
        let packet = case.bytes("packet");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = target_mask(
            &combat,
            &warriors,
            &areas,
            &SpellTarget {
                first: case.n::<i16>("first"),
                second: case.n::<i16>("second"),
                kind: case.n::<i16>("kind"),
            },
            &SpellContext {
                packet: &packet,
                id: case.n::<i16>("spell_id"),
                count: case.n::<i16>("count"),
                caster: 10,
                initiative: 1,
                collision: false,
            },
            &fields,
            &mut random,
        )
        .unwrap();
        support::native_expect! {
            bytes support::word_bytes(&result.mask) => &case["mask"], &format!("mask case {index}");
            bytes result.fields => &case["fields"], &format!("fields case {index}");
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_enemy_spells";
        }
    }
}

#[test]
fn reached_retained_packets_match_original_resolution_and_random_order() {
    let fixture = support::fixture("campaign-spell-boundaries.json");
    for case in fixture.array("cases").iter().take(3) {
        let state: Session = serde_json::from_value(case["session"].clone()).unwrap();
        let combat = state.combat.as_ref().unwrap();
        let packet = case.n::<Vec<u8>>("packet");
        let mask = case.n::<Vec<i16>>("mask");
        let mut random = state.random.clone();
        let (warriors, enemies, sounds) = if word(&packet, 40).unwrap() == 36 {
            let result = spells::resolve_support_for_caster(
                &state.warriors,
                &combat.roster.records,
                &mask,
                1703,
                &DirectSpell {
                    packet: &packet,
                    level: 6,
                    count: case.n("count"),
                    armor_filter: false,
                    weapon_sound: 0,
                },
                (combat.active_body(), None),
                &mut random,
            )
            .unwrap();
            let sounds = result
                .sounds
                .into_iter()
                .chain(
                    result
                        .targets
                        .into_iter()
                        .flat_map(|(_, target)| target.sounds),
                )
                .collect::<Vec<_>>();
            (result.warriors, result.enemies, sounds)
        } else {
            let result = resolve_bodies(
                &state.warriors,
                &combat.roster.records,
                &mask,
                &DirectSpell {
                    packet: &packet,
                    level: case["identity"][1].as_i64().unwrap() as i16,
                    count: case.n("count"),
                    armor_filter: false,
                    weapon_sound: 0,
                },
                &mut random,
            )
            .unwrap();
            let sounds = result
                .sounds
                .into_iter()
                .chain(
                    result
                        .targets
                        .into_iter()
                        .flat_map(|(_, target)| target.sounds),
                )
                .collect::<Vec<_>>();
            (result.warriors, result.enemies, sounds)
        };
        let native = &case["native"];
        support::native_expect! {
            records &warriors => &native["party_after"], case.text("name");
            records &enemies => &native["records_after"], case.text("name");
        };
        assert_eq!(random.seed, native.n::<u32>("seed"), "{}", case["name"]);
        assert_eq!(random.draws - state.random.draws, native.n::<u64>("draws"));
        support::expect_array(&sounds, &native["sounds"]);
    }
    let content = content();
    for case in fixture
        .array("cases")
        .iter()
        .filter(|c| c.text("name").starts_with("script"))
    {
        let (state, context_content) = campaign_boundary_context(case, &content);
        let combat = state.combat.as_ref().unwrap();
        let map = context_content
            .map_at(
                state.mission.as_deref(),
                state.scenario_level,
                state.exploration.as_ref().unwrap().map,
            )
            .unwrap();
        let point = words(&case["native"].bytes("point_alias"));
        let target = spells::SpellTarget {
            first: point[0] + combat.field.map_shift.0 + 5,
            second: point[1] + combat.field.map_shift.1 + 5,
            kind: 3,
        };
        let origin = spells::SpellTarget {
            first: case.get("first").map_or(63, |v| v.as_i64().unwrap() as i16),
            second: case
                .get("second")
                .map_or(38, |v| v.as_i64().unwrap() as i16),
            kind: 0,
        };
        let packet = case.n::<Vec<u8>>("packet");
        let mut random = state.random.clone();
        let position = state.exploration.as_ref().unwrap();
        let aliases = centurions_core::combat::map_alias::MapAliases::of_mission(map)
            .with_land(&position.objects, &position.events)
            .with_stages(&context_content.scenario.as_ref().unwrap().stages);
        let result = spells::target_mask_with_map_aliases(
            combat,
            &state.warriors,
            &map.areas,
            (&target, &origin),
            &spells::SpellContext {
                packet: &packet,
                id: 2606,
                count: 2,
                caster: state.combat_story.as_ref().unwrap().actor,
                initiative: combat.battle_order_index as i16,
                collision: false,
            },
            &combat.spell_queue,
            &mut random,
            aliases,
        )
        .unwrap();
        assert_eq!(result.mask.as_slice(), words(&case["native"].bytes("mask")));
        assert_eq!(result.fields, case["native"].bytes("fields_after"));
        assert_eq!(random.seed, case["native"].n::<u32>("seed"));
        assert_eq!(
            random.draws - state.random.draws,
            case["native"].n::<u64>("draws")
        );
    }
}

#[test]
fn recovered_campaign_boundaries_resume_without_interventions() {
    let fixture = support::fixture("campaign-spell-boundaries.json");
    let content = content();
    let path = std::env::temp_dir().join(format!(
        "centurions-campaign-boundary-{}.json",
        std::process::id()
    ));
    for (index, case) in fixture.array("cases").iter().enumerate() {
        if case["guest"].as_bool().unwrap_or(false)
            && case.get("mission_sentinel").is_none()
            && case.get("event_sentinel").is_none()
        {
            continue;
        }
        let (mut state, content) = campaign_boundary_context(case, &content);
        if index == 3 {
            let mut unknown = state.clone();
            unknown.combat_story.as_mut().unwrap().entry.x = 64;
            let before = unknown.clone();
            let error = apply(Command::Continue, &mut unknown, &content).unwrap_err();
            assert!(matches!(error.root(), RuleError::Unsupported { .. }));
            assert_eq!(unknown, before);
        }
        if index == 6 {
            let mut unknown = state.clone();
            unknown.combat.as_mut().unwrap().field.map_shift.0 = -9;
            let before = unknown.clone();
            let error = apply(Command::Continue, &mut unknown, &content).unwrap_err();
            assert!(matches!(error.root(), RuleError::Unsupported { .. }));
            assert_eq!(unknown, before);
        }
        save(&path, &state, &content).unwrap();
        let mut resumed = load(&path, &content).unwrap();
        let mut command = if case.text("name").starts_with("script") {
            Command::Continue
        } else {
            Command::RunEnemyTurn
        };
        if case["small_field"].as_bool().unwrap_or(false) {
            command = Command::WithPreferences {
                command: Box::new(command),
                preferences: centurions_core::CommandPreferences {
                    small_combat_field: true,
                    ..Default::default()
                },
            };
        }
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert_eq!(resumed, state);
        if case.get("mission_sentinel").is_some() || case.get("event_sentinel").is_some() {
            assert!(events.iter().any(|event| matches!(
                event,
                Event::SpellResolved {
                    spell: 2606,
                    target: 1,
                    ..
                }
            )));
        }
        if case.text("name").starts_with("last-hostile") {
            assert!(
                events
                    .iter()
                    .any(|event| matches!(event, Event::AttackResolved { defeated: true, .. }))
            );
        }
    }
    std::fs::remove_file(path).unwrap();
}

fn campaign_boundary_context(case: &Value, content: &Content) -> (Session, Content) {
    let mut state = support::source_frame(
        case.get("command_session")
            .unwrap_or(&case["session"])
            .clone(),
    );
    let mut content = content.clone();
    if let Some(address) = case["native"]["layout"]["mission_sentinel_address"].as_str() {
        let offset = usize::from_str_radix(address, 16).unwrap() - 0x41d1c4;
        content.scenario.as_mut().unwrap().stages[offset / 464].record
            [offset % 464..offset % 464 + 2]
            .copy_from_slice(&1_i16.to_be_bytes());
    }
    if let Some(address) = case["native"]["layout"]["event_sentinel_address"].as_str() {
        let offset = usize::from_str_radix(address, 16).unwrap() - 0x410448;
        let value = &mut state.exploration.as_mut().unwrap().events[offset / 4];
        *value = if offset.is_multiple_of(4) {
            (*value as u32 & 0xffff) | 0x10000
        } else {
            (*value as u32 & 0xffff0000) | 1
        } as i32;
    }
    // VerifyCampaignSpells loads the cold A5 zero pair before its declared entry.
    let selected = state.combat.as_ref().unwrap().monster_up;
    crate::native_common::monster_command_input(&mut state, selected, [0, 0]);
    (state, content)
}
