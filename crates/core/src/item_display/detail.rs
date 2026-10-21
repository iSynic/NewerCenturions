//! The quickinfo panel (0x132b0c): direct item calculations and drawing output.
//! File data is already loaded by content; no emulated I/O, stack or SANE calls.
use super::{BTreeMap, BodyRect, Content, InventoryRecord, NativeText, RenderingMemory, RuleError};
use crate::{
    recovered_detail_ui::{DetailEffect, DetailPlan},
    recovered_drawing::DrawingEffect,
    recovered_text::PascalString,
};
use centurions_content::{
    mission::OpeningMission,
    records::{InventoryModifier as Modifier, ItemDefinition, ItemPanelRecord},
    span,
};

mod drawing;
use drawing::Panel;

pub fn detail(
    content: &Content,
    mission: &OpeningMission,
    item: &InventoryRecord,
    stock: bool,
    native: &NativeText,
    strings: &BTreeMap<i16, Vec<u8>>,
) -> Result<DetailPlan, RuleError> {
    native.bytes(0x402900, 0x640)?;
    // Retain the original adapter's admission of all three records before drawing.
    for id in [0, item.id(), item.ammunition_id()?] {
        content
            .items
            .get(usize::from(id.unsigned_abs()))
            .ok_or("detail item unavailable")?;
    }
    let definition = crate::ammunition::definition(content, item.id())?;
    let base = ItemDefinition(&definition);
    let original = ItemPanelRecord(&definition);
    let spell = base.spell()?;
    if (1101..4000).contains(&spell) {
        let attacks = &mission
            .combat
            .as_ref()
            .ok_or("detail spell table unavailable")?
            .attacks;
        crate::combat::spells::load_packet(attacks, spell)?;
    }
    let id = usize::from(item.id().unsigned_abs());
    let description = if id < 800 {
        span(&mission.stock_item_descriptions, id * 256, 256)?
    } else {
        span(&mission.item_descriptions, (id - 800) * 256, 256)?
    };
    let has_ammunition = !stock && item.ammunition_id()? != 0;
    let ammo_bytes = crate::ammunition::definition(
        content,
        if has_ammunition {
            item.ammunition_id()?
        } else {
            0
        },
    )?;
    let ammunition = ItemDefinition(&ammo_bytes);
    let ammo_panel = ItemPanelRecord(&ammo_bytes);
    let modifier = |field: Modifier| i16::from(field.read(item));
    let (required_level, armor, shield, quantity, capacity, range, accuracy, melee_accuracy) =
        if stock {
            (
                original.required_level()?,
                original.armor()?,
                original.shield_chance()?,
                base.supplied_quantity()?,
                base.magazine_capacity()?,
                original.range()?,
                base.base_accuracy(),
                base.base_melee_accuracy(),
            )
        } else {
            (
                modifier(Modifier::RequiredLevel),
                original.armor()?.wrapping_add(modifier(Modifier::Armor)),
                original
                    .shield_chance()?
                    .wrapping_add(modifier(Modifier::ShieldChance)),
                item.quantity()?,
                item.capacity()?,
                original.range()?.wrapping_add(
                    modifier(Modifier::Range)
                        .wrapping_add(i16::from(ammo_panel.range_modifier()))
                        .wrapping_add(ammo_panel.range()?),
                ),
                (modifier(Modifier::Accuracy).wrapping_add(i16::from(ammunition.base_accuracy())))
                    as i8,
                Modifier::MeleeAccuracy.read(item),
            )
        };
    let mut panel = Panel::new(native, strings);
    panel.start(i16::from_be_bytes(
        native.bytes(0x402f3c, 2)?.try_into().unwrap(),
    ));
    for (vertical, value, plus) in [
        (479, required_level, false),
        (498, armor, false),
        (517, original.durability(), true),
        (536, shield, false),
        (555, quantity, false),
        (574, original.shield_cost()?, false),
    ] {
        panel.number_at(450, vertical, value, plus)?;
    }
    // quickinfo rounds the factor to Single, then multiplies in extended precision.
    let factor = (100_i32 + i32::from(item.damage_enhancement())) as f32 / 100.0;
    let mut damage = base.missile_damage()?.wrapping_add(base.melee_damage()?);
    if has_ammunition {
        damage = damage.wrapping_add(
            ammunition
                .missile_damage()?
                .wrapping_add(ammunition.melee_damage()?),
        );
    }
    let rounds = base.rounds_per_shot()?;
    if rounds != 0 {
        damage = damage.wrapping_mul(rounds);
    }
    for element in 0..7 {
        damage = damage.wrapping_add(
            ammunition
                .elemental_damage(element)?
                .wrapping_add(base.elemental_damage(element)?)
                .wrapping_mul(rounds),
        );
    }
    let damage = (f64::from(damage) * f64::from(factor)).trunc() as i32 as i16;
    panel.draw(DrawingEffect::ForeColor(205));
    panel.number_at(652, 479, damage.wrapping_mul(base.attack_count()?), false)?;
    panel.draw(DrawingEffect::ForeColor(69));
    panel.number_at(580, 479, damage, false)?;
    for (vertical, value, plus) in [
        (498, range, true),
        (517, rounds, false),
        (536, capacity, false),
        (555, base.attack_count()?, false),
        (
            574,
            i16::from(melee_accuracy).wrapping_add(i16::from(accuracy)),
            true,
        ),
    ] {
        panel.number_at(580, vertical, value, plus)?;
    }
    panel.draw(DrawingEffect::TextSize(10));
    panel.draw(DrawingEffect::TextFont(10));
    panel.draw(DrawingEffect::ForeColor(273));
    special_information(&mut panel, &base, &original)?;
    panel.draw(DrawingEffect::ForeColor(69));
    panel.draw(DrawingEffect::BackPixelPattern(0));
    panel.draw(DrawingEffect::ForeColor(69));
    let mut bytes = description[1..].to_vec();
    bytes.push(0);
    panel.text_box(
        0x479ed9,
        bytes,
        BodyRect {
            top: 473,
            left: 13,
            bottom: 534,
            right: 350,
        },
    );
    panel.move_to(168, 551);
    let label = if base.ammunition_caliber()? != 0 {
        if has_ammunition {
            panel.literal(0x402976)?;
            let selection = ammunition.description_id()?;
            let base = crate::rules::selection_base(selection);
            panel.indexed(
                base.wrapping_add(1),
                selection.wrapping_sub(base).wrapping_add(1),
            )?
        } else {
            panel.literal(0x402964)?;
            panel.indexed(142, base.ammunition_caliber()?)?
        }
    } else if base.reserve_ammunition_kind()? != 0 {
        panel.literal(0x40294e)?;
        panel.indexed(142, base.reserve_ammunition_kind()?)?
    } else if original.equipment_type() != 0 {
        panel.literal(0x402938)?;
        panel.indexed(-11000, i16::from(original.equipment_type()).wrapping_add(1))?
    } else {
        panel.literal(0x402922)?;
        panel.indexed(-11000, base.category()?.wrapping_add(1))?
    };
    panel.move_to(168, 571);
    panel.draw(DrawingEffect::DrawString(label));
    panel.icon(base.icon()?, 15);
    if has_ammunition {
        panel.icon(ammunition.icon()?, 65);
    }
    let mut animation = original.animation()?;
    if has_ammunition && ammo_panel.animation()? != 0 {
        animation = ammo_panel.animation()?;
    }
    if animation == 0 && original.target_mode()? != 0 {
        animation = 1;
    }
    if animation != 0 {
        panel.icon(animation.wrapping_add(9999), 115);
    }
    panel.finish();
    Ok(panel.plan)
}

fn special_information(
    panel: &mut Panel<'_>,
    item: &ItemDefinition<'_>,
    values: &ItemPanelRecord<'_>,
) -> Result<(), RuleError> {
    // Keep a local encoded text buffer because description_box changes it from
    // C to Pascal between branches. Unknown subselectors retain those bytes.
    let mut text = vec![0];
    let effect = item.effect_selector()?;
    let amount = item.spell()?;
    if effect > 19 {
        text = panel
            .indexed(136, effect.wrapping_sub(19))?
            .payload()
            .to_vec();
        text.push(0);
        panel.description(&mut text)?;
    }
    if effect == -10 {
        text = panel.native_c(0x402a3a)?;
        append(
            &mut text,
            panel.indexed(133, amount.wrapping_sub(19))?.payload(),
        );
        panel.description(&mut text)?;
    }
    if (1101..4000).contains(&amount) {
        text = panel.native_c(0x402a30)?;
        let tier = amount % 1000 / 100 - 1;
        let selection = amount % 100 - 1;
        append(
            &mut text,
            panel
                .indexed(
                    (amount / 1000).wrapping_mul(1000).wrapping_add(tier),
                    selection.wrapping_add(1),
                )?
                .payload(),
        );
        panel.description(&mut text)?;
    }
    if effect == 122 {
        if let Some(address) = match amount {
            1 => Some(0x402a12),
            2 => Some(0x4029f2),
            3 => Some(0x4029d2),
            4 => Some(0x4029b2),
            _ => None,
        } {
            text = panel.native_c(address)?;
        }
        panel.description(&mut text)?;
    }
    let (attributes, adjustment) = values.attribute_changes()?;
    for attribute in attributes {
        if (1..16).contains(&attribute) {
            text = panel.native_c(if adjustment > 0 { 0x4029b0 } else { 0x402d2d })?;
            let number = panel.convert_number(adjustment)?;
            append(&mut text, number.payload());
            append(&mut text, &panel.native_c(0x4029aa)?);
            append(&mut text, panel.indexed(1201, attribute)?.payload());
            panel.description(&mut text)?;
        }
    }
    Ok(())
}

fn append(text: &mut Vec<u8>, suffix: &[u8]) {
    text.truncate(
        text.iter()
            .position(|byte| *byte == 0)
            .unwrap_or(text.len()),
    );
    text.extend(suffix.iter().take_while(|byte| **byte != 0));
    text.push(0);
}

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../../tests/unit/detail_reference.rs"]
mod reference;
