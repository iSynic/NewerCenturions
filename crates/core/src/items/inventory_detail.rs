//! The optional Items detail panel: it loads the selected row's item, its
//! ammunition, the instance's own values and the item's spell. The weapon, the
//! action points, the random numbers, the warriors and the selection are left
//! alone.
use crate::combat::spells::{FIRST_SPELL_DESCRIPTOR, spell_identity};
use crate::{RuleError, Session};
use centurions_content::Content;
use centurions_content::records::{ItemDefinition, ItemPanelMut, SpellInfoRecord};

/// Shows the optional detail panel for one slot of a warrior's pack: the item, its ammunition and
/// spell are loaded as the panel displays them.
pub(crate) fn quick_info(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
) -> Result<(), RuleError> {
    let record = state.warriors[owner].record().inventory_slot(slot);
    state.load_item(content, record.id())?;
    let ammo = crate::ammunition::definition(content, record.ammunition_id()?)?;
    let item = state.globals.loaded_items.item.as_mut().unwrap();
    ItemPanelMut(item).merge_quick_info(&record.0, &ammo)?;
    let spell = ItemDefinition(item).spell()?;
    state.globals.weapon_traits.load_ammunition(&ammo)?;
    if let Some(combat) = state.combat.as_mut() {
        combat.weapon_traits.load_ammunition(&ammo)?;
    }
    state.globals.loaded_items.ammo = Some(ammo);
    load_spell(state, content, spell)
}

/// Loads an item spell in 1101..3999: its `spell_info`, identity and health effect.
///
/// Original: loadspell2
pub(super) fn load_spell(
    state: &mut Session,
    content: &Content,
    spell: i16,
) -> Result<(), RuleError> {
    if (FIRST_SPELL_DESCRIPTOR..4000).contains(&spell) {
        let attacks = &state
            .land(content)
            .and_then(|land| land.combat.as_ref())
            .ok_or("quick information spell tables missing")?
            .attacks;
        let packet = crate::combat::spells::load_packet(attacks, spell)?;
        let identity = spell_identity(spell);
        let health = &mut state.globals.health_context;
        health.effect = SpellInfoRecord(&packet).health_effect();
        if let Some(combat) = state.combat.as_mut() {
            combat.spell_info = packet.clone();
            combat.cast_identity = identity;
        }
        state.globals.spell_info = packet;
        state.globals.cast_identity = identity;
    }
    Ok(())
}
