//! Original combat estimate panel, expressed as drawing commands rather than
//! emulated stack and global-memory writes. No session state or RNG is changed.
use super::{BTreeMap, BodyRect, Content, NativeText, RenderingMemory, RuleError};
use crate::{
    recovered_detail_ui::{DetailEffect, DetailPlan},
    recovered_drawing::DrawingEffect,
    recovered_economy::ArithmeticQualification,
    recovered_text::PascalString,
};
use centurions_content::records::ItemDefinition;

pub fn attack_estimate(
    state: &crate::Session,
    content: &Content,
    body: i16,
    native: &NativeText,
    _strings: &BTreeMap<i16, Vec<u8>>,
) -> Result<DetailPlan, RuleError> {
    let combat = state.combat.as_ref().ok_or("estimate combat missing")?;
    let actor = combat
        .active_body()
        .filter(|body| (0..state.warriors.len() as i16).contains(body))
        .ok_or("estimate warrior missing")?;
    let font = i16::from_be_bytes(native.bytes(0x402f3c, 2)?.try_into().unwrap());
    let warrior = &state.warriors[actor as usize];
    let record = warrior.record();
    let equipped = record.displayed_weapon(record.firing_mode())?;
    if equipped != 0
        && !warrior
            .inventory()?
            .iter()
            .any(|item| item.id() == equipped && item.equipment_flag() != 0)
    {
        return Err("estimate requires the original equipped-instance multiplier".into());
    }
    if record.firing_mode() && equipped == 0 {
        return Err("estimate requires the original retained firearm damage context".into());
    }
    let target = if body >= 10 {
        Some(
            combat
                .enemies
                .records
                .get((body - 10) as usize)
                .ok_or("estimate target missing")?,
        )
    } else {
        None
    };
    // Keep the existing admission checks even when the panel has no target.
    for id in [
        record.equipped_weapon()?,
        record.melee_weapon()?,
        record.displayed_weapon(true)?,
        record.displayed_weapon(false)?,
        record.loaded_ammunition()?,
    ] {
        if id.unsigned_abs() / 200 >= 5 {
            return Err("estimate item family unavailable".into());
        }
        crate::ammunition::definition(content, id)?;
    }
    let mut plan = DetailPlan {
        memory: RenderingMemory::default(),
        effects: vec![],
        native_return: None,
        arithmetic_qualification: None,
    };
    let panel = BodyRect {
        top: 300,
        left: 480,
        bottom: 408,
        right: 800,
    };
    draw(
        &mut plan,
        DrawingEffect::DrawPicture {
            handle: if target.is_some() { 201 } else { 224 },
            bounds: panel,
        },
    );
    let Some(target) = target else {
        retain_definition_receipt(&mut plan, &state.ammunition_work);
        return Ok(plan);
    };
    let enemy = target.record();
    let mut icon_bounds = BodyRect {
        top: 309,
        left: 487,
        bottom: 373,
        right: 551,
    };
    match enemy.size()? {
        0 => inset(&mut icon_bounds, 16, 16),
        1 => inset(&mut icon_bounds, 16, 0),
        2 => inset(&mut icon_bounds, 0, 16),
        _ => (),
    }
    let mut icon = enemy.tactical_icon()?;
    if enemy.size()? != -3 && enemy.facing_flag() == 1 {
        icon = icon.wrapping_add(1000);
    }
    draw(
        &mut plan,
        DrawingEffect::PlotColorIcon {
            handle: u32::from(icon as u16),
            rect: icon_bounds,
        },
    );
    let weapon = if record.firing_mode() {
        Some(record.equipped_weapon()?)
    } else if record.melee_weapon()? != 0 {
        Some(record.melee_weapon()?)
    } else {
        None
    };
    let mut work = state.ammunition_work.clone();
    let icon = match weapon {
        Some(id) => {
            let definition = crate::ammunition::definition(content, id)?;
            let icon = ItemDefinition(&definition).icon()?;
            work.item = Some(definition);
            icon
        }
        None => 7002,
    };
    draw(
        &mut plan,
        DrawingEffect::PlotColorIcon {
            handle: u32::from(icon as u16),
            rect: BodyRect {
                top: 375,
                left: 502,
                bottom: 407,
                right: 534,
            },
        },
    );
    let estimate = crate::combat::estimates::calculate(
        warrior,
        &target.bytes,
        combat.difficulty,
        &work,
        content,
    )?;
    draw(&mut plan, DrawingEffect::TextSize(12));
    draw(&mut plan, DrawingEffect::ForeColor(30));
    draw(&mut plan, DrawingEffect::TextFont(font));
    plan.effects.push(DetailEffect::TextFaceSlot(0));
    for (horizontal, vertical, value) in [
        (685, 337, estimate.chance),
        (650, 375, estimate.minimum),
        (730, 375, estimate.maximum),
        (650, 394, estimate.average),
    ] {
        draw(
            &mut plan,
            DrawingEffect::MoveTo {
                horizontal,
                vertical,
            },
        );
        let text = PascalString::from_payload(value.to_string().as_bytes())?;
        plan.effects.push(DetailEffect::NumberToString {
            value: i32::from(value),
            destination: 0x479ed8,
            result: text.clone(),
        });
        draw(&mut plan, DrawingEffect::DrawString(text));
    }
    retain_definition_receipt(&mut plan, &estimate.work);
    plan.arithmetic_qualification = Some(ArithmeticQualification::SourceDerivedBinary32);
    Ok(plan)
}

fn draw(plan: &mut DetailPlan, drawing: DrawingEffect) {
    plan.effects.push(DetailEffect::Drawing(drawing));
}

fn inset(rect: &mut BodyRect, horizontal: i16, vertical: i16) {
    rect.top += vertical;
    rect.bottom -= vertical;
    rect.left += horizontal;
    rect.right -= horizontal;
}

// Legacy DetailPlan consumers and native receipts identify these three byte
// records by their original addresses. This is an output codec only: no rule
// reads this map, and no stack, CPU registers or SANE operations are executed.
fn retain_definition_receipt(plan: &mut DetailPlan, work: &crate::ammunition::WorkBuffers) {
    for (address, bytes) in [
        (0x45363c, &work.item),
        (0x4536d2, &work.ammo),
        (0x453768, &work.weapon),
    ] {
        plan.memory
            .insert_region(address, bytes.as_deref().unwrap_or(&[0; 150]));
    }
}
