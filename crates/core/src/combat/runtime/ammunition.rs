//! Ammunition during combat.
use super::finish_player_action;
use crate::combat::Combat;
use crate::{Event, Phase, RuleError, Session};
use centurions_content::Content;

/// Fits reserve ammunition into the firearm in an inventory slot. No message may be showing; out
/// of combat no script may be running either, and in combat it must be the warrior's turn with an
/// attack left. In combat the action then ends, and the common firearm classes play the reload
/// sound first. Pairings of firearm and ammunition that are not recovered are unsupported.
pub fn fit_ammunition(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
    ammunition: i16,
) -> Result<Vec<Event>, RuleError> {
    if warrior >= state.warriors.len()
        || !matches!(state.phase, Phase::Exploration | Phase::Combat)
        || state.exploration.as_ref().is_none_or(|p| {
            p.presentation.is_some() || (state.phase == Phase::Exploration && p.script.is_some())
        })
        || (state.phase == Phase::Combat
            && (state.combat.as_ref().and_then(Combat::active_body) != Some(warrior as i16)
                || state.warriors[warrior].record().attacks()? < 1))
    {
        return Err(RuleError::Invalid(
            "ammunition selection requires an unobstructed active mission turn".into(),
        ));
    }
    let item = state.warriors[warrior]
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("ammunition fitting slot missing".into()))?;
    if matches!(
        centurions_content::records::ItemDefinition(&crate::ammunition::definition(
            content,
            item.id()
        )?)
        .item_type()?,
        2 | 15 | 20
    ) {
        if item.category_code() == 20 && state.phase == Phase::Combat {
            return Err(RuleError::Refused {
                feedback: vec![crate::feedback::GameFeedback::Silent],
            });
        }
        let reserve_index = usize::try_from(ammunition.wrapping_sub(200))
            .map_err(|_| RuleError::Invalid("ammunition outside reserve table".into()))?;
        let quality = state
            .globals
            .loot_quality
            .as_deref()
            .map(|quality| quality[..4].try_into().unwrap());
        let in_combat = state.phase == Phase::Combat;
        let result = crate::ammunition::fit(
            &state.warriors[warrior],
            &state.ammunition,
            content,
            crate::ammunition::FitRequest {
                slot,
                reserve_index,
                in_combat,
                loot_quality: quality,
            },
            &state.globals.loaded_items,
            &mut state.random,
        )?;
        let mut events = if in_combat {
            vec![Event::Sound { resource_id: 141 }]
        } else {
            vec![]
        };
        events.extend(crate::ammunition::commit_result(state, warrior, result)?);
        if in_combat {
            events.extend(finish_player_action(state, content, warrior as i16)?);
        }
        return Ok(events);
    }
    let reserve = match (item.id(), ammunition) {
        (2, 201) => 1,
        (38, 203) => 3,
        (637, 307) if state.phase == Phase::Exploration => 107,
        _ => {
            return Err(RuleError::Unsupported {
                action: "equipment.fit_ammunition",
                dependency: "other firearm and selected ammunition pair",
            });
        }
    };
    if state.ammunition.get(reserve).copied().unwrap_or(0) <= 0 {
        return Err(RuleError::Invalid(
            "selected ammunition is unavailable".into(),
        ));
    }
    let in_combat = state.phase == Phase::Combat;
    let result = crate::combat::firearm::fit_ammunition(
        &state.warriors[warrior],
        slot,
        content,
        &state.ammunition,
        in_combat,
        &mut state.random,
    )?;
    state.warriors[warrior] = result.warrior;
    state.ammunition = result.ammunition;
    state.globals.loaded_item_field32 = result.loaded_item_field32;
    state.globals.loaded_items.load_item(content, item.id())?;
    if result.loaded {
        state.globals.loaded_items.ammo = Some(crate::ammunition::definition(content, ammunition)?);
        if let Some(combat) = state.combat.as_mut() {
            combat
                .weapon_traits
                .load_ammunition(&content.items[ammunition as usize])?;
        } else {
            state
                .globals
                .weapon_traits
                .load_ammunition(&content.items[ammunition as usize])?;
        }
    }
    let mut events: Vec<_> = result
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect();
    if in_combat {
        events.extend(finish_player_action(state, content, warrior as i16)?);
    }
    Ok(events)
}
