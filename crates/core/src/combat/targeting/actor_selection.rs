//! A monster's choice of target: cached targets, visibility scanning and held monster slots.
use crate::combat::{Combat, lifecycle};
use crate::{RandomState, RuleError};
use centurions_content::{Warrior, mission::OpeningMission};

/// Whether a body can be a monster's target: a warrior in battle, or a monster standing, on the
/// other side.
fn eligible(
    combat: &Combat,
    warriors: &[Warrior],
    side: i16,
    body: i16,
) -> Result<bool, RuleError> {
    if (0..warriors.len() as i16).contains(&body) {
        let warrior = warriors[body as usize].record();
        Ok(warrior.in_battle() != 0 && warrior.faction()? != side)
    } else if (10..110).contains(&body) {
        let monster = combat.story_record((body - 10) as usize)?.record();
        Ok(monster.stamina()? > 0 && monster.faction()? != side)
    } else {
        Ok(false)
    }
}

/// Where a warrior or monster stands.
fn position(combat: &Combat, body: i16) -> Result<(i16, i16), RuleError> {
    if (0..6).contains(&body) {
        Ok(combat.squad.pos[body as usize])
    } else {
        combat.story_position((body - 10) as usize)
    }
}

/// Chooses whom a monster aims at. It keeps its current target if that is still eligible, else
/// draws a random eligible body (an error if none turns up). If the target is out of sight it scans
/// the bodies in order for one that is eligible, not excluded and in sight. When none remains the
/// monster is left with no target (-1).
///
/// Original: combat
pub(crate) fn select_actor_target(
    combat: &mut Combat,
    warriors: &[Warrior],
    mission: &OpeningMission,
    random: &mut RandomState,
    body: i16,
) -> Result<i16, RuleError> {
    let actor = body
        .checked_sub(10)
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < combat.roster.records.len())
        .ok_or_else(|| RuleError::Invalid("invalid targeting enemy".into()))?;
    let side = combat.roster.records[actor].record().faction()?;
    let mut target = combat.roster.records[actor].record().target()?;
    if !eligible(combat, warriors, side, target)? {
        let mut selected = None;
        for _ in 0..10000 {
            let candidate = random.bound(combat.roster.records.len() as i16 + 10) - 1;
            if eligible(combat, warriors, side, candidate)? {
                selected = Some(candidate);
                break;
            }
        }
        target = selected.ok_or_else(|| RuleError::Invalid("no eligible enemy target".into()))?;
    }
    let mut scan = 0;
    loop {
        combat.roster.records[actor].record_mut().set_target(target);
        if lifecycle::can_see(
            combat,
            mission,
            position(combat, body)?,
            position(combat, target)?,
            0,
        )? {
            return Ok(target);
        }
        let mut next = None;
        while scan < 110 {
            let candidate = scan;
            scan += 1;
            if !eligible(combat, warriors, side, candidate)? {
                continue;
            }
            // The exclusion byte is checked on a fresh scan, not cached-target reuse.
            if candidate >= 10
                && combat
                    .story_record((candidate - 10) as usize)?
                    .record()
                    .excluded_from_target_scan()
            {
                continue;
            }
            next = Some(candidate);
            break;
        }
        let Some(next) = next else {
            // movemonster reads this -1 as its target coordinate (docs/source-notes/combat.md).
            combat.roster.records[actor].record_mut().set_target(-1);
            return Ok(-1);
        };
        target = next;
    }
}
