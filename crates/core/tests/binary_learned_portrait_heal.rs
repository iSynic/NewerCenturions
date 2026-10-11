use centurions_content::{Content, Warrior, word};
use centurions_core::combat::{medical, spells};
use centurions_core::{
    Event, MedicalContinuation, MedicalOrigin, MedicalTarget, RandomState, Session,
};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/learned-portrait-heal.json"
    ))
    .unwrap()
}

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|word| i16::from_be_bytes(*word))
        .collect()
}

fn party(value: &Value) -> Vec<Warrior> {
    bytes(value)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn assert_party(actual: &[Warrior], expected: &Value) {
    for (member, (actual, expected)) in actual.iter().zip(party(expected)).enumerate() {
        assert!(
            actual.bytes == expected.bytes,
            "member {member}: {:?}",
            actual
                .bytes
                .iter()
                .zip(&expected.bytes)
                .enumerate()
                .filter(|(_, (actual, expected))| actual != expected)
                .take(24)
                .collect::<Vec<_>>()
        );
    }
}

fn sounds(events: &[Event]) -> Vec<i16> {
    events
        .iter()
        .filter_map(|event| {
            if let Event::Sound { resource_id } = event {
                Some(*resource_id)
            } else {
                None
            }
        })
        .collect()
}

fn state(fixture: &Value, case: &Value, snapshot: &str) -> Session {
    let foundation = case["foundation"].as_u64().unwrap() as usize;
    let mut state: Session =
        serde_json::from_value(fixture["foundations"][foundation]["session"].clone()).unwrap();
    let snapshot = &case[snapshot];
    state.warriors = party(&snapshot["party"]);
    state.random = RandomState {
        seed: snapshot["seed"].as_u64().unwrap() as u32,
        draws: snapshot["draws"].as_u64().unwrap(),
    };
    let combat = state.combat.as_mut().unwrap();
    combat.attack_packet = Some(bytes(&snapshot["packet"]));
    combat.attack_identity = Some(words(&snapshot["identity"])[..3].try_into().unwrap());
    combat.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    combat.round = snapshot["round"].as_i64().unwrap() as i16;
    combat.order = words(&snapshot["order"]);
    combat.map.tiles = words(&snapshot["map"]);
    combat.squad.positions = words(&snapshot["party_positions"])
        .as_chunks::<2>()
        .0
        .iter()
        .map(|position| (position[0], position[1]))
        .collect();
    combat.squad.under_tiles = words(&snapshot["under_tiles"]);
    combat.unavailable = snapshot["unavailable"].as_i64().unwrap() as i8;
    combat.spell_fields = bytes(&snapshot["fields"]);
    combat.field_used = bytes(&snapshot["used"]);
    combat
        .weapon_work
        .load_pair(
            &bytes(&snapshot["weapon_work"]),
            &bytes(&snapshot["ammo_work"]),
        )
        .unwrap();
    state
}

fn pending(case: &Value) -> MedicalContinuation {
    MedicalContinuation {
        origin: MedicalOrigin::LearnedHeal,
        actor: 5,
        item: 0,
        spell: 1101,
        power: case["power"].as_i64().unwrap() as i16,
    }
}

fn assert_world_unchanged(before: &Session, after: &Session) {
    assert_eq!(after.ammunition, before.ammunition);
    assert_eq!(after.crate_items, before.crate_items);
    assert_eq!(after.booty_items, before.booty_items);
    assert_eq!(after.psi_memory, before.psi_memory);
    assert_eq!(after.party_buffs, before.party_buffs);
    assert_eq!(after.health_context, before.health_context);
    let actual = after.combat.as_ref().unwrap();
    let previous = before.combat.as_ref().unwrap();
    assert_eq!(actual.enemies, previous.enemies);
    assert_eq!(actual.spell_fields, previous.spell_fields);
    assert_eq!(actual.field_used, previous.field_used);
    assert_eq!(actual.death_queue, previous.death_queue);
    assert_eq!(actual.weapon_work, previous.weapon_work);
    assert_eq!(actual.cursor, previous.cursor);
    assert_eq!(actual.round, previous.round);
    assert_eq!(actual.round_scripts, previous.round_scripts);
}

#[test]
fn nine_accepted_preparations_match_native_party_packet_work_and_charges() {
    let fixture = fixture();
    let content = content();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["prepared"] == true)
    {
        let before = state(&fixture, case, "before");
        let original = before.clone();
        let power = case["power"].as_i64().unwrap() as i16;
        let result = medical::begin_learned_heal(&before, &content, power).unwrap();
        assert_eq!(before, original);
        let mut actual = before.warriors.clone();
        actual[5] = result.warrior;
        assert_party(&actual, &case["after_preparation"]["party"]);
        assert_eq!(result.packet, bytes(&case["after_preparation"]["packet"]));
        assert_eq!(result.identity, [0, 0, 0]);
        assert_eq!(result.count, power);
        assert_eq!(
            result.weapon_work,
            before.combat.as_ref().unwrap().weapon_work
        );
        assert_eq!(
            result.weapon_work.sound().unwrap(),
            if case["foundation"] == 0 { 350 } else { 363 }
        );
        assert_eq!(
            result.loaded_item_field32,
            before.loaded_item_field32.unwrap()
        );
        assert_eq!(
            sounds(&result.events),
            if power == 1 {
                vec![141, 20002, 141]
            } else {
                vec![141, 20002, 141, 137]
            }
        );
        for (a, b) in actual.iter().zip(&before.warriors) {
            assert_eq!(a.inventory().unwrap(), b.inventory().unwrap());
        }
        assert_eq!(word(&actual[5].bytes, 0x40).unwrap(), 3);
        assert_eq!(word(&actual[5].bytes, 0x3c).unwrap(), 8);
        assert_eq!(
            word(&actual[5].bytes, 0x58).unwrap(),
            word(&before.warriors[5].bytes, 0x58).unwrap() - 6 * power
        );
        assert_eq!(
            word(&actual[5].bytes, 0xac8).unwrap(),
            word(&before.warriors[5].bytes, 0xac8).unwrap() + 1
        );
        assert_eq!(
            i32::from_be_bytes(actual[5].bytes[0xa66..0xa6a].try_into().unwrap()),
            i32::from_be_bytes(before.warriors[5].bytes[0xa66..0xa6a].try_into().unwrap()) + 1
        );
        assert_eq!(case["after_preparation"]["learned_marker"], 1);
        assert_eq!(case["after_preparation"]["heal_marks_affected"], 0);
        assert_eq!(case["before"]["draws"], case["after_preparation"]["draws"]);
    }
}

#[test]
fn four_organic_portraits_and_race_immunity_match_native_resolver_and_restoration() {
    let fixture = fixture();
    let content = content();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] == "actual" || case["kind"] == "race_three")
    {
        let mut actual = state(&fixture, case, "after_preparation");
        let before = actual.clone();
        let pending = pending(case);
        let target = case["target"].as_u64().unwrap() as usize;
        let mission = content
            .map(actual.exploration.as_ref().unwrap().map)
            .unwrap();
        let mut direct_random = before.random.clone();
        let mut mask = [0; 110];
        mask[target] = 1;
        let direct = spells::resolve_player_support(
            &before.warriors,
            &before.combat.as_ref().unwrap().enemies.records,
            &mask,
            before
                .combat
                .as_ref()
                .unwrap()
                .attack_packet
                .as_ref()
                .unwrap(),
            1101,
            pending.power,
            &mut direct_random,
        )
        .unwrap();
        assert_party(&direct.warriors, &case["resolutions"][0]["party_after"]);
        let result = medical::resolve(
            actual.combat.as_mut().unwrap(),
            &mut actual.warriors,
            mission,
            &mut actual.random,
            &pending,
            &MedicalTarget::Portrait { warrior: target },
        )
        .unwrap();
        assert_eq!(result.follow_up, medical::FollowUp::EndTurn);
        assert_eq!(
            sounds(&result.events),
            if case["kind"] == "actual" {
                vec![-141, 624, 631]
            } else {
                vec![-141, 651]
            }
        );
        assert_party(&actual.warriors, &case["after"]["party"]);
        assert_eq!(actual.random, direct_random);
        let mut receipt_random = before.random.clone();
        for draw in case["rng"]
            .as_array()
            .unwrap()
            .iter()
            .take(if case["kind"] == "actual" {
                1 + pending.power as usize
            } else {
                1
            })
        {
            assert_eq!(
                receipt_random.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(actual.random, receipt_random);
        assert_world_unchanged(&before, &actual);
        let combat = actual.combat.as_ref().unwrap();
        assert_eq!(
            combat.squad.positions,
            words(&case["after"]["party_positions"])
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| (p[0], p[1]))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            combat.squad.under_tiles,
            words(&case["after"]["under_tiles"])
        );
        assert_eq!(combat.map.tiles, words(&case["after"]["map"]));
        assert_eq!(combat.order, words(&case["after"]["order"]));
        assert_eq!(
            combat.unavailable,
            case["after"]["unavailable"].as_i64().unwrap() as i8
        );
        let mut packet = bytes(&case["after_preparation"]["packet"]);
        if case["kind"] == "actual" {
            packet[18] = 4;
        }
        assert_eq!(combat.attack_packet, Some(packet));
        assert_eq!(word(&actual.warriors[5].bytes, 0x40).unwrap(), 3);
        assert_eq!(word(&actual.warriors[5].bytes, 0x3c).unwrap(), 8);
        assert_eq!(
            word(&actual.warriors[5].bytes, 0x58).unwrap(),
            word(&before.warriors[5].bytes, 0x58).unwrap()
        );
    }
}

#[test]
fn confirmed_cancel_and_qualified_rejections_preserve_native_cost_lifecycles() {
    let fixture = fixture();
    let content = content();
    for case in fixture["cases"].as_array().unwrap() {
        let kind = case["kind"].as_str().unwrap();
        if kind == "confirmed_cancel" {
            let mut actual = state(&fixture, case, "after_preparation");
            let before = actual.clone();
            let pending = pending(case);
            let mission = content
                .map(actual.exploration.as_ref().unwrap().map)
                .unwrap();
            let result = medical::resolve(
                actual.combat.as_mut().unwrap(),
                &mut actual.warriors,
                mission,
                &mut actual.random,
                &pending,
                &MedicalTarget::ConfirmedCancel,
            )
            .unwrap();
            assert_party(&actual.warriors, &case["after"]["party"]);
            assert_eq!(actual.random, before.random);
            assert_eq!(result.follow_up, medical::FollowUp::None);
            assert_eq!(sounds(&result.events), vec![6000, 20005]);
            assert_world_unchanged(&before, &actual);
            assert_eq!(actual.combat, before.combat);
            assert_eq!(
                word(&actual.warriors[5].bytes, 0xac8).unwrap(),
                word(&party(&case["before"]["party"])[5].bytes, 0xac8).unwrap()
            );
            assert_eq!(
                actual.warriors[5].bytes[0xa66..0xa6a],
                before.warriors[5].bytes[0xa66..0xa6a]
            );
        } else if matches!(kind, "living_portrait" | "terminal_portrait") {
            let mut actual = state(&fixture, case, "after_preparation");
            let before = actual.clone();
            let pending = pending(case);
            let mission = content
                .map(actual.exploration.as_ref().unwrap().map)
                .unwrap();
            assert!(
                medical::resolve(
                    actual.combat.as_mut().unwrap(),
                    &mut actual.warriors,
                    mission,
                    &mut actual.random,
                    &pending,
                    &MedicalTarget::Portrait {
                        warrior: case["target"].as_u64().unwrap() as usize
                    }
                )
                .is_err()
            );
            assert_eq!(actual, before);
            assert_eq!(case["after"]["learned_marker"], 1);
            assert_eq!(case["after"]["party"], case["after_preparation"]["party"]);
        } else if matches!(kind, "insufficient_psi" | "cannot_cast" | "not_learned") {
            let actual = state(&fixture, case, "before");
            let before = actual.clone();
            assert!(medical::begin_learned_heal(&actual, &content, 1).is_err());
            assert_eq!(actual, before);
            assert_eq!(case["after"]["party"], case["before"]["party"]);
            assert!(case["rng"].as_array().unwrap().is_empty());
        }
    }
    let case = &fixture["cases"][0];
    let mut actual = state(&fixture, case, "after_preparation");
    let before = actual.clone();
    let pending = pending(case);
    let mission = content
        .map(actual.exploration.as_ref().unwrap().map)
        .unwrap();
    let p = actual.combat.as_ref().unwrap().squad.positions[5];
    assert!(
        medical::resolve(
            actual.combat.as_mut().unwrap(),
            &mut actual.warriors,
            mission,
            &mut actual.random,
            &pending,
            &MedicalTarget::LivingPoint { x: p.0, y: p.1 }
        )
        .is_err()
    );
    assert_eq!(actual, before);
    let original = state(&fixture, case, "before");
    for power in [-1, 0, 5, 7, 8] {
        assert!(medical::begin_learned_heal(&original, &content, power).is_err());
    }
}
