use crate::combat::{Combat, firearm, lifecycle, player_psi, runtime};
use crate::{Event, RuleError, Session};
use centurions_content::records::AttackPacket;
use centurions_content::{Content, Warrior};
use serde::{Deserialize, Serialize};

/// Native combatchoice retains up to seven targets and charges ammunition as each is selected.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Selection {
    pub actor: usize,
    pub weapon: i16,
    pub targets: Vec<i16>,
    pub original_first_item: Vec<u8>,
    /// Kind-3 location triples the target key collects for a packet byte 1 above 1
    /// (combatchoice 11b13c..11b1b8), as combat positions. Bodies stay in `targets`;
    /// one selection never holds both.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<(i16, i16)>,
    /// combatchoice's local BYTE at -0x19: an enemy key in firing mode (11b462..11b482)
    /// or a location key (11b218..11b228) left slot 0 empty for an ITEM work WORD 0x26
    /// item, so the fire key drops it first (11ae08..11ae1c).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub discard_empty: bool,
}

/// How `player_shoot_action_at` reaches combatchoice's fire-key dropitem.
#[derive(Clone, Copy)]
pub(crate) enum Discard {
    /// Not a target-key shot.
    Never,
    /// The target key and fire key at once: the key's own test after its charge.
    AfterLoad,
    /// The fire key over collected triples, with the keys' flag.
    Collected(bool),
}

const TARGET_KEY_TUTORIAL: crate::feedback::NativeFeedback =
    crate::feedback::NativeFeedback::Tutorial {
        first: 76,
        last: 77,
    };
const LOCATION_TUTORIAL: crate::feedback::NativeFeedback =
    crate::feedback::NativeFeedback::Tutorial {
        first: 81,
        last: 81,
    };

/// The target key on a body: firstt(76, 77) (combatchoice 11b026) precedes its range
/// test and every later refusal.
pub fn target_key(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    let mut events = runtime::player_shoot(state, content, target)?;
    events.insert(0, Event::Feedback(TARGET_KEY_TUTORIAL));
    Ok(events)
}

/// A friendly triple stays unwritten above packet byte 31 of 1 unsigned (11b300), so cast
/// meets the slot's earlier triple, normally cleartarget's (-1, -1, -1): its location arm
/// at an off-map cell and spelltargets' warrior position -1.
fn unwritten_friendly(packet: &[u8], targets: &[i16]) -> Result<(), RuleError> {
    if packet.get(31).is_some_and(|byte| *byte > 1) && targets.iter().any(|body| *body < 10) {
        return Err(RuleError::Unsupported {
            action: "combat.firearm.targeting",
            dependency: "friendly target triple left unwritten above packet byte 31 of 1",
        });
    }
    Ok(())
}

/// The target key's discard test (11b218..11b228, 11b462..11b482) after its charge:
/// slot 0 is empty and ITEM work WORD 0x26 is set.
pub(crate) fn empties_discarded(state: &Session, actor: usize) -> Result<bool, RuleError> {
    let item = state
        .native_work
        .ammunition_work
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

/// combatchoice 11ae0e..11ae1c: dropitem(WORD4435d2, WORD44719a, 0), which loads
/// WORD44719a and drops slot 0 silently.
pub(crate) fn drop_empty_weapon(
    state: &mut Session,
    content: &Content,
    actor: usize,
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    let item = state.native_work.fire_item;
    state.load_item_work(content, item)?;
    let map = state.land(content).ok_or("firing map missing")?;
    let warrior = state
        .warriors
        .get_mut(actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?;
    let (loaded, globals) = crate::equipment::drop_loaded(warrior, 0, item, content, map, events)?;
    state.apply_equipment_globals(&globals)?;
    state.load_item_work(content, loaded)
}

impl Selection {
    /// WORD4435c0, the triples collected so far.
    pub fn count(&self) -> usize {
        self.targets.len() + self.locations.len()
    }

    pub fn validate(&self, combat: &Combat, warriors: &[Warrior]) -> Result<(), RuleError> {
        if self.actor >= warriors.len()
            || combat.active_body() != Some(self.actor as i16)
            || self.targets.is_empty() == self.locations.is_empty()
            || self.targets.len() > 8
            || self.locations.len() > 7
            || self.locations.iter().any(|location| {
                centurions_content::mission::OpeningMission::index(
                    i32::from(location.0) + i32::from(combat.map.shift.0) + 5,
                    i32::from(location.1) + i32::from(combat.map.shift.1) + 5,
                )
                .is_err()
            })
            || self.original_first_item.len() != 22
            || warriors[self.actor].record().equipped_weapon()? != self.weapon
            || self.targets.iter().any(|body| {
                *body < 0
                    || (*body >= warriors.len() as i16 && *body < 10)
                    || *body >= combat.enemies.records.len() as i16 + 10
            })
        {
            return Err(RuleError::Invalid("invalid firearm selection".into()));
        }
        Ok(())
    }
}

pub(crate) fn prepare(
    state: &Session,
    content: &Content,
) -> Result<firearm::PreparedFirearm, RuleError> {
    let c = state
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let body =
        c.active_body()
            .filter(|b| (0..6).contains(b))
            .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))? as usize;
    let warrior = &state.warriors[body];
    let id = warrior.record().equipped_weapon()?;
    let ammunition = warrior
        .inventory()?
        .iter()
        .find(|item| item.id() == id && item.equipment_flag() == 1)
        .map(|item| item.ammunition_id())
        .transpose()?
        .ok_or_else(|| RuleError::Invalid("equipped firearm absent from inventory".into()))?;
    if ammunition == 0 {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Combat {
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
        quantity.validate(c, &state.warriors)?;
        prepared.shots = quantity.power;
    }
    Ok(prepared)
}

/// WORD4435be, combatchoice's target ceiling. The fire button (118932) stores WEAPON
/// work +0x4c, the whole item WORD 76; without a fire-button entry it is that WORD.
pub(crate) fn ceiling(state: &Session, content: &Content) -> Result<i16, RuleError> {
    if let Some(quantity) = &state.target_quantity {
        return Ok(quantity.target_ceiling());
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

pub fn location_targeting(state: &Session, content: &Content) -> bool {
    prepare(state, content)
        .is_ok_and(|prepared| AttackPacket(&prepared.packet).requires_area_mask())
}

pub(crate) fn admit(
    state: &Session,
    content: &Content,
    target: i16,
    p: &firearm::PreparedFirearm,
    paid: bool,
) -> Result<(), RuleError> {
    let c = state
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    let actor = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active warrior missing".into()))?;
    let end = if (0..state.warriors.len() as i16).contains(&target)
        && state.warriors[target as usize].stamina()? > 0
    {
        c.squad.positions[target as usize]
    } else if (10..10 + c.enemies.records.len() as i16).contains(&target)
        && c.enemies.records[(target - 10) as usize].stamina()? > 0
    {
        c.enemies.positions[(target - 10) as usize]
    } else {
        return Err(RuleError::Invalid(
            "firearm target does not identify a living combatant".into(),
        ));
    };
    let start = c.squad.positions[actor as usize];
    let dx = f32::from(start.0 - end.0);
    let dy = f32::from(start.1 - end.1);
    let refused = |kind| RuleError::Refused {
        feedback: vec![crate::feedback::NativeFeedback::Combat { kind }],
    };
    if (dx * dx + dy * dy).sqrt().trunc() as i16 > player_psi::range(&p.packet, p.shots)? {
        return Err(refused(crate::feedback::CombatRefusal::OutOfRange));
    }
    let mission = state.land(content).unwrap();
    if !lifecycle::can_see(c, mission, start, end, 0)? {
        return Err(refused(crate::feedback::CombatRefusal::Blocked));
    }
    let warrior = &state.warriors[actor as usize];
    if warrior.record().action_points()? < 2 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if !paid && warrior.record().inventory_quantity(p.slot)? < p.rounds {
        return Err(RuleError::Refused {
            feedback: vec![
                crate::feedback::NativeFeedback::Tutorial {
                    first: 78,
                    last: 78,
                },
                crate::feedback::NativeFeedback::Combat {
                    kind: crate::feedback::CombatRefusal::NotEnoughAmmo,
                },
            ],
        });
    }
    Ok(())
}

pub(crate) fn admit_location(
    state: &Session,
    content: &Content,
    location: (i16, i16),
    prepared: &firearm::PreparedFirearm,
) -> Result<(), RuleError> {
    let c = state.combat.as_ref().ok_or("combat missing")?;
    let actor = c
        .active_body()
        .filter(|body| (0..6).contains(body))
        .ok_or("active warrior missing")? as usize;
    let point = (
        i32::from(location.0) + i32::from(c.map.shift.0) + 5,
        i32::from(location.1) + i32::from(c.map.shift.1) + 5,
    );
    centurions_content::mission::OpeningMission::index(point.0, point.1)?;
    if !AttackPacket(&prepared.packet).requires_area_mask() {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Combat {
                kind: crate::feedback::CombatRefusal::InvalidTarget,
            }],
        });
    }
    let start = c.squad.positions[actor];
    let dx = f32::from(start.0 - location.0);
    let dy = f32::from(start.1 - location.1);
    let packet = AttackPacket(&prepared.packet);
    let kind = if (dx * dx + dy * dy).sqrt().trunc() as i16
        > player_psi::range(&prepared.packet, prepared.shots)?
    {
        Some(crate::feedback::CombatRefusal::OutOfRange)
    } else if packet.base_range() + packet.range_per_power() > 0 {
        // 11b090..11b0ea: cansee only when packet bytes 2 and 3 sum above 0.
        let map = state.land(content).ok_or("combat map unavailable")?;
        (!lifecycle::can_see(c, map, start, location, 0)?)
            .then_some(crate::feedback::CombatRefusal::Blocked)
    } else {
        None
    };
    if let Some(kind) = kind {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Combat { kind }],
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
                vec![Event::Feedback(crate::feedback::NativeFeedback::Silent)]
            } else {
                feedback.into_iter().map(Event::Feedback).collect()
            }
        }
        result => result?,
    };
    events.insert(0, Event::Feedback(TARGET_KEY_TUTORIAL));
    Ok(events)
}

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

pub fn select(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    let p = prepare(state, content)?;
    admit(state, content, target, &p, false)?;
    state.set_ready(1);
    // The target key adds bodies while WORD4435c0 < WORD4435be (11b2f6, 11b3d8) for
    // any packet byte 1 below 2; the fire key casts them (11ae08). A single-target
    // ceiling takes both keys at once.
    let ceiling = ceiling(state, content)?;
    let selecting = state
        .combat
        .as_ref()
        .is_some_and(|c| c.firearm_selection.is_some());
    if AttackPacket(&p.packet).requires_area_mask() && selecting {
        // 11b13c: above packet byte 1 of 1 the target key takes the cell, not the body.
        let c = state.combat.as_ref().unwrap();
        let location = if target < 10 {
            c.squad.positions[target as usize]
        } else {
            c.enemies.positions[(target - 10) as usize]
        };
        return select_location(state, content, location);
    }
    if AttackPacket(&p.packet).requires_area_mask() {
        // The location key's firstt(81, 81) (11b158) follows its refusals.
        let mut events = runtime::player_shoot_immediate(state, content, target)?;
        if matches!(events.first(), Some(Event::Sound { resource_id: 145 })) {
            events.insert(0, Event::Feedback(LOCATION_TUTORIAL));
        }
        return Ok(events);
    }
    if !selecting && ceiling < 2 {
        unwritten_friendly(&p.packet, &[target])?;
        return runtime::player_shoot_immediate(state, content, target);
    }
    let c = state.combat.as_mut().unwrap();
    c.ready = 1;
    let actor = c.active_body().unwrap() as usize;
    let warrior = &mut state.warriors[actor];
    let selection = c.firearm_selection.get_or_insert_with(|| Selection {
        actor,
        weapon: warrior.record().equipped_weapon().unwrap(),
        targets: vec![],
        original_first_item: warrior.record().first_inventory_work().to_vec(),
        locations: vec![],
        discard_empty: false,
    });
    if selection.targets.len() >= ceiling.max(0) as usize {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    selection.targets.push(target);
    if selection.targets.len() == 8 {
        overwrite_with_eighth_body(&mut c.attack_packet, target);
        overwrite_with_eighth_body(&mut state.native_work.attack_packet, target);
    }
    let slot = p.slot;
    let loaded = warrior
        .record()
        .inventory_quantity(slot)?
        .wrapping_sub(p.rounds);
    warrior.record_mut().set_inventory_quantity(slot, loaded);
    // Preserve the native signed-word load adjustment, independently of inventory rollback.
    let weight = warrior
        .record()
        .carried_weight()?
        .wrapping_sub(p.rounds.wrapping_mul(p.weight));
    warrior.record_mut().set_carried_weight(weight);
    // Only the enemy arm in firing mode sets the discard flag (11b41a..11b482).
    if target >= 10 && warrior.record().firing_mode() && empties_discarded(state, actor)? {
        let c = state.combat.as_mut().unwrap();
        c.firearm_selection.as_mut().unwrap().discard_empty = true;
    }
    Ok(vec![Event::Sound { resource_id: 145 }])
}

/// Triple 7 sits at 0x44374c, so its kind WORD (0x443750) is attack packet bytes 2..3:
/// base range 0 and a range per power of the kind. The target key writes a friendly
/// triple only while packet byte 31 is at most 1 unsigned (11b300); enemies always
/// (11b3f2..11b40a). Body triples leave their second WORD, packet bytes 0..1, alone.
pub(crate) fn overwrite_with_eighth_body(packet: &mut [u8], target: i16) {
    let kind: i16 = if target < 10 { 1 } else { 2 };
    if packet.len() > 31 && (kind == 2 || packet[31] <= 1) {
        packet[2..4].copy_from_slice(&kind.to_be_bytes());
    }
}

/// The target key at a location for a packet byte 1 above 1 (combatchoice
/// 11b026..11b27a): range (11b046), sight only when packet bytes 2 and 3 sum above 0
/// (11b090), WEAPON work's rounds (11b102), then a kind-3 triple while WORD4435c0 is
/// below WORD4435be (11b14e), charged at once. A single-target ceiling takes the fire
/// key as well, as `select` does for bodies.
pub fn select_location(
    state: &mut Session,
    content: &Content,
    location: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    let p = prepare(state, content)?;
    let ceiling = ceiling(state, content)?;
    let c = state.combat.as_ref().ok_or("combat missing")?;
    let selected = c.firearm_selection.as_ref().map_or(0, Selection::count);
    if selected == 0 && ceiling < 2 {
        return fire_at_once(state, content, location);
    }
    if c.firearm_selection
        .as_ref()
        .is_some_and(|selection| !selection.targets.is_empty())
    {
        return Err(RuleError::Invalid(
            "firearm selection already holds bodies".into(),
        ));
    }
    admit_location(state, content, location, &p)?;
    if selected >= ceiling.max(0) as usize {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Combat {
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
    state.set_ready(1);
    let c = state.combat.as_mut().unwrap();
    let actor = c.active_body().unwrap() as usize;
    let warrior = &mut state.warriors[actor];
    let selection = c.firearm_selection.get_or_insert_with(|| Selection {
        actor,
        weapon: warrior.record().equipped_weapon().unwrap(),
        targets: vec![],
        original_first_item: warrior.record().first_inventory_work().to_vec(),
        locations: vec![],
        discard_empty: false,
    });
    selection.locations.push(location);
    let loaded = warrior
        .record()
        .inventory_quantity(p.slot)?
        .wrapping_sub(p.rounds);
    warrior.record_mut().set_inventory_quantity(p.slot, loaded);
    let weight = warrior
        .record()
        .carried_weight()?
        .wrapping_sub(p.rounds.wrapping_mul(p.weight));
    warrior.record_mut().set_carried_weight(weight);
    if empties_discarded(state, actor)? {
        let c = state.combat.as_mut().unwrap();
        c.firearm_selection.as_mut().unwrap().discard_empty = true;
    }
    Ok(vec![
        Event::Feedback(LOCATION_TUTORIAL),
        Event::Sound { resource_id: 145 },
    ])
}

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

pub fn cancel(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    if state
        .combat
        .as_ref()
        .and_then(|c| c.firearm_selection.as_ref())
        .is_some_and(|selection| selection.discard_empty)
    {
        // Cancel (11acd8..11ad06) keeps the local flag, so a later fire key would
        // drop slot 0 after the restored load.
        return Err(RuleError::Unsupported {
            action: "combat.firearm.targeting",
            dependency: "cancel after the target key set the discard flag",
        });
    }
    state.set_ready(0);
    state.target_quantity = None;
    if let Some(selection) = state
        .combat
        .as_mut()
        .and_then(|c| c.firearm_selection.take())
    {
        // Cancel restores the native first inventory work record, not the separate load field.
        state.warriors[selection.actor]
            .record_mut()
            .restore_first_inventory_work(&selection.original_first_item);
    }
    Ok(vec![])
}
