//! quickinfo 0x132b0c: the optional Items detail loads the selected row's work.
//!
//! loaditem(record ID) -> loadammo(record WORD+4) -> signed instance merge ->
//! loadspell2 when 1100 < spell < 4000. Weapon work is not loaded; the AP,
//! RNG, warriors and selection state are unchanged.
use crate::{RuleError, Session};
use centurions_content::Content;
use centurions_content::records::{AttackPacket, ItemDefinition, ItemPanelWorkMut};

pub(crate) fn quick_info(
    state: &mut Session,
    content: &Content,
    owner: usize,
    slot: usize,
) -> Result<(), RuleError> {
    let record = state.warriors[owner].record().inventory_slot(slot);
    state.load_item_work(content, record.id())?;
    let ammo = crate::ammunition::definition(content, record.ammunition_id()?)?;
    let item = state.native_work.ammunition_work.item.as_mut().unwrap();
    ItemPanelWorkMut(item).merge_quick_info(&record.0, &ammo)?;
    let spell = ItemDefinition(item).spell()?;
    state.native_work.weapon_work.load_ammunition(&ammo)?;
    if let Some(combat) = state.combat.as_mut() {
        combat.weapon_work.load_ammunition(&ammo)?;
    }
    state.native_work.ammunition_work.ammo = Some(ammo);
    load_spell(state, content, spell)
}

/// loadspell2 for an item spell in 1101..3999: the packet, identity and health effect.
pub(super) fn load_spell(
    state: &mut Session,
    content: &Content,
    spell: i16,
) -> Result<(), RuleError> {
    if (1101..4000).contains(&spell) {
        let attacks = &state
            .land(content)
            .and_then(|land| land.combat.as_ref())
            .ok_or("quick information spell tables missing")?
            .attacks;
        let packet = crate::combat::spells::load_packet(attacks, spell)?;
        let identity = [spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1];
        let health = &mut state.native_work.health_context;
        health.effect = AttackPacket(&packet).health_effect();
        if let Some(combat) = state.combat.as_mut() {
            combat.attack_packet = packet.clone();
            combat.attack_identity = identity;
        }
        state.native_work.attack_packet = packet;
        state.native_work.attack_identity = identity;
    }
    Ok(())
}
