use crate::prelude::*;
use crate::support::warriors as party;

fn positions(hex: &str) -> Vec<(i16, i16)> {
    words(hex)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect()
}
fn fixture() -> Value {
    crate::support::fixture("player-psi.json")
}
use crate::support::assert_bytes;

fn controlled(case: &Value) -> Combat {
    let mut order = vec![-1; 110];
    order[1] = 1;
    Combat {
        difficulty: 0,
        battle: 85,
        map: TacticalMap {
            tiles: words(case.text("map_before")),
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(case.text("warrior_positions")),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: crate::support::monsters(&case["records_before"]),
            positions: positions(case.text("monster_positions")),
        },
        order,
        spell_fields: case.bytes("fields_before"),
        attack_packet: None,
        attack_identity: None,
        ..crate::support::battle()
    }
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
            spell: id,
            power,
            target,
        },
    )
    .unwrap();
    let actual_party: Vec<_> = crate::support::record_bytes(&warriors);
    let actual_records: Vec<_> = crate::support::record_bytes(&combat.enemies.records);
    crate::support::native_expect! {
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
    crate::support::native_expect! {
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
    crate::support::native_expect! {
    scalar random.seed => &case["seed_after"], "binary_player_psi";
    scalar random.draws => &case["rng_draws"], "binary_player_psi";
    }
    let sounds: Vec<_> = crate::support::event_sounds(&events);
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
    let data = fixture();
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
    let data = fixture();
    let content = content();
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
}
#[test]
fn player_psi_admission_matches_16_native_gates_without_mutation() {
    let data = fixture();
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
        let actual: Vec<_> = crate::support::record_bytes(&warriors);
        assert_bytes(&actual, &case.bytes("after"), "admission unchanged");
    }
}
#[test]
fn player_psi_affordability_and_range_match_32_native_boundaries() {
    let data = fixture();
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
        crate::support::expect(
            mp >= player_psi::cost(&packet, power).unwrap(),
            &case["affordable"],
        );
        let range = player_psi::range(&packet, power).unwrap();
        crate::support::native_expect! {
        scalar range => &case["range"], "binary_player_psi";
        scalar player_psi::in_range((20, 20), (20 + distance, 20), range) => &case["in_range"], "binary_player_psi";
        }
    }
}
