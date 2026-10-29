//! Player psi in the original combat execution order.
use super::{begin_learned_heal, finish_player_action, select_medical_target};
use crate::combat::Combat;
use crate::{Event, RuleError, Session};
use centurions_content::Content;

pub fn player_psi(
    state: &mut Session,
    content: &Content,
    spell: i16,
    power: i16,
    target: Option<(i16, i16)>,
    rotation: u8,
) -> Result<Vec<Event>, RuleError> {
    player_psi_target(state, content, spell, power, target, rotation, None)
}

pub fn player_psi_warrior(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    if spell == 1101
        && state
            .warriors
            .get(warrior)
            .is_some_and(|target| (-9..=0).contains(&target.stamina().unwrap_or(-10)))
    {
        let mut events = begin_learned_heal(state, content, power)?;
        events.extend(select_medical_target(
            state,
            content,
            crate::MedicalTarget::Portrait { warrior },
        )?);
        return Ok(events);
    }
    let target = *state
        .combat
        .as_ref()
        .ok_or("Psi combat missing")?
        .squad
        .positions
        .get(warrior)
        .ok_or("Psi warrior target missing")?;
    player_psi_target(
        state,
        content,
        spell,
        power,
        Some(target),
        0,
        Some(warrior as i16),
    )
}

pub(super) fn player_psi_target(
    state: &mut Session,
    content: &Content,
    spell: i16,
    power: i16,
    target: Option<(i16, i16)>,
    rotation: u8,
    body: Option<i16>,
) -> Result<Vec<Event>, RuleError> {
    let actor = state
        .combat
        .as_ref()
        .and_then(Combat::active_body)
        .filter(|body| (0..6).contains(body))
        .ok_or_else(|| RuleError::Invalid("psi casting requires an active warrior".into()))?;
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    crate::combat::player_psi::select_native_caster(state);
    // cast's loop over castspell's single triple.
    if let Some(combat) = state.combat.as_mut() {
        combat.target_work.enter(0, 1);
    }
    let mut events = crate::combat::player_psi::cast_with_buffs(
        state.combat.as_mut().unwrap(),
        &mut state.warriors,
        mission,
        &mut state.random,
        &mut state.native_work.psi_memory,
        &mut state.party_buffs,
        state.psi_disabled,
        crate::combat::player_psi::CastRequest {
            rotation,
            spell,
            power,
            target,
        },
        crate::combat::player_psi::CastContext {
            movement_gate: state.bandage_ready,
            body,
            animation_delay: state.command_preferences.animation_delay,
            item_work_alias: crate::combat::spells::retained_item_alias(
                &state.native_work.ammunition_work,
            )?,
            owners: crate::combat::warrior_alias::Owners::new(
                None,
                &state.native_work.shop_work,
                state
                    .exploration
                    .as_ref()
                    .map(|position| position.event_headers.as_slice()),
            )
            .with_backup(&state.native_work.warrior_table),
            land: crate::combat::map_alias::session_map_aliases!(state),
            refresh: Some((&content.items, &mut state.native_work.ammunition_work)),
        },
    )?;
    state.commit_alias_writes()?;
    // castspell clears the mask at entry; cast and the global arm clear it afterwards.
    if let Some(combat) = state.combat.as_mut() {
        combat.clear_target_tail();
        // cast's last increment and closing cleartarget (10e658..10e678).
        combat.target_work.leave();
    }
    if matches!(
        centurions_content::records::AttackPacket(&state.combat.as_ref().unwrap().attack_packet)
            .effect()?,
        55 | 56
    ) {
        state.bandage_ready = 0;
    }
    state.native_work.attack_count = power;
    // castspell 0x10d328 stores the cast power in WORD 0x437a7a + 2 * member.
    state.native_work.psi_powers[actor as usize] = None;
    state.psi_menu_choices[actor as usize] = Some(crate::PsiMenuChoice {
        row: (spell % 1000 / 100 - 1) as u8,
        column: (spell % 100 - 1) as u8,
        power,
    });
    events.extend(finish_player_action(state, content, actor)?);
    Ok(events)
}
