use centurions_content::{Content, InventoryRecord, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    Command, Event, Phase, RuleError, Session, apply,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, weapon_work::WeaponWork},
    load, save,
};
use serde_json::Value;

fn fixture() -> Value {
    fixture_named("owned-player-firearm.json")
}
fn fixture_named(name: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}
fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}
fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| i16::from_be_bytes(*p))
        .collect()
}
fn positions(value: &Value) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect()
}
fn sounds(events: &[Event]) -> Vec<i16> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Sound { resource_id } => Some(*resource_id),
            _ => None,
        })
        .collect()
}
fn native_sounds(snapshot: &Value) -> Vec<i16> {
    snapshot["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i16)
        .collect()
}
fn work(snapshot: &Value) -> WeaponWork {
    let weapon = bytes(&snapshot["weapon"]);
    let ammo = bytes(&snapshot["ammo"]);
    WeaponWork {
        sound: Some(word(&weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(&ammo, 136).unwrap()),
    }
}
fn project(state: &mut Session, snapshot: &Value) {
    for (warrior, native) in state
        .warriors
        .iter_mut()
        .zip(bytes(&snapshot["party"]).as_chunks::<3138>().0)
    {
        warrior.bytes = Warrior::read(native).unwrap().bytes;
    }
    state.crate_items = bytes(&snapshot["crate"])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|p| InventoryRecord(*p))
        .collect();
    state.ammunition = bytes(&snapshot["ammunition"])
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| i32::from_be_bytes(*p))
        .collect();
    state.party_buffs = bytes(&snapshot["party_buffs"]);
    state.psi_memory = Some(bytes(&snapshot["memory"]));
    state.attack_packet = Some(bytes(&snapshot["packet"]));
    state.attack_identity = Some(words(&snapshot["identity"]).try_into().unwrap());
    state.attack_count = snapshot["attack_count"].as_i64().unwrap() as i16;
    state.loaded_item_field32 = Some(snapshot["item_context"].as_i64().unwrap() as i16);
    state.random.seed = snapshot["rng_seed"].as_u64().unwrap() as u32;
    state.random.draws = snapshot["rng_draws"].as_u64().unwrap();
    state.weapon_work = work(snapshot);
    let mut health = centurions_core::story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    health.affected = words(&snapshot["markers"])[..6].try_into().unwrap();
    health.effect = bytes(&snapshot["packet"])[18] as i8;
    state.health_context = Some(health);
    if let Some(combat) = state.combat.as_mut() {
        combat.enemies.records[0].bytes = bytes(&snapshot["target"]);
        combat.squad.positions = positions(&snapshot["party_positions"]);
        combat.enemies.positions = positions(&snapshot["monster_positions"])[..1].to_vec();
        combat.map.tiles = words(&snapshot["map"]);
        if let (Some(x), Some(y)) = (snapshot["shift_x"].as_i64(), snapshot["shift_y"].as_i64()) {
            combat.map.shift = (x as i16, y as i16);
        }
        combat.order = words(&snapshot["order"]);
        combat.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
        combat.round = snapshot["round"].as_i64().unwrap() as i16;
        combat.unavailable = snapshot["unavailable"].as_i64().unwrap() as i8;
        combat.spell_fields = bytes(&snapshot["fields"]);
        combat.field_used = bytes(&snapshot["used"]);
        combat.attack_packet = state.attack_packet.clone();
        combat.attack_identity = state.attack_identity;
        combat.weapon_work = state.weapon_work;
    }
}
fn assert_snapshot(state: &Session, snapshot: &Value) {
    let effect = bytes(&snapshot["packet"])[18] as i8;
    assert_eq!(state.health_context.as_ref().unwrap().effect, effect);
    assert_eq!(state.attack_packet.as_ref().unwrap()[18] as i8, effect);
    if let Some(combat) = state.combat.as_ref() {
        assert_eq!(combat.attack_packet.as_ref().unwrap()[18] as i8, effect);
    }
    let mut expected = state.clone();
    project(&mut expected, snapshot);
    assert_session(&expected, state);
}
fn assert_session(expected: &Session, actual: &Session) {
    let left = serde_json::to_value(expected).unwrap();
    let right = serde_json::to_value(actual).unwrap();
    let changed: Vec<_> = left
        .as_object()
        .unwrap()
        .iter()
        .filter_map(|(key, value)| (right[key] != *value).then_some(key))
        .collect();
    assert!(
        changed.is_empty(),
        "Changed Session fields: {changed:?}; expected work {:?}, actual work {:?}",
        expected.weapon_work,
        actual.weapon_work
    );
}
fn input(fixture: &Value, snapshot: &Value, in_combat: bool) -> Session {
    let mut state: Session =
        serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
    if in_combat {
        state.phase = Phase::Combat;
        state.combat = Some(Combat {
            battle: 85,
            map: TacticalMap {
                tiles: words(&snapshot["map"]),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: positions(&snapshot["party_positions"]),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: bytes(&snapshot["target"]),
                }],
                positions: positions(&snapshot["monster_positions"])[..1].to_vec(),
            },
            order: words(&snapshot["order"]),
            cursor: snapshot["cursor"].as_u64().unwrap() as usize,
            round: snapshot["round"].as_i64().unwrap() as i16,
            round_scripts: Default::default(),
            death_queue: Default::default(),
            spell_fields: bytes(&snapshot["fields"]),
            field_used: bytes(&snapshot["used"]),
            attack_packet: Some(bytes(&snapshot["packet"])),
            attack_identity: Some(words(&snapshot["identity"]).try_into().unwrap()),
            weapon_work: work(snapshot),
            unavailable: snapshot["unavailable"].as_i64().unwrap() as i8,
        });
    }
    project(&mut state, snapshot);
    state
}
fn checkpoint(state: &Session, content: &Content, path: &std::path::Path) -> Session {
    save(path, state, content).unwrap();
    let resumed = load(path, content).unwrap();
    assert_eq!(&resumed, state);
    resumed
}
fn assert_atomic(state: &Session, content: &Content, command: Command) {
    let mut candidate = state.clone();
    assert!(
        apply(command.clone(), &mut candidate, content).is_err(),
        "unexpectedly admitted {command:?}"
    );
    assert_eq!(&candidate, state);
}

fn source_caches(
    state: &mut Session,
    content: &Content,
    snapshot: &Value,
    ammunition: bool,
    prepared: bool,
) {
    let mut item = content.items[2].clone();
    let class = word(&item, 26).unwrap().wrapping_abs();
    item[26..28].copy_from_slice(&class.to_be_bytes());
    assert_eq!(item, bytes(&snapshot["item_work"]));
    state.ammunition_work.item = Some(item.clone());
    if ammunition {
        let mut ammo = content.items[201].clone();
        let class = word(&ammo, 26).unwrap().wrapping_abs();
        ammo[26..28].copy_from_slice(&class.to_be_bytes());
        assert_eq!(ammo, bytes(&snapshot["ammo"]));
        if prepared {
            // The spell-derived prepattack branch applies nonzero ammo sounds
            // to the raw weapon WORDs before resolving the shot.
            for offset in [12, 14] {
                if word(&ammo, offset).unwrap() != 0 {
                    item[offset..offset + 2].copy_from_slice(&ammo[offset..offset + 2]);
                }
            }
            assert_eq!(item, bytes(&snapshot["weapon"]));
            state.ammunition_work.weapon = Some(item);
        }
        state.ammunition_work.ammo = Some(ammo);
    }
}

#[test]
fn earned_serial_pickup_wear_fit_matches_native_phases_and_complete_resume() {
    let fixture = fixture();
    let content = content();
    let case = &fixture["equipment_cases"][0];
    let mut state = input(&fixture, &case["before"], false);
    state.validate(&content).unwrap();
    let before = state.clone();
    let path = std::env::temp_dir().join(format!(
        "centurions-owned-firearm-equipment-{}.json",
        std::process::id()
    ));
    let mut resumed = checkpoint(&state, &content, &path);
    let mut all_events = vec![];
    for step in case["equipment"].as_array().unwrap() {
        let member = step["member"].as_u64().unwrap() as usize;
        let source = state.crate_items[step["crate_slot"].as_u64().unwrap() as usize].clone();
        for operation in 0..3 {
            let slot = state.warriors[member]
                .inventory()
                .unwrap()
                .iter()
                .position(|item| item.id() == 2);
            let command = match operation {
                0 => Command::TakeCrate {
                    slot: step["crate_slot"].as_u64().unwrap() as usize,
                    warrior: member,
                },
                1 => Command::EquipItem {
                    warrior: member,
                    slot: slot.unwrap(),
                },
                _ => Command::FitAmmunition {
                    warrior: member,
                    slot: slot.unwrap(),
                    ammunition: 201,
                },
            };
            let prior = state.clone();
            let events = apply(command.clone(), &mut state, &content).unwrap();
            assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
            assert_eq!(resumed, state);
            all_events.extend(events);
            let mut expected = prior;
            let snapshot = match operation {
                0 => &step["picked"],
                1 => &step["compacted"],
                _ => &step["fitted"],
            };
            project(&mut expected, snapshot);
            source_caches(&mut expected, &content, snapshot, operation == 2, false);
            if operation == 0 {
                expected.crate_items = bytes(&step["worn"]["crate"])
                    .as_chunks::<22>()
                    .0
                    .iter()
                    .map(|p| InventoryRecord(*p))
                    .collect();
            }
            assert_session(&expected, &state);
            resumed = checkpoint(&resumed, &content, &path);
        }
        let inventory = state.warriors[member].inventory().unwrap();
        assert_eq!(inventory.iter().filter(|item| item.id() == 2).count(), 1);
        assert_eq!(inventory[0].id(), source.id());
        assert_eq!(&inventory[0].0[13..], &source.0[13..]);
        assert!(
            inventory
                .iter()
                .any(|item| item.id() == 1 && item.0[9] == 0)
        );
    }
    assert_snapshot(&state, &case["after"]);
    assert_eq!(sounds(&all_events), native_sounds(&case["after"]));
    assert_eq!(
        all_events,
        native_sounds(&case["after"])
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id })
            .collect::<Vec<_>>()
    );
    assert_eq!(before.ammunition[1] - state.ammunition[1], 24);
    for member in [0, 1, 4, 5] {
        assert_eq!(state.warriors[member], before.warriors[member]);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn owned_shoot_matches_complete_native_initiative_and_resume() {
    let fixture = fixture_named("owned-player-firearm-target.json");
    let content = content();
    for (index, case) in fixture["fire_cases"].as_array().unwrap().iter().enumerate() {
        let mut state = input(&fixture, &case["combat_before"], true);
        state.validate(&content).unwrap();
        let before = state.clone();
        let path = std::env::temp_dir().join(format!(
            "centurions-owned-firearm-shot-{}-{index}.json",
            std::process::id()
        ));
        let mut resumed = checkpoint(&state, &content, &path);
        let events = apply(Command::ShootEnemy { body: 10 }, &mut state, &content).unwrap();
        assert_eq!(
            apply(Command::ShootEnemy { body: 10 }, &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(state, resumed);
        assert_snapshot(&state, &case["advanced"]);
        let mut expected = before;
        project(&mut expected, &case["advanced"]);
        source_caches(&mut expected, &content, &case["advanced"], true, true);
        expected.bandage_ready = Some(1);
        expected.health_context = Some(centurions_core::story_health::context(
            &expected.warriors,
            expected.selected_warrior,
            expected.health_context.as_ref(),
        ));
        assert_session(&expected, &state);
        let old = native_sounds(&case["combat_before"]).len();
        assert_eq!(sounds(&events), native_sounds(&case["advanced"])[old..]);
        let mut expected_events: Vec<_> = native_sounds(&case["advanced"])[old..]
            .iter()
            .map(|resource_id| Event::Sound {
                resource_id: *resource_id,
            })
            .collect();
        expected_events.push(Event::AttackResolved {
            attacker: case["member"].as_i64().unwrap() as i16,
            target: 10,
            damage: word(&bytes(&case["combat_before"]["target"]), 0x110)
                .unwrap()
                .wrapping_sub(word(&bytes(&case["advanced"]["target"]), 0x110).unwrap()),
            defeated: false,
        });
        expected_events.push(Event::TurnStarted { body: 10, round: 1 });
        assert_eq!(events, expected_events);
        assert!(matches!(
            events.last(),
            Some(Event::TurnStarted { body: 10, round: 1 })
        ));
        checkpoint(&state, &content, &path);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn owned_combat_fit_matches_complete_native_initiative_and_resume() {
    let fixture = fixture();
    let content = content();
    for (index, case) in fixture["ammunition_cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, c)| c["kind"] == "combat_fit_and_advance")
    {
        let mut state = input(&fixture, &case["before"], true);
        let before = state.clone();
        let path = std::env::temp_dir().join(format!(
            "centurions-owned-firearm-fit-{}-{index}.json",
            std::process::id()
        ));
        let mut resumed = checkpoint(&state, &content, &path);
        let command = Command::FitAmmunition {
            warrior: 2,
            slot: 0,
            ammunition: 201,
        };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert_eq!(state, resumed);
        assert_snapshot(&state, &case["after"]);
        let mut expected = before;
        project(&mut expected, &case["after"]);
        source_caches(&mut expected, &content, &case["after"], true, false);
        expected.health_context = Some(centurions_core::story_health::context(
            &expected.warriors,
            expected.selected_warrior,
            expected.health_context.as_ref(),
        ));
        expected.bandage_ready = Some(1);
        assert_session(&expected, &state);
        let old = native_sounds(&case["before"]).len();
        assert_eq!(sounds(&events), native_sounds(&case["after"])[old..]);
        let mut expected_events: Vec<_> = native_sounds(&case["after"])[old..]
            .iter()
            .map(|resource_id| Event::Sound {
                resource_id: *resource_id,
            })
            .collect();
        expected_events.push(Event::TurnStarted { body: 10, round: 1 });
        assert_eq!(events, expected_events);
        assert!(matches!(
            events.last(),
            Some(Event::TurnStarted { body: 10, round: 1 })
        ));
        checkpoint(&state, &content, &path);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn owned_manual_fills_match_native_stock_audio_and_saved_public_commands() {
    let fixture = fixture();
    let content = content();
    for (index, case) in fixture["ammunition_cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c["kind"].as_str(), Some("manual_full" | "manual_partial")))
    {
        let mut state = input(&fixture, &case["before"], false);
        let before = state.clone();
        let path = std::env::temp_dir().join(format!(
            "centurions-owned-firearm-refill-{}-{index}.json",
            std::process::id()
        ));
        let mut resumed = checkpoint(&state, &content, &path);
        let command = Command::FitAmmunition {
            warrior: 2,
            slot: 0,
            ammunition: 201,
        };
        let events = apply(command.clone(), &mut state, &content).unwrap();
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        assert_eq!(state, resumed);
        let mut expected = before;
        project(&mut expected, &case["after"]);
        source_caches(&mut expected, &content, &case["after"], true, false);
        assert_session(&expected, &state);
        let old = native_sounds(&case["before"]).len();
        assert_eq!(sounds(&events), native_sounds(&case["after"])[old..]);
        let expected_events: Vec<_> = native_sounds(&case["after"])[old..]
            .iter()
            .map(|resource_id| Event::Sound {
                resource_id: *resource_id,
            })
            .collect();
        assert_eq!(events, expected_events);
        checkpoint(&state, &content, &path);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn owned_invalid_admission_and_later_unsupported_turn_preserve_whole_session() {
    let fixture = fixture();
    let content = content();
    let before = input(&fixture, &fixture["equipment_cases"][0]["before"], false);
    let mut loaded = before.clone();
    loaded.crate_items[3].0[2..4].copy_from_slice(&5_i16.to_be_bytes());
    loaded.crate_items[3].0[4..6].copy_from_slice(&201_i16.to_be_bytes());
    assert_atomic(
        &loaded,
        &content,
        Command::TakeCrate {
            warrior: 2,
            slot: 3,
        },
    );
    let mut enhanced = before.clone();
    enhanced.crate_items[3].0[15] = 1;
    assert_atomic(
        &enhanced,
        &content,
        Command::TakeCrate {
            warrior: 2,
            slot: 3,
        },
    );
    let mut checked = 0;
    for case in fixture["admission_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| {
            matches!(
                c["kind"].as_str(),
                Some(
                    "items_wear_ap_zero"
                        | "items_fit_ap_zero"
                        | "wear_inactive"
                        | "items_fit_inactive"
                        | "fire_ap_zero"
                        | "fire_movement_zero"
                )
            )
        })
    {
        checked += 1;
        let state = input(&fixture, &case["before"], true);
        let command = match case["kind"].as_str().unwrap() {
            "items_wear_ap_zero" | "wear_inactive" => Command::EquipItem {
                warrior: 2,
                slot: 0,
            },
            "items_fit_ap_zero" | "items_fit_inactive" => Command::FitAmmunition {
                warrior: 2,
                slot: 0,
                ammunition: 201,
            },
            _ => Command::ShootEnemy { body: 10 },
        };
        assert_atomic(&state, &content, command);
    }
    assert_eq!(checked, 6);
    let fitted = input(&fixture, &fixture["fire_cases"][0]["combat_before"], true);
    let mut outside_range = fitted.clone();
    assert!(matches!(
        apply(Command::ShootEnemy { body: 10 }, &mut outside_range, &content),
        Err(RuleError::Invalid(message)) if message == "enemy is outside firearm range or line of sight"
    ));
    assert_eq!(outside_range, fitted);
    assert_atomic(
        &fitted,
        &content,
        Command::FitAmmunition {
            warrior: 2,
            slot: 0,
            ammunition: 219,
        },
    );
    let mut unverified_removal = fitted.clone();
    unverified_removal.warriors[2].bytes[0x132 + 14] = 1;
    assert_atomic(
        &unverified_removal,
        &content,
        Command::UnequipItem {
            warrior: 2,
            slot: 0,
        },
    );
    let target_fixture = fixture_named("owned-player-firearm-target.json");
    let mut late = input(
        &target_fixture,
        &target_fixture["fire_cases"][0]["combat_before"],
        true,
    );
    let combat = late.combat.as_mut().unwrap();
    combat.order.fill(-1);
    combat.order[109] = 2;
    combat.cursor = 109;
    late.warriors[4].bytes[0xa2..0xa4].copy_from_slice(&1_i16.to_be_bytes());
    let unchanged = late.clone();
    assert!(matches!(
        apply(Command::ShootEnemy { body: 10 }, &mut late, &content),
        Err(RuleError::Unsupported {
            action: "combat.round",
            dependency: "ongoing regeneration, damage or bleeding"
        })
    ));
    assert_eq!(late, unchanged);
}

#[test]
fn owned_capacity_and_inventory_count_rejections_match_native_records_and_audio() {
    let fixture = fixture();
    let content = content();
    let mut checked = 0;
    for case in fixture["admission_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| {
            matches!(
                c["kind"].as_str(),
                Some("pickup_capacity_zero" | "pickup_count_99")
            )
        })
    {
        checked += 1;
        let mut state = input(&fixture, &case["before"], false);
        let mut expected = state.clone();
        let ordinary = bytes(&fixture["equipment_cases"][0]["before"]["crate"]);
        let slot = state
            .crate_items
            .iter()
            .position(|item| item.0 == ordinary[66..88])
            .unwrap();
        let events = apply(
            Command::TakeCrate { warrior: 2, slot },
            &mut state,
            &content,
        )
        .unwrap();
        project(&mut expected, &case["after"]);
        source_caches(&mut expected, &content, &case["after"], false, false);
        assert_session(&expected, &state);
        let old = native_sounds(&case["before"]).len();
        assert_eq!(sounds(&events), native_sounds(&case["after"])[old..]);
        assert_eq!(
            events,
            vec![
                Event::Sound { resource_id: 241 },
                Event::CollectionRejected {
                    warning: if case["kind"] == "pickup_capacity_zero" {
                        19
                    } else {
                        147
                    },
                },
            ]
        );
    }
    assert_eq!(checked, 2);
}
