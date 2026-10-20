//! combatchoice0011a4f4/0011a650: quantity, target ceiling and incremental Psi payment.
use crate::combat::Combat;
use crate::{Event, RuleError, Session};
use centurions_content::{Content, Warrior, word};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FirearmQuantity {
    pub actor: i16,
    pub weapon: i16,
    pub power: i16,
}
impl FirearmQuantity {
    pub fn validate(&self, combat: &Combat, warriors: &[Warrior]) -> Result<(), RuleError> {
        if !(0..warriors.len() as i16).contains(&self.actor)
            || combat.active_body() != Some(self.actor)
            || !(1..=7).contains(&self.power)
            || warriors[self.actor as usize].record().equipped_weapon()? != self.weapon
        {
            return Err("invalid targeting quantity".into());
        }
        Ok(())
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

pub fn firearm_quantity(state: &Session, content: &Content) -> Result<i16, RuleError> {
    Ok(super::shooting::prepare(state, content)?.shots)
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
    super::controls::actor(state)?;
    state.target_quantity = None;
    let prepared = super::shooting::prepare(state, content)?;
    let actor = state.combat.as_ref().unwrap().active_body().unwrap();
    let warrior = &state.warriors[actor as usize];
    let weapon_id = warrior.record().equipped_weapon()?;
    let ammo_id = word(&warrior.bytes, 0x132 + prepared.slot * 22 + 4)?;
    let weapon = &content.items[weapon_id.unsigned_abs() as usize];
    let ammunition = &content.items[ammo_id.unsigned_abs() as usize];
    let spell = word(weapon, 44)?;
    let identity = [spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1];
    let combat = state.combat.as_mut().unwrap();
    combat.weapon_work.prepare(weapon, ammunition)?;
    combat.attack_packet = Some(prepared.packet.clone());
    combat.attack_identity = Some(identity);
    state.attack_packet = Some(prepared.packet);
    state.attack_identity = Some(identity);
    state.attack_count = prepared.shots;
    state.weapon_work = combat.weapon_work;
    state.ammunition_work.prepare_pair(weapon, ammunition)?;
    state.ammunition_work.item = Some(crate::ammunition::definition(content, weapon_id)?);
    state.loaded_item_field32 = Some(word(weapon, 32)?);
    state.target_quantity = Some(FirearmQuantity {
        actor,
        weapon: weapon_id,
        power: prepared.shots,
    });
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
            .and_then(|c| c.attack_packet.as_ref())
            .ok_or("medical target packet missing")?;
        let power = adjusted_quantity(
            pending.power,
            if packet[1] == 0 { pending.power } else { 1 },
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
            state.warriors[actor].bytes[0x58..0x5a].copy_from_slice(&psi.to_be_bytes());
            if let Some(choice) = &mut state.psi_menu_choices[actor] {
                choice.power = power;
            }
        }
        state.medical_continuation.as_mut().unwrap().power = power;
        state.attack_count = power;
    } else if let Some(pending) = &state.psi_selection {
        pending.validate(state)?;
        let actor = pending.actor;
        let packet = state
            .combat
            .as_ref()
            .unwrap()
            .attack_packet
            .as_ref()
            .unwrap();
        let power = adjusted_quantity(
            pending.power,
            if packet[1] == 0 { pending.power } else { 1 },
            pending.targets.len(),
            true,
            word(&state.warriors[actor].bytes, 0x58)?,
            packet[30],
            increase,
        )?;
        let psi = word(&state.warriors[actor].bytes, 0x58)?
            .wrapping_add(i16::from(packet[30]).wrapping_mul(if increase { -1 } else { 1 }));
        state.warriors[actor].bytes[0x58..0x5a].copy_from_slice(&psi.to_be_bytes());
        state.psi_selection.as_mut().unwrap().power = power;
        state.attack_count = power;
        if let Some(choice) = &mut state.psi_menu_choices[actor] {
            choice.power = power;
        }
    } else {
        let firearm_mode = state
            .combat
            .as_ref()
            .and_then(|c| c.active_body())
            .and_then(|body| state.warriors.get(usize::try_from(body).ok()?))
            .is_some_and(|warrior| warrior.bytes[0x9dd] != 0);
        if !firearm_mode {
            return Err(RuleError::Invalid("no targeting quantity is active".into()));
        }
        let prepared = super::shooting::prepare(state, content)?;
        let c = state.combat.as_ref().unwrap();
        let selected = c.firearm_selection.as_ref().map_or(0, |p| p.targets.len());
        let ceiling = if prepared.packet[1] == 0 {
            prepared.shots
        } else {
            1
        };
        let power = adjusted_quantity(prepared.shots, ceiling, selected, false, 0, 0, increase)?;
        let actor = c.active_body().ok_or("targeting actor missing")?;
        let weapon = state.warriors[actor as usize].record().equipped_weapon()?;
        state.target_quantity = Some(FirearmQuantity {
            actor,
            weapon,
            power,
        });
        state.attack_count = power;
    }
    Ok(vec![Event::Sound { resource_id: -130 }])
}
