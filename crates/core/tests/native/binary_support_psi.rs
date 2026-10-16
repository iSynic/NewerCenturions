use crate::prelude::*;

use crate::support::warriors as party;
use centurions_core::combat::attack;
use centurions_core::combat::damage;
use centurions_core::combat::party_buffs;

use crate::support::positions;

use crate::support::assert_bytes;

fn combat(s: &Value, caster: i16) -> Combat {
    let mut order = vec![-1; 110];
    order[1] = caster;
    Combat {
        battle: 85,
        map: TacticalMap {
            tiles: words(s.text("map")),
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(s.text("party_positions")),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: crate::support::monsters(&s["records"]),
            positions: positions(s.text("monster_positions")),
        },
        order,
        spell_fields: s.bytes("fields"),
        field_used: s.bytes("used"),
        attack_packet: Some(s.bytes("packet")),
        attack_identity: Some(words(s.text("identity")).try_into().unwrap()),
        unavailable: s.n::<i8>("unavailable"),
        ..crate::support::battle()
    }
}
fn flatten_party(w: &[Warrior]) -> Vec<u8> {
    crate::support::record_bytes(w)
}
fn compare(c: &Combat, w: &[Warrior], s: &Value, label: &str) {
    crate::support::native_expect! {
    bytes flatten_party(w) => &s["party"], &format!("{label} party");
    bytes crate::support::record_bytes(&c.enemies.records) => &s["records"], &format!("{label} records");
    words &(c.map.tiles) => &s["map"], "binary_support_psi";
    }
    assert_eq!(
        c.squad.positions,
        positions(s.text("party_positions")),
        "{label} party positions"
    );
    assert_eq!(
        c.enemies.positions,
        positions(s.text("monster_positions")),
        "{label} enemy positions"
    );
    crate::support::native_expect! {
    bytes c.spell_fields => &s["fields"], &format!("{label} fields");
    bytes c.field_used => &s["used"], &format!("{label} used");
    bytes c.attack_packet.as_ref().unwrap() => &s["packet"], &format!("{label} packet");
    words &(c.attack_identity.unwrap().to_vec()) => &s["identity"], "binary_support_psi";
    scalar c.unavailable as u8 => &s["unavailable"], "binary_support_psi";
    }
}
fn rng(r: &RandomState, s: &Value, _label: &str) {
    crate::support::native_expect! {
    scalar r.seed => &s["seed"], "binary_support_psi";
    scalar r.draws => &s["draws"], "binary_support_psi";
    }
}
use crate::support::event_sounds as sounds;

fn expected_sounds(c: &Value) -> Vec<i16> {
    c.n::<Vec<i16>>("sounds")
}
#[test]
fn support_casts_match_complete_native_records_packets_memory_rng_and_audio() {
    let f = crate::support::fixture("player-support-psi.json");
    let content = content();
    let mission = content.map(0).unwrap();

    for (i, case) in f
        .array("cases")
        .iter()
        .enumerate()
        .filter(|(_, case)| case["spell_id"] != 1103)
    {
        let label = format!(
            "cast {i} id{} power{} variant{}",
            case["spell_id"], case["power"], case["variant"]
        );
        let before = &case["before"];
        let after = &case["after"];
        let id = case.n::<i16>("spell_id");
        let power = case.n::<i16>("power");
        let mut w = party(before.text("party"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let mut memory = before.bytes("memory");
        let mut c = combat(before, 5);
        let target = if id == 1101 {
            Some(c.squad.positions[case.n::<usize>("target")])
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
                rotation: 0,
                spell: id,
                power,
                target,
            },
        )
        .unwrap_or_else(|e| panic!("{label} {e:?}"));
        compare(&c, &w, after, &label);
        assert_bytes(&memory, &after.bytes("memory"), &format!("{label} memory"));
        assert_eq!(sounds(&events), expected_sounds(case), "{label} sounds");
        assert_eq!(
            before["party_buffs"], after["party_buffs"],
            "{label} unchanged buffs"
        );
        rng(&random, after, &label);
    }
}
#[test]
fn support_protection_matches_native_melee_and_natural_follow_on_attacks() {
    let f = crate::support::fixture("player-support-psi.json");
    let content = content();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;

    for (i, case) in f.array("attack_cases").iter().enumerate() {
        let label = format!("attack {i}");
        let before = &case["before"];
        let after = &case["after"];
        let mut c = combat(before, 10);
        let mut w = party(before.text("party"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let weapon = case.n::<usize>("weapon");
        let buffs = before.bytes("party_buffs");
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
                0,
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
    let f = crate::support::fixture("player-support-psi.json");
    for (i, case) in f.array("tick_cases").iter().enumerate() {
        let mut buffs = case["before"].bytes("party_buffs");
        party_buffs::reduce(&mut buffs).unwrap();
        assert_bytes(
            &buffs,
            &case["after"].bytes("party_buffs"),
            &format!("tick {i}"),
        );
    }
    for (i, case) in f.array("mitigation_cases").iter().enumerate() {
        let buffs = case["before"].bytes("party_buffs");
        let amount = case["before"].n::<i16>("last_amount");
        let result = party_buffs::attack2_damage(&buffs, 0, amount).unwrap();
        crate::support::expect(result.unwrap_or(amount), &case["after"]["last_amount"]);
        assert_eq!(
            if result.is_some() { vec![-694] } else { vec![] },
            expected_sounds(case),
            "modifier {i} sounds"
        );
    }
}
#[test]
fn electric_ward_matches_native_damage_component_rng_and_target_flags() {
    let f = crate::support::fixture("player-support-psi.json");
    for (i, case) in f.array("electric_cases").iter().enumerate() {
        let label = format!("electric {i}");
        let before = &case["before"];
        let after = &case["after"];
        let mut w = party(before.text("party"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = damage::resolve_damage(
            &mut w[0].bytes,
            false,
            &damage::DamagePacket {
                bytes: before.bytes("packet"),
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
        crate::support::native_expect! {
        scalar result.total => &case["damage"], "binary_support_psi";
        bytes flatten_party(&w) => &after["party"], &format!("{label} party");
        }
        rng(&random, after, &label);
        assert!(!result.shield_sound);
        assert!(expected_sounds(case).is_empty());
    }
}
#[test]
fn mode_affordability_and_geometry_match_native_preflight_without_mutation() {
    let f = crate::support::fixture("player-support-psi.json");
    let content = content();
    let mission = content.map(0).unwrap();
    for case in f.array("admission_cases").iter() {
        let before = &case["before"];
        let packet = before.bytes("packet");
        let w = party(before.text("party"));
        crate::support::native_expect! {
        scalar player_psi::mode_allowed(&packet, case.n::<bool>("combat")).unwrap() => &case["mode_allowed"], "binary_support_psi";
        scalar w[5].psi_points().unwrap() >= player_psi::cost(&packet, 1).unwrap() => &case["affordable"], "binary_support_psi";
        }
    }
    for case in f.array("geometry_cases").iter() {
        let before = &case["before"];
        let c = combat(before, 0);
        let packet = before.bytes("packet");
        let range = player_psi::range(&packet, 1).unwrap();
        crate::support::expect(range, &case["range"]);
        let target = (20 + case.n::<i16>("distance"), 20);
        crate::support::expect(
            player_psi::in_range((20, 20), target, range),
            &case["in_range"],
        );
        let sight = case["spell_id"] == 1101
            || centurions_core::combat::lifecycle::can_see(&c, mission, (20, 20), target, 0)
                .unwrap();
        crate::support::expect(sight, &case["sight_allowed"]);
    }
}
