use crate::native_common::decoded_positions as positions;
use crate::prelude::*;
fn fixture() -> Value {
    crate::support::fixture("warm-psi-work.json")
}

fn native_work(s: &Value) -> WeaponWork {
    let weapon = s.bytes("weapon_work");
    let ammo = s.bytes("ammo_work");
    let work = WeaponWork {
        sound: Some(word(&weapon, 0x0e).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 0x88).unwrap()),
        ammunition_armor_filter: Some(word(&ammo, 0x88).unwrap()),
    };
    let projection = &s["work_projection"];
    crate::support::native_expect! {
    scalar work.sound.unwrap() => &projection["sound"], "binary_warm_psi_work";
    scalar work.weapon_armor_filter.unwrap() => &projection["weapon_armor_filter"], "binary_warm_psi_work";
    scalar work.ammunition_armor_filter.unwrap() => &projection["ammunition_armor_filter"], "binary_warm_psi_work";
    }
    assert_eq!(work.armor_filter().unwrap(), projection["armor_filter"]);
    work
}

fn state(content: &Content, s: &Value) -> Session {
    let mut state = crate::support::mission_session(content);
    state.warriors = crate::support::warriors(&s["party"]);
    state.psi_memory = Some(s.bytes("memory"));
    state.party_buffs = s.bytes("party_buffs");
    state.loaded_item_field32 = Some(s.n::<i16>("item_context"));
    state.attack_count = s.n::<i16>("count");
    state.weapon_work = native_work(s);
    state.attack_packet = Some(s.bytes("packet"));
    state.attack_identity = Some(s.words("identity").try_into().unwrap());
    state.random = RandomState {
        seed: s.n::<u32>("seed"),
        draws: s.n::<u64>("draws"),
    };
    state.combat = Some(Combat {
        battle: 85,
        map: TacticalMap {
            tiles: s.words("map"),
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(&s["party_positions"]),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: crate::support::monsters(&s["records"]),
            positions: positions(&s["monster_positions"]),
        },
        order: s.words("order"),
        cursor: s.n::<usize>("cursor"),
        round: s.n::<i16>("round"),
        spell_fields: s.bytes("fields"),
        field_used: s.bytes("used"),
        attack_packet: Some(s.bytes("packet")),
        attack_identity: Some(s.words("identity").try_into().unwrap()),
        weapon_work: state.weapon_work,
        unavailable: s.n::<i8>("unavailable"),
        ..crate::support::battle()
    });
    state.phase = Phase::Combat;
    state
}

fn assert_snapshot(state: &Session, expected: &Value, events: &[Event], label: &str) {
    let c = state.combat.as_ref().unwrap();
    crate::support::native_expect! {
    bytes crate::support::record_bytes(&state.warriors) => &expected["party"], &format!("{label} full party");
    bytes crate::support::record_bytes(&c.enemies.records) => &expected["records"], &format!("{label} full enemies");
    words c.map.tiles => &expected["map"], "{label} map";
    }
    assert_eq!(c.squad.positions, positions(&expected["party_positions"]));
    assert_eq!(
        c.enemies.positions,
        positions(&expected["monster_positions"])
    );
    crate::support::native_expect! {
    bytes c.spell_fields => &expected["fields"], "{label} fields";
    bytes c.field_used => &expected["used"], "{label} usage";
    bytes c.attack_packet.as_ref().unwrap() => &expected["packet"], "binary_warm_psi_work";
    words c.attack_identity.unwrap().to_vec() => &expected["identity"], "binary_warm_psi_work";
    bytes state.psi_memory.as_ref().unwrap() => &expected["memory"], "binary_warm_psi_work";
    bytes state.party_buffs => &expected["party_buffs"], "binary_warm_psi_work";
    }
    assert_eq!(state.loaded_item_field32.unwrap(), expected["item_context"]);
    assert_eq!(state.attack_count, expected["count"]);
    assert_eq!(c.unavailable, expected["unavailable"]);
    assert_eq!(c.order, expected.words("order"));
    assert_eq!(c.cursor, expected["cursor"]);
    assert_eq!(c.round, expected["round"]);
    assert_eq!(c.weapon_work, native_work(expected));
    assert_eq!(state.weapon_work, c.weapon_work);
    assert_eq!(state.random.seed, expected["seed"]);
    assert_eq!(state.random.draws, expected["draws"]);
    let sounds: Vec<_> = crate::support::event_sounds(events);
    assert_eq!(
        serde_json::json!(sounds),
        expected["sounds"],
        "{label} audio"
    );
    let mut receipt = RandomState::new(12);
    for draw in expected.array("rng_records") {
        crate::support::expect(receipt.bound(draw.n::<i16>("bound")), &draw["result"]);
    }
    assert_eq!(receipt, state.random, "{label} RNG receipt state");
}

#[test]
fn native_loading_and_preparation_preserve_independent_weapon_and_ammunition_fields() {
    let content = content();
    let fixture = fixture();
    for case in fixture.array("cases") {
        let mut work = native_work(&case["unwarmed"]);
        let weapon = &content.items[case.n::<usize>("warm_weapon")];
        let ammunition = &content.items[case.n::<usize>("warm_ammo")];
        work.load_pair(weapon, ammunition).unwrap();
        assert_eq!(work, native_work(&case["loaded"]));
        work.prepare(weapon, ammunition).unwrap();
        assert_eq!(work, native_work(&case["prepared"]));
        for phase in ["before", "charged", "targeted", "cast", "after", "collided"] {
            assert_eq!(work, native_work(&case[phase]));
            for buffer in ["weapon_work", "ammo_work"] {
                assert_eq!(case[phase][buffer], case["prepared"][buffer]);
            }
        }
    }
}

#[test]
fn warmed_cast_automatic_initiative_and_field_collision_match_complete_native_cases() {
    let content = content();
    let mission = content.map(0).unwrap();
    let fixture = fixture();

    for case in fixture.array("cases") {
        let label = format!(
            "warm weapon{} ammo{}",
            case["warm_weapon"], case["warm_ammo"]
        );
        let mut state = state(&content, &case["before"]);
        let original = state.combat.as_ref().unwrap().clone();
        let point = words(&case["casts"][0]["targets"]);
        let mut mask_rng = state.random.clone();
        let c = state.combat.as_ref().unwrap();
        let packet = c.attack_packet.as_ref().unwrap();
        let mask = spells::target_mask(
            c,
            &state.warriors,
            &mission.areas,
            &spells::SpellTarget {
                first: point[0],
                second: point[1],
                kind: 3,
            },
            &spells::SpellContext {
                packet,
                id: 2101,
                count: 1,
                caster: 1,
                initiative: c.cursor as i16,
                collision: false,
            },
            &c.spell_fields,
            &mut mask_rng,
        )
        .unwrap();
        assert_eq!(mask.mask.to_vec(), words(&case["resolutions"][0]["mask"]));
        assert_eq!(mask.fields, bytes(&case["field_events"][1]["fields"]));
        assert_eq!(mask_rng.draws, case["resolutions"][0]["draws_before"]);
        let mut events = player_psi::cast(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            mission,
            &mut state.random,
            state.psi_memory.as_mut().unwrap(),
            0,
            CastRequest {
                spell: 2101,
                power: 1,
                target: Some((point[0] - 5, point[1] - 5)),
            },
        )
        .unwrap_or_else(|error| panic!("{label}: {error:?}"));
        assert_snapshot(&state, &case["cast"], &events, &format!("{label} cast"));
        events.extend(runtime::end_turn(&mut state, &content).unwrap());
        assert_eq!(state.combat.as_ref().unwrap().active_body(), Some(10));
        assert_snapshot(&state, &case["after"], &events, &format!("{label} advance"));
        events.extend(
            runtime::collide_fields(
                state.combat.as_mut().unwrap(),
                &mut state.warriors,
                mission,
                &mut state.attack_count,
                &mut state.random,
                10,
            )
            .unwrap(),
        );
        assert_snapshot(
            &state,
            &case["collided"],
            &events,
            &format!("{label} collision"),
        );
        let c = state.combat.as_ref().unwrap();
        assert_eq!(c.death_queue, original.death_queue);
        assert_eq!(c.round_scripts, original.round_scripts);
        let expected_filter = i16::from(c.weapon_work.armor_filter().unwrap());
        for call in case.array("damage_arguments") {
            assert_eq!(call["arguments"][4], expected_filter);
        }
    }
}
