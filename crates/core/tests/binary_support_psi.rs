use centurions_content::{Content, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    Event, RandomState,
    combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap, attack, damage, enemy_weapons,
        party_buffs,
        player_psi::{self, CastRequest, PartyCastRequest},
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
fn party(hex: &str) -> Vec<Warrior> {
    bytes(hex)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|b| Warrior::read(b).unwrap())
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
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/player-support-psi.json"
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
fn assert_bytes(a: &[u8], b: &[u8], context: &str) {
    assert_eq!(a.len(), b.len(), "{context} size");
    if let Some(i) = a.iter().zip(b).position(|(a, b)| a != b) {
        panic!("{context} byte {i:x}: {} != {}", a[i], b[i]);
    }
}
fn combat(s: &Value, caster: i16) -> Combat {
    let mut order = vec![-1; 110];
    order[1] = caster;
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
        order,
        cursor: 1,
        round: 0,
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
fn flatten_party(w: &[Warrior]) -> Vec<u8> {
    w.iter().flat_map(|w| w.bytes.iter().copied()).collect()
}
fn compare(c: &Combat, w: &[Warrior], s: &Value, label: &str) {
    assert_bytes(
        &flatten_party(w),
        &bytes(s["party"].as_str().unwrap()),
        &format!("{label} party"),
    );
    assert_bytes(
        &c.enemies
            .records
            .iter()
            .flat_map(|m| m.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(s["records"].as_str().unwrap()),
        &format!("{label} records"),
    );
    assert_eq!(
        c.map.tiles,
        words(s["map"].as_str().unwrap()),
        "{label} map"
    );
    assert_eq!(
        c.squad.positions,
        positions(s["party_positions"].as_str().unwrap()),
        "{label} party positions"
    );
    assert_eq!(
        c.enemies.positions,
        positions(s["monster_positions"].as_str().unwrap()),
        "{label} enemy positions"
    );
    assert_bytes(
        &c.spell_fields,
        &bytes(s["fields"].as_str().unwrap()),
        &format!("{label} fields"),
    );
    assert_bytes(
        &c.field_used,
        &bytes(s["used"].as_str().unwrap()),
        &format!("{label} used"),
    );
    assert_bytes(
        c.attack_packet.as_ref().unwrap(),
        &bytes(s["packet"].as_str().unwrap()),
        &format!("{label} packet"),
    );
    assert_eq!(
        c.attack_identity.unwrap().to_vec(),
        words(s["identity"].as_str().unwrap()),
        "{label} identity"
    );
    assert_eq!(
        c.unavailable as u8,
        s["unavailable"].as_u64().unwrap() as u8,
        "{label} unavailable"
    );
}
fn rng(r: &RandomState, s: &Value, label: &str) {
    assert_eq!(r.seed, s["seed"].as_u64().unwrap() as u32, "{label} seed");
    assert_eq!(r.draws, s["draws"].as_u64().unwrap(), "{label} draws");
}
fn sounds(events: &[Event]) -> Vec<i16> {
    events
        .iter()
        .filter_map(|e| {
            if let Event::Sound { resource_id } = e {
                Some(*resource_id)
            } else {
                None
            }
        })
        .collect()
}
fn expected_sounds(c: &Value) -> Vec<i16> {
    c["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i16)
        .collect()
}
#[test]
fn support_casts_match_complete_native_records_packets_memory_rng_and_audio() {
    let f = fixture();
    let content = content();
    let mission = content.map(0).unwrap();
    assert_eq!(f["cases"].as_array().unwrap().len(), 68);
    for (i, case) in f["cases"].as_array().unwrap().iter().enumerate() {
        let label = format!(
            "cast {i} id{} power{} variant{}",
            case["spell_id"], case["power"], case["variant"]
        );
        let before = &case["before"];
        let after = &case["after"];
        let id = case["spell_id"].as_i64().unwrap() as i16;
        let power = case["power"].as_i64().unwrap() as i16;
        let mut w = party(before["party"].as_str().unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let mut memory = bytes(before["memory"].as_str().unwrap());
        if id == 1103 {
            let result = player_psi::cast_party(
                &w,
                &memory,
                &bytes(before["party_buffs"].as_str().unwrap()),
                mission,
                &mut random,
                0,
                PartyCastRequest {
                    warrior: 5,
                    spell: id,
                    power,
                },
            )
            .unwrap_or_else(|e| panic!("{label} {e:?}"));
            assert_bytes(
                &flatten_party(&result.warriors),
                &bytes(after["party"].as_str().unwrap()),
                &format!("{label} party"),
            );
            assert_bytes(
                &result.memory,
                &bytes(after["memory"].as_str().unwrap()),
                &format!("{label} memory"),
            );
            assert_bytes(
                &result.buffs,
                &bytes(after["party_buffs"].as_str().unwrap()),
                &format!("{label} buffs"),
            );
            assert_bytes(
                &result.packet,
                &bytes(after["packet"].as_str().unwrap()),
                &format!("{label} packet"),
            );
            assert_eq!(
                result.identity.to_vec(),
                words(after["identity"].as_str().unwrap())
            );
            assert_eq!(
                sounds(&result.events),
                expected_sounds(case),
                "{label} sounds"
            );
            for key in [
                "records",
                "map",
                "fields",
                "used",
                "party_positions",
                "monster_positions",
            ] {
                assert_eq!(before[key], after[key], "{label} native unchanged {key}");
            }
        } else {
            let mut c = combat(before, 5);
            let target = if id == 1101 {
                Some(c.squad.positions[case["target"].as_u64().unwrap() as usize])
            } else {
                Some((20, 20))
            };
            let events = player_psi::cast(
                &mut c,
                &mut w,
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
            .unwrap_or_else(|e| panic!("{label} {e:?}"));
            compare(&c, &w, after, &label);
            assert_bytes(
                &memory,
                &bytes(after["memory"].as_str().unwrap()),
                &format!("{label} memory"),
            );
            assert_eq!(sounds(&events), expected_sounds(case), "{label} sounds");
            assert_eq!(
                before["party_buffs"], after["party_buffs"],
                "{label} unchanged buffs"
            );
        }
        rng(&random, after, &label);
    }
}
#[test]
fn support_protection_matches_native_melee_and_natural_follow_on_attacks() {
    let f = fixture();
    let content = content();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
    assert_eq!(f["attack_cases"].as_array().unwrap().len(), 8);
    for (i, case) in f["attack_cases"].as_array().unwrap().iter().enumerate() {
        let label = format!("attack {i}");
        let before = &case["before"];
        let after = &case["after"];
        let mut c = combat(before, 10);
        let mut w = party(before["party"].as_str().unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let weapon = case["weapon"].as_u64().unwrap() as usize;
        let buffs = bytes(before["party_buffs"].as_str().unwrap());
        let retained = c.attack_packet.as_ref().unwrap().clone();
        let (result, packet) = if weapon == 187 {
            enemy_weapons::melee_warrior_with_buffs(
                &mut c,
                &mut w,
                &mut random,
                (10, 0, 0, 10),
                &content.items[weapon],
                0,
                attacks,
                &retained,
                &buffs,
            )
        } else {
            attack::natural_attack_with_buffs(
                &mut w[0],
                &mut c.enemies.records[0],
                0,
                &mut random,
                attacks,
                &retained,
                &buffs,
            )
        }
        .unwrap_or_else(|e| panic!("{label} {e:?}"));
        c.attack_packet = Some(packet);
        compare(&c, &w, after, &label);
        rng(&random, after, &label);
        let mut sounds = result.sounds;
        sounds.push(if result.damage == 0 {
            650
        } else if weapon == 187 {
            word(&content.items[weapon], 14).unwrap()
        } else {
            i16::from(c.enemies.records[0].bytes[0x95] as i8)
        });
        assert_eq!(sounds, expected_sounds(case), "{label} sounds");
        assert!(!result.killed);
    }
}
#[test]
fn party_timers_and_damage_modifier_match_independent_native_branches() {
    let f = fixture();
    for (i, case) in f["tick_cases"].as_array().unwrap().iter().enumerate() {
        let mut buffs = bytes(case["before"]["party_buffs"].as_str().unwrap());
        party_buffs::reduce(&mut buffs).unwrap();
        assert_bytes(
            &buffs,
            &bytes(case["after"]["party_buffs"].as_str().unwrap()),
            &format!("tick {i}"),
        );
        for key in [
            "party", "records", "map", "fields", "used", "memory", "packet", "identity", "seed",
            "draws",
        ] {
            assert_eq!(
                case["before"][key], case["after"][key],
                "tick {i} native unchanged {key}"
            );
        }
    }
    for (i, case) in f["mitigation_cases"].as_array().unwrap().iter().enumerate() {
        let buffs = bytes(case["before"]["party_buffs"].as_str().unwrap());
        let amount = case["before"]["last_amount"].as_i64().unwrap() as i16;
        let result = party_buffs::attack2_damage(&buffs, 0, amount).unwrap();
        assert_eq!(
            result.unwrap_or(amount),
            case["after"]["last_amount"].as_i64().unwrap() as i16,
            "modifier {i}"
        );
        assert_eq!(
            if result.is_some() { vec![-694] } else { vec![] },
            expected_sounds(case),
            "modifier {i} sounds"
        );
        for key in [
            "party",
            "records",
            "map",
            "fields",
            "used",
            "memory",
            "party_buffs",
            "packet",
            "identity",
            "seed",
            "draws",
        ] {
            assert_eq!(
                case["before"][key], case["after"][key],
                "modifier {i} native unchanged {key}"
            );
        }
    }
}
#[test]
fn electric_ward_matches_native_damage_component_rng_and_target_flags() {
    let f = fixture();
    for (i, case) in f["electric_cases"].as_array().unwrap().iter().enumerate() {
        let label = format!("electric {i}");
        let before = &case["before"];
        let after = &case["after"];
        let mut w = party(before["party"].as_str().unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = damage::resolve_damage(
            &mut w[0].bytes,
            false,
            &damage::DamagePacket {
                bytes: bytes(before["packet"].as_str().unwrap()),
                attack_count: 0,
            },
            damage::DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter: false,
            },
            &mut random,
        )
        .unwrap();
        assert_eq!(
            result.total,
            case["damage"].as_i64().unwrap() as i16,
            "{label} amount"
        );
        assert_bytes(
            &flatten_party(&w),
            &bytes(after["party"].as_str().unwrap()),
            &format!("{label} party"),
        );
        rng(&random, after, &label);
        assert!(!result.shield_sound);
        assert!(expected_sounds(case).is_empty());
        for key in [
            "records",
            "map",
            "fields",
            "used",
            "memory",
            "party_buffs",
            "packet",
            "identity",
        ] {
            assert_eq!(before[key], after[key], "{label} native unchanged {key}");
        }
    }
}
#[test]
fn mode_affordability_and_geometry_match_native_preflight_without_mutation() {
    let f = fixture();
    let content = content();
    let mission = content.map(0).unwrap();
    for (i, case) in f["admission_cases"].as_array().unwrap().iter().enumerate() {
        let before = &case["before"];
        let packet = bytes(before["packet"].as_str().unwrap());
        let w = party(before["party"].as_str().unwrap());
        assert_eq!(
            player_psi::mode_allowed(&packet, case["combat"].as_bool().unwrap()).unwrap(),
            case["mode_allowed"].as_bool().unwrap(),
            "mode {i}"
        );
        assert_eq!(
            w[5].psi_points().unwrap() >= player_psi::cost(&packet, 1).unwrap(),
            case["affordable"].as_bool().unwrap(),
            "affordability {i}"
        );
        for key in [
            "party",
            "records",
            "map",
            "fields",
            "used",
            "memory",
            "party_buffs",
            "packet",
            "identity",
            "seed",
            "draws",
        ] {
            assert_eq!(
                before[key], case["after"][key],
                "admission {i} native unchanged {key}"
            );
        }
    }
    for (i, case) in f["geometry_cases"].as_array().unwrap().iter().enumerate() {
        let before = &case["before"];
        let c = combat(before, 0);
        let packet = bytes(before["packet"].as_str().unwrap());
        let range = player_psi::range(&packet, 1).unwrap();
        assert_eq!(range, case["range"].as_i64().unwrap() as i16, "range {i}");
        let target = (20 + case["distance"].as_i64().unwrap() as i16, 20);
        assert_eq!(
            player_psi::in_range((20, 20), target, range),
            case["in_range"].as_bool().unwrap(),
            "in range {i}"
        );
        let sight = case["spell_id"] == 1101
            || centurions_core::combat::lifecycle::can_see(&c, mission, (20, 20), target, 0)
                .unwrap();
        assert_eq!(sight, case["sight_allowed"].as_bool().unwrap(), "sight {i}");
        for key in [
            "party",
            "records",
            "map",
            "fields",
            "used",
            "memory",
            "party_buffs",
            "packet",
            "identity",
            "seed",
            "draws",
        ] {
            assert_eq!(
                before[key], case["after"][key],
                "geometry {i} native unchanged {key}"
            );
        }
    }
}
