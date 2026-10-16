use centurions_content::Content;
use centurions_core::{Command, Event, RuleError, Session};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum Assistance {
    Health,
    Ammunition,
    Psi,
    WeaponDamage,
    FragileEnemies,
    DefeatEnemies,
    Region {
        slot: usize,
        chance: i16,
    },
    Flag {
        index: usize,
        value: i8,
    },
    QuestItems(Vec<i16>),
    Position {
        x: i32,
        y: i32,
        #[serde(default)]
        map: Option<i32>,
    },
}

pub fn apply_recorded(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    match centurions_core::apply(command, state, content) {
        Err(RuleError::Refused { feedback } | RuleError::RefusedAfterEffects { feedback }) => {
            Ok(feedback.into_iter().map(Event::Feedback).collect())
        }
        result => result,
    }
}

pub fn assist(state: &mut Session, assistance: &[Assistance], content: &Content) {
    for setting in assistance {
        match setting {
            Assistance::Region { slot, chance } => {
                assert!(*slot < 20);
                let offset = 160 + slot * 2;
                state.exploration.as_mut().unwrap().region_state[offset..offset + 2]
                    .copy_from_slice(&chance.to_be_bytes());
            }
            Assistance::Flag { index, value } => {
                state.flags[*index] = *value as u8;
            }
            Assistance::DefeatEnemies => {
                if let Some(combat) = state.combat.as_mut() {
                    for index in 0..combat.enemies.records.len() {
                        let enemy = &mut combat.enemies.records[index];
                        if i16::from_be_bytes(enemy.bytes[0x110..0x112].try_into().unwrap()) > 0
                            && i16::from_be_bytes(enemy.bytes[14..16].try_into().unwrap()) != 0
                        {
                            enemy.bytes[0x110..0x112].fill(0);
                            centurions_core::combat::lifecycle::kill_body(
                                combat,
                                &mut state.warriors,
                                index as i16 + 10,
                            )
                            .unwrap();
                        }
                    }
                    assert!(combat.round_scripts.pending.is_none());
                    combat.cursor = combat
                        .order
                        .iter()
                        .position(|body| (0..10).contains(body))
                        .unwrap();
                    combat.selection_pending = false;
                }
            }
            Assistance::Position { x, y, map } => {
                assert_eq!(state.phase, centurions_core::Phase::Exploration);
                centurions_content::mission::OpeningMission::index(*x, *y).unwrap();
                if let Some(map) = map
                    && state.exploration.as_ref().unwrap().map != *map
                {
                    let authored = content
                        .map_at(state.mission.as_deref(), state.scenario_level, *map)
                        .unwrap();
                    let position = state.exploration.as_mut().unwrap();
                    assert!(position.presentation.is_none() && position.script.is_none());
                    let records = state
                        .visited_maps
                        .remove(map)
                        .unwrap_or_else(|| centurions_core::MapState::authored(authored));
                    state
                        .visited_maps
                        .insert(position.map, position.map_state());
                    position.restore_map(*map, records);
                }
                let position = state.exploration.as_mut().unwrap();
                assert!(position.presentation.is_none() && position.script.is_none());
                (position.x, position.y) = (*x, *y);
            }
            Assistance::QuestItems(ids) => {
                for id in ids {
                    if state.warriors.iter().any(|warrior| {
                        warrior
                            .inventory()
                            .unwrap()
                            .iter()
                            .any(|item| item.id() == *id)
                    }) {
                        continue;
                    }
                    let context = content
                        .context(state.mission.as_deref(), state.scenario_level)
                        .unwrap();
                    let item = centurions_core::items::initialize_at_difficulty(
                        *id,
                        0,
                        state.difficulty,
                        context.as_ref(),
                        &mut state.random,
                    )
                    .unwrap();
                    let map = context
                        .map(state.exploration.as_ref().unwrap().map)
                        .unwrap();
                    let collected = centurions_core::equipment::collect_item(
                        &state.warriors[0],
                        &item,
                        context.as_ref(),
                        map,
                    )
                    .unwrap();
                    assert!(collected.collected);
                    state.warriors[0] = collected.warrior;
                }
            }
            Assistance::Health => {
                for warrior in &mut state.warriors {
                    if warrior.stamina().unwrap() > 0 {
                        for offset in [0x50, 0x52] {
                            warrior.bytes[offset..offset + 2]
                                .copy_from_slice(&10000_i16.to_be_bytes());
                        }
                    }
                }
            }
            Assistance::Ammunition => state.ammunition.fill(30000),
            Assistance::Psi => {
                for warrior in &mut state.warriors {
                    warrior.bytes[0x58..0x5a].copy_from_slice(&10000_i16.to_be_bytes());
                }
            }
            Assistance::WeaponDamage => {
                for warrior in &mut state.warriors {
                    for slot in 0..warrior.inventory().unwrap().len() {
                        let offset = 0x132 + slot * 22;
                        if matches!(warrior.bytes[offset + 10], 2 | 15) {
                            warrior.bytes[offset + 15] = 100;
                        }
                    }
                }
            }
            Assistance::FragileEnemies => {
                if let Some(combat) = state.combat.as_mut() {
                    for enemy in &mut combat.enemies.records {
                        if i16::from_be_bytes(enemy.bytes[0x110..0x112].try_into().unwrap()) > 0 {
                            for offset in [0x110, 0x112] {
                                enemy.bytes[offset..offset + 2]
                                    .copy_from_slice(&1_i16.to_be_bytes());
                            }
                        }
                    }
                }
            }
        }
    }
}
