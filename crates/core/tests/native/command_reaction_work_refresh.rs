use crate::{native_common, prelude::*, snapshots, support};

use centurions_core::ammunition::LoadedItems;

use support::buffer as bytes;

use native_common::buffer_words as words;
use native_common::snapshot_word as number;

use native_common::random_state as random;

fn initial(fixture: &Value, case: &Value) -> Session {
    let snapshot = &case["before"];
    let member = snapshot.n::<usize>("active_member");
    let mut state = snapshots::helper_frame(fixture, snapshot, member);
    // Captured neutral input buffers establish scalar projections only.
    // They are not authored definition0; raw records remain unknown.
    for key in ["item_work", "weapon_work", "ammo_work"] {
        assert_eq!(bytes(fixture, &snapshot[key]), vec![0; 150]);
    }
    state.globals.loaded_items = LoadedItems::default();
    state.globals.weapon_traits = WeaponTraits {
        natural_attack_index: 0,
        sound: 0,
        weapon_armor_filter: 0,
        ammunition_armor_filter: 0,
    };
    state.globals.loaded_item_field32 = fixture["foundation"].n::<i16>("item_context");
    let combat = state.combat.as_mut().unwrap();
    let npcs = bytes(fixture, &snapshot["npcs"]);
    assert_eq!(npcs.len(), 100 * MONSTER_BYTES);
    assert!(npcs[MONSTER_BYTES..].iter().all(|byte| *byte == 0));
    let positions = words(fixture, &snapshot["npc_positions"]);
    assert!(positions[2..].iter().all(|word| *word == 0));
    combat.roster.records = vec![MonsterRecord {
        bytes: npcs[..288].to_vec(),
    }];
    combat.roster.mon_pos = vec![(positions[0], positions[1])];
    combat.squad.char_under = combat
        .squad
        .pos
        .iter()
        .map(|(x, y)| {
            let index = (*x as usize + 5 + combat.field.map_shift.0 as usize) * 90
                + *y as usize
                + 5
                + combat.field.map_shift.1 as usize;
            combat.field.tiles[index]
        })
        .collect();
    combat.weapon_traits = state.globals.weapon_traits;
    native_common::source_victory_context(&mut state);
    state
}

fn normalized(content: &Content, id: usize) -> Vec<u8> {
    let mut record = content.items[id].clone();
    let class = word(&record, 26).unwrap().wrapping_abs();
    record[26..28].copy_from_slice(&class.to_be_bytes());
    record
}

fn full_party(state: &Session) -> Vec<u8> {
    support::record_bytes(&state.warriors)
}

fn native_loaders(fixture: &Value, case: &Value, content: &Content) {
    for phase in case.array("native_phases") {
        let key = match phase.text("address") {
            "130d38" => "weapon_work",
            "130e54" => "ammo_work",
            "130afe" => "item_work",
            _ => continue,
        };
        let id = phase["arguments"][0].as_u64().unwrap() as usize;
        assert_eq!(
            bytes(fixture, &phase["after"][key]),
            normalized(content, id)
        );
        for retained in ["item_work", "weapon_work", "ammo_work"] {
            if retained != key {
                assert_eq!(
                    bytes(fixture, &phase["before"][retained]),
                    bytes(fixture, &phase["after"][retained])
                );
            }
        }
    }
}

fn compose_reaction(state: &Session, fixture: &Value, case: &Value, content: &Content) -> Session {
    let mut expected = state.clone();
    let mode = number(case, "mode");
    assert_eq!(current_combat(&expected).active_body(), Some(0));
    assert_eq!(expected.warriors[0].bytes[0x9dd], 1);
    assert_eq!(word(&expected.warriors[0].bytes, 0xe4).unwrap(), 1);
    assert_eq!(word(&expected.warriors[0].bytes, 0xca).unwrap(), 0);
    let record = &mut expected.warriors[0].bytes;
    let dodged = u32::from_be_bytes(record[0xa56..0xa5a].try_into().unwrap());
    record[0xa56..0xa5a].copy_from_slice(&dodged.wrapping_add(1).to_be_bytes());
    assert_eq!(
        full_party(&expected),
        bytes(fixture, &case["after"]["party"])
    );
    let combat = expected.combat.as_mut().unwrap();
    assert_eq!(word(&combat.roster.records[0].bytes, 0x4e).unwrap(), 0);
    assert_eq!(
        combat.roster.records[0].bytes[0x104],
        if mode == 0 { 1 } else { 0 }
    );
    combat.roster.records[0].bytes[0x104] = 0;
    let mut npcs = vec![0; 100 * MONSTER_BYTES];
    npcs[..288].copy_from_slice(&combat.roster.records[0].bytes);
    assert_eq!(npcs, bytes(fixture, &case["after"]["npcs"]));
    // Natural helper136398 establishes all three definitions before the miss;
    // active-party refresh1115a8 subsequently replaces only ITEM with ID1.
    let natural = normalized(content, 0);
    assert_eq!(word(&natural, 100).unwrap(), 0);
    let item = normalized(content, 1);
    assert_eq!(natural, bytes(fixture, &case["after"]["weapon_work"]));
    assert_eq!(natural, bytes(fixture, &case["after"]["ammo_work"]));
    assert_eq!(item, bytes(fixture, &case["after"]["item_work"]));
    expected.globals.loaded_items = LoadedItems {
        item: Some(item.clone()),
        weapon: Some(natural.clone()),
        ammo: Some(natural.clone()),
    };
    expected.globals.loaded_item_field32 = word(&item, 32).unwrap();
    expected.globals.weapon_traits = WeaponTraits {
        natural_attack_index: 0,
        sound: word(&natural, 14).unwrap(),
        weapon_armor_filter: word(&natural, 136).unwrap(),
        ammunition_armor_filter: word(&natural, 136).unwrap(),
    };
    combat.weapon_traits = expected.globals.weapon_traits;
    let attacks = &content
        .map(state.exploration.as_ref().unwrap().map)
        .unwrap()
        .combat
        .as_ref()
        .unwrap()
        .attacks;
    let packet = attacks[0x21c2..0x21c2 + 58].to_vec();
    assert_eq!(packet, bytes(fixture, &case["after"]["packet"]));
    combat.spell_info = packet.clone();
    expected.globals.spell_info = packet.clone();
    expected.globals.health_context.effect = packet[18] as i8;
    assert_eq!(
        expected.globals.health_context.effect,
        number(&case["after"], "health_effect") as i8
    );

    assert_eq!(expected.random.bound(100), 48);

    assert_eq!(expected.random, random(&case["after"]));
    assert_eq!(case["sounds"], serde_json::json!([650]));
    native_loaders(fixture, case, content);
    expected
}

fn cell(state: &Session, x: i16, y: i16) -> usize {
    let shift = current_combat(state).field.map_shift;
    (usize::try_from(x + shift.0 + 5).unwrap() * 90) + usize::try_from(y + shift.1 + 5).unwrap()
}
use support::checkpoint;
use support::same_session as same;

#[test]
fn public_moves_match_native_reactions_and_source_composed_outer_effects() {
    let fixture = support::fixture("reaction-work-refresh.json");
    let content = content();

    for (index, case) in fixture.array("cases").iter().enumerate() {
        assert_eq!(number(case, "mode"), [0, 2][index]);
        let mut state = initial(&fixture, case);
        assert_eq!(current_combat(&state).squad.pos[0], (20, 20));
        assert_eq!(current_combat(&state).roster.mon_pos, vec![(21, 20)]);
        let old_cell = cell(&state, 20, 20);
        let next_cell = cell(&state, 19, 20);
        let mission = content
            .map(state.exploration.as_ref().unwrap().map)
            .unwrap();
        let tile = if index == 0 { 1061 } else { 1155 };
        assert_eq!(mission.tile_word(155, 0).unwrap(), 82);
        assert_eq!(mission.tile_word(155, 2).unwrap(), 2);
        assert_eq!(mission.tile_word(155, 4).unwrap(), 0);
        if index == 0 {
            assert_ne!(mission.tile_word(61, 4).unwrap(), 0);
            // Declared blocking destination does not change native adjacency,
            // the miss roll, or any helper's retained rule writer inputs.
            state.combat.as_mut().unwrap().field.tiles[next_cell] = tile;
        }
        state.validate(&content).unwrap();
        let mut expected = compose_reaction(&state, &fixture, case, &content);

        // The original helper is the native oracle boundary. Outer Move is
        // source-composed in a fixed logical frame; native camera recentering
        // and a complete original walking caller are not certified here.
        // Facing11c0cc and blocked sounds11c932/11ce50 follow Mode0.
        expected.warriors[0].bytes[0x9e0] = 1;
        expected.bandage_ready = 0;
        // Combatchoice writes442736/442737 before Mode0. A blocked walk
        // clears those shared bytes; a continuing step retains its direction.
        expected.exploration.as_mut().unwrap().last_move =
            if index == 0 { [0, 0] } else { [-1, 0] };
        // AttackResolved uses the established synthesized neighbor convention;
        // native attack2 requests650 internally and captures no typed event.
        // The Sound subsequence alone has the native audio ordering.
        let mut events = vec![
            Event::AttackResolved {
                attacker: 10,
                target: 0,
                damage: 0,
                defeated: false,
                icon: Default::default(),
                flight: Default::default(),
            },
            Event::Sound { resource_id: 650 },
        ];
        if index == 0 {
            events.extend([
                Event::Sound { resource_id: -129 },
                Event::Sound { resource_id: -130 },
            ]);
        } else {
            // Getmovecost153de6 halves signed WORD2, subtracts1, clamps0,
            // then adds1 for nonzero dx.11c966 is the captured Mode2 helper;
            // 11c99a/11ca0e/11ca76/11cad4/11caf4 commit MV/map/position.
            let property = mission.tile_word(tile - 1000, 2).unwrap();
            let half = property.wrapping_add((property as u16 >> 15) as i16) >> 1;
            let cost = half.wrapping_sub(1).max(0).wrapping_add(1);
            assert_eq!(cost, 1);
            let remaining = word(&expected.warriors[0].bytes, 0x3c)
                .unwrap()
                .wrapping_sub(cost);
            expected.warriors[0].bytes[0x3c..0x3e].copy_from_slice(&remaining.to_be_bytes());
            let combat = expected.combat.as_mut().unwrap();
            combat.field.tiles[old_cell] = combat.squad.char_under[0];
            combat.squad.char_under[0] = tile;
            combat.field.tiles[next_cell] = 0;
            combat.squad.pos[0] = (19, 20);
            events.extend([
                Event::TacticalMoved {
                    body: 0,
                    x: 19,
                    y: 20,
                },
                Event::Sound {
                    resource_id: mission.tile_word(tile - 1000, 0).unwrap(),
                },
            ]);
        }
        expected.validate(&content).unwrap();
        let mut uninterrupted = state.clone();
        let mut resumed = checkpoint(&state, &content, &format!("{index}-before"));
        let command = Command::Move { dx: -1, dy: 0 };
        assert_eq!(
            apply(command.clone(), &mut uninterrupted, &content).unwrap(),
            events
        );
        support::same_captured_session(&uninterrupted, &expected);
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        support::same_captured_session(&resumed, &expected);
        same(
            &checkpoint(&uninterrupted, &content, &format!("{index}-after")),
            &uninterrupted,
        );
    }
}
