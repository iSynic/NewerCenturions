use crate::{native_common, prelude::*, support};

use native_common::snapshot_party as party;

fn monsters(snapshot: &Value) -> Vec<MonsterRecord> {
    support::monsters(&snapshot["all_records"])
}
use support::compare_party as assert_party;

fn assert_monsters(actual: &[MonsterRecord], snapshot: &Value) {
    support::compare_record_bytes(actual, &monsters(snapshot));
}
use native_common::random_state as random;

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn prepare(content: &Content, fixture: &Value, case: &Value) -> firearm::PreparedFirearm {
    let state: Session = serde_json::from_value(
        fixture["foundations"][case.n::<usize>("foundation")]["session"].clone(),
    )
    .unwrap();
    let records = party(&case["before"]);
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
        .unwrap();
    firearm::prepare(
        &records[case.n::<usize>("actor")],
        &content.items[165],
        &content.items[280],
        &mission.combat.as_ref().unwrap().attacks,
    )
    .unwrap()
}

fn caller_debits(warrior: &mut Warrior) {
    let w = &mut warrior.bytes;
    let movement = word(w, 0x3c).unwrap().wrapping_sub(3).max(0);
    let actions = word(w, 0x40).unwrap().wrapping_sub(2);
    put_word(w, 0x3c, movement);
    put_word(w, 0x40, actions);
}

#[test]
fn actual_loaded_shot_preparations_match_native_packets_and_resource_work() {
    let content = support::content();
    let fixture: Value = support::fixture("earned-loaded-shot.json");
    for case in fixture.array("cases") {
        let prepared = prepare(&content, &fixture, case);
        support::native_expect! {
            bytes prepared.packet => &case["prepared"]["packet"], "prepared packet";
            scalar centurions_core::combat::player_psi::range(&prepared.packet, prepared.shots).unwrap() => &case["prepared"]["range"], "binary_earned_loaded_shot";
        }
        let mut weapon = content.items[165].clone();
        put_word(&mut weapon, 14, 406);
        support::native_expect! {
            bytes weapon => &case["prepared"]["weapon_work"], "weapon work";
            bytes content.items[280] => &case["prepared"]["ammo_work"], "ammo work";
        }
    }
}

#[test]
fn actual_miss_and_lethal_resolver_match_native_before_death_and_getup() {
    let content = support::content();
    let fixture: Value = support::fixture("earned-loaded-shot.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let prepared = prepare(&content, &fixture, case);
        let actor = case.n::<usize>("actor");
        let target = case.n::<usize>("target") - 10;
        let mut warriors = party(&case["targeted"]);
        let mut records = monsters(&case["targeted"]);
        caller_debits(&mut warriors[actor]);
        let mut random = random(&case["targeted"]);
        let initial = random.clone();
        let result = firearm::resolve(
            &mut warriors[actor],
            &mut records[target],
            &prepared,
            &mut random,
            0,
        )
        .unwrap();
        let checkpoint = case.get("pre_kill").unwrap_or(&case["resolved"]);
        assert_party(&warriors, checkpoint);
        assert_monsters(&records, checkpoint);
        assert_eq!(random, self::random(checkpoint));
        let draw_count = checkpoint.n::<u64>("draws") - initial.draws;
        let mut native_random = initial;
        support::replay_rng(
            &mut native_random,
            &case.array("rng")[..draw_count as usize],
        );
        assert_eq!(random, native_random);
        let phase = case
            .array("phase_receipts")
            .iter()
            .find(|phase| phase["address"] == "151022" && phase["spell_id"] == 4101)
            .unwrap();
        let native_sounds: Vec<i16> = checkpoint.array("ordered_sounds")
            [phase.n::<usize>("sound_count_before")..]
            .iter()
            .map(|sound| sound.as_i64().unwrap() as i16)
            .collect();
        assert_eq!(result.sounds, native_sounds);
        assert_eq!(result.damage, [0, 7][index]);
        assert_eq!(result.killed, index == 1);
    }
}
