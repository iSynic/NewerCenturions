use super::*;

/// Read-only projection of updateodds161068. Scratch writes never enter the session.
fn reference_attack_estimate(
    state: &crate::Session,
    content: &Content,
    body: i16,
    native: &NativeText,
    strings: &BTreeMap<i16, Vec<u8>>,
) -> Result<DetailPlan, RuleError> {
    const FRAME: u32 = 0x700200;
    let combat = state.combat.as_ref().ok_or("estimate combat missing")?;
    let actor = combat
        .active_body()
        .filter(|body| (0..state.warriors.len() as i16).contains(body))
        .ok_or("estimate warrior missing")?;
    let mut memory = RenderingMemory::default();
    for (address, length) in [
        (0x41f714, 2),
        (0x439598, 4),
        (0x4434b6, 4),
        (0x45363c, 450),
        (0x479ed8, 256),
        (0x700000, 1024),
    ] {
        memory.insert_region(address, &vec![0; length]);
    }
    memory.insert_region(0x402f3c, native.bytes(0x402f3c, 2)?);
    memory.insert_region(0x442576, &combat.difficulty.to_be_bytes());
    memory.insert_region(0x4435d2, &actor.to_be_bytes());
    memory.insert_region(
        FRAME + 8,
        &(if body >= 10 { body - 10 } else { -1 }).to_be_bytes(),
    );
    let warrior = &state.warriors[actor as usize];
    memory.insert_region(0x448aa0 + actor as u32 * 3138, &warrior.bytes);
    let equipment = if warrior.bytes[0x9dd] == 0 {
        0x12e
    } else {
        0x12c
    };
    let equipped = word(&warrior.bytes, equipment)?;
    if equipped != 0
        && !warrior
            .inventory()?
            .iter()
            .any(|item| item.id() == equipped && item.0[9] != 0)
    {
        return Err("estimate requires the original equipped-instance multiplier".into());
    }
    if warrior.bytes[0x9dd] != 0 && equipped == 0 {
        return Err("estimate requires the original retained firearm damage context".into());
    }
    if body >= 10 {
        let enemy = combat
            .enemies
            .records
            .get((body - 10) as usize)
            .ok_or("estimate target missing")?;
        memory.insert_region(0x439d3c + (body - 10) as u32 * 288, &enemy.bytes);
    }
    for id in [
        warrior.record().equipped_weapon()?,
        word(&warrior.bytes, 0xca)?,
        word(&warrior.bytes, 0x12c)?,
        word(&warrior.bytes, 0x12e)?,
        word(&warrior.bytes, 0x126)?,
    ] {
        let index = id.unsigned_abs() as usize;
        if index / 200 >= ITEM_TABLE_ADDRESSES.len() {
            return Err("estimate item family unavailable".into());
        }
        let definition = crate::ammunition::definition(content, id)?;
        memory.insert_region(
            ITEM_TABLE_ADDRESSES[index / 200] + (index % 200 * 150) as u32,
            &definition,
        );
    }
    for (address, bytes) in [
        (0x45363c, &state.ammunition_work.item),
        (0x4536d2, &state.ammunition_work.ammo),
        (0x453768, &state.ammunition_work.weapon),
    ] {
        if let Some(bytes) = bytes {
            memory.insert_region(address, bytes);
        }
    }
    crate::recovered_detail_ui::update_odds(
        &memory,
        FRAME,
        crate::recovered_attack::DamageFrame {
            unmatched_multiplier_single_bits: 1_f32.to_bits(),
        },
        &mut Services {
            strings,
            description: &[],
        },
    )
}

fn visible(plan: &DetailPlan) -> Vec<crate::recovered_detail_ui::DetailEffect> {
    use crate::{recovered_detail_ui::DetailEffect as E, recovered_drawing::DrawingEffect as D};
    plan.effects
        .iter()
        .filter(|effect| {
            matches!(
                effect,
                E::Drawing(
                    D::ForeColor(_)
                        | D::TextSize(_)
                        | D::TextFont(_)
                        | D::MoveTo { .. }
                        | D::DrawString(_)
                        | D::DrawPicture { .. }
                        | D::PlotColorIcon { .. }
                ) | E::TextFaceSlot(_)
                    | E::NumberToString { .. }
                    | E::TextBox { .. }
            )
        })
        .cloned()
        .collect()
}

#[test]
fn attack_estimate_preserves_displayed_odds_and_damage() {
    // Reconstruction equivalence, not an additional native receipt. Exercise the
    // public desktop adapter with earned state and controlled branch variations.
    let content = crate::test_support::content();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let native = NativeText::load(&content, &root).unwrap();
    let fixture = crate::test_support::fixture("earned-combat-reload.json");
    let strings = BTreeMap::new();
    let mut displayed = [0; 2];
    for foundation in fixture["foundations"].as_array().unwrap() {
        let initial: crate::Session =
            serde_json::from_value(foundation["session"].clone()).unwrap();
        for firing in [false, true] {
            for difficulty in -2..=2 {
                for target in [-1, 10] {
                    let mut state = initial.clone();
                    let combat = state.combat.as_mut().unwrap();
                    combat.difficulty = difficulty;
                    let actor = combat.active_body().unwrap() as usize;
                    state.warriors[actor].bytes[0x9dd] = u8::from(firing);
                    let before = serde_json::to_value(&state).unwrap();
                    let expected =
                        reference_attack_estimate(&state, &content, target, &native, &strings);
                    let actual = crate::item_display::attack_estimate(
                        &state, &content, target, &native, &strings,
                    );
                    match (expected, actual) {
                        (Ok(expected), Ok(actual)) => {
                            if target >= 10 {
                                displayed[usize::from(firing)] += 1;
                            }
                            assert_eq!(
                                visible(&actual),
                                visible(&expected),
                                "firing={firing} difficulty={difficulty} target={target}"
                            );
                            assert_eq!(
                                actual.arithmetic_qualification,
                                expected.arithmetic_qualification
                            );
                            for base in [0x45363c, 0x4536d2, 0x453768] {
                                for offset in 0..150 {
                                    assert_eq!(
                                        actual.memory.byte(base + offset).unwrap(),
                                        expected.memory.byte(base + offset).unwrap()
                                    );
                                }
                            }
                        }
                        (Err(expected), Err(actual)) => {
                            assert_eq!(actual.to_string(), expected.to_string())
                        }
                        (expected, actual) => panic!("expected {expected:?}, actual {actual:?}"),
                    }
                    assert_eq!(serde_json::to_value(&state).unwrap(), before);
                }
            }
        }
    }
    assert!(
        displayed.iter().all(|count| *count > 0),
        "both modes must produce estimates: {displayed:?}"
    );
}

#[test]
fn item_panels_preserve_numeric_fields_descriptions_and_icons() {
    let content = crate::test_support::content();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let native = NativeText::load(&content, &root).unwrap();
    let strings = content
        .resources
        .iter()
        .filter(|r| r.r#type == "STR#")
        .map(|r| {
            (
                r.id,
                centurions_content::read_resource(&root.join("analysis/extracted"), r).unwrap(),
            )
        })
        .collect();
    let fixture = crate::test_support::fixture("inventory-details.json");
    let mut samples: Vec<(InventoryRecord, bool)> = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| {
            let raw = case["record"].as_str().unwrap();
            let bytes: Vec<u8> = (0..raw.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&raw[i..i + 2], 16).unwrap())
                .collect();
            (
                InventoryRecord(bytes.try_into().unwrap()),
                case["stock"].as_bool().unwrap(),
            )
        })
        .collect();
    // One supplied item per distinct special-description branch, beyond the
    // independent numeric receipts. This is reconstruction equivalence only.
    let mut branches = std::collections::BTreeSet::new();
    for bytes in &content.items {
        let definition = centurions_content::records::ItemDefinition(bytes);
        let effect = definition.effect_selector().unwrap();
        let amount = definition.spell().unwrap();
        let key = (
            effect,
            (1101..4000).contains(&amount),
            effect == 122 && (1..=4).contains(&amount),
        );
        if (effect > 19 || effect == -10 || key.1) && branches.insert(key) {
            samples.push((definition.fresh_stock().unwrap(), false));
        }
    }
    for (item, stock) in samples {
        let visible_row = |effects: Vec<crate::recovered_item_ui::ItemUiEffect>| {
            effects
                .into_iter()
                .filter(|effect| {
                    !matches!(effect, crate::recovered_item_ui::ItemUiEffect::Write(_))
                })
                .collect::<Vec<_>>()
        };
        let expected_row = reference_row(&content, &item, &native, &strings).unwrap();
        let actual_row = crate::item_display::row(&content, &item, &native, &strings).unwrap();
        assert_eq!(
            visible_row(actual_row),
            visible_row(expected_row),
            "row {}",
            item.id()
        );
        let mission = content.map(5).unwrap();
        let expected = reference_detail(&content, mission, &item, stock, &native, &strings);
        let actual =
            crate::item_display::detail(&content, mission, &item, stock, &native, &strings);
        match (expected, actual) {
            (Ok(expected), Ok(actual)) => assert_eq!(
                visible(&actual),
                visible(&expected),
                "item={} stock={stock}",
                item.id()
            ),
            (Err(expected), Err(actual)) => assert_eq!(actual.to_string(), expected.to_string()),
            (expected, actual) => panic!(
                "item={} stock={stock}: expected {expected:?}, actual {actual:?}",
                item.id()
            ),
        }
    }
}
