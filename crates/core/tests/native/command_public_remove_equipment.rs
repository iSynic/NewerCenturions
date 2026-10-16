use crate::prelude::*;

use centurions_core::ammunition::WorkBuffers;

use crate::support::buffer as bytes;

use crate::native_common::buffer_words as words;

use crate::snapshots::captured_party as project_party;

fn initial(fixture: &Value, case: &Value) -> Session {
    let snapshot = &case["before"];
    let member = case.n::<usize>("member");
    let mut state = crate::snapshots::helper_frame(fixture, snapshot, member);
    state.ammunition_work = WorkBuffers {
        item: Some(bytes(fixture, &snapshot["item_work"])),
        weapon: Some(bytes(fixture, &snapshot["weapon_work"])),
        ammo: Some(bytes(fixture, &snapshot["ammo_work"])),
    };
    let weapon = state.ammunition_work.weapon.as_ref().unwrap();
    let ammo = state.ammunition_work.ammo.as_ref().unwrap();
    state.weapon_work = WeaponWork {
        sound: Some(word(weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(ammo, 136).unwrap()),
    };
    state.loaded_item_field32 =
        Some(word(state.ammunition_work.item.as_ref().unwrap(), 32).unwrap());
    let combat = state.combat.as_mut().unwrap();
    // No active NPC is established by these zeroed native helper inputs.
    combat.enemies.records.clear();
    combat.enemies.positions.clear();
    combat.weapon_work = state.weapon_work;
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
    expected.loaded_item_field32 = Some(word(&definition, 32).unwrap());
    expected.ammunition_work.item = Some(definition);
    let member = case.n::<usize>("member");
    expected.health_context.as_mut().unwrap().affected[member] = 1;
    assert_eq!(
        expected.health_context.as_ref().unwrap().affected,
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
use crate::support::checkpoint;
use crate::support::same_session as same;

#[test]
fn four_public_removals_at_zero_ap_match_complete_native_state_and_saved_replay() {
    let fixture = crate::support::fixture("public-remove-equipment.json");
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

        let events: Vec<_> = case
            .array("sounds")
            .iter()
            .map(|sound| Event::Sound {
                resource_id: sound.as_i64().unwrap() as i16,
            })
            .collect();
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
        same(&uninterrupted, &expected);
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        same(&resumed, &expected);
        checkpoint(&uninterrupted, &content, &format!("{index}-after"));
        if index == 0 {
            assert_eq!(state.health_context.as_ref().unwrap().affected[0], 0);
            assert_eq!(expected.health_context.as_ref().unwrap().affected[0], 1);
        }
    }
}
