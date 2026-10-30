use crate::{native_common, prelude::*, support};

use centurions_core::combat::ally_turns;
use centurions_core::combat::attack;
use native_common::original_file as data;

fn combat(records: &[u8], member: i16) -> Combat {
    let mut combat = support::battle();
    combat.battle = 117;
    combat.field.tiles.fill(1155);
    combat.squad.pos = (0..6).map(|i| (40 + i, 40)).collect();
    combat.roster.records = support::monsters(records);
    combat.roster.mon_pos = vec![(40 + member, 40); records.len() / MONSTER_BYTES];
    combat.cast_identity = [2, 4, 6];
    combat
}

#[test]
fn attack_packets_match_native_complete_writers_and_retention() {
    let fixture = support::fixture("attack-packets.json");
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
        let mut monster = MonsterRecord {
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
                    &mut monster,
                    &mut random,
                    &attacks,
                    &retained,
                    0,
                )
                .unwrap();
                (result, packet, monster.bytes.clone())
            }
            "natural" => {
                let (result, packet) = attack::natural_attack_with_packet(
                    &mut warrior,
                    &mut monster,
                    slot,
                    &mut random,
                    &attacks,
                    &retained,
                    0,
                )
                .unwrap();
                (result, packet, monster.bytes.clone())
            }
            "melee" => {
                let id = i16::from_be_bytes(monster.bytes[0x28..0x2a].try_into().unwrap()) as usize;
                let (result, packet) = enemy_weapons::melee_with_packet(
                    &mut warrior,
                    &mut monster,
                    &items[id * 150..(id + 1) * 150],
                    &mut random,
                    0,
                    40 + member,
                    40 + member,
                    &attacks,
                    &retained,
                )
                .unwrap();
                (result, packet, monster.bytes.clone())
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
                let records = support::record_bytes(&c.roster.records);
                warrior = party[0].clone();
                (result, packet, records)
            }
            _ => panic!("unknown native packet kind"),
        };
        support::native_expect! {
            bytes packet => &case["packet_after"], &format!("case{index} {kind} packet");
            bytes warrior.bytes => &case["warrior_after"], &format!("case{index} {kind} warrior");
            bytes after_records => &case["records_after"], &format!("case{index} {kind} enemies");
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_attack_packets";
        }

        support::expect_array(&(result.sounds), &case["sounds"]);
    }
}
