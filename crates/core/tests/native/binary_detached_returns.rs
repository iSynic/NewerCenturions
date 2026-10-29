//! Saved source joins around bounded original walking/reaction instruction tails.
use crate::{prelude::*, support};
use centurions_core::combat::neighbor_work::NeighborWork;
use centurions_core::combat::retained::{InitiativeWork, NpcBacking, NpcSlot};
use centurions_core::combat::runtime::{CombatReturn, DeathStory};
use centurions_core::combat::tactical_work::TacticalWork;

#[test]
fn detached_returns_match_native_work_records_rng_and_sound_order() {
    let fixture = support::fixture("detached-returns.json");
    let content = content();
    for case in fixture.array("cases") {
        assert_eq!(case.text("status"), "completed");
        let input = &case["declarations"]["source"];
        let mut state = initial(input, &content);
        let events = support::saved_command(Command::Continue, &mut state, &content);
        let work = &state.native_work.tactical_work;
        let npcs: Vec<_> = state
            .native_work
            .npc_backing
            .slots
            .iter()
            .flat_map(|slot| slot.as_ref().unwrap().record.bytes.iter().copied())
            .collect();
        let positions: Vec<_> = work
            .positions
            .iter()
            .flat_map(|point| [point.0, point.1])
            .collect();
        let neighbors = &state.native_work.neighbor_work;
        let presence: Vec<_> = neighbors
            .presence
            .iter()
            .flat_map(|row| row.iter().copied())
            .collect();
        let bodies = neighbors.bodies.clone();
        for (name, actual) in [
            ("warriors", support::record_bytes(&state.warriors)),
            ("npcs", npcs),
            (
                "map",
                support::word_bytes(&work.map.tiles),
            ),
            ("party_positions", support::word_bytes(&positions)),
            (
                "under",
                support::word_bytes(&work.under_tiles),
            ),
            ("fields", work.spell_fields.clone()),
            ("used", work.field_used.clone()),
            ("presence", support::word_bytes(&presence)),
            ("bodies", support::word_bytes(&bodies)),
            ("mask", support::word_bytes(&state.native_work.party_mask)),
            (
                "departure",
                support::word_bytes(&[neighbors.departure]),
            ),
            (
                "natural_index",
                support::word_bytes(&[state.native_work.weapon_work.natural_attack_index]),
            ),
            (
                "direction",
                state
                    .exploration
                    .as_ref()
                    .unwrap()
                    .last_move
                    .map(|v| v as u8)
                    .to_vec(),
            ),
        ] {
            let expected = case["memory"][name].bytes("hex");
            assert!(
                actual == expected,
                "{} {name}: first byte {:?}",
                case.text("name"),
                actual.iter().zip(&expected).position(|(a, b)| a != b)
            );
        }
        assert_eq!(state.random.seed, case.n::<u32>("seed"));
        assert_eq!(state.random.draws, case.n::<u64>("draws"));
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::Sound { resource_id } => Some(*resource_id),
                _ => None,
            })
            .collect();
        let native: Vec<_> = case
            .array("calls")
            .iter()
            .filter(|call| call.text("id") == "sound")
            .map(|call| {
                i16::from_str_radix(call["arguments"].text("id"), 16).unwrap_or_else(|_| {
                    u16::from_str_radix(call["arguments"].text("id"), 16).unwrap() as i16
                })
            })
            .collect();
        assert_eq!(sounds, native, "{} sound order", case.text("name"));
        assert!(state.combat.is_none() && state.combat_story.is_none());
    }
}

pub(super) fn initial(input: &Value, content: &Content) -> Session {
    let before = &input["before"];
    let mut state = support::mission_session(content);
    state.warriors = support::warriors(&before["warriors"]);
    state.selected_warrior = input.n("selected");
    state
        .native_work
        .health_context
        .selected_warrior = state.selected_warrior;
    state.exploration.as_mut().unwrap().last_move = before
        .bytes("direction")
        .iter()
        .map(|v| *v as i8)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let npc_positions = before.words("npc_positions");
    state.native_work.npc_backing = NpcBacking {
        active_count: 1,
        origin: (5, 5),
        slots: support::monsters(&before["npcs"])
            .into_iter()
            .enumerate()
            .map(|(i, record)| {
                Some(NpcSlot {
                    record,
                    position: Some((
                        i32::from(npc_positions[i * 2]) + 5,
                        i32::from(npc_positions[i * 2 + 1]) + 5,
                    )),
                })
            })
            .collect(),
    };
    let positions = before.words("party_positions");
    let under = before.words("under");
    state.native_work.tactical_work = TacticalWork {
        map: TacticalMap {
            tiles: before.words("map"),
            shift: (0, 0),
            edge_word: 0,
        },
        saved_land: None,
        positions: std::array::from_fn(|i| (positions[i * 2], positions[i * 2 + 1])),
        under_tiles: std::array::from_fn(|i| under[i]),
        spell_fields: before.bytes("fields"),
        field_used: before.bytes("used"),
        battle: None,
    };
    let mut order = vec![-1; 110];
    order[0] = input.n("actor");
    state.native_work.initiative_work = InitiativeWork { order, cursor: 0 };
    state.native_work.victory_work.total = 5;
    state.native_work.victory_work.defeated = 0;
    state.native_work.weapon_work.natural_attack_index = 0;
    let presence = before.words("presence");
    state.native_work.neighbor_work = NeighborWork {
        departure: 0,
        presence: std::array::from_fn(|i| presence[i * 110..(i + 1) * 110].to_vec()),
        bodies: before.words("bodies"),
    };
    state.native_work.party_mask = before.words("mask").try_into().unwrap();
    state.random = RandomState::new(input.n("seed"));
    state.random.draws = input.n("draws");
    let caller: CombatReturn =
        serde_json::from_value(json!({"WarriorMove":input["frame"]})).unwrap();
    state.combat_story = Some(DeathStory {
        return_to: Some(Box::new(caller)),
        direct: true,
        child_returned: true,
        body: 0,
        actor: 0,
        script: 1,
        cursor: 8,
        ..Default::default()
    });
    state.flags.fill(0);
    support::pause_script(&mut state, vec![0; 42], 0);
    state
}
