//! Original item-panel and encounter-return comparisons, shared with the desktop workflow.
use super::*;

pub(super) fn compare_previews(
    content: &Content,
    state: &Session,
    root: &std::path::Path,
    receipt: &serde_json::Value,
) {
    for case in receipt["instances"].as_array().unwrap() {
        assert_eq!(
            centurions_core::shop::preview(content, case["item"].as_i64().unwrap() as i16)
                .unwrap()
                .0
                .to_vec(),
            decode(case["record"].as_str().unwrap())
        );
    }
    let native = centurions_content::native_text::NativeText::load(
        content,
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mut strings = std::collections::BTreeMap::new();
    for resource in content.resources.iter().filter(|r| r.r#type == "STR#") {
        strings.insert(
            resource.id,
            centurions_content::read_resource(&root.join("analysis/extracted"), resource).unwrap(),
        );
    }
    for warrior in &state.warriors {
        for item in warrior.inventory().unwrap() {
            centurions_core::item_display::row(content, &item, &native, &strings).unwrap();
            centurions_core::item_display::detail(
                content,
                content.map(5).unwrap(),
                &item,
                false,
                &native,
                &strings,
            )
            .unwrap();
        }
    }
    for case in receipt["instances"].as_array().unwrap() {
        let item =
            centurions_core::shop::preview(content, case["item"].as_i64().unwrap() as i16).unwrap();
        centurions_core::item_display::detail(
            content,
            content.map(5).unwrap(),
            &item,
            true,
            &native,
            &strings,
        )
        .unwrap();
    }
    let panels = native_fixture("inventory-details.json");
    for case in panels["cases"].as_array().unwrap() {
        let item = centurions_content::InventoryRecord(
            decode(case["record"].as_str().unwrap()).try_into().unwrap(),
        );
        let plan = centurions_core::item_display::detail(
            content,
            content.map(5).unwrap(),
            &item,
            case["stock"].as_bool().unwrap(),
            &native,
            &strings,
        )
        .unwrap();
        let mut position = (0, 0);
        let mut fields = vec![];
        for effect in plan.effects {
            use centurions_core::item_display::output::{Draw, PanelEffect};
            match effect {
                PanelEffect::Drawing(Draw::MoveTo {
                    horizontal,
                    vertical,
                }) => position = (horizontal, vertical),
                PanelEffect::NumberToString { value, .. } if fields.len() < 13 => {
                    fields.push(serde_json::json!([position.0, position.1, value]))
                }
                _ => {}
            }
        }
        assert_eq!(serde_json::Value::Array(fields), case["fields"]);
    }
}

pub(super) fn compare_encounter_returns(
    app: &mut Application,
    ui: &mut UiState,
    initial: &Session,
    content: &Content,
    path: &std::path::Path,
) {
    let initial = initial.clone();
    for (name, index, regional) in [
        ("encounter-inert-items.json", 1),
        ("encounter-script-items.json", 0),
        ("encounter-script-items.json", 2),
        ("item-modifiers.json", 2),
        ("item-modifiers.json", 4),
        ("encounter-medical.json", 2),
        ("walking-encounter-item.json", 0),
        ("calendar-encounter-item.json", 0),
        ("outdoor-indexed-items.json", 1),
    ]
    .into_iter()
    .map(|(name, index)| (name, index, false))
    .chain(std::iter::once(("calendar-encounter-item.json", 0, true)))
    {
        let mut restored = initial.clone();
        ui.feedback.clear();
        let mut content = content.clone();
        let encounter = native_fixture(name);
        let case = &encounter["cases"][index];
        let walking = case["caller"] == "walking";
        let calendar = case["caller"] == "calendar" && !regional;
        let global = case["global"] == true;
        if let Some(definition) = case.get("definition") {
            content.items[case["item"].as_u64().unwrap() as usize] =
                decode(definition.as_str().unwrap());
        }
        if case.get("active_body").is_some() {
            let combat: serde_json::Value =
                crate::original_content::fixture("earned-combat-reload.json");
            restored = serde_json::from_value(combat["foundations"][0]["session"].clone()).unwrap();
            let battle = restored.combat.as_mut().unwrap();
            battle.order[battle.cursor] = 2;
        }
        let native_party = case["before"]
            .as_str()
            .unwrap_or_else(|| case["before"]["party"].as_str().unwrap());
        restored.warriors = decode(native_party)
            .chunks(3138)
            .enumerate()
            .map(|(i, bytes)| {
                let mut warrior = restored.warriors[i].clone();
                warrior.bytes = bytes.to_vec();
                warrior
            })
            .collect();
        if case["before"].get("seed").is_some() {
            restored.random = centurions_core::RandomState {
                seed: case["before"]["seed"].as_u64().unwrap() as u32,
                draws: case["before"]["draws"].as_u64().unwrap(),
            };
        }
        if let Some(tail) = case["before"].get("scenario_duration_tail") {
            restored.scenario_duration_tail = Some(tail.as_i64().unwrap() as i16);
        }
        let position = restored.exploration.as_mut().unwrap();
        let mut record = vec![0; 42];
        record[8..10].copy_from_slice(&5_i16.to_be_bytes());
        record[24..26].copy_from_slice(&8_i16.to_be_bytes());
        position.script = Some(centurions_core::ScriptState {
            item_return: name == "encounter-script-items.json",
            scheduled: !walking,
            origin: walking.then_some(centurions_core::ScriptOrigin {
                map: position.map,
                x: position.x,
                y: position.y,
                level: restored.scenario_level,
            }),
            ..centurions_core::ScriptState::scheduled(record, 0)
        });
        position.presentation = Some(centurions_core::Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        if calendar {
            position.day = 2;
            position.hour = 0;
            position.minute = 0;
            restored.time_continuation = Some(centurions_core::TimeContinuation {
                remaining_ticks: 0,
                minutes_per_tick: 5,
                schedule_cursor: position.schedule.len() / 40,
            });
        }
        if regional {
            position.last_move = [1, -1];
            position.floor.fill(0);
            position.event_headers.fill(0);
            restored.movement_continuation =
                Some(centurions_core::MovementContinuation::RegionalReturn { header: true });
        }
        if global {
            position.script = None;
            position.presentation = None;
        } else {
            apply(Command::Continue, &mut restored, &content).unwrap();
        }
        if case["encounter_active"] == false {
            let position = restored.exploration.as_mut().unwrap();
            position.script = None;
            position.presentation = None;
        }
        ui.inspection = Some(crate::presentation::Inspection::Items);
        centurions_core::save(path, &restored, &content).unwrap();
        restored = centurions_core::load(path, &content).unwrap();
        let before = restored.clone();
        ui.inventory.open(
            case.get("actor")
                .map_or(2, |v| v.as_u64().unwrap() as usize),
            restored.warriors.len(),
        );
        assert!(matches!(
            ui.resolve_inventory(
                Some(Action::Inventory(crate::inventory::InventoryAction::Done)),
                &restored,
                &content,
                30.
            )
            .unwrap(),
            Some(Action::Close)
        ));
        assert_eq!(restored, before);
        for _ in 0..2 {
            let action = crate::inventory::InventoryAction::Item {
                side: 0,
                slot: case["slot"].as_u64().unwrap() as usize,
            };
            if let Some(Action::Command(command)) = ui
                .resolve_inventory(Some(Action::Inventory(action)), &restored, &content, 31.)
                .unwrap()
            {
                let inspecting = matches!(command, Command::InspectInventory { .. });
                match apply(app.command(command), &mut restored, &content) {
                    Ok(events) => {
                        if !inspecting {
                            ui.medical_result(&restored, &events);
                        }
                        ui.events(&events);
                    }
                    Err(error) => {
                        assert!(case.get("warning").is_some());
                        ui.failure(error);
                    }
                }
            }
        }
        if name == "encounter-medical.json" {
            assert!(restored.outdoor_psi.is_some());
            assert!(ui.inspection.is_none());
            centurions_core::save(path, &restored, &content).unwrap();
            restored = centurions_core::load(path, &content).unwrap();
            let events = apply(
                app.command(Command::CancelOutdoorPsi),
                &mut restored,
                &content,
            )
            .unwrap();
            ui.medical_result(&restored, &events);
            ui.events(&events);
            assert!(restored.outdoor_psi.is_none());
        }
        if case.get("warning").is_some() {
            assert_eq!(
                ui.feedback.back(),
                Some(&centurions_core::feedback::NativeFeedback::Warning {
                    index: case["warning"].as_i64().unwrap() as i16
                })
            );
            assert!(ui.technical_error.is_none());
            assert!(ui.inspection == Some(crate::presentation::Inspection::Items));
            assert!(matches!(
                restored.exploration.as_ref().unwrap().presentation,
                Some(centurions_core::Presentation::Typed { .. })
            ));
        } else if case.get("dialog_open").is_some() {
            assert!(ui.inspection == Some(crate::presentation::Inspection::Items));
            assert!(ui.inventory.selected.is_none());
            assert!(
                restored
                    .exploration
                    .as_ref()
                    .unwrap()
                    .presentation
                    .is_none()
            );
        } else if walking || calendar || global || regional {
            let position = restored.exploration.as_ref().unwrap();
            assert!(position.presentation.is_none() && position.script.is_none());
            assert!(ui.inspection.is_none());
            assert!(restored.time_continuation.is_none());
            if regional {
                assert_eq!(position.last_move, [0; 2]);
                assert!(restored.movement_continuation.is_none());
            }
            if global {
                assert_eq!(
                    restored.scenario_duration_tail,
                    Some(case["after"]["scenario_duration_tail"].as_i64().unwrap() as i16)
                );
            }
        } else {
            let expected = if name == "encounter-script-items.json" {
                851
            } else {
                950
            };
            assert!(
                matches!(restored.exploration.as_ref().unwrap().presentation, Some(centurions_core::Presentation::Dialogue { id, .. }) if id == expected)
            );
        }
        assert_eq!(
            restored
                .warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            decode(
                case["after"]
                    .as_str()
                    .unwrap_or_else(|| case["after"]["party"].as_str().unwrap())
            )
        );
        assert_eq!(restored.ammunition, before.ammunition);
        let expected_random = if case["before"].get("seed").is_some() {
            centurions_core::RandomState {
                seed: case["after"]["seed"].as_u64().unwrap() as u32,
                draws: case["after"]["draws"].as_u64().unwrap(),
            }
        } else {
            before.random
        };
        assert_eq!(restored.random, expected_random);
        if global {
            centurions_core::save(path, &restored, &content).unwrap();
            assert_eq!(centurions_core::load(path, &content).unwrap(), restored);
        }
    }
}

pub(super) fn compare_special_purchases(
    content: &Content,
    state: &Session,
    path: &std::path::Path,
) {
    let special = native_fixture("special-shop-purchase.json");
    let generated = native_fixture("special-shop.json");
    for case in special["cases"].as_array().unwrap() {
        let mut content = content.clone();
        let map_index = state.exploration.as_ref().unwrap().map as usize;
        let map = &mut content.scenario.as_mut().unwrap().maps[map_index];
        std::sync::Arc::make_mut(&mut map.shared)
            .combat
            .as_mut()
            .unwrap()
            .encounters[3650..3660]
            .copy_from_slice(&[255, 255, 0, 1, 0, 2, 0, 0, 0, 0]);
        let script = map.script(191).unwrap().to_vec();
        let mut probe = state.clone();
        probe.exploration.as_mut().unwrap().presentation =
            Some(centurions_core::Presentation::Dialogue {
                id: 740,
                text: String::new(),
            });
        probe.exploration.as_mut().unwrap().script = Some(story_frame(script, 1, 0));
        apply(Command::Continue, &mut probe, &content).unwrap();
        let id = case["item"].as_i64().unwrap() as i16;
        probe.warriors[0] =
            centurions_content::Warrior::read(&decode(case["before"].as_str().unwrap())).unwrap();
        probe.shops.get_mut(&probe.scenario_level).unwrap()[3002..6004]
            .copy_from_slice(&decode(case["stock_before"].as_str().unwrap()));
        // The restricted moveicon receipt declares current work, after loadshop.
        probe.native_work.shop_work = serde_json::from_value(serde_json::json!({
            "bytes": decode(case["stock_before"].as_str().unwrap())
        }))
        .unwrap();
        let stock = centurions_core::shop::stock(&probe)
            .unwrap()
            .into_iter()
            .find(|(_, item, _)| *item == id)
            .unwrap()
            .0;
        assert_eq!(
            centurions_core::shop::price(&probe, &content, id, false).unwrap(),
            case["price"].as_i64().unwrap() as i32
        );
        probe.party_currency = 900000;
        probe.random = centurions_core::RandomState::new(12);
        centurions_core::save(path, &probe, &content).unwrap();
        probe = centurions_core::load(path, &content).unwrap();
        let before = probe.clone();
        assert!(
            matches!(apply(Command::SellItem { warrior: 0, slot: 0 }, &mut probe, &content), Err(centurions_core::RuleError::Refused { feedback }) if feedback == vec![centurions_core::feedback::NativeFeedback::Warning { index: generated["refusals"][0]["warning"].as_i64().unwrap() as i16 }])
        );
        assert_eq!(probe, before);
        apply(Command::BuyItem { warrior: 0, stock }, &mut probe, &content).unwrap();
        let reference = generated["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["item"] == case["item"])
            .unwrap();
        assert_eq!(
            probe.warriors[0]
                .inventory()
                .unwrap()
                .into_iter()
                .find(|record| record.id() == id)
                .unwrap()
                .0
                .as_slice(),
            &decode(reference["record"].as_str().unwrap())
        );
        assert_eq!(
            probe.random.seed,
            reference["seed_after"].as_u64().unwrap() as u32
        );
        assert_eq!(probe.random.draws, reference["rng_draws"].as_u64().unwrap());
        assert_eq!(
            probe.warriors[0].bytes,
            decode(case["after"].as_str().unwrap())
        );
        // moveicon changes current work; this receipt stops before saveshop.
        assert_eq!(
            self::stock(&probe),
            decode(case["stock_after"].as_str().unwrap())
        );
        assert_eq!(probe.random.seed, case["seed"].as_u64().unwrap() as u32);
        assert_eq!(probe.random.draws, case["draws"].as_u64().unwrap());
        assert_eq!(
            probe.party_currency,
            case["currency"].as_i64().unwrap() as i32
        );
        assert_eq!(
            probe.shop_access.as_ref().unwrap().remaining,
            case["remaining"].as_i64().unwrap() as i16
        );
        let before = probe.clone();
        assert!(
            matches!(apply(Command::BuyItem { warrior: 0, stock }, &mut probe, &content), Err(centurions_core::RuleError::Refused { feedback }) if feedback == vec![centurions_core::feedback::NativeFeedback::Warning { index: generated["refusals"][1]["warning"].as_i64().unwrap() as i16 }])
        );
        assert_eq!(probe, before);
        centurions_core::save(path, &probe, &content).unwrap();
        assert_eq!(centurions_core::load(path, &content).unwrap(), probe);
        apply(Command::CloseShop, &mut probe, &content).unwrap();
        assert!(matches!(
            probe.exploration.as_ref().unwrap().presentation,
            Some(centurions_core::Presentation::Dialogue { id: 741, .. })
        ));
    }
}

pub(super) fn information_entry(actual: &Session, before: &Session, member: usize) {
    let mut expected = before.clone();
    // viewcharacter 163a54..163a7c copies the displayed record into character work.
    expected.native_work.character_bare_hand_bonus =
        before.warriors[member].record().bare_hand_bonus().unwrap();
    assert_eq!(*actual, expected);
}

pub(super) fn compare_character_fields(content: &Content, state: &Session, root: &std::path::Path) {
    let extracted = root.join("analysis/extracted");
    let native = centurions_content::native_text::NativeText::load(content, &extracted).unwrap();
    let strings = content
        .resources
        .iter()
        .filter(|r| {
            r.original.ends_with("/Cargo Pod")
                && r.r#type == "STR#"
                && matches!(r.id, 129 | 131 | 132 | 133)
        })
        .map(|r| {
            (
                r.id,
                centurions_content::read_resource(&extracted, r).unwrap(),
            )
        })
        .collect();
    let fields =
        centurions_core::character_display::information(state, 0, &native, &strings).unwrap();
    assert_eq!(fields[&67].0, state.warriors[0].name);
    for (reference, detailed) in [
        (
            include_str!("../../../../evidence/fixtures/warrior-information.json"),
            false,
        ),
        (
            include_str!("../../../../evidence/fixtures/warrior-statistics.json"),
            true,
        ),
    ] {
        let reference: serde_json::Value = serde_json::from_str(reference).unwrap();
        for case in reference["cases"].as_array().unwrap() {
            let bytes = decode(case["before"].as_str().unwrap());
            let mut sample = Session::new(content);
            sample.warriors = vec![centurions_content::Warrior::read(&bytes).unwrap()];
            let mut fields = if detailed {
                centurions_core::character_display::statistics(&sample, 0).unwrap()
            } else {
                centurions_core::character_display::information(&sample, 0, &native, &strings)
                    .unwrap()
            };
            if detailed {
                fields.remove(&37);
            }
            assert_eq!(
                serde_json::to_value(fields).unwrap(),
                case["fields"],
                "{}",
                case["case"]
            );
        }
    }
}

pub(super) fn stock(state: &Session) -> Vec<u8> {
    serde_json::from_value(
        serde_json::to_value(&state.native_work.shop_work).unwrap()["bytes"].clone(),
    )
    .unwrap()
}

pub(super) fn compare_shop_stock(
    content: &Content,
    state: &mut Session,
    receipt: &serde_json::Value,
    source: &[u8],
) {
    // Independent cases and the main state's next script use the receipt's cold DATA work.
    state.native_work.shop_work = serde_json::from_value(serde_json::json!({
        "bytes": vec![0u8; 3002]
    }))
    .unwrap();
    for case in receipt["shops"].as_array().unwrap().iter().skip(1) {
        let mut probe = state.clone();
        let mut record = source.to_vec();
        record[26..28].copy_from_slice(&(-(case["shop"].as_i64().unwrap() as i16)).to_be_bytes());
        probe.exploration.as_mut().unwrap().script = Some(story_frame(record, 1, 92));
        probe.exploration.as_mut().unwrap().presentation =
            Some(centurions_core::Presentation::Dialogue {
                id: 919,
                text: String::new(),
            });
        apply(Command::Continue, &mut probe, content).unwrap();
        assert_eq!(stock(&probe), decode(case["stock"].as_str().unwrap()));
    }
}

/// Every original DATA address and expected byte comparison is retained.
pub(super) fn compare_editor_data(content: &Content, root: &std::path::Path) {
    let text = centurions_content::native_text::NativeText::load(
        content,
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let original = std::fs::read(root.join("analysis/ghidra/startup/a5-mapped.bin")).unwrap();
    for (address, pascal) in [
        (0x403658, true),
        (0x403670, true),
        (0x40361a, false),
        (0x4035fa, false),
        (0x4035be, false),
        (0x403686, false),
    ] {
        let slice = &original[(address - 0x3fe210) as usize..];
        let bytes = if pascal {
            &slice[1..1 + usize::from(slice[0])]
        } else {
            &slice[..slice.iter().position(|b| *b == 0).unwrap()]
        };
        assert_eq!(text.text(address, pascal).unwrap().as_bytes(), bytes);
    }
}
