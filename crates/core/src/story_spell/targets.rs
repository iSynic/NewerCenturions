//! newland17's caller parameter and its native coordinate/index alias.
use crate::combat::spells::{SpellContext, SpellMask, SpellTarget, target_mask_with_map_aliases};
use crate::{RuleError, Session};
use centurions_content::{Content, mission::OpeningMission};

pub(super) fn caster(state: &Session) -> i16 {
    state
        .turn_dispatch
        .as_ref()
        .filter(|frame| frame.owns_script(state))
        .map(|frame| frame.body)
        .or_else(|| state.combat_story.as_ref().map(|story| story.actor))
        .unwrap_or(state.selected_warrior as i16)
}

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
        // getup1269cc passes the selected NPC index as newland's fifth argument.
        // newland143a62 treats values >=2 as body IDs and subtracts ten, so NPCs 2..9
        // read before the NPC positions 0x446f0a. NPCs 0..1 instead read the LONGs at
        // 0x437a9a and 0x437a9e + (abs(index) - 1) * 14 and keep their low WORDs
        // (143aa8..143ae2): the high WORDs of death-queue row 0's x and y LONGs for
        // NPC 1. NPC 0's row -1 is scaled unsigned (mulu.w), far past the queue.
        let index = frame.body - 20;
        if index == -9 {
            let entry = combat.death_queue.entries[0];
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
                point.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
                point.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
            )
        }
    } else {
        let story = state
            .combat_story
            .as_ref()
            .ok_or("scripted tactical spell origin missing")?;
        (story.entry.x as i16, story.entry.y as i16)
    };
    let target = SpellTarget {
        first,
        second,
        kind: 3,
    };
    let context = SpellContext {
        packet,
        id,
        count,
        caster: caster(state),
        initiative: combat.cursor as i16,
        collision: false,
    };
    // The original overwrites coordinates while leaving the cleared kind WORD.
    let point = combat.script_target_position(
        target.first,
        state.command_preferences.small_combat_field,
        &state.native_work.tactical_work.positions,
    )?;
    let selected_target = SpellTarget {
        first: point.0 + combat.map.shift.0 + 5,
        second: point.1 + combat.map.shift.1 + 5,
        kind: 3,
    };
    let position = state
        .exploration
        .as_ref()
        .ok_or("scripted tactical map missing")?;
    let aliases = crate::combat::map_alias::MapAliases::of_mission(map)
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
        &combat.spell_fields,
        &mut state.random,
        aliases,
    )?;
    Ok((mask, (selected_target.first, selected_target.second)))
}
