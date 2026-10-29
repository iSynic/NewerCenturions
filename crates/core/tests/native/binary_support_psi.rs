use crate::{native_common, prelude::*, support};

use centurions_core::combat::attack;
use centurions_core::combat::damage;
use centurions_core::combat::party_buffs;
use support::warriors as party;

use support::{assert_bytes, record_bytes as flatten_party};

#[test]
fn npc_revival_matches_inert_backup_aliases_and_keeps_writing_cases_atomic() {
    let fixture = support::fixture("revival-boundary.json");
    let content = content();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
    let packet = spells::load_packet(attacks, 1705).unwrap();
    let warriors = party(fixture.text("warrior"));
    let enemies = support::monsters(&fixture.bytes("npc").repeat(5));
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let body = case.n::<usize>("body");
        let backup = case.bytes("backup_before");
        let memory = if index < 4 {
            spells::RestrictionMemoryView::ordinary_startup(&[], None)
        } else {
            spells::RestrictionMemoryView::with_editor_backup(&[], None, &backup).unwrap()
        };
        let mut mask = vec![0; 15];
        mask[body] = 1;
        let context = DirectSpell {
            packet: &packet,
            level: 6,
            count: 1,
            armor_filter: false,
            weapon_sound: 0,
        };
        // The receipt executes spelllist, without the enclosing duration draw.
        let mut random = RandomState::new(1);
        let result = spells::resolve_support_for_caster(
            &warriors,
            &enemies,
            &mask,
            1705,
            &context,
            (Some(0), Some(memory)),
            &mut random,
        );
        if index < 4 {
            let result = result.unwrap();
            assert_bytes(&backup, &case.bytes("backup_after"), "inert native backup");
            assert_eq!(result.warriors, warriors);
            assert_eq!(result.enemies, enemies);
            assert_eq!(result.targets[0].0, body);
            assert_eq!(result.targets[0].1.sounds, [word(&packet, 38).unwrap()]);
            let mut expected = RandomState::new(1);
            expected.range(1, 1);
            assert_eq!(random, expected);
        } else {
            assert_ne!(backup, case.bytes("backup_after"));
            assert!(matches!(
                result,
                Err(RuleError::Unsupported {
                    dependency: "warrior-indexed NPC write into the editor backup without the session table",
                    ..
                })
            ));
            assert_eq!(random, RandomState::new(1));
        }
    }
}

fn combat(snapshot: &Value, caster: i16) -> Combat {
    let mut combat = support::psi_frame::combat(snapshot);
    combat.order = vec![-1; 110];
    combat.order[1] = caster;
    combat.cursor = 1;
    combat.round = 0;
    combat
}
fn compare(c: &Combat, w: &[Warrior], s: &Value, label: &str) {
    support::psi_frame::compare(c, w, s, label);
    assert_eq!(c.unavailable as u8, s.n::<u8>("unavailable"), "{label}");
}
use support::event_sounds as sounds;
#[test]
fn support_casts_match_complete_native_records_packets_memory_rng_and_audio() {
    let f = support::fixture("player-support-psi.json");
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
        assert_eq!(
            sounds(&events),
            case.n::<Vec<i16>>("sounds"),
            "{label} sounds"
        );
        assert_eq!(random, native_common::random_state(after));
    }
}
#[test]
fn support_protection_matches_native_melee_and_natural_follow_on_attacks() {
    let f = support::fixture("player-support-psi.json");
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
        let retained = c.attack_packet.clone();
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
        c.attack_packet = packet;
        compare(&c, &w, after, &label);
        assert_eq!(random, native_common::random_state(after));
        let mut sounds = result.sounds;
        sounds.push(if result.damage == 0 {
            650
        } else if weapon == 187 {
            word(&content.items[weapon], 14).unwrap()
        } else {
            i16::from(c.enemies.records[0].bytes[0x95] as i8)
        });
        assert_eq!(sounds, case.n::<Vec<i16>>("sounds"), "{label} sounds");
        assert!(!result.killed);
    }
}
#[test]
fn party_timers_and_damage_modifier_match_independent_native_branches() {
    let f = support::fixture("player-support-psi.json");
    for (i, case) in f.array("tick_cases").iter().enumerate() {
        let mut buffs = case["before"].bytes("party_buffs");
        party_buffs::reduce(&mut buffs).unwrap();
        support::native_expect! {
            bytes &buffs => &case["after"].bytes("party_buffs"), &format!("tick {i}");
        };
    }
    for (i, case) in f.array("mitigation_cases").iter().enumerate() {
        let buffs = case["before"].bytes("party_buffs");
        let amount = case["before"].n::<i16>("last_amount");
        let result = party_buffs::attack2_damage(&buffs, 0, amount).unwrap();
        support::expect(result.unwrap_or(amount), &case["after"]["last_amount"]);
        assert_eq!(
            if result.is_some() { vec![-694] } else { vec![] },
            case.n::<Vec<i16>>("sounds"),
            "modifier {i} sounds"
        );
    }
}
#[test]
fn electric_ward_matches_native_damage_component_rng_and_target_flags() {
    let f = support::fixture("player-support-psi.json");
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
        support::native_expect! {
            scalar result.total => &case["damage"], "binary_support_psi";
            bytes flatten_party(&w) => &after["party"], &format!("{label} party");
        }
        assert_eq!(random, native_common::random_state(after));
        assert!(!result.shield_sound);
        assert!(case.n::<Vec<i16>>("sounds").is_empty());
    }
}
#[test]
fn mode_affordability_and_geometry_match_native_preflight_without_mutation() {
    let f = support::fixture("player-support-psi.json");
    let content = content();
    let mission = content.map(0).unwrap();
    for case in f.array("admission_cases").iter() {
        let before = &case["before"];
        let packet = before.bytes("packet");
        let w = party(before.text("party"));
        support::native_expect! {
            scalar player_psi::mode_allowed(&packet, case.n::<bool>("combat")).unwrap() => &case["mode_allowed"], "binary_support_psi";
            scalar w[5].psi_points().unwrap() >= player_psi::cost(&packet, 1).unwrap() => &case["affordable"], "binary_support_psi";
        }
    }
    for case in f.array("geometry_cases").iter() {
        let before = &case["before"];
        let c = combat(before, 0);
        let packet = before.bytes("packet");
        let range = player_psi::range(&packet, 1).unwrap();
        support::expect(range, &case["range"]);
        let target = (20 + case.n::<i16>("distance"), 20);
        support::expect(
            player_psi::in_range((20, 20), target, range),
            &case["in_range"],
        );
        let sight = case["spell_id"] == 1101
            || centurions_core::combat::lifecycle::can_see(&c, mission, (20, 20), target, 0)
                .unwrap();
        support::expect(sight, &case["sight_allowed"]);
    }
}
