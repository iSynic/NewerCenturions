//! Silent wear/removal used by equipment commands, refit and retained item callers.
//! These are game transactions over records, not a projected Mac address space.
use super::globals::GlobalWrite;
use super::modifiers::{self, change};
use super::record_rules::{can_use, unsupported};
use crate::RuleError;
use centurions_content::records::{
    EquipmentModifiers, EquipmentStat as Stat, RetainedEquipmentModifiers,
};
use centurions_content::{Content, InventoryRecord, Warrior, word};

pub(crate) struct RetainedEquipment {
    pub warrior: Warrior,
    pub record: InventoryRecord,
    pub definition: Vec<u8>,
    pub warnings: Vec<i16>,
    pub native_warnings: Vec<i16>,
    pub native_return: u32,
    pub level_refusal: Option<i16>,
    /// Ordered party-wide writes for the Session caller to apply.
    pub global_writes: Vec<GlobalWrite>,
}

struct Operation<'a> {
    result: RetainedEquipment,
    content: &'a Content,
    global_effects: Vec<GlobalWrite>,
    editor_active: bool,
}

pub(crate) fn apply(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    wearing: bool,
    forced: bool,
    retained: &InventoryRecord,
) -> Result<RetainedEquipment, RuleError> {
    warrior.inventory()?;
    content.map(0).and_then(|map| map.combat.as_ref()).ok_or(RuleError::Unsupported {
        action: "editor.item", dependency: "the reached presentation or arithmetic service is not part of item editing",
    })?;
    let mut operation = Operation {
        result: RetainedEquipment {
            warrior: warrior.clone(),
            record: retained.clone(),
            definition: vec![0; 150],
            warnings: vec![],
            native_warnings: vec![],
            native_return: 0,
            level_refusal: None,
            global_writes: vec![],
        },
        content,
        global_effects: vec![],
        editor_active: false,
    };
    if wearing {
        operation.wear(slot)?;
    } else {
        operation.remove(slot, forced)?;
        operation.result.warnings = operation.result.native_warnings.clone();
    }
    Warrior::read(&operation.result.warrior.bytes)?;
    operation.result.global_writes = operation.global_effects;
    Ok(operation.result)
}

pub(crate) fn edit_record(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    control: u16,
) -> Result<(Warrior, InventoryRecord, Vec<GlobalWrite>), RuleError> {
    content.map(0).and_then(|map| map.combat.as_ref()).ok_or(RuleError::Unsupported {
        action: "editor.item", dependency: "the reached presentation or arithmetic service is not part of item editing",
    })?;
    let mut item = warrior.record().inventory_slot(slot);
    let equipped = item.equipment_flag() != 0;
    let mut operation = Operation {
        result: RetainedEquipment {
            warrior: warrior.clone(),
            record: item.clone(),
            definition: vec![0; 150],
            warnings: vec![],
            native_warnings: vec![],
            native_return: 0,
            level_refusal: None,
            global_writes: vec![],
        },
        content,
        global_effects: vec![],
        editor_active: true,
    };
    if equipped {
        operation.remove(slot, true)?;
    }
    crate::editor_items::change_record(&mut item, control)?;
    if equipped {
        item.0[9] = operation.item(slot).equipment_flag();
    }
    operation
        .result
        .warrior
        .record_mut()
        .set_inventory_slot(slot, &item);
    operation.result.record = item;
    if equipped {
        operation.wear(slot)?;
    }
    Warrior::read(&operation.result.warrior.bytes)?;
    Ok((
        operation.result.warrior,
        operation.result.record,
        operation.global_effects,
    ))
}

impl Operation<'_> {
    fn load(&mut self, id: i16) -> Result<EquipmentModifiers, RuleError> {
        let index = usize::from(id.wrapping_abs() as u16);
        // loaditem's out-of-family branch retains the previous definition.
        if index < 1000 {
            self.result.definition = self
                .content
                .items
                .get(index)
                .ok_or_else(|| unsupported("equipment definition missing"))?
                .clone();
        }
        let kind = word(&self.result.definition, 26)?.wrapping_abs();
        self.result.definition[26..28].copy_from_slice(&kind.to_be_bytes());
        Ok(EquipmentModifiers::read(&self.result.definition)?)
    }
    fn item(&self, slot: usize) -> InventoryRecord {
        self.result.warrior.record().inventory_slot(slot)
    }
    fn read(&self, stat: Stat) -> Result<i16, RuleError> {
        Ok(stat.read(&self.result.warrior)?)
    }
    fn write(&mut self, stat: Stat, value: i16) -> Result<(), RuleError> {
        Ok(stat.write(&mut self.result.warrior, value)?)
    }
    fn refuse(&mut self, warning: i16) {
        self.result.warnings.push(warning);
        self.result.native_return = 0;
    }

    fn wear(&mut self, slot: usize) -> Result<(), RuleError> {
        // Each repeat follows a removal that unequips one of at most 101 searched
        // slots. A further repeat means a removal kept its item worn, and the
        // original loop would never end.
        let mut remaining = 102_usize;
        loop {
            if remaining == 0 {
                return Err(RuleError::Unsupported {
                    action: "equipment.processing",
                    dependency: "equipment wear repeats without unequipping; the original loop never ends",
                });
            }
            remaining -= 1;
            self.result.native_return = 0;
            let item = self.item(slot);
            let mut definition = self.load(item.id())?;
            let warrior = &self.result.warrior;
            let allowed = if warrior.record().race()? == 0 || warrior.record().profession()? == 0 {
                false
            } else {
                can_use(
                    warrior,
                    &self.result.definition,
                    self.content.map(0).unwrap(),
                )?
            };
            if !allowed {
                self.refuse(29);
                return Ok(());
            }
            if definition.kind > 23 {
                self.refuse(48);
                return Ok(());
            }
            if i16::from(item.editor_modifier(0)) > warrior.record().level()? && !self.editor_active
            {
                // wear returns zero after its level flash; refit keeps the charge
                // and reserve changes and continues without equipping the battery.
                self.result.level_refusal = Some(i16::from(item.required_level()));
                return Ok(());
            }
            if definition.kind > 1 && self.read(Stat::Equipment(definition.kind))? != 0 {
                let mut replaced = false;
                let count = self.result.warrior.inventory()?.len();
                // Native search includes the slot immediately after inventory.
                for search in 0..=count {
                    let old = self.item(search);
                    let previous = self.load(old.id())?;
                    if previous.kind == definition.kind && old.equipment_flag() != 0 {
                        self.result.record = old;
                        if !self.remove(search, false)? {
                            return Ok(());
                        }
                        self.result.record = self.item(slot);
                        replaced = true;
                        break;
                    }
                }
                if replaced {
                    continue;
                }
                definition = self.load(item.id())?;
                self.write(Stat::Equipment(definition.kind), 0)?;
                if definition.hands != 0 {
                    self.write(Stat::Hands, 0)?;
                }
            }
            let selected = self.select_slot(&definition, &item)?;
            if !selected {
                return Ok(());
            }
            if definition.energy_requirement == 1 && item.quantity()? < 1 {
                self.refuse(7);
                return Ok(());
            }
            if definition.energy_requirement == 2
                && definition
                    .shield_cost
                    .wrapping_add(self.read(Stat::ShieldCost)?)
                    > self.read(Stat::ShieldCharge)?
            {
                self.refuse(144);
                return Ok(());
            }
            let retained = RetainedEquipmentModifiers::read(&self.result.record);
            modifiers::defenses(
                &mut self.result.warrior,
                &item,
                &definition,
                &retained,
                false,
            )?;
            self.write(Stat::Equipment(definition.kind), definition.id)?;
            self.result
                .warrior
                .record_mut()
                .set_inventory_equipment_flag(slot, 1);
            modifiers::statistics(&mut self.result.warrior, &definition, &retained, false)?;
            modifiers::movement(&mut self.result.warrior, self.content)?;
            modifiers::conditions(&mut self.result.warrior, &definition, false)?;
            modifiers::attributes(
                &mut self.result.warrior,
                &definition,
                &mut self.global_effects,
                false,
            )?;
            self.result.native_return = 1;
            return Ok(());
        }
    }

    fn select_slot(
        &mut self,
        definition: &EquipmentModifiers,
        item: &InventoryRecord,
    ) -> Result<bool, RuleError> {
        match definition.kind {
            0 => {
                if self.read(Stat::Equipment(0))? != 0 && self.read(Stat::Equipment(1))? != 0 {
                    self.refuse(9);
                    return Ok(false);
                }
                let slot = if self.read(Stat::Equipment(0))? == 0 {
                    0
                } else {
                    1
                };
                self.write(Stat::Equipment(slot), definition.id)?;
                if self.read(Stat::Equipment(0))? == self.read(Stat::Equipment(1))? {
                    self.write(Stat::Equipment(1), 0)?;
                    self.refuse(31);
                    return Ok(false);
                }
            }
            2 => {
                if self.read(Stat::Hands)? >= 2 || self.read(Stat::Equipment(2))? != 0 {
                    return Ok(false);
                }
                if definition.hands.wrapping_add(self.read(Stat::Hands)?) > 2 {
                    self.refuse(8);
                    return Ok(false);
                }
                self.write(Stat::Equipment(2), definition.id)?;
                self.write(Stat::MeleeSound, definition.impact_sound)?;
                change(
                    &mut self.result.warrior,
                    Stat::Hands,
                    definition.hands,
                    false,
                    false,
                )?;
                self.write(Stat::DisplayedMelee, definition.id)?;
                self.result.warrior.record_mut().set_firing_mode(false);
            }
            3 => {
                if definition.hands.wrapping_add(self.read(Stat::Hands)?) > 2 {
                    self.refuse(8);
                    return Ok(false);
                }
                self.write(Stat::Equipment(3), definition.id)?;
                change(&mut self.result.warrior, Stat::Hands, 1, false, false)?;
            }
            13 | 14 => {
                let other = if definition.kind == 13 { 14 } else { 13 };
                if self.read(Stat::Equipment(18))? == 0 && self.read(Stat::Equipment(other))? != 0 {
                    self.refuse(146);
                    return Ok(false);
                }
            }
            15 => {
                if self.read(Stat::Equipment(15))? != 0 {
                    return Ok(false);
                }
                self.write(Stat::DisplayedMissile, definition.id)?;
                self.write(Stat::Equipment(15), definition.id)?;
                self.write(Stat::LoadedAmmunition, item.ammunition_id()?)?;
                self.result.warrior.record_mut().set_firing_mode(true);
            }
            kind => {
                if kind < 23 {
                    self.write(Stat::Equipment(kind), definition.id)?;
                }
            }
        }
        Ok(true)
    }

    fn remove(&mut self, slot: usize, forced: bool) -> Result<bool, RuleError> {
        let item = self.item(slot);
        let definition = self.load(item.id())?;
        self.result.native_return = 0;
        if item.quantity()? == -1 {
            self.write(Stat::Equipment(definition.kind), 0)?;
            self.result
                .warrior
                .record_mut()
                .set_inventory_equipment_flag(slot, 0);
        } else {
            match definition.kind {
                0 => {
                    let slot = if definition.id == self.read(Stat::Equipment(0))? {
                        0
                    } else {
                        1
                    };
                    self.write(Stat::Equipment(slot), 0)?;
                }
                2 => {
                    self.write(Stat::DisplayedMelee, 0)?;
                    self.write(Stat::Equipment(2), 0)?;
                    self.write(Stat::MeleeSound, 38)?;
                    change(
                        &mut self.result.warrior,
                        Stat::Hands,
                        definition.hands,
                        true,
                        true,
                    )?;
                    if self.read(Stat::Equipment(15))? != 0 {
                        self.result.warrior.record_mut().set_firing_mode(true);
                    }
                }
                3 => {
                    change(&mut self.result.warrior, Stat::Hands, 1, true, true)?;
                    self.write(Stat::Equipment(3), 0)?;
                }
                15 => {
                    self.write(Stat::DisplayedMissile, 0)?;
                    self.result.warrior.record_mut().set_firing_mode(false);
                    self.write(Stat::Equipment(15), 0)?;
                    self.write(Stat::LoadedAmmunition, 0)?;
                    change(
                        &mut self.result.warrior,
                        Stat::Hands,
                        definition.hands,
                        true,
                        true,
                    )?;
                }
                20 => {
                    if !forced
                        && (self.read(Stat::Equipment(13))? != 0
                            || self.read(Stat::Equipment(14))? != 0)
                    {
                        self.result.native_warnings.push(145);
                        return Ok(false);
                    }
                }
                kind if kind < 24 => self.write(Stat::Equipment(kind.wrapping_abs()), 0)?,
                _ => (),
            }
            if definition.effect == 122 {
                change(
                    &mut self.result.warrior,
                    Stat::SpecialEffect122,
                    definition.effect_amount,
                    true,
                    false,
                )?;
            }
            modifiers::attributes(
                &mut self.result.warrior,
                &definition,
                &mut self.global_effects,
                true,
            )?;
            let retained = RetainedEquipmentModifiers::read(&self.result.record);
            modifiers::defenses(
                &mut self.result.warrior,
                &item,
                &definition,
                &retained,
                true,
            )?;
            self.write(Stat::Equipment(definition.kind), 0)?;
            self.result
                .warrior
                .record_mut()
                .set_inventory_equipment_flag(slot, 0);
            modifiers::statistics(&mut self.result.warrior, &definition, &retained, true)?;
            if self.read(Stat::Equipment(13))? == 0 && self.read(Stat::Equipment(14))? == 0 {
                self.write(Stat::ShieldCost, 0)?;
                self.write(Stat::ShieldChance, 0)?;
            }
            if self.read(Stat::Equipment(20))? == 0 {
                self.write(Stat::ShieldChance, 0)?;
            }
            modifiers::movement(&mut self.result.warrior, self.content)?;
            modifiers::conditions(&mut self.result.warrior, &definition, true)?;
        }
        self.result.native_return = 1;
        Ok(true)
    }
}
