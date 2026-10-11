use centurions_content::{Content, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    Event, RandomState,
    combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap,
        player_psi::{self, CastRequest},
        spells,
    },
};
use serde_json::Value;
fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn words(hex: &str) -> Vec<i16> {
    bytes(hex)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| i16::from_be_bytes(*b))
        .collect()
}
fn positions(hex: &str) -> Vec<(i16, i16)> {
    words(hex)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| (b[0], b[1]))
        .collect()
}
fn party(s: &Value) -> Vec<Warrior> {
    bytes(s["party"].as_str().unwrap())
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|b| Warrior::read(b).unwrap())
        .collect()
}
fn data() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/crystalize-psi.json"),
        )
        .unwrap(),
    )
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
fn assert_bytes(a: &[u8], b: &[u8], label: &str) {
    assert_eq!(a.len(), b.len(), "{label} length");
    if let Some(i) = a.iter().zip(b).position(|(a, b)| a != b) {
        panic!("{label} byte{i:#x}:{} != {}", a[i], b[i]);
    }
}
fn combat(s: &Value) -> Combat {
    Combat {
        battle: 85,
        map: TacticalMap {
            tiles: words(s["map"].as_str().unwrap()),
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(s["party_positions"].as_str().unwrap()),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: bytes(s["records"].as_str().unwrap())
                .as_chunks::<288>()
                .0
                .iter()
                .map(|b| MonsterRecord { bytes: b.to_vec() })
                .collect(),
            positions: positions(s["monster_positions"].as_str().unwrap()),
        },
        order: words(s["order"].as_str().unwrap()),
        cursor: s["cursor"].as_u64().unwrap() as usize,
        round: s["round"].as_u64().unwrap() as i16,
        round_scripts: Default::default(),
        death_queue: Default::default(),
        spell_fields: bytes(s["fields"].as_str().unwrap()),
        field_used: bytes(s["used"].as_str().unwrap()),
        attack_packet: Some(bytes(s["packet"].as_str().unwrap())),
        attack_identity: Some(words(s["identity"].as_str().unwrap()).try_into().unwrap()),
        weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
        unavailable: s["unavailable"].as_u64().unwrap() as i8,
    }
}
#[test]
fn learned_crystalize_matches_complete_native_cast_field_creation_cost_rng_and_audio() {
    let f = data();
    let content = content();
    let mission = content.map(0).unwrap();
    assert_eq!(f["cases"].as_array().unwrap().len(), 4);
    for (i, case) in f["cases"].as_array().unwrap().iter().enumerate() {
        let label = format!("crystalize{i}");
        let before = &case["before"];
        let cast = &case["cast"];
        let power = case["power"].as_i64().unwrap() as i16;
        let mut c = combat(before);
        let original = c.clone();
        let mut w = party(before);
        let mut memory = bytes(before["memory"].as_str().unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let point = words(case["casts"][0]["targets"].as_str().unwrap());
        let target = (point[0] - 5, point[1] - 5);
        let packet = bytes(before["packet"].as_str().unwrap());
        let mut mask_rng = random.clone();
        let target_descriptor = spells::SpellTarget {
            first: point[0],
            second: point[1],
            kind: 3,
        };
        let mask = spells::target_mask(
            &c,
            &w,
            &mission.areas,
            &target_descriptor,
            &spells::SpellContext {
                packet: &packet,
                id: 2101,
                count: power,
                caster: 1,
                initiative: c.cursor as i16,
                collision: false,
            },
            &c.spell_fields,
            &mut mask_rng,
        )
        .unwrap();
        assert_eq!(
            mask.mask.to_vec(),
            words(case["resolutions"][0]["mask"].as_str().unwrap()),
            "{label} full110 mask"
        );
        assert_bytes(
            &mask.fields,
            &bytes(case["field_events"][1]["fields"].as_str().unwrap()),
            &format!("{label} field creation"),
        );
        assert_eq!(
            mask_rng.draws,
            case["resolutions"][0]["draws_before"].as_u64().unwrap(),
            "{label} field-before-resolution draws"
        );
        let events = player_psi::cast(
            &mut c,
            &mut w,
            mission,
            &mut random,
            &mut memory,
            0,
            CastRequest {
                spell: 2101,
                power,
                target: Some(target),
            },
        )
        .unwrap_or_else(|e| panic!("{label}:{e:?}"));
        assert_bytes(
            &w.iter()
                .flat_map(|w| w.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(cast["party"].as_str().unwrap()),
            &format!("{label} complete party"),
        );
        assert_bytes(
            &c.enemies
                .records
                .iter()
                .flat_map(|m| m.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(cast["records"].as_str().unwrap()),
            &format!("{label} enemies"),
        );
        assert_eq!(
            c.map.tiles,
            words(cast["map"].as_str().unwrap()),
            "{label} map"
        );
        assert_eq!(
            c.squad.positions,
            positions(cast["party_positions"].as_str().unwrap()),
            "{label} party positions"
        );
        assert_eq!(
            c.enemies.positions,
            positions(cast["monster_positions"].as_str().unwrap()),
            "{label} monster positions"
        );
        assert_bytes(
            &c.spell_fields,
            &bytes(cast["fields"].as_str().unwrap()),
            &format!("{label} all60fields"),
        );
        assert_bytes(
            &c.field_used,
            &bytes(cast["used"].as_str().unwrap()),
            &format!("{label} all128usage"),
        );
        assert_bytes(
            c.attack_packet.as_ref().unwrap(),
            &bytes(cast["packet"].as_str().unwrap()),
            &format!("{label} packet"),
        );
        assert_eq!(
            c.attack_identity.unwrap().to_vec(),
            words(cast["identity"].as_str().unwrap()),
            "{label} identity"
        );
        assert_bytes(
            &memory,
            &bytes(cast["memory"].as_str().unwrap()),
            &format!("{label} memory"),
        );
        assert_eq!(c.unavailable, cast["unavailable"].as_i64().unwrap() as i8);
        assert_eq!(c.order, words(cast["order"].as_str().unwrap()));
        assert_eq!(c.cursor, cast["cursor"].as_u64().unwrap() as usize);
        assert_eq!(c.round, cast["round"].as_u64().unwrap() as i16);
        assert_eq!(c.death_queue, original.death_queue);
        assert_eq!(c.round_scripts, original.round_scripts);
        assert_eq!(
            random.seed,
            cast["seed"].as_u64().unwrap() as u32,
            "{label} seed"
        );
        assert_eq!(
            random.draws,
            cast["draws"].as_u64().unwrap(),
            "{label} draws"
        );
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Sound { resource_id } = e {
                    Some(*resource_id)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            serde_json::json!(sounds),
            cast["sounds"],
            "{label} ordered raw audio"
        );
        let damage = word(&party(before)[1].bytes, 0x50).unwrap();
        assert_eq!(
            word(&w[1].bytes, 0x50).unwrap(),
            damage,
            "{label} caster health"
        );
        let result = events
            .iter()
            .find_map(|e| {
                if let Event::SpellResolved {
                    caster: Some(1),
                    spell: 2101,
                    target: 10,
                    damage,
                    defeated: false,
                } = e
                {
                    Some(*damage)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(
            result,
            cast["last_amount"].as_i64().unwrap() as i16,
            "{label} resolved damage"
        );
        for key in [
            "party_buffs",
            "item_context",
            "weapon_work",
            "ammo_work",
            "active_member",
            "monster_index",
            "death_count",
            "enemy_phase",
        ] {
            assert_eq!(before[key], cast[key], "{label} native unchanged{key}");
        }
        let mut receipt_rng = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        for draw in cast["rng_records"].as_array().unwrap() {
            assert_eq!(
                receipt_rng.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16,
                "{label} native ordered RNG"
            );
        }
        assert_eq!(receipt_rng, random);
    }
}
