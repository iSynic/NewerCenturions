//! Silent wearing and removal, used by equipment commands, refit and other item
//! actions.
use super::globals::GlobalWrite;
use super::modifiers::{self, Floor, change};
use super::record_rules::{can_use, unsupported};
use crate::RuleError;
use centurions_content::ITEM_BYTES;
use centurions_content::records::{EquipmentModifiers, EquipmentStat as Stat, InstanceModifiers};
use centurions_content::{Content, InventoryRecord, Warrior, word};

/// Whether an item goes on or comes off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fit {
    Wear,
    Remove,
}

/// A forced change works on a blank record and skips the refusal that keeps a
/// shield battery in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Force {
    Checked,
    Forced,
}

/// The outcome of wearing or removing an item without messages: the warrior, the item record and
/// definition used, the warnings raised, and the party-wide values written.
pub(crate) struct SilentFit {
    pub warrior: Warrior,
    pub record: InventoryRecord,
    pub definition: Vec<u8>,
    pub warnings: Vec<i16>,
    pub raised_warnings: Vec<i16>,
    pub return_code: u32,
    pub level_refusal: Option<i16>,
    /// Ordered party-wide writes for the Session caller to apply.
    pub global_writes: Vec<GlobalWrite>,
}

/// One wear or removal in progress.
struct Operation<'a> {
    result: SilentFit,
    content: &'a Content,
    global_effects: Vec<GlobalWrite>,
    editor_active: bool,
}

/// Wears or removes the item in a slot as the game does without messages, returning what the item
/// did.
pub(crate) fn apply(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    fit: Fit,
    force: Force,
    inventory_record: &InventoryRecord,
) -> Result<SilentFit, RuleError> {
    warrior.inventory()?;
    content.map(0).and_then(|map| map.combat.as_ref()).ok_or(RuleError::Unsupported {
        action: "editor.item", dependency: "the reached presentation or arithmetic service is not part of item editing",
    })?;
    let mut operation = Operation {
        result: SilentFit {
            warrior: warrior.clone(),
            record: inventory_record.clone(),
            definition: vec![0; ITEM_BYTES],
            warnings: vec![],
            raised_warnings: vec![],
            return_code: 0,
            level_refusal: None,
            global_writes: vec![],
        },
        content,
        global_effects: vec![],
        editor_active: false,
    };
    if fit == Fit::Wear {
        operation.wear(slot)?;
    } else {
        operation.remove(slot, force == Force::Forced)?;
        operation.result.warnings = operation.result.raised_warnings.clone();
    }
    Warrior::read(&operation.result.warrior.bytes)?;
    operation.result.global_writes = operation.global_effects;
    Ok(operation.result)
}

/// Applies an item-editor control to the item in a slot, with the item taken off and worn again
/// around the change when it is worn.
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
    let equipped = item.equipped() != 0;
    let mut operation = Operation {
        result: SilentFit {
            warrior: warrior.clone(),
            record: item.clone(),
            definition: vec![0; ITEM_BYTES],
            warnings: vec![],
            raised_warnings: vec![],
            return_code: 0,
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
        item.0[9] = operation.item(slot).equipped();
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
    /// Loads the definition of an item id. An id outside the item families leaves the previous
    /// definition loaded.
    fn load(&mut self, id: i16) -> Result<EquipmentModifiers, RuleError> {
        let index = usize::from(id.wrapping_abs() as u16);
        // An ID outside the item families leaves the previous definition loaded.
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
    /// The item record in a slot of the warrior being changed.
    fn item(&self, slot: usize) -> InventoryRecord {
        self.result.warrior.record().inventory_slot(slot)
    }
    /// Reads a stat of the warrior being changed.
    fn read(&self, stat: Stat) -> Result<i16, RuleError> {
        Ok(stat.read(&self.result.warrior)?)
    }
    /// Writes a stat of the warrior being changed.
    fn write(&mut self, stat: Stat, value: i16) -> Result<(), RuleError> {
        Ok(stat.write(&mut self.result.warrior, value)?)
    }
    /// Refuses the wear or removal with a warning; the item is not worn.
    fn refuse(&mut self, warning: i16) {
        self.result.warnings.push(warning);
        self.result.return_code = 0;
    }

    /// Wears the item in a slot. The warrior must be allowed to use it and (outside the item
    /// editor) be high enough level; a worn item of the same class is taken off first, and the item
    /// needs charge or shield energy when its definition says so.
    fn wear(&mut self, slot: usize) -> Result<(), RuleError> {
        // Each repeat follows a removal that unequips one of at most 101 searched
        // slots. A further repeat means a removal kept its item worn, where the
        // game would loop forever.
        let mut remaining = 102_usize;
        loop {
            if remaining == 0 {
                return Err(RuleError::Unsupported {
                    action: "equipment.processing",
                    dependency: "equipment wear repeats without unequipping; the original loop never ends",
                });
            }
            remaining -= 1;
            self.result.return_code = 0;
            let item = self.item(slot);
            let mut definition = self.load(item.id())?;
            let warrior = &self.result.warrior;
            let allowed = if warrior.record().race()? == 0 || warrior.record().mos()? == 0 {
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
                // Too low a level flashes and refuses; refit keeps the charge and
                // reserve changes and continues without equipping the battery.
                self.result.level_refusal = Some(i16::from(item.required_level()));
                return Ok(());
            }
            if definition.kind > 1 && self.read(Stat::Equipment(definition.kind))? != 0 {
                let mut replaced = false;
                let count = self.result.warrior.inventory()?.len();
                // The search includes the slot just past the inventory.
                for search in 0..=count {
                    let old = self.item(search);
                    let previous = self.load(old.id())?;
                    if previous.kind == definition.kind && old.equipped() != 0 {
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
            if definition.energy_requirement == 1 && item.charge()? < 1 {
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
            let inventory_record = InstanceModifiers::read(&self.result.record);
            modifiers::defenses(
                &mut self.result.warrior,
                &item,
                &definition,
                &inventory_record,
                Fit::Wear,
            )?;
            self.write(Stat::Equipment(definition.kind), definition.id)?;
            self.result
                .warrior
                .record_mut()
                .set_inventory_equipment_flag(slot, 1);
            modifiers::statistics(
                &mut self.result.warrior,
                &definition,
                &inventory_record,
                Fit::Wear,
            )?;
            modifiers::movement(&mut self.result.warrior, self.content)?;
            modifiers::conditions(&mut self.result.warrior, &definition, Fit::Wear)?;
            modifiers::attributes(
                &mut self.result.warrior,
                &definition,
                &mut self.global_effects,
                Fit::Wear,
            )?;
            self.result.return_code = 1;
            return Ok(());
        }
    }

    /// Chooses the place a worn item takes by its class. A class 0 item takes the first free of two
    /// places, refused with warning 9 when both are taken and 31 when it matches the other; a
    /// class 2 or 3 item claims hands, refused with warning 8 when it would need more than two.
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
                    Fit::Wear,
                    Floor::Unbounded,
                )?;
                self.write(Stat::DisplayedMelee, definition.id)?;
                self.result.warrior.record_mut().set_toggle(false);
            }
            3 => {
                if definition.hands.wrapping_add(self.read(Stat::Hands)?) > 2 {
                    self.refuse(8);
                    return Ok(false);
                }
                self.write(Stat::Equipment(3), definition.id)?;
                change(
                    &mut self.result.warrior,
                    Stat::Hands,
                    1,
                    Fit::Wear,
                    Floor::Unbounded,
                )?;
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
                self.result.warrior.record_mut().set_toggle(true);
            }
            kind => {
                if kind < 23 {
                    self.write(Stat::Equipment(kind), definition.id)?;
                }
            }
        }
        Ok(true)
    }

    /// Takes the item in a slot off and undoes its effects, clearing the place it occupied. A
    /// shield battery is refused, with warning 145 raised and false returned, while a power shield
    /// is worn unless the removal is forced.
    fn remove(&mut self, slot: usize, forced: bool) -> Result<bool, RuleError> {
        let item = self.item(slot);
        let definition = self.load(item.id())?;
        self.result.return_code = 0;
        if item.charge()? == -1 {
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
                        Fit::Remove,
                        Floor::Zero,
                    )?;
                    if self.read(Stat::Equipment(15))? != 0 {
                        self.result.warrior.record_mut().set_toggle(true);
                    }
                }
                3 => {
                    change(
                        &mut self.result.warrior,
                        Stat::Hands,
                        1,
                        Fit::Remove,
                        Floor::Zero,
                    )?;
                    self.write(Stat::Equipment(3), 0)?;
                }
                15 => {
                    self.write(Stat::DisplayedMissile, 0)?;
                    self.result.warrior.record_mut().set_toggle(false);
                    self.write(Stat::Equipment(15), 0)?;
                    self.write(Stat::LoadedAmmunition, 0)?;
                    change(
                        &mut self.result.warrior,
                        Stat::Hands,
                        definition.hands,
                        Fit::Remove,
                        Floor::Zero,
                    )?;
                }
                20 => {
                    if !forced
                        && (self.read(Stat::Equipment(13))? != 0
                            || self.read(Stat::Equipment(14))? != 0)
                    {
                        self.result.raised_warnings.push(145);
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
                    Fit::Remove,
                    Floor::Unbounded,
                )?;
            }
            modifiers::attributes(
                &mut self.result.warrior,
                &definition,
                &mut self.global_effects,
                Fit::Remove,
            )?;
            let inventory_record = InstanceModifiers::read(&self.result.record);
            modifiers::defenses(
                &mut self.result.warrior,
                &item,
                &definition,
                &inventory_record,
                Fit::Remove,
            )?;
            self.write(Stat::Equipment(definition.kind), 0)?;
            self.result
                .warrior
                .record_mut()
                .set_inventory_equipment_flag(slot, 0);
            modifiers::statistics(
                &mut self.result.warrior,
                &definition,
                &inventory_record,
                Fit::Remove,
            )?;
            if self.read(Stat::Equipment(13))? == 0 && self.read(Stat::Equipment(14))? == 0 {
                self.write(Stat::ShieldCost, 0)?;
                self.write(Stat::ShieldChance, 0)?;
            }
            if self.read(Stat::Equipment(20))? == 0 {
                self.write(Stat::ShieldChance, 0)?;
            }
            modifiers::movement(&mut self.result.warrior, self.content)?;
            modifiers::conditions(&mut self.result.warrior, &definition, Fit::Remove)?;
        }
        self.result.return_code = 1;
        Ok(true)
    }
}
