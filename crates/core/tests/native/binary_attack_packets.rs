use crate::prelude::*;

use crate::native_common::original_file as data;
use centurions_core::combat::ally_turns;
use centurions_core::combat::attack;

fn fixture() -> serde_json::Value {
    crate::support::fixture("attack-packets.json")
}

fn combat(records: &[u8], member: i16) -> Combat {
    Combat {
        battle: 117,
        map: TacticalMap {
            tiles: vec![1155; 8100],
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: (0..6).map(|i| (40 + i, 40)).collect(),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: crate::support::monsters(records),
            positions: vec![(40 + member, 40); records.len() / 288],
        },
        attack_identity: Some([2, 4, 6]),
        ..crate::support::battle()
    }
}

#[test]
fn attack_packets_match_native_complete_writers_and_retention() {
    let fixture = fixture();
    let attacks = data("Data Files/Data S");
    let items = data("Data Files/Data ID");
    let cases = fixture.array("cases");

    assert_eq!(
        &attacks[3 * 2842 + 2 * 58..3 * 2842 + 3 * 58],
        fixture.bytes("base4103")
    );
    for (index, case) in cases.iter().enumerate() {
        let kind = case.text("kind");
        let mut warrior = Warrior::read(&case.bytes("warrior_before")).unwrap();
        let records = case.bytes("records_before");
        let mut enemy = MonsterRecord {
            bytes: records[..288].to_vec(),
        };
        let retained = case.bytes("packet_before");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let slot = case.n::<usize>("attack");
        let member = case.n::<i16>("member");
        let (result, packet, after_records) = match kind {
            "unarmed" => {
                let (result, packet) = attack::unarmed_attack_with_packet(
                    &mut warrior,
                    &mut enemy,
                    &mut random,
                    &attacks,
                    &retained,
                )
                .unwrap();
                (result, packet, enemy.bytes.clone())
            }
            "natural" => {
                let (result, packet) = attack::natural_attack_with_packet(
                    &mut warrior,
                    &mut enemy,
                    slot,
                    &mut random,
                    &attacks,
                    &retained,
                )
                .unwrap();
                (result, packet, enemy.bytes.clone())
            }
            "melee" => {
                let id = i16::from_be_bytes(enemy.bytes[0x28..0x2a].try_into().unwrap()) as usize;
                let (result, packet) = enemy_weapons::melee_with_packet(
                    &mut warrior,
                    &mut enemy,
                    &items[id * 150..(id + 1) * 150],
                    &mut random,
                    0,
                    40 + member,
                    40 + member,
                    &attacks,
                    &retained,
                )
                .unwrap();
                (result, packet, enemy.bytes.clone())
            }
            "ally" => {
                let mut c = combat(&records, member);
                let mut party = vec![warrior.clone(); 6];
                let (result, packet) = ally_turns::natural_attack_enemy_with_packet(
                    &mut c,
                    &mut party,
                    &mut random,
                    10,
                    11,
                    slot,
                    &attacks,
                    &retained,
                )
                .unwrap();
                let records = crate::support::record_bytes(&c.enemies.records);
                warrior = party[0].clone();
                (result, packet, records)
            }
            _ => panic!("unknown native packet kind"),
        };
        crate::support::native_expect! {
        bytes packet => &case["packet_after"], &format!("case{index} {kind} packet");
        bytes warrior.bytes => &case["warrior_after"], &format!("case{index} {kind} warrior");
        bytes after_records => &case["records_after"], &format!("case{index} {kind} enemies");
        scalar random.seed => &case["seed_after"], "binary_attack_packets";
        scalar random.draws => &case["rng_draws"], "binary_attack_packets";
        }

        crate::support::expect_array(&(result.sounds), &case["sounds"]);
    }
}
