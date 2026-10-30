//! Selected Spell's target from the script's caller.
use crate::combat::map_alias::MapAliases;
use crate::combat::spells::{SpellContext, SpellMask, SpellTarget, target_mask_with_map_aliases};
use crate::{RuleError, Session};
use centurions_content::{Content, mission::OpeningMission};

/// The warrior casting a script's spell: the warrior whose turn it is, the battle's actor, or the
/// selected warrior.
pub(super) fn caster(state: &Session) -> i16 {
    state
        .turn_dispatch
        .as_ref()
        .filter(|frame| frame.owns_script(state))
        .map(|frame| frame.body)
        .or_else(|| state.combat_story.as_ref().map(|story| story.actor))
        .unwrap_or(state.char_select_new as i16)
}

/// Picks the targets of the spell a script casts.
pub(super) fn select(
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
    packet: &[u8],
    id: i16,
    count: i16,
) -> Result<(SpellMask, (i16, i16)), RuleError> {
    let combat = state
        .combat
        .as_ref()
        .ok_or("scripted tactical roster missing")?;
    let (first, second) = if let Some(frame) = state
        .turn_dispatch
        .as_ref()
        .filter(|frame| frame.owns_script(state))
    {
        // getup passes the monster slot, but the script treats it as a body number,
        // so monsters 0..9 read the wrong position: an original quirk (see
        // docs/source-notes/rules.md).
        let index = frame.body - 20;
        if index == -9 {
            let entry = combat.todo_queue.entries[0];
            (
                entry.x.wrapping_shr(16) as i16,
                entry.y.wrapping_shr(16) as i16,
            )
        } else {
            if index < -8 {
                return Err(RuleError::Unsupported {
                    action: "story.spell",
                    dependency: "enemy-turn target from NPC 0 reads far past the death queue",
                });
            }
            if index < 0 {
                return Err(RuleError::Unsupported {
                    action: "story.spell",
                    dependency: "enemy-turn target reads before the NPC position array",
                });
            }
            let point = combat.story_position(index as usize)?;
            (
                point
                    .0
                    .wrapping_add(combat.field.map_shift.0)
                    .wrapping_add(5),
                point
                    .1
                    .wrapping_add(combat.field.map_shift.1)
                    .wrapping_add(5),
            )
        }
    } else {
        let story = state
            .combat_story
            .as_ref()
            .ok_or("scripted tactical spell origin missing")?;
        (story.entry.x as i16, story.entry.y as i16)
    };
    let target = SpellTarget::cell(first, second);
    let context = SpellContext {
        packet,
        id,
        count,
        caster: caster(state),
        initiative: combat.battle_order_index as i16,
        collision: false,
    };
    // The coordinates are replaced while the cleared target kind stays.
    let point = combat.script_target_position(
        target.first,
        state.command_preferences.small_combat_field,
        &state.globals.battlefield.pos,
    )?;
    let selected_target = SpellTarget::cell(
        point.0 + combat.field.map_shift.0 + 5,
        point.1 + combat.field.map_shift.1 + 5,
    );
    let position = state
        .exploration
        .as_ref()
        .ok_or("scripted tactical map missing")?;
    let aliases = MapAliases::of_mission(map)
        .with_land(&position.objects, &position.events)
        .with_stages(
            &content
                .scenario
                .as_ref()
                .ok_or("scripted tactical scenario missing")?
                .stages,
        );
    let mask = target_mask_with_map_aliases(
        combat,
        &state.warriors,
        &map.areas,
        (&selected_target, &target),
        &context,
        &combat.spell_queue,
        &mut state.random,
        aliases,
    )?;
    Ok((mask, (selected_target.first, selected_target.second)))
}
