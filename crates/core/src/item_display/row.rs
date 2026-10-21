//! showitemstats (0x157758) row annotations. Composition is deliberately distinct
//! from quickinfo: only positive elemental damage contributes to this maximum.
use super::{
    BTreeMap, Content, DrawingEffect, DrawingServices, InventoryRecord, ItemUiEffect, NativeText,
    PascalString, RuleError, Services,
};
use centurions_content::records::{InventoryModifier as Modifier, ItemDefinition, ItemPanelRecord};

pub fn row(
    content: &Content,
    item: &InventoryRecord,
    native: &NativeText,
    strings: &BTreeMap<i16, Vec<u8>>,
) -> Result<Vec<ItemUiEffect>, RuleError> {
    native.bytes(0x400ef8, 140)?;
    let definition = crate::ammunition::definition(content, item.id())?;
    let base = ItemDefinition(&definition);
    let panel = ItemPanelRecord(&definition);
    let ammo = item.ammunition_id()?;
    let ammunition = if ammo != 0 {
        content
            .items
            .get(ammo.unsigned_abs() as usize)
            .ok_or("display ammunition unavailable")?
            .as_slice()
    } else {
        &[0; 150]
    };
    let ammo = ItemDefinition(ammunition);
    let mut elements = [0_i16; 7];
    for (index, value) in elements.iter_mut().enumerate() {
        *value = base
            .elemental_damage(index)?
            .wrapping_add(ammo.elemental_damage(index)?);
    }
    let mut damage = base
        .missile_damage()?
        .wrapping_add(ammo.missile_damage()?)
        .wrapping_add(base.melee_damage()?.wrapping_add(ammo.melee_damage()?));
    for value in elements {
        if value > 0 {
            damage = damage.wrapping_add(value);
        }
    }
    for multiplier in [base.attack_count()?, base.rounds_per_shot()?] {
        if multiplier != 0 {
            damage = (i32::from(damage) * i32::from(multiplier)) as i16;
        }
    }
    let modifier = |field: Modifier| i16::from(field.read(item));
    let mut row = Row {
        effects: vec![],
        native,
    };
    row.draw(DrawingEffect::TextSize(10));
    row.draw(DrawingEffect::TextFont(10));
    row.effects.push(ItemUiEffect::TextFaceSlot(0));
    row.draw(DrawingEffect::MoveTo {
        horizontal: 42,
        vertical: 30,
    });
    row.draw(DrawingEffect::ForeColor(33));
    row.draw(DrawingEffect::TextSize(10));
    row.effects.push(ItemUiEffect::TextMode(1));
    row.effects.push(ItemUiEffect::TextFaceSlot(0));
    if panel.equipment_type() != 0 {
        row.draw(DrawingEffect::ForeColor(409));
        row.literal(0x400f7c)?;
        let index = i16::from(panel.equipment_type()).wrapping_add(1);
        let text = Services { strings }.indexed_string(-11000, index)?;
        row.effects.push(ItemUiEffect::IndexedString {
            resource_id: -11000,
            index,
            destination: 0x479ed8,
            result: text.clone(),
        });
        row.draw(DrawingEffect::DrawString(text));
        row.literal(0x400f78)?;
    }
    for (color, prefix, value, plus, suffix) in [
        (409, 0x400f74, base.attack_count()?, false, Some(0x400f72)),
        (205, 0x400f68, damage, false, None),
        (
            409,
            0x400f64,
            modifier(Modifier::Damage),
            false,
            Some(0x400f5e),
        ),
        (
            33,
            0x400f52,
            panel.capacity_modifier()?,
            false,
            Some(0x400f4e),
        ),
        (
            341,
            0x400f48,
            modifier(Modifier::Accuracy),
            true,
            Some(0x400f44),
        ),
        (
            33,
            0x400f3c,
            i16::from(panel.missile_dodge()).wrapping_add(modifier(Modifier::MissileDodge)),
            true,
            Some(0x400f72),
        ),
        (
            33,
            0x400f34,
            i16::from(panel.melee_dodge()).wrapping_add(modifier(Modifier::MeleeDodge)),
            true,
            Some(0x400f72),
        ),
        (
            409,
            0x400f2e,
            modifier(Modifier::Range).wrapping_add(
                panel
                    .range()?
                    .wrapping_add(ItemPanelRecord(ammunition).range()?),
            ),
            true,
            Some(0x400f72),
        ),
    ] {
        if value != 0 {
            row.stat(color, prefix, value, plus, suffix)?;
        }
    }
    let armor = modifier(Modifier::Armor).wrapping_add(panel.armor()?);
    if armor > 0 {
        row.stat(409, 0x400f28, armor, false, Some(0x400f72))?;
    }
    if panel.shield_chance()? > 0 {
        row.stat(
            409,
            0x400f22,
            modifier(Modifier::ShieldChance).wrapping_add(panel.shield_chance()?),
            true,
            Some(0x400f72),
        )?;
    }
    if modifier(Modifier::ShieldCost) != 0 || panel.shield_cost()? != 0 {
        row.stat(
            341,
            0x400f1c,
            modifier(Modifier::ShieldCost).wrapping_add(panel.shield_cost()?),
            true,
            Some(0x400f72),
        )?;
    }
    // Chemical reduction alone does not enable the original DRVs prefix.
    if elements[1..].iter().any(|value| *value < 0) {
        row.draw(DrawingEffect::ForeColor(341));
        row.literal(0x400f14)?;
        for (value, label) in elements.into_iter().zip([
            0x400f10, 0x400f0c, 0x400f08, 0x400f04, 0x400f00, 0x400efc, 0x400ef8,
        ]) {
            if value < 0 {
                row.literal(label)?;
            }
        }
    }
    row.draw(DrawingEffect::ForeColor(33));
    row.draw(DrawingEffect::TextFont(0));
    row.effects.push(ItemUiEffect::TextFaceSlot(0));
    row.draw(DrawingEffect::TextSize(0));
    row.effects.push(ItemUiEffect::TextMode(0));
    Ok(row.effects)
}

struct Row<'a> {
    effects: Vec<ItemUiEffect>,
    native: &'a NativeText,
}

impl Row<'_> {
    fn draw(&mut self, effect: DrawingEffect) {
        self.effects.push(ItemUiEffect::Drawing(effect));
    }
    fn literal(&mut self, address: u32) -> Result<(), RuleError> {
        let length = usize::from(self.native.bytes(address, 1)?[0]);
        self.draw(DrawingEffect::DrawString(PascalString::from_raw(
            self.native.bytes(address, length + 1)?,
        )?));
        Ok(())
    }
    fn stat(
        &mut self,
        color: i32,
        prefix: u32,
        value: i16,
        plus: bool,
        suffix: Option<u32>,
    ) -> Result<(), RuleError> {
        self.draw(DrawingEffect::ForeColor(color));
        self.literal(prefix)?;
        let text = PascalString::from_payload(value.to_string().as_bytes())?;
        self.effects.push(ItemUiEffect::NumberToString {
            value: i32::from(value),
            destination: 0x479ed8,
            result: text.clone(),
        });
        if plus && value > 0 {
            self.draw(DrawingEffect::DrawString(PascalString::from_payload(b"+")?));
        }
        self.draw(DrawingEffect::DrawString(text));
        if let Some(suffix) = suffix {
            self.literal(suffix)?;
        }
        Ok(())
    }
}
