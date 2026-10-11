use centurions_content::{Content, Warrior, word};
use centurions_core::combat::{medical, spells};
use centurions_core::{
    Event, MedicalContinuation, MedicalOrigin, MedicalTarget, RandomState, Session,
};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/learned-portrait-power.json"
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

fn assert_native_party_snapshot(state: &Session, snapshot: &Value) {
    assert_party(&state.warriors, &snapshot["party"]);
}

fn projected_after(before: &Session, snapshot: &Value) -> Session {
    let mut expected = before.clone();
    for (warrior, native) in expected.warriors.iter_mut().zip(party(&snapshot["party"])) {
        warrior.bytes = native.bytes;
    }
    expected
}

#[test]
fn eight_accepted_preparations_match_native_costs_counters_packet_and_work() {
    let fixture = fixture();
    let content = content();
    let mut checked = 0;
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] != "psi_below_cost")
    {
        checked += 1;
        let before = state(&fixture, case, "before");
        let original = before.clone();
        let power = case["power"].as_i64().unwrap() as i16;
        let result = medical::begin_learned_heal(&before, &content, power).unwrap();
        assert_eq!(before, original);
        let mut actual = before.clone();
        actual.warriors[5] = result.warrior;
        let expected = projected_after(&before, &case["prepared"]);
        assert_eq!(actual, expected);
        assert_native_party_snapshot(&actual, &case["prepared"]);
        assert_eq!(result.packet, bytes(&case["prepared"]["packet"]));
        assert_eq!(result.packet[18], 13);
        assert_eq!(result.identity, [0, 0, 0]);
        assert_eq!(result.count, power);
        assert_eq!(case["prepared"]["required_target_count"], 1);
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
        let mut canonical = case["sounds"].as_array().unwrap()[..4]
            .iter()
            .map(|v| v.as_i64().unwrap() as i16)
            .collect::<Vec<_>>();
        canonical[2] = 141;
        assert_eq!(
            result.events,
            canonical
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id })
                .collect::<Vec<_>>()
        );
        assert_eq!(case["prepared"]["seed"], case["before"]["seed"]);
        assert_eq!(case["prepared"]["draws"], case["before"]["draws"]);
        assert_eq!(word(&actual.warriors[5].bytes, 0x40).unwrap(), 3);
        assert_eq!(word(&actual.warriors[5].bytes, 0x3c).unwrap(), 8);
        assert_eq!(
            word(&actual.warriors[5].bytes, 0x58).unwrap(),
            before.warriors[5].psi_points().unwrap() - 6 * power
        );
        assert_eq!(
            word(&actual.warriors[5].bytes, 0xac8).unwrap(),
            word(&before.warriors[5].bytes, 0xac8).unwrap() + 1
        );
        assert_eq!(
            i32::from_be_bytes(actual.warriors[5].bytes[0xa66..0xa6a].try_into().unwrap()),
            i32::from_be_bytes(before.warriors[5].bytes[0xa66..0xa6a].try_into().unwrap()) + 1
        );
        assert_eq!(case["prepared"]["learned_marker"], 1);
        assert_eq!(case["prepared"]["heal_marks_affected"], 0);
    }
    assert_eq!(checked, 8);
}

#[test]
fn four_portraits_match_native_resolver_records_restoration_events_and_natural_draws() {
    let fixture = fixture();
    let content = content();
    let mut checked = 0;
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] == "actual_portrait")
    {
        checked += 1;
        let mut actual = state(&fixture, case, "prepared");
        let before = actual.clone();
        let pending = pending(case);
        let target = case["target"].as_u64().unwrap() as usize;
        let mission = content
            .map(actual.exploration.as_ref().unwrap().map)
            .unwrap();
        let mask = words(&case["resolutions"][0]["mask"]);
        let mut direct_random = before.random.clone();
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
        assert_eq!(
            direct.enemies,
            before.combat.as_ref().unwrap().enemies.records
        );
        assert!(direct.healed);
        assert_eq!(direct.recovered, 1);
        assert_eq!(direct.targets.len(), 1);
        let native_damage = case["effect_events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|event| event["address"] == "1518d4")
            .unwrap()["d0"]
            .as_u64()
            .unwrap() as u16 as i16;
        assert_eq!(direct.targets[0].1.damage, native_damage);
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
            result.events,
            vec![
                Event::Sound { resource_id: -141 },
                Event::Sound { resource_id: 624 },
                Event::SpellResolved {
                    caster: Some(5),
                    spell: 1101,
                    target: target as i16,
                    damage: native_damage,
                    defeated: false
                },
                Event::Sound { resource_id: 631 }
            ]
        );
        assert_eq!(
            sounds(&result.events),
            case["sounds"].as_array().unwrap()[4..]
                .iter()
                .map(|value| value.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        let mut receipt_random = before.random.clone();
        for draw in case["rng"]
            .as_array()
            .unwrap()
            .iter()
            .take(1 + pending.power as usize)
        {
            assert_eq!(
                receipt_random.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(actual.random, direct_random);
        assert_eq!(actual.random, receipt_random);
        let mut expected = projected_after(&before, &case["after"]);
        expected.random = receipt_random.clone();
        let combat = expected.combat.as_mut().unwrap();
        combat.squad.positions = words(&case["after"]["party_positions"])
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| (p[0], p[1]))
            .collect();
        combat.squad.under_tiles = words(&case["after"]["under_tiles"]);
        combat.map.tiles = words(&case["after"]["map"]);
        combat.order = words(&case["after"]["order"]);
        combat.unavailable = case["after"]["unavailable"].as_i64().unwrap() as i8;
        combat.attack_packet.as_mut().unwrap()[18] = 4;
        assert_eq!(actual, expected);
        assert_native_party_snapshot(&actual, &case["after"]);
        assert_world_unchanged(&before, &actual);
        assert_eq!(
            actual.combat.as_ref().unwrap().squad.positions[target],
            if target == 1 { (9, 23) } else { (16, 13) }
        );
        assert_eq!(
            actual.warriors[5].psi_points().unwrap(),
            before.warriors[5].psi_points().unwrap()
        );
        let incoming_draw = case["rng"].as_array().unwrap().last().unwrap();
        assert_eq!(incoming_draw["bound"], 100);
        assert_eq!(
            receipt_random.bound(100),
            incoming_draw["result"].as_i64().unwrap() as i16
        );
        assert_eq!(
            receipt_random.seed,
            case["after"]["seed"].as_u64().unwrap() as u32
        );
        assert_eq!(
            receipt_random.draws,
            case["after"]["draws"].as_u64().unwrap()
        );
    }
    assert_eq!(checked, 4);
}

#[test]
fn finite_cancel_refunds_and_affordability_controls_match_native() {
    let fixture = fixture();
    let content = content();
    let mut cancellations = 0;
    let mut equalities = 0;
    let mut rejections = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let power = case["power"].as_i64().unwrap() as i16;
        match case["kind"].as_str().unwrap() {
            "confirmed_cancel" => {
                cancellations += 1;
                let mut actual = state(&fixture, case, "prepared");
                let before = actual.clone();
                let mission = content
                    .map(actual.exploration.as_ref().unwrap().map)
                    .unwrap();
                let result = medical::resolve(
                    actual.combat.as_mut().unwrap(),
                    &mut actual.warriors,
                    mission,
                    &mut actual.random,
                    &pending(case),
                    &MedicalTarget::ConfirmedCancel,
                )
                .unwrap();
                assert_eq!(actual, projected_after(&before, &case["after"]));
                assert_native_party_snapshot(&actual, &case["after"]);
                assert_eq!(result.follow_up, medical::FollowUp::None);
                assert_eq!(
                    result.events,
                    vec![
                        Event::Sound { resource_id: 6000 },
                        Event::Sound { resource_id: 20005 }
                    ]
                );
                assert_world_unchanged(&before, &actual);
                assert_eq!(actual.combat, before.combat);
                assert_eq!(actual.random, before.random);
                assert_eq!(word(&actual.warriors[5].bytes, 0x40).unwrap(), 3);
                assert_eq!(word(&actual.warriors[5].bytes, 0x3c).unwrap(), 5);
                assert_eq!(
                    actual.warriors[5].psi_points().unwrap(),
                    if power == 3 { 18 } else { 16 }
                );
                assert_eq!(
                    actual.warriors[5].bytes[0xa66..0xa6a],
                    before.warriors[5].bytes[0xa66..0xa6a]
                );
                assert_eq!(
                    word(&actual.warriors[5].bytes, 0xac8).unwrap(),
                    word(&party(&case["before"]["party"])[5].bytes, 0xac8).unwrap()
                );
                assert!(case["rng"].as_array().unwrap().is_empty());
            }
            "psi_equal_cost" => {
                equalities += 1;
                let original = state(&fixture, case, "before");
                let result = medical::begin_learned_heal(&original, &content, power).unwrap();
                assert_eq!(result.warrior.psi_points().unwrap(), 0);
                assert_eq!(original.warriors[5].psi_points().unwrap(), 6 * power);
                assert!(case["rng"].as_array().unwrap().is_empty());
            }
            "psi_below_cost" => {
                rejections += 1;
                let actual = state(&fixture, case, "before");
                let before = actual.clone();
                assert!(medical::begin_learned_heal(&actual, &content, power).is_err());
                assert_eq!(actual, before);
                assert_eq!(actual.warriors[5].psi_points().unwrap(), 6 * power - 1);
                assert_native_party_snapshot(&actual, &case["after"]);
                assert!(case["rng"].as_array().unwrap().is_empty());
                assert_eq!(case["after"]["learned_marker"], 0);
            }
            _ => {}
        }
    }
    assert_eq!((cancellations, equalities, rejections), (2, 2, 2));
}

#[test]
fn extended_power_requires_one_unconscious_party_body_and_rejects_other_continuations_atomically() {
    let fixture = fixture();
    let content = content();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["kind"] == "actual_portrait" && case["foundation"] == 0)
    {
        let base = state(&fixture, case, "prepared");
        let pending = pending(case);
        let mission = content.map(base.exploration.as_ref().unwrap().map).unwrap();
        let target = case["target"].as_u64().unwrap() as usize;
        for (hp, second, body, id) in [
            (1_i16, false, target, 1101),
            (-10, false, target, 1101),
            (-1, true, target, 1101),
            (-1, false, 10, 1101),
            (-1, false, target, 1305),
            (-1, false, target, 1401),
        ] {
            let mut warriors = base.warriors.clone();
            warriors[target].bytes[0x50..0x52].copy_from_slice(&hp.to_be_bytes());
            let mut mask = [0; 110];
            mask[body] = 1;
            if second {
                mask[2] = 1;
            }
            let packet = if id == 1101 {
                base.combat
                    .as_ref()
                    .unwrap()
                    .attack_packet
                    .as_ref()
                    .unwrap()
                    .clone()
            } else {
                spells::load_packet(&mission.combat.as_ref().unwrap().attacks, id).unwrap()
            };
            let mut random = base.random.clone();
            assert!(
                spells::resolve_player_support(
                    &warriors,
                    &base.combat.as_ref().unwrap().enemies.records,
                    &mask,
                    &packet,
                    id,
                    pending.power,
                    &mut random
                )
                .is_err()
            );
            assert_eq!(random, base.random);
        }
        for selection in [
            MedicalTarget::Portrait { warrior: 5 },
            MedicalTarget::LivingPoint {
                x: base.combat.as_ref().unwrap().squad.positions[5].0,
                y: base.combat.as_ref().unwrap().squad.positions[5].1,
            },
        ] {
            let mut actual = base.clone();
            assert!(
                medical::resolve(
                    actual.combat.as_mut().unwrap(),
                    &mut actual.warriors,
                    mission,
                    &mut actual.random,
                    &pending,
                    &selection
                )
                .is_err()
            );
            assert_eq!(actual, base);
        }
        let before = state(&fixture, case, "before");
        assert!(medical::begin_learned_heal(&before, &content, 5).is_err());
    }
}
