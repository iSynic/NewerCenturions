use crate::prelude::*;

use centurions_core::ammunition::WorkBuffers;
fn fixture() -> Value {
    crate::support::fixture("public-remove-equipment.json")
}

use crate::support::buffer as bytes;

fn words(fixture: &Value, value: &Value) -> Vec<i16> {
    crate::support::words(&bytes(fixture, value))
}
use crate::native_common::snapshot_word as number;

use crate::native_common::random_state as random;

fn project_party(state: &mut Session, fixture: &Value, snapshot: &Value) {
    let party = bytes(fixture, &snapshot["party"]);
    assert_eq!(party.len(), 6 * 3138);
    for (warrior, record) in state.warriors.iter_mut().zip(party.as_chunks::<3138>().0) {
        let captured = Warrior::read(record).unwrap();
        assert_eq!(warrior.name, captured.name);
        warrior.bytes = captured.bytes;
    }
}

fn initial(fixture: &Value, case: &Value) -> Session {
    let background: Value = crate::support::fixture("player-psi.json");
    // The mission supplies valid unobserved background for controlled native
    // inputs. These original helper controls do not describe an earned route.
    let mut state: Session =
        serde_json::from_value(background["route_foundation"].clone()).unwrap();
    let exploration = state.exploration.as_mut().unwrap();
    exploration.presentation = None;
    exploration.script = None;
    state.movement_continuation = None;
    state.medical_continuation = None;
    state.combat_story = None;
    state.handled_deaths.clear();
    let snapshot = &case["before"];
    let member = case.n::<usize>("member");
    project_party(&mut state, fixture, snapshot);
    let reserves = bytes(fixture, &snapshot["reserves"]);
    assert_eq!(reserves.len(), 804);
    state.ammunition = reserves[..800]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|long| i32::from_be_bytes(*long))
        .collect();
    state.loot_quality = Some(bytes(fixture, &snapshot["loot_quality"]));
    assert_eq!(&reserves[800..], &state.loot_quality.as_ref().unwrap()[..4]);
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
    state.random = random(snapshot);
    state.selected_warrior = member;
    state.flags = bytes(fixture, &snapshot["flags"]);
    state.party_buffs = bytes(fixture, &snapshot["party_buffs"]);
    state.psi_memory = Some(bytes(fixture, &snapshot["memory"]));
    state.crate_items = crate::snapshots::inventory(&|key| bytes(fixture, &snapshot[key]), "crate");
    state.booty_items = crate::snapshots::inventory(&|key| bytes(fixture, &snapshot[key]), "loot");
    state.attack_packet = Some(bytes(fixture, &snapshot["packet"]));
    state.attack_identity = Some(words(fixture, &snapshot["identity"]).try_into().unwrap());
    state.attack_count = number(snapshot, "count");
    state
        .party_mask
        .copy_from_slice(&words(fixture, &snapshot["mask"])[..6]);
    state.health_context = Some(HealthContext {
        unavailable: 0,
        selected_warrior: member,
        effect: number(snapshot, "health_effect") as i8,
        last_body: number(snapshot, "health_last_body"),
        marks_affected: number(snapshot, "heal_marks_affected") != 0,
        affected: words(fixture, &snapshot["markers"])[..6]
            .try_into()
            .unwrap(),
    });
    let combat = state.combat.as_mut().unwrap();
    combat.map.tiles = words(fixture, &snapshot["map"]);
    combat.map.shift = (
        number(snapshot, "shift_x") - 5,
        number(snapshot, "shift_y") - 5,
    );
    combat.squad.positions = words(fixture, &snapshot["party_positions"])
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| (pair[0], pair[1]))
        .collect();
    // No active NPC is established by these zeroed native helper inputs.
    assert!(
        bytes(fixture, &snapshot["npcs"])
            .iter()
            .all(|byte| *byte == 0)
    );
    assert!(
        bytes(fixture, &snapshot["npc_positions"])
            .iter()
            .all(|byte| *byte == 0)
    );
    combat.enemies.records.clear();
    combat.enemies.positions.clear();
    combat.order = words(fixture, &snapshot["order"]);
    combat.cursor = snapshot.n::<usize>("cursor");
    combat.round = number(snapshot, "round_script_counter") - 1;
    combat.round_scripts = RoundScripts::default();
    combat.death_queue = DeathQueue::default();
    combat.spell_fields = bytes(fixture, &snapshot["fields"]);
    combat.field_used = bytes(fixture, &snapshot["field_used"]);
    combat.attack_packet = state.attack_packet.clone();
    combat.attack_identity = state.attack_identity;
    combat.weapon_work = state.weapon_work;
    combat.unavailable = 0;
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
    assert_eq!(word(&definition, 46).unwrap(), 0);
    assert_eq!(word(&definition, 48).unwrap(), 0);
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
    let fixture = fixture();
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
        same(
            &checkpoint(&uninterrupted, &content, &format!("{index}-after")),
            &expected,
        );
        if index == 0 {
            assert_eq!(state.health_context.as_ref().unwrap().affected[0], 0);
            assert_eq!(expected.health_context.as_ref().unwrap().affected[0], 1);
        }
    }
}
