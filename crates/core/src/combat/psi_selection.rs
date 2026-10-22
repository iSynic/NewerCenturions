use crate::combat::{player_psi, runtime, spells};
use crate::{Event, Phase, PsiMenuChoice, RuleError, Session};
use centurions_content::mission::OpeningMission;
use centurions_content::{
    Content,
    records::{AttackPacket, CharacterStatisticsMut, Statistic},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Selection {
    pub actor: usize,
    pub spell: i16,
    pub power: i16,
    pub targets: Vec<i16>,
}

impl Selection {
    pub fn validate(&self, state: &Session) -> Result<(), RuleError> {
        let combat = state.combat.as_ref().ok_or("pending Psi combat missing")?;
        if state.phase != Phase::Combat
            || combat.active_body() != Some(self.actor as i16)
            || self.actor >= state.warriors.len()
            || !(1..=7).contains(&self.power)
            || self.targets.len() > self.power as usize
            || !player_psi::COMBAT_SPELLS.contains(&self.spell)
            || state.medical_continuation.is_some()
            || combat.firearm_selection.is_some()
            || combat.attack_identity
                != Some([
                    self.spell / 1000 - 1,
                    self.spell % 1000 / 100 - 1,
                    self.spell % 100 - 1,
                ])
            || combat.attack_packet.as_ref().is_none_or(|packet| {
                packet.len() != 58
                    || AttackPacket(packet).targeting_mode() > 6
                    || AttackPacket(packet).targeting_mode() != 0 && !self.targets.is_empty()
            })
            || self.targets.iter().any(|body| {
                *body < 0
                    || (*body >= state.warriors.len() as i16 && *body < 10)
                    || *body >= 10 + combat.enemies.records.len() as i16
            })
        {
            return Err(RuleError::Invalid("invalid pending Psi selection".into()));
        }
        for body in &self.targets {
            let active = if *body < 10 {
                state.warriors[*body as usize].record().active_flag() != 0
            } else {
                combat.enemies.records[(*body - 10) as usize]
                    .record()
                    .stamina()?
                    > 0
            };
            if !active {
                return Err(RuleError::Invalid(
                    "inactive body in pending Psi selection".into(),
                ));
            }
        }
        Ok(())
    }
}

fn mission<'a>(state: &Session, content: &'a Content) -> Result<&'a OpeningMission, RuleError> {
    state
        .map(
            content,
            state
                .exploration
                .as_ref()
                .ok_or("Psi position missing")?
                .map,
        )
        .ok_or_else(|| RuleError::Invalid("Psi map missing".into()))
}

pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    spell: i16,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    let map = mission(state, content)?;
    let packet = spells::load_packet(
        &map.combat.as_ref().ok_or("Psi content missing")?.attacks,
        spell,
    )?;
    if AttackPacket(&packet).targeting_mode() != 0 {
        return Err("Psi selection requires a multi-body spell".into());
    }
    begin_targeting(state, content, spell, power)
}

pub(crate) fn fire_at(
    state: &mut Session,
    content: &Content,
    point: (i16, i16),
    rotation: u8,
) -> Result<Vec<Event>, RuleError> {
    let pending = state.psi_selection.clone().ok_or("no pending Psi target")?;
    pending.validate(state)?;
    if state
        .combat
        .as_ref()
        .unwrap()
        .attack_packet
        .as_ref()
        .unwrap()[1]
        == 0
    {
        return Err("multi-body scan requires selected targets".into());
    }
    if state.native_work.psi_memory.is_none() {
        state.native_work.psi_memory = Some(crate::initial_psi_memory(&state.warriors)?);
    }
    let mut events = player_psi::cast_prepared_at(
        state,
        mission(state, content)?,
        &pending,
        point,
        rotation,
        None,
    )?;
    state.psi_selection = None;
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}

pub(crate) fn fire_warrior(
    state: &mut Session,
    content: &Content,
    warrior: usize,
) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .clone()
        .ok_or("no pending portrait scan")?;
    pending.validate(state)?;
    let combat = state.combat.as_ref().unwrap();
    let packet = combat.attack_packet.as_ref().unwrap();
    // combatchoice119dd2 accepts the healing effect57, not revival effect64.
    if AttackPacket(packet).targeting_mode() != 1 || AttackPacket(packet).effect()? != 57 {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Sound { resource_id: 143 }],
        });
    }
    state.psi_selection = None;
    state.selected_warrior = pending.actor;
    let mut health = crate::story_health::context(
        &state.warriors,
        pending.actor,
        state.native_work.health_context.as_ref(),
    );
    health.affected[pending.actor] = 1;
    health.effect = AttackPacket(packet).health_effect();
    state.native_work.health_context = Some(health);
    state.medical_continuation = Some(crate::MedicalContinuation {
        origin: crate::MedicalOrigin::LearnedHeal,
        actor: pending.actor,
        item: 0,
        spell: pending.spell,
        power: pending.power,
    });
    runtime::refresh_active_item_context(state, content)?;
    runtime::select_medical_target(state, content, crate::MedicalTarget::Portrait { warrior })
}

pub(crate) fn begin_targeting(
    state: &mut Session,
    content: &Content,
    spell: i16,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    if state.psi_selection.is_some()
        || state.medical_continuation.is_some()
        || state
            .combat_story
            .as_ref()
            .is_some_and(|story| !story.waiting_child)
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some())
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let combat = state.combat.as_ref().ok_or("Psi requires combat")?;
    if combat.firearm_selection.is_some() {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let actor = combat
        .active_body()
        .filter(|body| (0..state.warriors.len() as i16).contains(body))
        .ok_or("Psi caster missing")? as usize;
    let map = mission(state, content)?;
    player_psi::validate(
        combat,
        &state.warriors,
        map,
        state.psi_disabled,
        player_psi::CastRequest {
            spell,
            power,
            target: Some(combat.squad.positions[actor]),
            rotation: 0,
        },
    )?;
    let packet = spells::load_packet(
        &map.combat.as_ref().ok_or("Psi content missing")?.attacks,
        spell,
    )?;
    if AttackPacket(&packet).targeting_mode() > 6 {
        return Err("scan has no point-target continuation".into());
    }
    combat.weapon_work.armor_filter()?;
    let w = &mut state.warriors[actor];
    let points = w
        .record()
        .psi_points()?
        .wrapping_add(player_psi::cost(&packet, power)?.wrapping_neg());
    w.record_mut().set_psi_points(points);
    let count = w.record().scans_used()?.wrapping_add(1);
    w.record_mut().set_scans_used(count);
    CharacterStatisticsMut(&mut w.bytes).add(Statistic::ScansCast, 1);
    let identity = [spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1];
    let combat = state.combat.as_mut().unwrap();
    combat.attack_packet = Some(packet);
    combat.attack_identity = Some(identity);
    state.native_work.attack_packet = combat.attack_packet.clone();
    state.native_work.attack_identity = combat.attack_identity;
    state.native_work.weapon_work = combat.weapon_work;
    state.target_quantity = None;
    state.native_work.attack_count = power;
    state.psi_menu_choices[actor] = Some(PsiMenuChoice {
        row: identity[1] as u8,
        column: identity[2] as u8,
        power,
    });
    state.psi_selection = Some(Selection {
        actor,
        spell,
        power,
        targets: vec![],
    });
    Ok(vec![])
}

pub fn admit_target(
    state: &Session,
    content: &Content,
    point: (i16, i16),
) -> Result<i16, RuleError> {
    let pending = state
        .psi_selection
        .as_ref()
        .ok_or("no pending Psi selection")?;
    pending.validate(state)?;
    if pending.targets.len() >= pending.power as usize {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Combat {
                kind: crate::feedback::CombatRefusal::TooManyTargets,
            }],
        });
    }
    let combat = state.combat.as_ref().unwrap();
    let packet = combat.attack_packet.as_ref().unwrap();
    if AttackPacket(packet).targeting_mode() != 0 {
        return Err("point scan requires a point-target command".into());
    }
    let mut admission = state.warriors.clone();
    let w = &mut admission[pending.actor];
    let points = w
        .record()
        .psi_points()?
        .wrapping_add(player_psi::cost(packet, pending.power)?);
    w.record_mut().set_psi_points(points);
    let count = w.record().scans_used()?.wrapping_add(-1);
    w.record_mut().set_scans_used(count);
    player_psi::validate(
        combat,
        &admission,
        mission(state, content)?,
        state.psi_disabled,
        player_psi::CastRequest {
            spell: pending.spell,
            power: pending.power,
            target: Some(point),
            rotation: 0,
        },
    )?;
    player_psi::selected_body(combat, &state.warriors, point)
}

pub(crate) fn select(
    state: &mut Session,
    content: &Content,
    point: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    let body = admit_target(state, content, point)?;
    state.psi_selection.as_mut().unwrap().targets.push(body);
    Ok(vec![Event::Sound { resource_id: 145 }])
}

pub(crate) fn fire(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .clone()
        .ok_or("no pending Psi selection")?;
    pending.validate(state)?;
    if pending.targets.is_empty() {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Combat {
                kind: crate::feedback::CombatRefusal::NeedTarget,
            }],
        });
    }
    if state.native_work.psi_memory.is_none() {
        state.native_work.psi_memory = Some(crate::initial_psi_memory(&state.warriors)?);
    }
    let mut events = player_psi::cast_selection(state, mission(state, content)?, &pending)?;
    state.psi_selection = None;
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}

/// cleartarget13822a clears selection without charging or refunding the scan.
pub(crate) fn clear(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .as_ref()
        .ok_or("no pending Psi selection")?;
    pending.validate(state)?;
    state.psi_selection.as_mut().unwrap().targets.clear();
    Ok(vec![])
}

pub(crate) fn cancel(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .as_ref()
        .ok_or("no pending Psi selection")?;
    pending.validate(state)?;
    let actor = pending.actor;
    let packet = state
        .combat
        .as_ref()
        .unwrap()
        .attack_packet
        .as_ref()
        .unwrap();
    let refund = i32::from(player_psi::cost(packet, pending.power)?) * 66 / 100;
    let w = &mut state.warriors[actor];
    let points = w.record().psi_points()?.wrapping_add(refund as i16);
    w.record_mut().set_psi_points(points);
    let count = w.record().scans_used()?.wrapping_add(-1);
    w.record_mut().set_scans_used(count);
    let movement = w.record().movement_points()?.wrapping_sub(3).max(0);
    w.record_mut().set_movement_points(movement);
    state.psi_selection = None;
    runtime::refresh_active_item_context(state, content)?;
    Ok(vec![
        Event::Sound { resource_id: 6000 },
        Event::Sound { resource_id: 20005 },
    ])
}
