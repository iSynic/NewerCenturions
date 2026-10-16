use super::*;

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
            || word(&warriors[self.actor].bytes, 0xe4)? != self.weapon
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
    let id = word(&warrior.bytes, 0xe4)?;
    let ammunition = warrior
        .inventory()?
        .iter()
        .find(|item| item.id() == id && item.0[9] == 1)
        .map(|item| word(&item.0, 4))
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
    firearm::prepare(warrior, weapon, ammo, attacks)
}

pub fn location_targeting(state: &Session, content: &Content) -> bool {
    prepare(state, content).is_ok_and(|prepared| prepared.packet[1] >= 2)
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
    let w = &state.warriors[actor as usize].bytes;
    if word(w, 0x40)? < 2 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if !paid && word(w, 0x134 + 22 * p.slot)? < p.rounds {
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
    if prepared.packet[1] < 2 {
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
    if p.packet[1] != 0 || !(2..=7).contains(&p.shots) {
        return runtime::player_shoot_immediate(state, content, target);
    }
    let c = state.combat.as_mut().unwrap();
    let actor = c.active_body().unwrap() as usize;
    let w = &mut state.warriors[actor].bytes;
    let selection = c.firearm_selection.get_or_insert_with(|| Selection {
        actor,
        weapon: word(w, 0xe4).unwrap(),
        targets: vec![],
        original_first_item: w[0x132..0x148].to_vec(),
    });
    if selection.targets.len() >= p.shots as usize {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    selection.targets.push(target);
    let slot = 0x134 + 22 * p.slot;
    let loaded = word(w, slot)?.wrapping_sub(p.rounds);
    put_word(w, slot, loaded);
    // The original subtracts a WORD from the LONG load field at 0x9d6.
    let weight = word(w, 0x9d6)?.wrapping_sub(p.rounds.wrapping_mul(p.weight));
    put_word(w, 0x9d6, weight);
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
    if let Some(selection) = state
        .combat
        .as_mut()
        .and_then(|c| c.firearm_selection.take())
    {
        // Cancel restores the native first inventory work record, not the separate load field.
        state.warriors[selection.actor].bytes[0x132..0x148]
            .copy_from_slice(&selection.original_first_item);
    }
    Ok(vec![])
}
