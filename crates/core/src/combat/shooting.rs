use crate::combat::{Combat, firearm, lifecycle, player_psi, runtime};
use crate::{Event, RuleError, Session};
use centurions_content::records::SpellInfoRecord;
use centurions_content::{Content, Warrior};
use serde::{Deserialize, Serialize};

/// Up to seven targets a warrior picks; ammunition is spent as each is picked.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Selection {
    pub actor: usize,
    pub weapon: i16,
    pub targets: Vec<i16>,
    pub original_first_item: Vec<u8>,
    /// Cells picked for an area attack (packet byte 1 above 1). Bodies stay in
    /// `targets`; one selection never holds both.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<(i16, i16)>,
    /// Picking a target emptied a weapon that is thrown away when empty, so firing
    /// drops it first.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub discard_empty: bool,
}

/// When firing drops an emptied weapon.
#[derive(Clone, Copy)]
pub(crate) enum Discard {
    /// Not a target-key shot.
    Never,
    /// Picking and firing at once: tested after the ammunition is spent.
    AfterLoad,
    /// Firing at targets already picked, with the flag picking left.
    Collected(bool),
}

/// The tutorial shown when a target is picked with the target key.
const TARGET_KEY_TUTORIAL: crate::feedback::GameFeedback =
    crate::feedback::GameFeedback::Tutorial {
        first: 76,
        last: 77,
    };
/// The tutorial shown when a location is picked.
const LOCATION_TUTORIAL: crate::feedback::GameFeedback = crate::feedback::GameFeedback::Tutorial {
    first: 81,
    last: 81,
};

/// Picks a body as a target. The targeting sound plays before the range test and
/// every later refusal.
///
/// Original: combatchoice
pub fn target_key(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    let mut events = runtime::player_shoot(state, content, target)?;
    events.insert(0, Event::Feedback(TARGET_KEY_TUTORIAL));
    Ok(events)
}

/// A friendly target is not recorded when packet byte 31 is above 1, so the cast meets the slot's
/// earlier, normally cleared, target (see docs/source-notes/combat.md).
fn unwritten_friendly(packet: &[u8], targets: &[i16]) -> Result<(), RuleError> {
    if packet.get(31).is_some_and(|byte| *byte > 1) && targets.iter().any(|body| *body < 10) {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.targeting",
            dependency: "friendly target triple left unwritten above packet byte 31 of 1",
        });
    }
    Ok(())
}

/// After a pick spends ammunition: the weapon is empty and is thrown away when
/// empty.
///
/// Original: combatchoice
pub(crate) fn empties_discarded(state: &Session, actor: usize) -> Result<bool, RuleError> {
    let item = state
        .globals
        .loaded_items
        .item
        .as_deref()
        .ok_or(RuleError::Unsupported {
            action: "combat.firearm.targeting",
            dependency: "target key before any retained item work",
        })?;
    Ok(
        centurions_content::records::ItemDefinition(item).discarded_when_empty()? != 0
            && state.warriors[actor].record().inventory_quantity(0)? == 0,
    )
}

/// Silently drops the warrior's emptied weapon.
///
/// Original: combatchoice, dropitem
pub(crate) fn drop_empty_weapon(
    state: &mut Session,
    content: &Content,
    actor: usize,
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    let item = state.globals.item_used;
    state.load_item(content, item)?;
    let map = state.land(content).ok_or("firing map missing")?;
    let warrior = state
        .warriors
        .get_mut(actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?;
    let (loaded, globals) = crate::equipment::drop_loaded(warrior, 0, item, content, map, events)?;
    state.apply_equipment_globals(&globals)?;
    state.load_item(content, loaded)
}

impl Selection {
    /// The targets picked so far.
    pub fn count(&self) -> usize {
        self.targets.len() + self.locations.len()
    }

    /// Checks the selection belongs to the active warrior, holds either targets or locations within
    /// the limits, and names valid bodies and cells.
    pub fn validate(&self, combat: &Combat, warriors: &[Warrior]) -> Result<(), RuleError> {
        if self.actor >= warriors.len()
            || combat.active_body() != Some(self.actor as i16)
            || self.targets.is_empty() == self.locations.is_empty()
            || self.targets.len() > 8
            || self.locations.len() > 7
            || self.locations.iter().any(|location| {
                centurions_content::mission::OpeningMission::index(
                    i32::from(location.0) + i32::from(combat.field.map_shift.0) + 5,
                    i32::from(location.1) + i32::from(combat.field.map_shift.1) + 5,
                )
                .is_err()
            })
            || self.original_first_item.len() != 22
            || warriors[self.actor].record().equipped_weapon()? != self.weapon
            || self.targets.iter().any(|body| {
                *body < 0
                    || (*body >= warriors.len() as i16 && *body < 10)
                    || *body >= combat.roster.records.len() as i16 + 10
            })
        {
            return Err(RuleError::Invalid("invalid firearm selection".into()));
        }
        Ok(())
    }
}

/// Prepares the equipped firearm's shot. A firearm with no ammunition is refused, and a quantity
/// already chosen sets the number of shots.
pub(crate) fn prepare(
    state: &Session,
    content: &Content,
) -> Result<firearm::PreparedFirearm, RuleError> {
    let combat = state
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let body = combat
        .active_body()
        .filter(|b| (0..6).contains(b))
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?
        as usize;
    let warrior = &state.warriors[body];
    let id = warrior.record().equipped_weapon()?;
    let ammunition = warrior
        .inventory()?
        .iter()
        .find(|item| item.id() == id && item.equipped() == 1)
        .map(|item| item.ammunition_id())
        .transpose()?
        .ok_or_else(|| RuleError::Invalid("equipped firearm absent from inventory".into()))?;
    if ammunition == 0 {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Combat {
                kind: crate::feedback::CombatRefusal::NotEnoughAmmo,
            }],
        });
    }
    let weapon = content
        .items
        .get(id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("firearm definition missing".into()))?;
    let ammo = content
        .items
        .get(ammunition.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("ammunition definition missing".into()))?;
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat content missing".into()))?
        .attacks;
    let mut prepared = firearm::prepare(warrior, weapon, ammo, attacks)?;
    if let Some(quantity) = &state.target_quantity {
        quantity.validate(combat, &state.warriors)?;
        prepared.shots = quantity.power;
    }
    Ok(prepared)
}

/// The most targets a shot may pick: the loaded weapon's attack count.
///
/// Original: combatchoice
pub(crate) fn ceiling(state: &Session, content: &Content) -> Result<i16, RuleError> {
    if let Some(quantity) = &state.target_quantity {
        return Ok(quantity.num_of_targets());
    }
    let body = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..state.warriors.len() as i16).contains(body))
        .ok_or("active warrior missing")?;
    let id = state.warriors[body as usize].record().equipped_weapon()?;
    let weapon = content
        .items
        .get(id.unsigned_abs() as usize)
        .ok_or("firearm definition missing")?;
    Ok(centurions_content::records::ItemDefinition(weapon).attack_count()?)
}

/// Whether the prepared shot targets a cell rather than bodies.
pub fn location_targeting(state: &Session, content: &Content) -> bool {
    prepare(state, content)
        .is_ok_and(|prepared| SpellInfoRecord(&prepared.packet).requires_area_mask())
}

/// Checks that the body is a living target in range and in sight, that the warrior has two attacks
/// left, and that the ammunition is there unless it is already paid for.
pub(crate) fn admit(
    state: &Session,
    content: &Content,
    target: i16,
    p: &firearm::PreparedFirearm,
    paid: bool,
) -> Result<(), RuleError> {
    let combat = state
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let actor = combat
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    let end = if (0..state.warriors.len() as i16).contains(&target)
        && state.warriors[target as usize].stamina()? > 0
    {
        combat.squad.pos[target as usize]
    } else if (10..10 + combat.roster.records.len() as i16).contains(&target)
        && combat.roster.records[(target - 10) as usize].stamina()? > 0
    {
        combat.roster.mon_pos[(target - 10) as usize]
    } else {
        return Err(RuleError::Invalid(
            "firearm target does not identify a living combatant".into(),
        ));
    };
    let start = combat.squad.pos[actor as usize];
    let dx = f32::from(start.0 - end.0);
    let dy = f32::from(start.1 - end.1);
    let refused = |kind| RuleError::Refused {
        feedback: vec![crate::feedback::GameFeedback::Combat { kind }],
    };
    if (dx * dx + dy * dy).sqrt().trunc() as i16 > player_psi::range(&p.packet, p.shots)? {
        return Err(refused(crate::feedback::CombatRefusal::OutOfRange));
    }
    let mission = state.land(content).unwrap();
    if !lifecycle::can_see(combat, mission, start, end, 0)? {
        return Err(refused(crate::feedback::CombatRefusal::Blocked));
    }
    let warrior = &state.warriors[actor as usize];
    if warrior.record().action_points()? < 2 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if !paid && warrior.record().inventory_quantity(p.slot)? < p.rounds {
        return Err(RuleError::Refused {
            feedback: vec![
                crate::feedback::GameFeedback::Tutorial {
                    first: 78,
                    last: 78,
                },
                crate::feedback::GameFeedback::Combat {
                    kind: crate::feedback::CombatRefusal::NotEnoughAmmo,
                },
            ],
        });
    }
    Ok(())
}

/// The same checks for a cell: the shot must take a location, and the cell be in range and, if the
/// shot needs it, in sight.
pub(crate) fn admit_location(
    state: &Session,
    content: &Content,
    location: (i16, i16),
    prepared: &firearm::PreparedFirearm,
) -> Result<(), RuleError> {
    let combat = state.combat.as_ref().ok_or("combat missing")?;
    let actor = combat
        .active_body()
        .filter(|body| (0..6).contains(body))
        .ok_or("active warrior missing")? as usize;
    let point = (
        i32::from(location.0) + i32::from(combat.field.map_shift.0) + 5,
        i32::from(location.1) + i32::from(combat.field.map_shift.1) + 5,
    );
    centurions_content::mission::OpeningMission::index(point.0, point.1)?;
    if !SpellInfoRecord(&prepared.packet).requires_area_mask() {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Combat {
                kind: crate::feedback::CombatRefusal::InvalidTarget,
            }],
        });
    }
    let start = combat.squad.pos[actor];
    let dx = f32::from(start.0 - location.0);
    let dy = f32::from(start.1 - location.1);
    let packet = SpellInfoRecord(&prepared.packet);
    let kind = if (dx * dx + dy * dy).sqrt().trunc() as i16
        > player_psi::range(&prepared.packet, prepared.shots)?
    {
        Some(crate::feedback::CombatRefusal::OutOfRange)
    } else if packet.range_base() + packet.range_per_power() > 0 {
        // Line of sight is tested only when packet bytes 2 and 3 sum above 0.
        let map = state.land(content).ok_or("combat map unavailable")?;
        (!lifecycle::can_see(combat, map, start, location, 0)?)
            .then_some(crate::feedback::CombatRefusal::Blocked)
    } else {
        None
    };
    if let Some(kind) = kind {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Combat { kind }],
        });
    }
    admit(state, content, actor as i16, prepared, false)
}

/// The location target key, with its refusals as feedback events.
pub fn fire_at(
    state: &mut Session,
    content: &Content,
    location: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    let mut events = match select_location(state, content, location) {
        Err(RuleError::Refused { feedback }) => {
            if feedback.is_empty() {
                vec![Event::Feedback(crate::feedback::GameFeedback::Silent)]
            } else {
                feedback.into_iter().map(Event::Feedback).collect()
            }
        }
        result => result?,
    };
    events.insert(0, Event::Feedback(TARGET_KEY_TUTORIAL));
    Ok(events)
}

/// Fires the shot at the cell at once and finishes the warrior's action.
fn fire_at_once(
    state: &mut Session,
    content: &Content,
    location: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    let actor = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .ok_or("active warrior missing")?;
    let mut events = runtime::player_shoot_at_action(state, content, location)?;
    events.splice(
        0..0,
        [
            Event::Feedback(LOCATION_TUTORIAL),
            Event::Sound { resource_id: 145 },
        ],
    );
    events.extend(runtime::finish_player_action(state, content, actor)?);
    Ok(events)
}

/// Picks a body as the target of the firearm shot. A single-target shot fires at once; a shot that
/// takes a location picks the body's cell; otherwise the body is added to the targets, up to the
/// weapon's attack count.
pub fn select(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    let prepared = prepare(state, content)?;
    admit(state, content, target, &prepared, false)?;
    state.set_in_spell(1);
    // Picking adds bodies up to the ceiling for packet byte 1 below 2, and firing
    // shoots them. A single-target shot picks and fires at once.
    let ceiling = ceiling(state, content)?;
    let selecting = state
        .combat
        .as_ref()
        .is_some_and(|combat| combat.firearm_selection.is_some());
    if SpellInfoRecord(&prepared.packet).requires_area_mask() && selecting {
        // Above packet byte 1 of 1 the pick takes the cell, not the body.
        let combat = state.combat.as_ref().unwrap();
        let location = combat.body_position(target);
        return select_location(state, content, location);
    }
    if SpellInfoRecord(&prepared.packet).requires_area_mask() {
        // The cell's targeting sound follows its refusals.
        let mut events = runtime::player_shoot_immediate(state, content, target)?;
        if matches!(events.first(), Some(Event::Sound { resource_id: 145 })) {
            events.insert(0, Event::Feedback(LOCATION_TUTORIAL));
        }
        return Ok(events);
    }
    if !selecting && ceiling < 2 {
        unwritten_friendly(&prepared.packet, &[target])?;
        return runtime::player_shoot_immediate(state, content, target);
    }
    let combat = state.combat.as_mut().unwrap();
    combat.in_spell = 1;
    let actor = combat.active_body().unwrap() as usize;
    let warrior = &mut state.warriors[actor];
    let selection = combat.firearm_selection.get_or_insert_with(|| Selection {
        actor,
        weapon: warrior.record().equipped_weapon().unwrap(),
        targets: vec![],
        original_first_item: warrior.record().inventory_bytes().to_vec(),
        locations: vec![],
        discard_empty: false,
    });
    if selection.targets.len() >= ceiling.max(0) as usize {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    selection.targets.push(target);
    if selection.targets.len() == 8 {
        overwrite_with_eighth_body(&mut combat.spell_info, target);
        overwrite_with_eighth_body(&mut state.globals.spell_info, target);
    }
    let slot = prepared.slot;
    let loaded = warrior
        .record()
        .inventory_quantity(slot)?
        .wrapping_sub(prepared.rounds);
    warrior.record_mut().set_inventory_quantity(slot, loaded);
    // The load adjusts as a signed word, apart from any inventory rollback.
    let weight = warrior
        .record()
        .load()?
        .wrapping_sub(prepared.rounds.wrapping_mul(prepared.weight));
    warrior.record_mut().set_load(weight);
    // Only an enemy picked in firing mode sets the discard flag.
    if target >= 10 && warrior.record().toggle() && empties_discarded(state, actor)? {
        let combat = state.combat.as_mut().unwrap();
        combat.firearm_selection.as_mut().unwrap().discard_empty = true;
    }
    Ok(vec![Event::Sound { resource_id: 145 }])
}

/// An eighth target overwrites the start of the attack packet, which follows the
/// target list (see docs/source-notes/combat.md). A friendly target is written
/// only while packet byte 31 is at most 1; an enemy always.
///
/// Original: combatchoice
pub(crate) fn overwrite_with_eighth_body(packet: &mut [u8], target: i16) {
    let kind: i16 = if target < 10 { 1 } else { 2 };
    if packet.len() > 31 && (kind == 2 || packet[31] <= 1) {
        packet[2..4].copy_from_slice(&kind.to_be_bytes());
    }
}

/// Picks a cell for an area attack (packet byte 1 above 1): range, then line of
/// sight when packet bytes 2 and 3 sum above 0, then the weapon's rounds, then the
/// cell while the ceiling allows, paid at once. A single-target shot also fires,
/// as `select` does for bodies.
///
/// Original: combatchoice
pub fn select_location(
    state: &mut Session,
    content: &Content,
    location: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    let prepared = prepare(state, content)?;
    let ceiling = ceiling(state, content)?;
    let combat = state.combat.as_ref().ok_or("combat missing")?;
    let selected = combat
        .firearm_selection
        .as_ref()
        .map_or(0, Selection::count);
    if selected == 0 && ceiling < 2 {
        return fire_at_once(state, content, location);
    }
    if combat
        .firearm_selection
        .as_ref()
        .is_some_and(|selection| !selection.targets.is_empty())
    {
        return Err(RuleError::Invalid(
            "firearm selection already holds bodies".into(),
        ));
    }
    admit_location(state, content, location, &prepared)?;
    if selected >= ceiling.max(0) as usize {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Combat {
                kind: crate::feedback::CombatRefusal::TooManyTargets,
            }],
        });
    }
    if selected >= 7 {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.targeting",
            dependency: "eighth location triple overwrites the packet targeting mode",
        });
    }
    state.set_in_spell(1);
    let combat = state.combat.as_mut().unwrap();
    let actor = combat.active_body().unwrap() as usize;
    let warrior = &mut state.warriors[actor];
    let selection = combat.firearm_selection.get_or_insert_with(|| Selection {
        actor,
        weapon: warrior.record().equipped_weapon().unwrap(),
        targets: vec![],
        original_first_item: warrior.record().inventory_bytes().to_vec(),
        locations: vec![],
        discard_empty: false,
    });
    selection.locations.push(location);
    let loaded = warrior
        .record()
        .inventory_quantity(prepared.slot)?
        .wrapping_sub(prepared.rounds);
    warrior
        .record_mut()
        .set_inventory_quantity(prepared.slot, loaded);
    let weight = warrior
        .record()
        .load()?
        .wrapping_sub(prepared.rounds.wrapping_mul(prepared.weight));
    warrior.record_mut().set_load(weight);
    if empties_discarded(state, actor)? {
        let combat = state.combat.as_mut().unwrap();
        combat.firearm_selection.as_mut().unwrap().discard_empty = true;
    }
    Ok(vec![
        Event::Feedback(LOCATION_TUTORIAL),
        Event::Sound { resource_id: 145 },
    ])
}

/// Fires at everything picked so far, then finishes the warrior's action.
pub fn fire_selected(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let selection = state
        .combat
        .as_ref()
        .and_then(|c| c.firearm_selection.clone())
        .ok_or_else(|| RuleError::Invalid("no selected firearm targets".into()))?;
    unwritten_friendly(&prepare(state, content)?.packet, &selection.targets)?;
    let discard = Discard::Collected(selection.discard_empty);
    let mut events = if selection.locations.is_empty() {
        runtime::player_shoot_targets_action(state, content, &selection.targets, true, discard)?
    } else {
        runtime::player_shoot_locations_action(state, content, &selection.locations, discard)?
    };
    state.combat.as_mut().unwrap().firearm_selection = None;
    events.extend(runtime::finish_player_action(
        state,
        content,
        selection.actor as i16,
    )?);
    Ok(events)
}

/// Abandons the picking and puts the warrior's weapon back as it was. After the target key has set
/// the discard flag this is not supported.
pub fn cancel(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    if state
        .combat
        .as_ref()
        .and_then(|c| c.firearm_selection.as_ref())
        .is_some_and(|selection| selection.discard_empty)
    {
        // Cancelling keeps the discard flag, so a later shot would drop the
        // restored weapon.
        return Err(RuleError::Unsupported {
            action: "combat.firearm.targeting",
            dependency: "cancel after the target key set the discard flag",
        });
    }
    state.set_in_spell(0);
    state.target_quantity = None;
    if let Some(selection) = state
        .combat
        .as_mut()
        .and_then(|c| c.firearm_selection.take())
    {
        // Cancelling restores the first inventory slot, not the separate load.
        state.warriors[selection.actor]
            .record_mut()
            .restore_inventory_bytes(&selection.original_first_item);
    }
    Ok(vec![])
}
