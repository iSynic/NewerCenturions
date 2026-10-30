use crate::combat::item_activation::ordinary;
use crate::combat::spells::spell_identity;
use crate::combat::{player_psi, runtime, spells};
use crate::{Event, Phase, PsiMenuChoice, RuleError, Session};
use centurions_content::mission::OpeningMission;
use centurions_content::{
    Content,
    records::{CharacterStatisticsMut, SpellInfoRecord, Statistic},
};
use serde::{Deserialize, Serialize};

/// A psi power or item chosen and waiting for its targets: the caster, the spell, the power level
/// and the bodies picked so far.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Selection {
    pub actor: usize,
    pub spell: i16,
    pub power: i16,
    pub targets: Vec<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<i16>,
}

impl Selection {
    /// Checks that the choice is still sound: it is the active warrior's, in battle, with no more
    /// targets than the power level, each standing.
    pub fn validate(&self, state: &Session) -> Result<(), RuleError> {
        if self.item.is_some() {
            return ordinary::validate(self, state);
        }
        let combat = state.combat.as_ref().ok_or("pending Psi combat missing")?;
        if state.phase != Phase::Combat
            || combat.active_body() != Some(self.actor as i16)
            || self.actor >= state.warriors.len()
            || !(1..=7).contains(&self.power)
            || self.targets.len() > self.power as usize
            || !player_psi::COMBAT_SPELLS.contains(&self.spell)
            || state.medical_continuation.is_some()
            || combat.firearm_selection.is_some()
            || combat.cast_identity != spell_identity(self.spell)
            || combat.spell_info.len() != 58
            || SpellInfoRecord(&combat.spell_info).target_type() > 6
            || SpellInfoRecord(&combat.spell_info).target_type() != 0 && !self.targets.is_empty()
            || self.targets.iter().any(|body| {
                *body < 0
                    || (*body >= state.warriors.len() as i16 && *body < 10)
                    || *body >= 10 + combat.roster.records.len() as i16
            })
        {
            return Err(RuleError::Invalid("invalid pending Psi selection".into()));
        }
        for body in &self.targets {
            let active = if *body < 10 {
                state.warriors[*body as usize].record().in_battle() != 0
            } else {
                combat.roster.records[(*body - 10) as usize]
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

/// The map the party stands on.
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

/// Starts choosing bodies for a multi-body psi power.
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
    if SpellInfoRecord(&packet).target_type() != 0 {
        return Err("Psi selection requires a multi-body spell".into());
    }
    begin_targeting(state, content, spell, power)
}

/// Fires the chosen power at the cell clicked, turned by `rotation` quarter turns. A multi-body
/// power needs its targets chosen instead.
pub(crate) fn fire_at(
    state: &mut Session,
    content: &Content,
    point: (i16, i16),
    rotation: u8,
) -> Result<Vec<Event>, RuleError> {
    let pending = state.psi_selection.clone().ok_or("no pending Psi target")?;
    pending.validate(state)?;
    if pending.item.is_some() {
        return ordinary::fire_at(state, content, point, rotation);
    }
    if state.combat.as_ref().unwrap().spell_info[1] == 0 {
        return Err("multi-body scan requires selected targets".into());
    }
    // The chosen cell is stored in map coordinates.
    let combat = state.combat.as_ref().unwrap();
    let cell = (
        point.0 + combat.field.map_shift.0 + 5,
        point.1 + combat.field.map_shift.1 + 5,
    );
    state.target_picks_mut().select(Some(cell.0), Some(cell.1));
    let mut events = player_psi::cast_prepared_at(
        state,
        (mission(state, content)?, &content.items),
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

/// Fires the chosen healing power at a warrior's portrait; only healing (effect 57) qualifies, not
/// revival, and it goes on to choosing whom to treat.
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
    if pending.item.is_some() {
        return ordinary::portrait(state, content, warrior);
    }
    let combat = state.combat.as_ref().unwrap();
    let packet = &combat.spell_info;
    // Only healing (effect 57) qualifies here, not revival (effect 64).
    if SpellInfoRecord(packet).target_type() != 1 || SpellInfoRecord(packet).effect()? != 57 {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Sound { resource_id: 143 }],
        });
    }
    state.psi_selection = None;
    state.char_select_new = pending.actor;
    let mut health =
        crate::story::health::reselected_context(pending.actor, &state.globals.health_context);
    health.affected[pending.actor] = 1;
    health.effect = SpellInfoRecord(packet).health_effect();
    state.globals.health_context = health;
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

/// Starts a psi power that needs a target to be chosen. The power's cost is paid and the scan
/// counted when it is drawn, and the target list starts empty.
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
            target: Some(combat.squad.pos[actor]),
            rotation: 0,
        },
    )?;
    let packet = spells::load_packet(
        &map.combat.as_ref().ok_or("Psi content missing")?.attacks,
        spell,
    )?;
    if SpellInfoRecord(&packet).target_type() > 6 {
        return Err("scan has no point-target continuation".into());
    }
    combat.weapon_traits.armor_filter();
    player_psi::select_combat_caster(state);
    let warriors = &mut state.warriors[actor];
    let points = warriors
        .record()
        .psi_points()?
        .wrapping_add(player_psi::cost(&packet, power)?.wrapping_neg());
    warriors.record_mut().set_psi_points(points);
    let count = warriors.record().spells_so_far()?.wrapping_add(1);
    warriors.record_mut().set_spells_so_far(count);
    CharacterStatisticsMut(&mut warriors.bytes).add(Statistic::ScansCast, 1);
    let identity = spell_identity(spell);
    let combat = state.combat.as_mut().unwrap();
    combat.spell_info = packet;
    combat.cast_identity = identity;
    state.globals.spell_info = combat.spell_info.clone();
    state.globals.cast_identity = combat.cast_identity;
    state.globals.weapon_traits = combat.weapon_traits;
    state.target_quantity = None;
    state.globals.attack_count = power;
    // Drawing the spell empties the target list.
    state.target_picks_mut().count = 0;
    state.globals.old_power_level[actor] = None;
    state.psi_menu_choices[actor] = Some(PsiMenuChoice {
        row: identity[1] as u8,
        column: identity[2] as u8,
        power,
    });
    state.set_in_spell(1);
    state.psi_selection = Some(Selection {
        actor,
        spell,
        power,
        targets: vec![],
        item: None,
    });
    Ok(vec![])
}

/// Checks a cell clicked as the next target: there must still be room under the power level, and
/// the power must be cast on a point. The cast is checked as if its cost were refunded. Returns the
/// body in the cell.
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
    if pending.item.is_some() {
        if state.combat.as_ref().unwrap().spell_info[1] != 0 {
            return Err("item target list requires point targeting mode".into());
        }
        if pending.targets.len() as i32 >= i32::from(pending.power) {
            return Err(RuleError::Refused {
                feedback: vec![crate::feedback::GameFeedback::Combat {
                    kind: crate::feedback::CombatRefusal::TooManyTargets,
                }],
            });
        }
        if pending.targets.len() >= 7 {
            return Err(RuleError::Unsupported {
                action: "combat.item_activation",
                dependency: "eighth prepared item target aliases unrecorded native work",
            });
        }
        return ordinary::admit(state, content, pending, point);
    }
    if pending.targets.len() >= pending.power as usize {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Combat {
                kind: crate::feedback::CombatRefusal::TooManyTargets,
            }],
        });
    }
    let combat = state.combat.as_ref().unwrap();
    let packet = &combat.spell_info;
    if SpellInfoRecord(packet).target_type() != 0 {
        return Err("point scan requires a point-target command".into());
    }
    let mut admission = state.warriors.clone();
    let w = &mut admission[pending.actor];
    let points = w
        .record()
        .psi_points()?
        .wrapping_add(player_psi::cost(packet, pending.power)?);
    w.record_mut().set_psi_points(points);
    let count = w.record().spells_so_far()?.wrapping_add(-1);
    w.record_mut().set_spells_so_far(count);
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

/// Adds the body in the clicked cell to the targets.
pub(crate) fn select(
    state: &mut Session,
    content: &Content,
    point: (i16, i16),
) -> Result<Vec<Event>, RuleError> {
    let body = admit_target(state, content, point)?;
    let pending = state.psi_selection.as_mut().unwrap();
    pending.targets.push(body);
    if pending.item.is_none() {
        // A warrior target is recorded only while packet byte 31 is 0 or 1; a
        // monster target always is.
        let byte = state.combat.as_ref().unwrap().spell_info[31];
        let first = (body >= 9 || byte <= 1).then_some(body);
        state.target_picks_mut().select(first, None);
    }
    Ok(vec![Event::Sound { resource_id: 145 }])
}

/// Fires the chosen power at the targets picked; choosing none is refused.
pub(crate) fn fire(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .clone()
        .ok_or("no pending Psi selection")?;
    pending.validate(state)?;
    if pending.item.is_some() {
        return ordinary::fire_targets(state, content);
    }
    if pending.targets.is_empty() {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Combat {
                kind: crate::feedback::CombatRefusal::NeedTarget,
            }],
        });
    }
    let mut events =
        player_psi::cast_selection(state, (mission(state, content)?, &content.items), &pending)?;
    state.psi_selection = None;
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}

/// Clears the targets without charging or refunding the scan.
///
/// Original: cleartarget
pub(crate) fn clear(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .as_ref()
        .ok_or("no pending Psi selection")?;
    pending.validate(state)?;
    if pending.item.is_some() {
        state.clear_targets();
    } else {
        state.target_picks_mut().clear();
    }
    state.psi_selection.as_mut().unwrap().targets.clear();
    Ok(vec![])
}

/// Gives up the chosen power. The caster is refunded 66 percent of its cost, the scan is uncounted,
/// and 3 movement is lost.
pub(crate) fn cancel(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .as_ref()
        .ok_or("no pending Psi selection")?;
    pending.validate(state)?;
    if pending.item.is_some() {
        return ordinary::cancel(state, content);
    }
    let actor = pending.actor;
    let packet = &state.combat.as_ref().unwrap().spell_info;
    let refund = i32::from(player_psi::cost(packet, pending.power)?) * 66 / 100;
    let warriors = &mut state.warriors[actor];
    let points = warriors.record().psi_points()?.wrapping_add(refund as i16);
    warriors.record_mut().set_psi_points(points);
    let count = warriors.record().spells_so_far()?.wrapping_add(-1);
    warriors.record_mut().set_spells_so_far(count);
    let movement = warriors.record().movement()?.wrapping_sub(3).max(0);
    warriors.record_mut().set_movement(movement);
    state.set_in_spell(0);
    state.psi_selection = None;
    runtime::refresh_active_item_context(state, content)?;
    Ok(vec![
        Event::Sound { resource_id: 6000 },
        Event::Sound { resource_id: 20005 },
    ])
}
