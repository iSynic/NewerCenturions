//! The item information dialog's fields (DLOG 149), set in the game's order.
//! Each field keeps the last text set; item 24 is set in color 205.
use super::{BTreeMap, Content, GameText, InventoryRecord, RuleError, indexed_string};
use crate::combat::spells::{FIRST_SPELL_DESCRIPTOR, spell_level};
use centurions_content::{
    Warrior,
    mission::OpeningMission,
    records::{
        EquipmentEligibility, EquipmentModifiers, InventoryModifier as Modifier, ItemDefinition,
        ItemPanelMut, ItemPanelRecord, MosEquipment, ProfileTable, RaceEquipment,
    },
};

/// A field of the item information dialog: text (plain or red), or an icon.
#[derive(Clone, Debug, PartialEq)]
pub enum InformationField {
    Text { text: String, red: bool },
    Icon(i16),
}

/// The fields of the item information dialog as they are set, with the text they read from.
struct Fields<'a> {
    game_text: &'a GameText,
    strings: &'a BTreeMap<i16, Vec<u8>>,
    items: BTreeMap<u16, InformationField>,
}

impl Fields<'_> {
    /// Sets a text field.
    fn set(&mut self, item: u16, text: String) {
        self.items
            .insert(item, InformationField::Text { text, red: false });
    }
    /// The game's text at an address as a string.
    fn game_text(&self, address: u32) -> Result<String, RuleError> {
        Ok(self.game_text.text(address, false)?)
    }
    /// Sets a field to a text the game keeps as a Pascal string.
    fn pascal(&mut self, item: u16, address: u32) -> Result<(), RuleError> {
        let text = self.game_text.text(address, true)?;
        self.set(item, text);
        Ok(())
    }
    /// A string from a STR# resource by its index.
    fn indexed(&self, resource: i16, index: i16) -> Result<String, RuleError> {
        let text = indexed_string(self.strings, resource, index)?;
        Ok(encoding_rs::MACINTOSH.decode(text.payload()).0.into_owned())
    }
    /// DialogNumNZ: empty for zero, an explicit '+' for positive values.
    fn signed(&mut self, item: u16, value: i16) {
        let text = match value {
            0 => String::new(),
            1.. => format!("+{value}"),
            _ => value.to_string(),
        };
        self.set(item, text);
    }
}

/// `record` is the shown instance, or `None` for a bare item ID (no instance
/// values, no ammunition); `owner` is the selected warrior whose race and MOS
/// are checked.
///
/// Original: showiteminfo
pub fn information(
    content: &Content,
    mission: &OpeningMission,
    owner: &Warrior,
    (record, merged): (&InventoryRecord, bool),
    game_text: &GameText,
    strings: &BTreeMap<i16, Vec<u8>>,
) -> Result<BTreeMap<u16, InformationField>, RuleError> {
    let mut fields = Fields {
        game_text,
        strings,
        items: BTreeMap::new(),
    };
    for item in 7..52 {
        fields.set(item, String::new());
    }
    let ammo_id = if merged { record.ammunition_id()? } else { 0 };
    let loaded = ammo_id != 0;
    let ammo = crate::ammunition::definition(content, ammo_id)?;
    let mut definition = crate::ammunition::definition(content, record.id())?;
    if merged {
        ItemPanelMut(&mut definition).merge_item_information(&record.0, &ammo)?;
    }
    let item = ItemDefinition(&definition);
    let panel = ItemPanelRecord(&definition);
    let values = EquipmentModifiers::read(&definition)?;
    let ammunition = ItemDefinition(&ammo);
    let (label, text) = if item.ammunition_caliber()? != 0 {
        if loaded {
            let selection = ammunition.description_id()?;
            let base = crate::rules::selection_base(selection);
            (
                0x4010d8,
                fields.indexed(base.wrapping_add(1), selection.wrapping_sub(base) + 1)?,
            )
        } else {
            (0x4010c6, fields.indexed(142, item.ammunition_caliber()?)?)
        }
    } else if item.reserve_ammunition_kind()? != 0 {
        (
            0x4010b0,
            fields.indexed(142, item.reserve_ammunition_kind()?)?,
        )
    } else if panel.equipment_type() != 0 {
        let index = i16::from(panel.equipment_type()).wrapping_add(1);
        (0x40109a, fields.indexed(-11000, index)?)
    } else {
        (
            0x401084,
            fields.indexed(-11000, item.item_type()?.wrapping_add(1))?,
        )
    };
    fields.pascal(20, label)?;
    fields.set(21, text);
    if loaded {
        fields
            .items
            .insert(4, InformationField::Icon(ammunition.icon()?));
    }
    let mut animation = panel.animation()?;
    if loaded && ItemPanelRecord(&ammo).animation()? != 0 {
        animation = ItemPanelRecord(&ammo).animation()?;
    }
    if animation == 0 && panel.target_mode()? != 0 {
        animation = 1;
    }
    if animation != 0 {
        let icon = InformationField::Icon(animation.wrapping_add(9999));
        fields.items.insert(54, icon);
    }
    fields.items.insert(3, InformationField::Icon(item.icon()?));
    fields.set(6, mission.item_description(item.description_id()?)?);
    special_information(&mut fields, &item, &panel)?;
    let damage =
        crate::items::shown_damage(&definition, &ammo, loaded, record.damage_enhancement())?;
    if damage.base != 0 {
        fields.set(27, damage.base.to_string());
    }
    for (n, shown) in damage.elements.into_iter().enumerate() {
        let n = n as u16;
        if shown > 0 {
            fields.set(28 + n, shown.to_string());
        } else if item.elemental_damage(n as usize)? < 0 {
            fields.signed(36 + n, item.elemental_damage(n as usize)?.wrapping_neg());
        }
    }
    let mut attacks = item.attack_count()?;
    if damage.total != 0 {
        fields.set(26, damage.total.to_string());
        if attacks == 0 {
            attacks = 1;
        }
        let text = attacks.wrapping_mul(damage.total).to_string();
        let red = InformationField::Text { text, red: true };
        fields.items.insert(24, red);
    }
    if attacks != 0 {
        fields.set(25, attacks.to_string());
    }
    let level = if merged {
        i16::from(record.required_level())
    } else {
        panel.required_level()?
    };
    fields.set(17, level.to_string());
    fields.pascal(
        49,
        if item.bypasses_hit_roll()? {
            0x400fb2
        } else {
            0x400fae
        },
    )?;
    for (item, value) in [
        (52, i16::from(item.damage_enhancement())),
        (7, i16::from(item.base_accuracy())),
        (43, values.movement),
        (45, values.missile_dodge),
        (44, values.melee_dodge),
        (46, values.psi_resistance),
        (47, values.maximum_psi),
        (8, values.range),
        (11, values.armor),
        (13, values.shield_chance),
    ] {
        fields.signed(item, value);
    }
    if item.battery_capacity()? != 0 {
        fields.set(14, item.battery_capacity()?.to_string());
    }
    let cost = values.shield_cost.wrapping_add(i16::from(if merged {
        Modifier::ShieldCost.read(record)
    } else {
        panel.cost_modifier()
    }));
    if cost != 0 {
        fields.set(15, cost.to_string());
    }
    if item.rounds_per_shot()? != 0 {
        fields.set(50, item.rounds_per_shot()?.to_string());
    }
    let supplied = item.supplied_quantity()?;
    if record.capacity()? > 0 {
        fields.set(22, record.charge()?.to_string());
        fields.set(53, record.capacity()?.to_string());
    } else if supplied > 0 {
        fields.set(22, supplied.to_string());
        fields.set(53, supplied.to_string());
    } else if item.magazine_capacity()? != 0 {
        fields.set(22, "0".into());
        fields.set(53, item.magazine_capacity()?.to_string());
    } else {
        fields.pascal(22, 0x400faa)?;
        fields.pascal(53, 0x400fa6)?;
    }
    let weight = item
        .charge_weight()?
        .wrapping_mul(supplied)
        .wrapping_add(item.weight()?);
    fields.set(23, weight.to_string());
    let (race, mos) = conflicts(&definition, owner, mission)?;
    if race {
        fields.pascal(18, 0x400f96)?;
    }
    if mos {
        fields.pascal(19, 0x400f86)?;
    }
    Ok(fields.items)
}

/// Sets the special-effect line (item 48) from the item's effect, spell or attribute changes; a
/// later change replaces the line an earlier one set.
fn special_information(
    fields: &mut Fields<'_>,
    item: &ItemDefinition<'_>,
    values: &ItemPanelRecord<'_>,
) -> Result<(), RuleError> {
    let effect = item.effect_selector()?;
    let amount = item.spell()?;
    if effect > 19 {
        let text = fields.indexed(136, effect.wrapping_sub(19))?;
        fields.set(48, text);
    }
    if effect == -10 {
        let text = fields.game_text(0x401046)? + &fields.indexed(133, amount.wrapping_sub(19))?;
        fields.set(48, text);
    }
    if (FIRST_SPELL_DESCRIPTOR..4000).contains(&amount) {
        let tier = spell_level(amount);
        let resource = (amount / 1000).wrapping_mul(1000).wrapping_add(tier);
        let text = fields.game_text(0x40103c)? + &fields.indexed(resource, amount % 100)?;
        fields.set(48, text);
    }
    if effect == 122 {
        let address = match amount {
            1 => Some(0x40101e),
            2 => Some(0x400ffe),
            3 => Some(0x400fde),
            4 => Some(0x400fbe),
            _ => None,
        };
        if let Some(address) = address {
            fields.pascal(48, address)?;
        }
    }
    let (attributes, adjustment) = values.attribute_changes()?;
    for attribute in attributes {
        if (1..16).contains(&attribute) {
            let sign = if adjustment > 0 { "+" } else { "" };
            let text = format!(
                "{sign}{adjustment}{}{}",
                fields.game_text(0x400fb6)?,
                fields.indexed(1201, attribute)?
            );
            fields.set(48, text);
        }
    }
    Ok(())
}

/// Whether the item conflicts with the owner's race and, separately, with the owner's M.O.S.,
/// returned in that order.
///
/// Original: canuse
fn conflicts(
    definition: &[u8],
    owner: &Warrior,
    mission: &OpeningMission,
) -> Result<(bool, bool), RuleError> {
    let (race, mos) = profiles(owner, mission)?;
    let race_id = owner.record().race()?;
    let mos_id = owner.record().mos()?;
    let eligibility = EquipmentEligibility(definition);
    let mut racial = eligibility.required_race()? != 0 && eligibility.required_race()? != race_id;
    let mut caste = eligibility.required_mos()? != 0 && eligibility.required_mos()? != mos_id;
    if racial {
        return Ok((racial, caste));
    }
    let mut ability = 0;
    while ability < 58 && !eligibility.ability(ability)? {
        ability += 1;
    }
    if !mos.permits_ability(ability)? {
        return Ok((racial, true));
    }
    if !race.permits_ability(ability)? {
        return Ok((true, caste));
    }
    for index in 0..9 {
        let race_trait = race.has_trait(index)?;
        if (eligibility.forbids_trait(index)? && race_trait)
            || (eligibility.requires_trait(index)? && !race_trait)
        {
            racial = true;
        }
    }
    let school = mos.psi_school()?.wrapping_sub(1);
    if (0..7).contains(&school) && eligibility.forbids_school(school as usize)? {
        caste = true;
    }
    // A warrior without a school (-1) tests bit 7 of the byte before the
    // required-schools mask, as the game does.
    if eligibility.has_required_schools()? && !eligibility.permits_required_school_bit(school)? {
        caste = true;
    }
    Ok((racial, caste))
}

/// One row of the dialog's popups: its text, icon, whether it is disabled and
/// whether it is marked.
#[derive(Clone, Debug, PartialEq)]
pub struct UseRow {
    pub text: String,
    pub portrait: Option<i16>,
    pub icon: Option<i16>,
    pub enabled: bool,
    pub marked: bool,
}

/// A row of the list of users, enabled and without a mark.
fn row(text: String) -> UseRow {
    UseRow {
        text,
        portrait: None,
        icon: None,
        enabled: true,
        marked: false,
    }
}

/// The MOS popup: the abilities the selected warrior's race and MOS both
/// permit (`STR#` 6), with icon 385 on each ability the item has. `passes`
/// repeats the member loop when the shift key widens it to the squad.
///
/// Original: showcondition
pub fn mos_list(
    mission: &OpeningMission,
    owner: &Warrior,
    definition: &[u8],
    game_text: &GameText,
    strings: &BTreeMap<i16, Vec<u8>>,
    passes: usize,
) -> Result<Vec<UseRow>, RuleError> {
    let (race, mos) = profiles(owner, mission)?;
    let eligibility = EquipmentEligibility(definition);
    let names = centurions_content::ui::string_list(strings.get(&6).ok_or("STR# 6 missing")?)?;
    let mut rows = vec![];
    for _ in 0..passes {
        rows.push(row(game_text.text(0x401112, true)?));
        rows[0].portrait = Some(owner.record().portrait()?);
        let mut marks = vec![];
        for bit in 0..64 {
            let name = names.get(bit).cloned().unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            if mos.permits_ability(bit)? && race.permits_ability(bit)? {
                rows.push(row(name));
            }
            if eligibility.ability(bit)? {
                marks.push(bit + 1);
            }
        }
        for item in marks {
            if let Some(row) = rows.get_mut(item) {
                row.icon = Some(385);
            }
        }
    }
    Ok(rows)
}

/// The user popup: every squad member, disabled when the member cannot use the
/// item. Each check reloads the item from its description, as ListItemUsers does.
///
/// Original: showcondition
pub fn user_list(
    content: &Content,
    mission: &OpeningMission,
    warriors: &[Warrior],
    selected: usize,
    definition: &[u8],
    game_text: &GameText,
    passes: usize,
) -> Result<Vec<UseRow>, RuleError> {
    // Menu item numbers are absolute, so a widened second pass only appends text.
    let mut definition = definition.to_vec();
    let mut rows = vec![];
    for _ in 0..passes {
        rows.push(row(game_text.text(0x4010e8, true)?));
        rows[0].icon = Some(379);
        for (member, warrior) in warriors.iter().enumerate() {
            rows.push(row(warrior.name.clone()));
            rows[member + 1].portrait = Some(warrior.record().portrait()?);
            let id = ItemDefinition(&definition).description_id()?;
            if id > 0 {
                definition = crate::ammunition::definition(content, id)?;
            }
            let (racial, caste) = conflicts(&definition, warrior, mission)?;
            if racial || caste {
                rows[member + 1].enabled = false;
            }
        }
        if let Some(row) = rows.get_mut(selected + 1) {
            row.marked = true;
        }
    }
    Ok(rows)
}

/// The race and MOS profiles of the owner, for judging which items can be used.
fn profiles<'a>(
    owner: &Warrior,
    mission: &'a OpeningMission,
) -> Result<(RaceEquipment<'a>, MosEquipment<'a>), RuleError> {
    let race_id = owner.record().race()?;
    let mos_id = owner.record().mos()?;
    if race_id <= 0 || mos_id <= 0 {
        return Err("item information owner has no Race or MOS profile".into());
    }
    Ok((
        RaceEquipment(ProfileTable::races(&mission.races).row(race_id as usize - 1)?),
        MosEquipment(ProfileTable::mos_profiles(&mission.profiles).row(mos_id as usize - 1)?),
    ))
}
