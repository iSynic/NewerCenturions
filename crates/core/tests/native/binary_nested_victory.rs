use crate::prelude::*;
use crate::support::{checkpoint, record_bytes, replay_command};
use centurions_core::combat;

// The receipt acknowledges combat setup/turns, but executes original child generation.
fn controlled_battle(
    state: &mut Session,
    content: &Content,
    warriors: Vec<Warrior>,
    mut random: RandomState,
) -> Vec<u8> {
    let mission = content.map(0).unwrap();
    let tables = mission.combat.as_ref().unwrap();
    let arena = state.combat.as_mut().unwrap();
    arena.map = TacticalMap {
        tiles: vec![1155; 8100],
        shift: (0, 0),
        edge_word: 0,
    };
    arena.squad = SquadPlacement {
        positions: (20..26).map(|x| (x, 20)).collect(),
        under_tiles: vec![1155; 6],
    };
    arena.order.fill(-1);
    arena.order[1..7].copy_from_slice(&[0, 1, 2, 3, 4, 5]);
    arena.cursor = 1;
    state.warriors = warriors;
    if arena.battle == 14 {
        let mut record = MonsterRecord::read(&tables.monsters, 66).unwrap();
        record.bytes[0x110..0x114].copy_from_slice(&[0, 0, 0, 100]);
        for offset in [0x118, 0x11a, 0x11c, 0x11e] {
            record.bytes[offset..offset + 2].copy_from_slice(&1155_i16.to_be_bytes());
        }
        arena.enemies = EnemySetup {
            records: vec![record],
            positions: vec![(20, 20)],
        };
        for x in 24..=25 {
            for y in 24..=25 {
                arena.map.tiles[x * 90 + y] = 10;
            }
        }
        combat::lifecycle::kill_body(arena, &mut state.warriors, 10).unwrap();
    } else {
        assert_eq!(arena.battle, 15);
        let battle = centurions_content::combat::BattleRecord::read(&tables.battles, 15).unwrap();
        arena.enemies = combat::place_enemies(
            &mut arena.map,
            mission,
            &battle,
            &tables.monsters,
            (40, 40),
            &mut random,
        )
        .unwrap();
    }
    let generated = record_bytes(&arena.enemies.records);
    for record in &mut arena.enemies.records {
        record.bytes[0x110..0x112].fill(0);
    }
    state.random = random;
    generated
}

fn backing(state: &Session) -> Vec<u8> {
    state
        .npc_backing
        .slots
        .iter()
        .flat_map(|slot| slot.as_ref().unwrap().record.bytes.iter().copied())
        .collect()
}

fn body(state: &Session, content: &Content) -> Vec<u8> {
    let position = state.exploration.as_ref().unwrap();
    let mut record = position.script.as_ref().unwrap().record.clone();
    if let Some(story) = state
        .combat_story
        .as_ref()
        .filter(|story| !story.shared_dispatch)
    {
        record[8..40]
            .copy_from_slice(&content.map(1).unwrap().script(story.script).unwrap()[8..40]);
    }
    record
}

#[test]
fn nested_battle_rewards_consume_count_once_and_preserve_backing_records() {
    let nested = crate::support::fixture("nested-victory.json");
    let caller = crate::support::fixture("nested-caller.json");
    let content = content();
    for (case, original) in caller.array("cases").iter().zip(nested.array("cases")) {
        assert_eq!(case["seed"], original["seed"]);
        let mut state = crate::support::mission_session(&content);
        state.warriors = crate::support::warriors(&original["calls"][0]["party_before"]);
        state.allies.fill(0);
        state.ally_count = 0;
        state.random = RandomState::new(case.n("seed"));
        let position = state.exploration.as_mut().unwrap();
        position.restore_map(
            1,
            centurions_core::MapState::authored(content.map(1).unwrap()),
        );
        position.x = 77;
        position.y = 9;
        position.script = Some(crate::support::script_state(caller.bytes("script55"), 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let mut resumed = checkpoint(&state, &content, "native-nested-caller");
        let (mut battles, mut rewards) = (0, 0);
        let (mut sounds, mut texts) = (vec![], vec![]);
        for _ in 0..80 {
            let before = state.clone();
            let fighting = state.combat.is_some()
                && state.exploration.as_ref().unwrap().presentation.is_none();
            let command = if fighting {
                Command::EndTurn
            } else {
                Command::Continue
            };
            let events = replay_command(command, &mut state, &mut resumed, &content);
            sounds.extend(crate::support::event_sounds(&events));
            if events
                .iter()
                .any(|event| matches!(event, Event::CombatStarted { .. }))
            {
                battles += 1;
                let generated =
                    controlled_battle(&mut state, &content, before.warriors, before.random);
                if battles == 2 {
                    assert_eq!(generated, original.bytes("generated"));
                    assert_eq!(
                        (state.random.seed, state.random.draws),
                        (original.n("setup_seed_after"), original.n("setup_draws")),
                    );
                }
                resumed = checkpoint(&state, &content, "native-nested-controlled-battle");
                continue;
            }
            let position = state.exploration.as_ref().unwrap();
            if let Some(Presentation::Dialogue { id, .. }) = position.presentation {
                texts.push(id);
            }
            if position.presentation == Some(Presentation::Loot) {
                let expected = &case["rewards"][rewards];
                assert_eq!(body(&state, &content), expected.bytes("body"));
                assert_eq!(record_bytes(&state.warriors), expected.bytes("warriors"));
                assert_eq!(
                    (state.random.seed, state.random.draws),
                    (expected.n("seed"), expected.n("draws_total")),
                );
                if rewards < 2 {
                    let old = &original["calls"][rewards];
                    assert_eq!(state.party_currency, old.n::<i32>("currency"));
                    assert_eq!(record_bytes(&before.warriors), old.bytes("party_before"));
                    assert_eq!(record_bytes(&state.warriors), old.bytes("party_after"));
                    assert_eq!(backing(&state), old.bytes("records_after"));
                    assert_eq!(
                        (
                            before.random.seed,
                            state.random.seed,
                            state.random.draws - before.random.draws
                        ),
                        (
                            old.n("seed_before"),
                            old.n("seed_after"),
                            old.n("rng_draws")
                        ),
                    );
                    let loot: Vec<_> = state.booty_items.iter().flat_map(|item| item.0).collect();
                    assert_eq!(loot, old.bytes("loot_after"));
                }
                rewards += 1;
                if rewards == 3 {
                    break;
                }
            }
        }
        assert_eq!(
            (battles, rewards),
            (case.n("battle_calls"), case.n("reward_calls"))
        );
        let position = state.exploration.as_ref().unwrap();
        assert_eq!((position.map, position.x, position.y), (1, 77, 9));
        assert_eq!(state.phase, Phase::Exploration);
        let count = state
            .combat
            .as_ref()
            .map_or(0, |arena| arena.enemies.records.len());
        assert_eq!(count, case.n::<usize>("enemy_count_after"));
        assert_eq!(body(&state, &content), case.bytes("body_after"));
        assert_eq!(backing(&state), case.bytes("records_after"));
        let queue = state.retained_death_queue.as_ref().unwrap();
        assert_eq!(queue.count, case.n::<usize>("queue_count_after"));
        let bytes: Vec<_> = queue
            .entries
            .iter()
            .flat_map(|entry| entry.bytes())
            .collect();
        assert_eq!(bytes, case.bytes("queue_after"));
        // The receipt stops before replacement script56; exercise the handoff separately.
        replay_command(Command::Continue, &mut state, &mut resumed, &content);
        assert_eq!(record_bytes(&state.warriors), case.bytes("warriors_after"));
        assert_eq!(
            (state.random.seed, state.random.draws),
            (case.n("seed_after"), case.n("rng_draws"))
        );
        let native = |kind| {
            case.array("events")
                .iter()
                .filter(|event| event[0] == kind)
                .map(|event| event[1].as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        };
        assert_eq!(sounds, native("sound"));
        assert_eq!(texts, native("text"));
    }
}
