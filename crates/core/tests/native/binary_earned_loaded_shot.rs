use crate::prelude::*;

fn fixture() -> (Content, Value) {
    (
        crate::support::content(),
        crate::support::fixture("earned-loaded-shot.json"),
    )
}
use crate::native_common::snapshot_party as party;

fn monsters(snapshot: &Value) -> Vec<MonsterRecord> {
    crate::support::monsters(&snapshot["all_records"])
}
use crate::support::assert_bytes;
use crate::support::compare_party as assert_party;

fn assert_monsters(actual: &[MonsterRecord], snapshot: &Value) {
    assert_eq!(actual.len(), 100);
    for (member, (actual, expected)) in actual.iter().zip(monsters(snapshot)).enumerate() {
        assert_bytes(&actual.bytes, &expected.bytes, &format!("NPC {member}"));
    }
}
use crate::native_common::random_state as random;

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
    let (content, fixture) = fixture();
    for case in fixture.array("cases") {
        let prepared = prepare(&content, &fixture, case);
        crate::support::native_expect! {
        bytes prepared.packet => &case["prepared"]["packet"], "prepared packet";
        scalar centurions_core::combat::player_psi::range(&prepared.packet, prepared.shots).unwrap() => &case["prepared"]["range"], "binary_earned_loaded_shot";
        }
        assert_eq!(
            (
                prepared.slot,
                prepared.shots,
                prepared.rounds,
                prepared.weight,
                prepared.sound,
                prepared.armor_filter
            ),
            (0, 1, 1, 0, 406, false)
        );
        let mut weapon = content.items[165].clone();
        put_word(&mut weapon, 14, 406);
        crate::support::native_expect! {
        bytes weapon => &case["prepared"]["weapon_work"], "weapon work";
        bytes content.items[280] => &case["prepared"]["ammo_work"], "ammo work";
        }
        assert_party(&party(&case["before"]), &case["prepared"]);
        assert_monsters(&monsters(&case["before"]), &case["prepared"]);
        assert_eq!(random(&case["before"]), random(&case["prepared"]));
    }
}

#[test]
fn actual_miss_and_lethal_resolver_match_native_before_death_and_getup() {
    let (content, fixture) = fixture();
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
        assert_eq!(draw_count, [3, 18][index]);
        let mut native_random = initial;
        for draw in &case.array("rng")[..draw_count as usize] {
            crate::support::expect(native_random.bound(draw.n::<i16>("bound")), &draw["result"]);
        }
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
        let instance = &warriors[actor].inventory().unwrap()[0].0;
        assert_eq!(word(instance, 2).unwrap(), 0);
        assert_eq!(word(instance, 4).unwrap(), 280);
        assert_eq!(word(instance, 6).unwrap(), 1);
        assert_bytes(
            &prepared.packet,
            &case["prepared"].bytes("packet"),
            "retained prepared packet",
        );
        if index == 1 {
            assert_eq!(word(&records[target].bytes, 0x110).unwrap(), -2);
            assert_eq!(records[target].bytes[0x105], 1);
            assert_eq!(records[target].bytes[0x106], 9);
            assert_eq!(checkpoint["killed_count"], 1);
        }
    }
}

#[test]
fn firing_admission_does_not_admit_earned_reload_or_ammunition_fitting() {
    let (content, fixture) = fixture();
    for case in fixture.array("cases") {
        let state: Session = serde_json::from_value(
            fixture["foundations"][case.n::<usize>("foundation")]["session"].clone(),
        )
        .unwrap();
        let mut warrior = party(&case["targeted"])[4].clone();
        let before = warrior.clone();
        let mut reserve = state.ammunition[80];
        let original_reserve = reserve;
        let mut random = self::random(&case["targeted"]);
        let original_random = random.clone();
        assert!(firearm::reload(&mut warrior, &mut reserve, true, &mut random).is_err());
        assert_eq!(warrior, before);
        assert_eq!(reserve, original_reserve);
        assert_eq!(random, original_random);
        assert!(
            firearm::fit_ammunition(&warrior, 0, &content, &state.ammunition, false, &mut random)
                .is_err()
        );
        assert_eq!(warrior, before);
        assert_eq!(random, original_random);
    }
}
