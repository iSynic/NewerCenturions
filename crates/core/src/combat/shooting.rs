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
}

impl Selection {
    pub fn validate(&self, combat: &Combat, warriors: &[Warrior]) -> Result<(), RuleError> {
        if self.actor >= warriors.len()
            || combat.active_body() != Some(self.actor as i16)
            || self.targets.is_empty()
            || self.targets.len() > 7
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
    let mission = content
        .map(state.exploration.as_ref().map_or(0, |p| p.map))
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
    let mission = content
        .map(state.exploration.as_ref().map_or(0, |p| p.map))
        .unwrap();
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
    let kind = if (dx * dx + dy * dy).sqrt().trunc() as i16
        > player_psi::range(&prepared.packet, prepared.shots)?
    {
        Some(crate::feedback::CombatRefusal::OutOfRange)
    } else {
        let map = state
            .map(content, state.exploration.as_ref().map_or(0, |p| p.map))
            .ok_or("combat map unavailable")?;
        (!lifecycle::can_see(c, map, start, location, 0)?)
            .then_some(crate::feedback::CombatRefusal::Blocked)
    };
    if let Some(kind) = kind {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Combat { kind }],
        });
    }
    admit(state, content, actor as i16, prepared, false)
}

pub fn fire_at(
    state: &mut Session,
    content: &Content,
    location: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    let result = (|| {
        let actor = state
            .combat
            .as_ref()
            .and_then(Combat::active_body)
            .ok_or("active warrior missing")?;
        let mut events = runtime::player_shoot_at_action(state, content, location)?;
        events.insert(0, Event::Sound { resource_id: 145 });
        events.extend(runtime::finish_player_action(state, content, actor)?);
        Ok(events)
    })();
    match result {
        Err(RuleError::Refused { feedback }) => Ok(if feedback.is_empty() {
            vec![Event::Feedback(crate::feedback::NativeFeedback::Silent)]
        } else {
            feedback.into_iter().map(Event::Feedback).collect()
        }),
        result => result,
    }
}

pub fn select(
    state: &mut Session,
    content: &Content,
    target: i16,
) -> Result<Vec<Event>, RuleError> {
    let p = prepare(state, content)?;
    admit(state, content, target, &p, false)?;
    if !AttackPacket(&p.packet).divides_shots_between_targets() || !(2..=7).contains(&p.shots) {
        return runtime::player_shoot_immediate(state, content, target);
    }
    let c = state.combat.as_mut().unwrap();
    let actor = c.active_body().unwrap() as usize;
    let warrior = &mut state.warriors[actor];
    let selection = c.firearm_selection.get_or_insert_with(|| Selection {
        actor,
        weapon: warrior.record().equipped_weapon().unwrap(),
        targets: vec![],
        original_first_item: warrior.record().first_inventory_work().to_vec(),
    });
    if selection.targets.len() >= p.shots as usize {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    selection.targets.push(target);
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
    Ok(vec![Event::Sound { resource_id: 145 }])
}

pub fn fire_selected(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let selection = state
        .combat
        .as_ref()
        .and_then(|c| c.firearm_selection.clone())
        .ok_or_else(|| RuleError::Invalid("no selected firearm targets".into()))?;
    let mut events =
        runtime::player_shoot_targets_action(state, content, &selection.targets, true)?;
    state.combat.as_mut().unwrap().firearm_selection = None;
    events.extend(runtime::finish_player_action(
        state,
        content,
        selection.actor as i16,
    )?);
    Ok(events)
}

pub fn cancel(state: &mut Session) -> Result<Vec<Event>, RuleError> {
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
