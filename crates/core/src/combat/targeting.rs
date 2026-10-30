//! How many targets a shot or cast may pick, and paying psi for each extra one.
use crate::combat::Combat;
use crate::combat::spells::spell_identity;
use crate::{Event, RuleError, Session};
use centurions_content::{Content, Warrior, records::ItemDefinition};
use serde::{Deserialize, Serialize};
mod actor_selection;
pub mod target_picks;
pub(crate) use actor_selection::select_actor_target;

/// How many shots a firearm will take: the warrior, his weapon, the shot count, and the most
/// targets that can be picked.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FirearmQuantity {
    pub actor: i16,
    pub weapon: i16,
    pub power: i16,
    /// The target ceiling when it differs from `power`: an eight-shot weapon's
    /// random count, or a quantity step on a packet byte 1 other than 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ceiling: Option<i16>,
}
impl FirearmQuantity {
    /// A firearm quantity; the ceiling is kept only when it differs from the shot count.
    fn new(actor: i16, weapon: i16, power: i16, ceiling: i16) -> Self {
        Self {
            actor,
            weapon,
            power,
            ceiling: (ceiling != power).then_some(ceiling),
        }
    }
    /// The most targets a pick admits.
    pub fn num_of_targets(&self) -> i16 {
        self.ceiling.unwrap_or(self.power)
    }
    /// Checks that the quantity is for the active warrior and his equipped weapon, with 1 to 7
    /// shots and a ceiling from the shots up to 8.
    pub fn validate(&self, combat: &Combat, warriors: &[Warrior]) -> Result<(), RuleError> {
        if !(0..warriors.len() as i16).contains(&self.actor)
            || combat.active_body() != Some(self.actor)
            || !(1..=7).contains(&self.power)
            || !(self.power..=8).contains(&self.num_of_targets())
            || warriors[self.actor as usize].record().equipped_weapon()? != self.weapon
        {
            return Err("invalid targeting quantity".into());
        }
        Ok(())
    }
}

/// Refuses with a warning message.
fn warning(index: i16) -> RuleError {
    RuleError::Refused {
        feedback: vec![crate::feedback::GameFeedback::Warning { index }],
    }
}

/// The quantity after a step up or down; the same rule serves paid steps and the desktop preview. A
/// step up is refused (with a sound) at power 7, for a spell not learned, or without the psi; a
/// step down at power 1 or when the ceiling is no more than the targets picked.
///
/// Original: combatchoice
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
            feedback: vec![crate::feedback::GameFeedback::Sound { resource_id: 143 }],
        });
    }
    Ok(power.wrapping_add(if increase { 1 } else { -1 }))
}

/// A firearm's target ceiling, which the spell panel prints for a category-9 packet.
///
/// Original: combatchoice
pub fn firearm_ceiling(state: &Session, content: &Content) -> Result<i16, RuleError> {
    super::shooting::prepare(state, content)?;
    super::shooting::ceiling(state, content)
}

/// Starts choosing targets for a firearm shot. In firearm mode the warrior is warned when he has
/// fewer than two attacks, or when the load is below the rounds one shot takes; the weapon is then
/// readied with its shot count as the quantity of targets.
///
/// Original: combatchoice
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
    // In firing mode: a warning below two attacks, then a warning when the load is
    // below the rounds per shot of the item last loaded, before the weapon reloads.
    let record = state.warriors[actor].record();
    if record.toggle() {
        if record.attacks()? < 2 {
            return Err(warning(0x11));
        }
        let stale = state
            .globals
            .loaded_items
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
    // The weapon loads from its ammunition above descriptor 1100, else from itself;
    // at 1100 or below nothing further is checked.
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
    // After the attack is prepared, a warning when the load is below the weapon's
    // rounds per shot.
    if warrior.record().inventory_quantity(0)? < ItemDefinition(weapon).rounds_per_shot()? {
        return Err(warning(0x55));
    }
    // The weapon's attack count sets both the power and the target ceiling.
    let ceiling = ItemDefinition(weapon).attack_count()?;
    if !(1..=8).contains(&ceiling) {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.targeting",
            dependency: "weapon attack count outside 1..8",
        });
    }
    let identity = spell_identity(spell);
    let combat = state.combat.as_mut().unwrap();
    combat.weapon_traits.prepare(weapon, ammunition)?;
    combat.spell_info = prepared.packet.clone();
    combat.cast_identity = identity;
    state.globals.spell_info = prepared.packet;
    state.globals.cast_identity = identity;
    state.globals.weapon_traits = combat.weapon_traits;
    state
        .globals
        .loaded_items
        .prepare_pair(weapon, ammunition)?;
    state.globals.loaded_items.item = Some(crate::ammunition::definition(content, weapon_id)?);
    state.globals.loaded_item_field32 = ItemDefinition(weapon).ammunition_caliber()?;
    // The firearm in firing mode, otherwise the melee weapon.
    let record = state.warriors[actor as usize].record();
    state.globals.item_used = if record.toggle() {
        record.equipped_weapon()?
    } else {
        record.melee_weapon()?
    };
    // An eight-shot weapon fires a random count; the ceiling stays eight.
    let power = if ceiling == 8 {
        state.random.bound(7)
    } else {
        ceiling
    };
    state.globals.attack_count = power;
    state.set_in_spell(1);
    state.target_quantity = Some(FirearmQuantity::new(actor, weapon_id, power, ceiling));
    Ok(vec![])
}

/// Steps the number of targets or shots up or down by one: a spell pays or refunds its psi cost for
/// the step, and a firearm changes its shot count and the most targets it can pick. Plays the step
/// sound.
///
/// Original: combatchoice
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
            .map(|combat| &combat.spell_info)
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
        state.globals.attack_count = power;
        // Showing the spell after a step clears the item in use.
        state.globals.item_used = 0;
    } else if let Some(pending) = &state.psi_selection {
        pending.validate(state)?;
        let actor = pending.actor;
        let packet = &state.combat.as_ref().unwrap().spell_info;
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
        state.globals.attack_count = power;
        if let Some(choice) = &mut state.psi_menu_choices[actor] {
            choice.power = power;
        }
        state.globals.item_used = 0;
    } else {
        let firearm_mode = state
            .combat
            .as_ref()
            .and_then(|combat| combat.active_body())
            .and_then(|body| state.warriors.get(usize::try_from(body).ok()?))
            .is_some_and(|warrior| warrior.record().toggle());
        if !firearm_mode {
            return Err(RuleError::Invalid("no targeting quantity is active".into()));
        }
        let prepared = super::shooting::prepare(state, content)?;
        let ceiling = super::shooting::ceiling(state, content)?;
        let combat = state.combat.as_ref().unwrap();
        let selected = combat.firearm_selection.as_ref().map_or(0, |p| p.count());
        let power = adjusted_quantity(prepared.shots, ceiling, selected, false, 0, 0, increase)?;
        // The ceiling steps only for packet byte 1 of 0.
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
        let actor = combat.active_body().ok_or("targeting actor missing")?;
        let weapon = state.warriors[actor as usize].record().equipped_weapon()?;
        state.set_in_spell(1);
        state.target_quantity = Some(FirearmQuantity::new(actor, weapon, power, ceiling));
        state.globals.attack_count = power;
    }
    Ok(vec![Event::Sound { resource_id: -130 }])
}

/// A cast's target ceiling: the power when packet byte 1, read signed, is below 1,
/// so modes 128..255 count too.
///
/// Original: combatchoice
fn castspell_ceiling(packet: &[u8], power: i16) -> i16 {
    if (packet[1] as i8) < 1 { power } else { 1 }
}
