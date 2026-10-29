//! combatchoice0011a4f4/0011a650: quantity, target ceiling and incremental Psi payment.
use crate::combat::Combat;
use crate::{Event, RuleError, Session};
use centurions_content::{Content, Warrior, records::ItemDefinition};
use serde::{Deserialize, Serialize};
mod actor_selection;
pub(crate) use actor_selection::select_actor_target;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FirearmQuantity {
    pub actor: i16,
    pub weapon: i16,
    pub power: i16,
    /// WORD4435be when it differs from WORD4435cc (`power`): an eight-shot weapon's
    /// Rand(7) count, or a quantity step on a packet byte 1 other than 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ceiling: Option<i16>,
}
impl FirearmQuantity {
    fn new(actor: i16, weapon: i16, power: i16, ceiling: i16) -> Self {
        Self {
            actor,
            weapon,
            power,
            ceiling: (ceiling != power).then_some(ceiling),
        }
    }
    /// WORD4435be, the most targets the target key admits.
    pub fn target_ceiling(&self) -> i16 {
        self.ceiling.unwrap_or(self.power)
    }
    pub fn validate(&self, combat: &Combat, warriors: &[Warrior]) -> Result<(), RuleError> {
        if !(0..warriors.len() as i16).contains(&self.actor)
            || combat.active_body() != Some(self.actor)
            || !(1..=7).contains(&self.power)
            || !(self.power..=8).contains(&self.target_ceiling())
            || warriors[self.actor as usize].record().equipped_weapon()? != self.weapon
        {
            return Err("invalid targeting quantity".into());
        }
        Ok(())
    }
}

fn warning(index: i16) -> RuleError {
    RuleError::Refused {
        feedback: vec![crate::feedback::NativeFeedback::Warning { index }],
    }
}

/// The same admission rule serves paid command continuations and uncommitted desktop previews.
pub fn adjusted_quantity(
    power: i16,
    ceiling: i16,
    selected: usize,
    learned: bool,
    psi: i16,
    base_cost: u8,
    increase: bool,
) -> Result<i16, RuleError> {
    if if increase {
        power >= 7 || !learned || i16::from(base_cost) > psi
    } else {
        power < 2 || ceiling <= selected as i16
    } {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Sound { resource_id: 143 }],
        });
    }
    Ok(power.wrapping_add(if increase { 1 } else { -1 }))
}

/// WORD4435be, which showspellinfo prints for a category-9 packet.
pub fn firearm_ceiling(state: &Session, content: &Content) -> Result<i16, RuleError> {
    super::shooting::prepare(state, content)?;
    super::shooting::ceiling(state, content)
}

pub(crate) fn begin_firearm(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if state
        .combat
        .as_ref()
        .is_some_and(|c| c.firearm_selection.is_some())
    {
        return Ok(vec![]);
    }
    let actor = super::controls::actor(state)?;
    // combatchoice 1184d4..1186ce: in firing mode, warn 0x11 below two action points,
    // then warn 7 when slot 0's load is below the retained item work's WORD 22
    // (0x453652), which loaditem has not yet refreshed.
    let record = state.warriors[actor].record();
    if record.firing_mode() {
        if record.action_points()? < 2 {
            return Err(warning(0x11));
        }
        let stale =
            state
                .native_work
                .ammunition_work
                .item
                .as_deref()
                .ok_or(RuleError::Unsupported {
                    action: "combat.firearm.targeting",
                    dependency: "fire button before any retained item work",
                })?;
        if record.inventory_quantity(0)? < ItemDefinition(stale).rounds_per_shot()? {
            return Err(warning(7));
        }
    }
    state.target_quantity = None;
    let prepared = super::shooting::prepare(state, content)?;
    let actor = actor as i16;
    let warrior = &state.warriors[actor as usize];
    let weapon_id = warrior.record().equipped_weapon()?;
    let ammo_id = warrior.record().inventory_ammunition(prepared.slot)?;
    let weapon = &content.items[weapon_id.unsigned_abs() as usize];
    let ammunition = &content.items[ammo_id.unsigned_abs() as usize];
    let spell = ItemDefinition(weapon).spell()?;
    // 118744..118818: loaditem(+0xe4) and loadammo(slot 0's +0x136) fill WEAPON work
    // from the ammunition above descriptor 1100, else from the weapon; at most 1100
    // the fire button skips everything below.
    let slot_ammunition = content
        .items
        .get(warrior.record().inventory_ammunition(0)?.unsigned_abs() as usize)
        .ok_or("fire-button ammunition definition missing")?;
    let descriptor = match ItemDefinition(slot_ammunition).spell()? {
        descriptor if descriptor > 1100 => descriptor,
        _ => spell,
    };
    if descriptor <= 1100 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    // 11885a..1188a8: after prepattack, warn 0x55 when slot 0's load is below the
    // weapon's WORD 22.
    if warrior.record().inventory_quantity(0)? < ItemDefinition(weapon).rounds_per_shot()? {
        return Err(warning(0x55));
    }
    // combatchoice 118928..118932 stores WEAPON work +0x4c, the whole item WORD 76,
    // in WORD4435cc and WORD4435be.
    let ceiling = ItemDefinition(weapon).attack_count()?;
    if !(1..=8).contains(&ceiling) {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.targeting",
            dependency: "weapon attack count outside 1..8",
        });
    }
    let identity = [spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1];
    let combat = state.combat.as_mut().unwrap();
    combat.weapon_work.prepare(weapon, ammunition)?;
    combat.attack_packet = prepared.packet.clone();
    combat.attack_identity = identity;
    state.native_work.attack_packet = prepared.packet;
    state.native_work.attack_identity = identity;
    state.native_work.weapon_work = combat.weapon_work;
    state
        .native_work
        .ammunition_work
        .prepare_pair(weapon, ammunition)?;
    state.native_work.ammunition_work.item =
        Some(crate::ammunition::definition(content, weapon_id)?);
    state.native_work.loaded_item_field32 = ItemDefinition(weapon).retained_field32()?;
    // combatchoice 1188dc..118910: +0xe4 in firing mode, otherwise +0xca.
    let record = state.warriors[actor as usize].record();
    state.native_work.fire_item = if record.firing_mode() {
        record.equipped_weapon()?
    } else {
        record.melee_weapon()?
    };
    // 118944..118958: eight shots become Rand(7) in WORD4435cc only.
    let power = if ceiling == 8 {
        state.random.bound(7)
    } else {
        ceiling
    };
    state.native_work.attack_count = power;
    state.set_ready(1);
    state.target_quantity = Some(FirearmQuantity::new(actor, weapon_id, power, ceiling));
    Ok(vec![])
}

pub(crate) fn adjust(
    state: &mut Session,
    content: &Content,
    increase: bool,
) -> Result<Vec<Event>, RuleError> {
    if let Some(pending) = &state.medical_continuation {
        let actor = pending.actor;
        let learned = pending.origin == crate::MedicalOrigin::LearnedHeal;
        let packet = state
            .combat
            .as_ref()
            .map(|c| &c.attack_packet)
            .ok_or("medical target packet missing")?;
        let power = adjusted_quantity(
            pending.power,
            castspell_ceiling(packet, pending.power),
            0,
            learned,
            state.warriors[actor].psi_points()?,
            packet[30],
            increase,
        )?;
        if learned {
            let psi = state.warriors[actor]
                .psi_points()?
                .wrapping_add(i16::from(packet[30]) * if increase { -1 } else { 1 });
            state.warriors[actor].record_mut().set_psi_points(psi);
            if let Some(choice) = &mut state.psi_menu_choices[actor] {
                choice.power = power;
            }
        }
        state.medical_continuation.as_mut().unwrap().power = power;
        state.native_work.attack_count = power;
        // An accepted Psi step (combatchoice 11a622/11a794) ends in showspellinfo,
        // which clears WORD44719a at 15939a.
        state.native_work.fire_item = 0;
    } else if let Some(pending) = &state.psi_selection {
        pending.validate(state)?;
        let actor = pending.actor;
        let packet = &state.combat.as_ref().unwrap().attack_packet;
        let power = adjusted_quantity(
            pending.power,
            castspell_ceiling(packet, pending.power),
            pending.targets.len(),
            true,
            state.warriors[actor].record().psi_points()?,
            packet[30],
            increase,
        )?;
        let psi = state.warriors[actor]
            .record()
            .psi_points()?
            .wrapping_add(i16::from(packet[30]).wrapping_mul(if increase { -1 } else { 1 }));
        state.warriors[actor].record_mut().set_psi_points(psi);
        state.psi_selection.as_mut().unwrap().power = power;
        state.native_work.attack_count = power;
        if let Some(choice) = &mut state.psi_menu_choices[actor] {
            choice.power = power;
        }
        state.native_work.fire_item = 0;
    } else {
        let firearm_mode = state
            .combat
            .as_ref()
            .and_then(|c| c.active_body())
            .and_then(|body| state.warriors.get(usize::try_from(body).ok()?))
            .is_some_and(|warrior| warrior.record().firing_mode());
        if !firearm_mode {
            return Err(RuleError::Invalid("no targeting quantity is active".into()));
        }
        let prepared = super::shooting::prepare(state, content)?;
        let ceiling = super::shooting::ceiling(state, content)?;
        let c = state.combat.as_ref().unwrap();
        let selected = c.firearm_selection.as_ref().map_or(0, |p| p.count());
        let power = adjusted_quantity(prepared.shots, ceiling, selected, false, 0, 0, increase)?;
        // 11a576..11a580 and 11a6f8..11a702 step WORD4435be only for packet byte 1 of 0.
        let ceiling = if prepared.packet[1] == 0 {
            ceiling.wrapping_add(power.wrapping_sub(prepared.shots))
        } else {
            ceiling
        };
        if !(power..=8).contains(&ceiling) {
            return Err(RuleError::Unsupported {
                action: "combat.firearm.targeting",
                dependency: "weapon attack count outside 1..8",
            });
        }
        let actor = c.active_body().ok_or("targeting actor missing")?;
        let weapon = state.warriors[actor as usize].record().equipped_weapon()?;
        state.set_ready(1);
        state.target_quantity = Some(FirearmQuantity::new(actor, weapon, power, ceiling));
        state.native_work.attack_count = power;
    }
    Ok(vec![Event::Sound { resource_id: -130 }])
}

/// castspell's WORD4435be (combatchoice 119210..119226): WORD4435cc when packet
/// byte 1 is below 1 as a signed BYTE, so modes 128..255 take the count too.
fn castspell_ceiling(packet: &[u8], power: i16) -> i16 {
    if (packet[1] as i8) < 1 { power } else { 1 }
}
