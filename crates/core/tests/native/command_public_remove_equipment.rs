use crate::{native_common, prelude::*, snapshots, support};

use centurions_core::ammunition::WorkBuffers;

use support::buffer as bytes;

use native_common::buffer_words as words;

use snapshots::captured_party as project_party;

fn initial(fixture: &Value, case: &Value) -> Session {
    let snapshot = &case["before"];
    let member = case.n::<usize>("member");
    let mut state = snapshots::helper_frame(fixture, snapshot, member);
    state.native_work.ammunition_work = WorkBuffers {
        item: Some(bytes(fixture, &snapshot["item_work"])),
        weapon: Some(bytes(fixture, &snapshot["weapon_work"])),
        ammo: Some(bytes(fixture, &snapshot["ammo_work"])),
    };
    let weapon = state.native_work.ammunition_work.weapon.as_ref().unwrap();
    let ammo = state.native_work.ammunition_work.ammo.as_ref().unwrap();
    state.native_work.weapon_work = WeaponWork {
        natural_attack_index: 0,
        sound: word(weapon, 14).unwrap(),
        weapon_armor_filter: word(weapon, 136).unwrap(),
        ammunition_armor_filter: word(ammo, 136).unwrap(),
    };
    state.native_work.loaded_item_field32 =
        word(state.native_work.ammunition_work.item.as_ref().unwrap(), 32).unwrap();
    let combat = state.combat.as_mut().unwrap();
    // No active NPC is established by these zeroed native helper inputs.
    combat.enemies.records.clear();
    combat.enemies.positions.clear();
    combat.weapon_work = state.native_work.weapon_work;
    state
}

fn expected(state: &Session, fixture: &Value, case: &Value, content: &Content) -> Session {
    let mut expected = state.clone();
    let after = &case["after"];
    project_party(&mut expected, fixture, after);
    let id = case.n::<usize>("id");
    let mut definition = content.items[id].clone();
    let class = word(&definition, 26).unwrap().wrapping_abs();
    definition[26..28].copy_from_slice(&class.to_be_bytes());
    assert_eq!(definition, bytes(fixture, &after["item_work"]));
    expected.native_work.loaded_item_field32 = word(&definition, 32).unwrap();
    expected.native_work.ammunition_work.item = Some(definition);
    let member = case.n::<usize>("member");
    expected
        .native_work
        .health_context
        .affected[member] = 1;
    assert_eq!(
        expected
            .native_work
            .health_context
            .affected,
        words(fixture, &after["markers"])[..6]
    );
    expected
}

fn compaction_is_unchanged(state: &Session, member: usize) {
    let warrior = &state.warriors[member];
    let count = warrior.inventory().unwrap().len();
    let records = warrior.bytes[0x132..0x132 + 2200].as_chunks::<22>().0;
    assert!(records[..count].iter().all(|record| record[9] == 0));
    assert!(
        records[count..]
            .iter()
            .all(|record| record[..4] == [0; 4] && record[9] == 0)
    );
    // At original compaction12006e no equipped row can move, and the unused
    // record fields it clears are already zero in these four native results.
}
use support::checkpoint;

#[test]
fn four_public_removals_at_zero_ap_match_complete_native_state_and_saved_replay() {
    let fixture = support::fixture("public-remove-equipment.json");
    let content = content();

    for (index, case) in fixture.array("cases")[..4].iter().enumerate() {
        let state = initial(&fixture, case);
        let member = case.n::<usize>("member");
        let expected = expected(&state, &fixture, case, &content);
        state.validate(&content).unwrap();
        expected.validate(&content).unwrap();
        assert_eq!(word(&state.warriors[member].bytes, 0x40).unwrap(), 0);
        assert_eq!(word(&expected.warriors[member].bytes, 0x40).unwrap(), 0);
        assert_eq!(
            word(&state.warriors[member].bytes, 0x42).unwrap(),
            word(&expected.warriors[member].bytes, 0x42).unwrap()
        );
        assert_eq!(expected.combat, state.combat);
        compaction_is_unchanged(&expected, member);

        let events: Vec<_> = support::sound_events(case.n::<Vec<i16>>("sounds"));
        let definition = &content.items[case.n::<usize>("id")];
        let sound = word(definition, 12).unwrap();
        let sound = if sound == 0 {
            word(definition, 14).unwrap()
        } else {
            sound
        };
        assert_eq!(events, vec![Event::Sound { resource_id: sound }]);
        let command = Command::UnequipItem {
            warrior: member,
            slot: case.n::<usize>("slot"),
        };
        let mut uninterrupted = state.clone();
        let mut resumed = checkpoint(&state, &content, &format!("{index}-before"));
        assert_eq!(
            apply(command.clone(), &mut uninterrupted, &content).unwrap(),
            events
        );
        support::same_captured_session(&uninterrupted, &expected);
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        support::same_captured_session(&resumed, &expected);
        checkpoint(&uninterrupted, &content, &format!("{index}-after"));
        if index == 0 {
            assert_eq!(
                state.native_work.health_context.affected[0],
                0
            );
            assert_eq!(
                expected
                    .native_work
                    .health_context
                    .affected[0],
                1
            );
        }
    }

    // Native wear/remove retries must not corrupt the subsequently loaded Psi packet.
    let fixture = support::fixture("equipment-spell-cache-alias.json");
    for case in fixture.array("cases") {
        let input = Warrior::read(&bytes(&fixture, &case["before"]["member_record"])).unwrap();
        let result = if case["variant"] == "wear_retry_old1_to38" {
            let worn = equipment::wear(
                &input,
                &content,
                content.map(0).unwrap(),
                case.n("slot"),
                false,
            )
            .unwrap();
            support::expect_array(&worn.sounds, &case["sounds"]);
            worn.warrior
        } else {
            input
        };
        support::native_expect! {
            bytes &result.bytes => &bytes(&fixture, &case["after_equipment"]["member_record"]), "equipment/cache alias";
        };
        let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
        assert_eq!(
            *attacks,
            bytes(&fixture, &case["after_equipment"]["spell_cache"])
        );
        let packet = spells::load_packet(attacks, case.n("spell")).unwrap();
        assert_eq!(packet, bytes(&fixture, &case["after_spell_load"]["packet"]));
    }
}
