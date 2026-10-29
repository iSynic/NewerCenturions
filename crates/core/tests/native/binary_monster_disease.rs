use crate::{prelude::*, support};

use centurions_core::combat::ally_turns::natural_attack_enemy_with_actor;
use centurions_core::combat::runtime::check_neighbors_with_context;

use support::event_sounds as sound_ids;
#[test]
fn monster_disease_matches_native_signed_conditions_and_whole_caller() {
    let fixture = support::fixture("monster-disease.json");

    compare(fixture, false);
}

pub(super) fn compare(fixture: Value, elemental_audio: bool) {
    let content = support::content();
    let mission = content.map(0).unwrap();
    let attacks = &mission.combat.as_ref().unwrap().attacks;

    for family in ["attack_cases", "caller_cases"] {
        for (index, case) in fixture[family].as_array().unwrap().iter().enumerate() {
            let context = format!("{family} case{index}");
            let source = case.n::<i16>("source");
            let target = case.n::<i16>("target");
            let before_records = case.bytes("records_before");
            let positions = if source == 10 {
                vec![(20, 20), (21, 20), (40, 40)]
            } else {
                vec![(21, 20), (40, 40), (20, 20)]
            };
            let mut combat = support::battle();
            combat.battle = 15;
            combat.map.tiles = case.words("map_before");
            combat.squad.positions = (0..6).map(|i| (40 + i, 40)).collect();
            combat.enemies.records = support::monsters(&before_records);
            combat.enemies.positions = positions;
            combat.order = case.words("order_before");
            let mut warriors: Vec<_> = support::warriors(&fixture["party"]);
            let mut random = RandomState::new(case.n::<u32>("seed_before"));
            let sounds = if family == "attack_cases" {
                let (result, packet) = natural_attack_enemy_with_actor(
                    &mut combat,
                    &mut warriors,
                    &mut random,
                    (
                        source,
                        target,
                        case.n::<usize>("index"),
                        case.n::<i16>("actor"),
                    ),
                    attacks,
                    &[0; 58],
                )
                .unwrap_or_else(|error| panic!("{context}: {error:?}"));
                combat.attack_packet = packet;
                let mut sounds = result.sounds;
                sounds.push(if elemental_audio {
                    centurions_core::combat::ally_turns::result_audio(
                        &combat.enemies.records[(source - 10) as usize],
                        case.n::<usize>("index"),
                        result.damage,
                        0,
                    )
                    .unwrap()
                } else {
                    if result.damage == 0 {
                        650
                    } else {
                        let attack = case.n::<usize>("index");
                        let base = (source - 10) as usize * MONSTER_BYTES;
                        let offset = if before_records[base + 0x93 + attack * 4] == 0 {
                            0x95
                        } else {
                            0x95 + attack * 4
                        };
                        i16::from(before_records[base + offset] as i8)
                    }
                });
                sounds
            } else {
                let events = check_neighbors_with_context(
                    &mut combat,
                    &mut warriors,
                    &mut random,
                    source,
                    &content.items,
                    attacks,
                )
                .unwrap_or_else(|error| panic!("{context}: {error:?}"));
                sound_ids(&events)
            };
            let actual: Vec<_> = support::record_bytes(&combat.enemies.records);
            support::native_expect! {
                bytes actual => &case["records_after"], &format!("{context} records");
                bytes combat.attack_packet => &case["packet_after"], &format!("{context} packet");
            }
            assert_eq!(combat.map.tiles, case.words("map_after"), "{context} map");
            assert_eq!(combat.order, case.words("order_after"), "{context} order");

            support::native_expect! {
            positions combat.enemies.positions => &case["positions_after"], "{context} positions";
            records &warriors => &fixture["party"], &format!("{context} party");
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_monster_disease";
                }
            assert_eq!(
                serde_json::json!(sounds),
                case["sounds"],
                "{context} sounds"
            );
        }
    }
}
