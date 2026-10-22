use crate::{prelude::*, support};
use support::warriors as party;

use support::assert_bytes;
use support::positions;

fn controlled(case: &Value) -> Combat {
    let mut order = vec![-1; 110];
    order[1] = 1;
    let mut combat = support::battle();
    combat.battle = 85;
    combat.map.tiles = case.words("map_before");
    combat.squad.positions = positions(case.text("warrior_positions"));
    combat.enemies.records = support::monsters(&case["records_before"]);
    combat.enemies.positions = positions(case.text("monster_positions"));
    combat.order = order;
    combat.spell_fields = case.bytes("fields_before");
    combat.attack_packet = None;
    combat.attack_identity = None;
    combat
}
fn verify(
    case: &Value,
    mut combat: Combat,
    mission: &centurions_content::mission::OpeningMission,
    context: &str,
) {
    let mut warriors = party(case.text("warriors_before"));
    let mut random = RandomState::new(case.n::<u32>("seed_before"));
    let id = case.n::<i16>("spell_id");
    let power = case.n::<i16>("power");
    let target = if id == 2204 {
        let raw = words(case["casts"][0]["targets"].as_str().unwrap());
        Some((
            raw[0] - combat.map.shift.0 - 5,
            raw[1] - combat.map.shift.1 - 5,
        ))
    } else {
        None
    };
    let mut memory = vec![0; 480];
    let events = player_psi::cast(
        &mut combat,
        &mut warriors,
        mission,
        &mut random,
        &mut memory,
        0,
        CastRequest {
            rotation: 0,
            spell: id,
            power,
            target,
        },
    )
    .unwrap();
    let actual_party: Vec<_> = support::record_bytes(&warriors);
    let actual_records: Vec<_> = support::record_bytes(&combat.enemies.records);
    support::native_expect! {
        bytes actual_party => &case["warriors_after"], &format!("{context} party");
        bytes actual_records => &case["records_after"], &format!("{context} enemies");
        words &(combat.map.tiles) => &case["map_after"], "binary_player_psi";
    }
    assert_eq!(
        combat.squad.positions,
        positions(case.text("warrior_positions")),
        "{context} warrior positions"
    );
    assert_eq!(
        combat.enemies.positions,
        positions(case.text("monster_positions")),
        "{context} monster positions"
    );
    support::native_expect! {
        bytes combat.spell_fields => &case["fields_after"], &format!("{context} fields");
        bytes combat.field_used => &case["used_after"], &format!("{context} used");
        bytes memory => &case["memory_after"], &format!("{context} memory");
    }
    assert_eq!(
        combat.attack_packet,
        Some(case["resolutions"][0].bytes("packet")),
        "{context} packet"
    );
    assert_eq!(
        combat.attack_identity,
        Some([id / 1000 - 1, id % 1000 / 100 - 1, id % 100 - 1]),
        "{context} descriptor"
    );
    support::assert_rng(&random, &case["seed_after"], &case["rng_draws"]);
    let sounds: Vec<_> = support::event_sounds(&events);
    let expected: Vec<_> = case.n::<Vec<i16>>("sounds");
    assert_eq!(sounds, expected, "{context} sounds");
    let mut applied = vec![0; 110];
    for event in events {
        if let Event::SpellResolved { target, .. } = event {
            applied[target as usize] = 1;
        }
    }
    assert_eq!(
        applied,
        words(case["resolutions"][0]["applied_mask"].as_str().unwrap()),
        "{context} applied mask"
    );
}

#[test]
fn learned_player_psi_matches_96_complete_native_casts() {
    let data = support::fixture("player-psi.json");
    let content = content();
    let mission = content.map(0).unwrap();
    let cases = data.array("cases");

    for (i, case) in cases.iter().enumerate() {
        verify(
            case,
            controlled(case),
            mission,
            &format!("player psi case {i}"),
        );
    }
}
#[test]
fn learned_player_psi_matches_two_native_campaign_inputs() {
    let data = support::fixture("player-psi.json");
    let content = content();
    let startup =
        std::fs::read(support::root().join("analysis/ghidra/startup/a5-initial.bin")).unwrap();
    assert_eq!(
        digest(&startup),
        support::fixture("startup.json")["initialized_sha256"]
    );
    let retained = centurions_core::combat::weapon_work::WeaponWork {
        natural_attack_index: Some(0),
        sound: Some(word(&startup, 0x453776 - 0x3fe210).unwrap()),
        weapon_armor_filter: Some(word(&startup, 0x4537f0 - 0x3fe210).unwrap()),
        ammunition_armor_filter: Some(word(&startup, 0x45375a - 0x3fe210).unwrap()),
    };
    assert_eq!(Session::new(&content).native_work.weapon_work, retained);
    let foundation = &data["route_foundation"];
    let combat: Combat = serde_json::from_value(foundation["combat"].clone()).unwrap();
    let mission = content
        .map(foundation["exploration"].n::<i32>("map"))
        .unwrap();
    let cases = data.array("route_cases");

    for (i, case) in cases.iter().enumerate() {
        let mut combat = combat.clone();
        combat.weapon_work = centurions_core::combat::weapon_work::WeaponWork::zero();
        combat.spell_fields.fill(0);
        combat.field_used.fill(0);
        verify(case, combat, mission, &format!("campaign psi case {i}"));
    }
    // Resolution extensions are source-derived; ray dispatch also compares original instructions.
    let rays = support::fixture("ray-dispatch.json");
    let movements = support::fixture("movement-psi.json");
    let drains = support::fixture("drain-caller.json");
    let charms = support::fixture("charm-result.json");
    for (spell, gate, blocked, ray_case) in [
        1106, 1403, 1406, 1502, 1506, 1603, 1606, 1607, 1705, 2103, 2201, 2305, 2401, 2405, 2505,
        2506, 2604, 2607, 2701, 2707, 3104, 3105, 3207, 3307, 3503, 3506, 3604, 3607, 3705,
    ]
    .into_iter()
    .filter(|spell| !matches!(spell, 2103 | 2506 | 2607 | 2707 | 3104))
    .map(|spell| (spell, 1, false, None))
    .chain([
        (1101, 1, false, None),
        (1106, 0, false, None),
        (1106, 1, true, None),
    ])
    .chain(
        rays.array("cases")
            .iter()
            .map(|case| (case.n::<i16>("spell"), 1, false, Some(case))),
    ) {
        let mut state: Session = serde_json::from_value(foundation.clone()).unwrap();
        state.native_work.psi_memory = Some(vec![0; 480]);
        state.bandage_ready = Some(gate);
        let actor = 1;
        let overflow = spell == 1101;
        let power = if ray_case.is_some() {
            3
        } else if overflow {
            2
        } else {
            1
        };
        let w = &mut state.warriors[actor].bytes;
        w[0x36..0x38].copy_from_slice(&(spell / 1000_i16).to_be_bytes());
        w[0x9f0..0xa21].fill(1);
        for (offset, value) in [
            (0x58, 30000_i16),
            (0x50, 30000),
            (0x40, 5),
            (0x3c, 5),
            (0xac6, 100),
            (0xac8, 0),
            (0x14, 0),
        ] {
            w[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        let combat = state.combat.as_mut().unwrap();
        combat.order[combat.cursor] = actor as i16;
        combat.weapon_work = WeaponWork::zero();
        let charm = matches!(spell, 1406 | 1506 | 3506);
        let drain = matches!(spell, 1502 | 1603 | 3503);
        let movement = matches!(spell, 1106 | 1403 | 2305 | 2604 | 3105);
        let revival = matches!(spell, 1705 | 3705);
        if movement || overflow {
            state.random = RandomState::new(12);
        }
        if let Some(case) = ray_case {
            combat.map.tiles = case.words("map");
            combat.map.shift = (0, 0);
            for index in 0..3 {
                let position = (
                    39 + case.n::<i16>("dx") * (index + 1),
                    50 + case.n::<i16>("dy") * (index + 1),
                );
                if index < 2 {
                    combat.enemies.positions[index as usize] = position;
                    combat.enemies.records[index as usize].bytes[0x110..0x112]
                        .copy_from_slice(&30000_i16.to_be_bytes());
                } else {
                    combat.squad.positions[0] = position;
                    state.warriors[0].bytes[0x50..0x52].copy_from_slice(&30000_i16.to_be_bytes());
                }
            }
        }
        let target = if let Some(case) = ray_case {
            (39 + 4 * case.n::<i16>("dx"), 50 + 4 * case.n::<i16>("dy"))
        } else if charm || drain {
            let m = &mut combat.enemies.records[1].bytes;
            m[0xa..0xc].fill(0);
            m[0xc2..0xc9].fill(0);
            m[0x90] = 0;
            m[0x8c..0x8e].fill(0);
            m[0x10e..0x110].copy_from_slice(&1000_i16.to_be_bytes());
            combat.enemies.positions[1]
        } else if overflow {
            for offset in [0x50, 0x52] {
                state.warriors[0].bytes[offset..offset + 2]
                    .copy_from_slice(&i16::MAX.to_be_bytes());
            }
            state.warriors[0].bytes[0x9a..0x9c].fill(0);
            combat.squad.positions[0]
        } else if revival {
            state.warriors[0].bytes[0x50..0x52].copy_from_slice(&(-10_i16).to_be_bytes());
            centurions_core::combat::lifecycle::kill_body(combat, &mut state.warriors, 0).unwrap();
            combat.squad.positions[0]
        } else if movement {
            if blocked {
                combat.squad.positions[0]
            } else {
                (38, 50)
            }
        } else {
            combat.squad.positions[actor]
        };
        if spell == 1705 {
            let mut portrait = state.clone();
            support::saved_command(
                Command::BeginPsiTargeting { spell, power },
                &mut portrait,
                &content,
            );
            let native = support::fixture("portrait-eligibility.json");
            let sound = native["portrait_eligibility"][0].n::<Vec<i16>>("sounds")[0];
            support::refuse_command(
                &portrait,
                &content,
                Command::FirePreparedPsiOnWarrior { warrior: 0 },
                vec![centurions_core::feedback::NativeFeedback::Sound { resource_id: sound }],
            );
        }
        let target_before = state.warriors[0].bytes.clone();
        let before = state.warriors[actor].psi_points().unwrap();
        let mut resumed = support::checkpoint(&state, &content, "learned-family-before");
        let command = if matches!(spell, 1406 | 2405 | 2701 | 3506) {
            replay_command(
                Command::BeginPsiSelection { spell, power },
                &mut state,
                &mut resumed,
                &content,
            );
            replay_command(
                Command::SelectPsiTarget {
                    x: target.0,
                    y: target.1,
                },
                &mut state,
                &mut resumed,
                &content,
            );
            resumed = support::checkpoint(&state, &content, "learned-family-selection");
            Command::FirePsiSelection
        } else if revival || overflow {
            Command::CastPsiOnWarrior {
                warrior: 0,
                spell,
                power,
            }
        } else {
            Command::CastPsi {
                rotation: 0,
                spell,
                power,
                target: Some(target),
            }
        };
        let command = Command::WithPreferences {
            command: Box::new(command),
            preferences: centurions_core::CommandPreferences {
                animation_delay: ray_case.map_or(0, |case| case.n("delay")),
                ..Default::default()
            },
        };
        let events = replay_command(command, &mut state, &mut resumed, &content);
        if gate == 0 {
            assert!(events.contains(&Event::Feedback(
                centurions_core::feedback::NativeFeedback::Warning { index: 87 }
            )));
        } else {
            assert!(events.iter().any(|event| matches!(event, Event::SpellResolved { spell: actual, .. } if *actual == spell)), "spell {spell}");
        }
        assert!(state.warriors[actor].psi_points().unwrap() < before);
        if movement && !blocked {
            let case = movements
                .array("cases")
                .iter()
                .find(|case| case.n::<i16>("spell") == spell && case.n::<i16>("gate") == gate)
                .unwrap();
            let combat = current_combat(&state);
            support::native_expect! {
                records &state.warriors => &case["warriors_after"], "movement party";
                words combat.map.tiles => &case["map_after"], "movement terrain";
                bytes state.native_work.psi_memory.as_ref().unwrap() => &case["memory_after"], "movement memory";
                scalar state.bandage_ready.unwrap() => &case["gate_after"], "movement gate";
                scalar state.random.draws => &case["draws"], "movement draws";
            }
            assert_eq!(combat.squad.positions, positions(&case["positions_after"]));
            support::expect_array(
                &support::event_sounds(&events)[1..].to_vec(),
                &case["sounds"],
            );
        } else if movement {
            assert_eq!(state.warriors[actor].stamina().unwrap(), -10);
            assert!(current_combat(&state).active_body().is_some());
        }
        if charm || drain {
            let case = (if charm { &charms } else { &drains })
                .array("cases")
                .iter()
                .find(|case| case.n::<i16>("spell") == spell)
                .unwrap();
            support::native_expect! {
                records &state.warriors => &case["warriors_after"], "Psi party";
                bytes support::record_bytes(&current_combat(&state).enemies.records) => &case["records_after"], "Psi NPCs";
            }
            if charm {
                support::expect(state.bandage_ready.unwrap(), &case["gate_after"]);
            }
        }
        if overflow {
            let native = support::fixture("healing-overflow.json");
            let defeated: Vec<_> = events
                .iter()
                .filter_map(|e| match e {
                    Event::SpellResolved {
                        target,
                        defeated: true,
                        ..
                    } => Some(*target),
                    _ => None,
                })
                .collect();
            support::expect(defeated, &native["killbody_arguments"]);
            assert_eq!(
                state.warriors[0].stamina().unwrap(),
                word(&native.bytes("party_after"), 0x50).unwrap()
            );
            support::assert_rng(&state.random, &native["seed_after"], &native["rng_draws"]);
        }
        if revival {
            let mut expected = target_before;
            support::patch_bytes(&mut expected, &support::fixture("revival.json")["changes"]);
            assert_eq!(state.warriors[0].bytes, expected);
        }
        if let Some(case) = ray_case {
            let targets: Vec<_> = events
                .iter()
                .filter_map(|event| match event {
                    Event::SpellResolved { target, .. } => Some(*target),
                    _ => None,
                })
                .collect();
            support::expect_array(&targets, &case["targets"]);
        }
    }
}
#[test]
fn player_psi_admission_matches_16_native_gates_without_mutation() {
    let data = support::fixture("player-psi.json");
    let cases = data.array("gate_cases");

    for case in cases {
        let warriors = party(case.text("before"));
        let admission = player_psi::admission(
            &warriors,
            case.n::<i16>("member"),
            u8::from(case.n::<bool>("disabled")),
        )
        .unwrap();
        assert_eq!(admission.allowed, case.n::<i64>("allowed") != 0);
        let sounds: Vec<_> = case.n::<Vec<i16>>("sounds");
        assert_eq!(admission.sounds, sounds);
        let actual: Vec<_> = support::record_bytes(&warriors);
        assert_bytes(&actual, &case.bytes("after"), "admission unchanged");
    }
}
#[test]
fn player_psi_affordability_and_range_match_32_native_boundaries() {
    let data = support::fixture("player-psi.json");
    let content = content();
    let packet = spells::load_packet(
        &content.map(0).unwrap().combat.as_ref().unwrap().attacks,
        2204,
    )
    .unwrap();
    let cases = data.array("prerequisite_cases");

    for case in cases {
        let power = case.n::<i16>("power");
        let mp = case.n::<i16>("mp");
        let distance = case.n::<i16>("distance");
        support::expect(
            mp >= player_psi::cost(&packet, power).unwrap(),
            &case["affordable"],
        );
        let range = player_psi::range(&packet, power).unwrap();
        support::native_expect! {
            scalar range => &case["range"], "binary_player_psi";
            scalar player_psi::in_range((20, 20), (20 + distance, 20), range) => &case["in_range"], "binary_player_psi";
        }
    }
}
