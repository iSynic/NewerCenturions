//! combat115214..115520: cached targets, visibility scanning and retained NPC slots.
use crate::combat::{Combat, lifecycle};
use crate::{RandomState, RuleError};
use centurions_content::{Warrior, mission::OpeningMission};

fn eligible(
    combat: &Combat,
    warriors: &[Warrior],
    side: i16,
    body: i16,
) -> Result<bool, RuleError> {
    if (0..warriors.len() as i16).contains(&body) {
        let warrior = warriors[body as usize].record();
        Ok(warrior.active_flag() != 0 && warrior.faction()? != side)
    } else if (10..110).contains(&body) {
        let enemy = combat.story_record((body - 10) as usize)?.record();
        Ok(enemy.stamina()? > 0 && enemy.faction()? != side)
    } else {
        Ok(false)
    }
}

fn position(combat: &Combat, body: i16) -> Result<(i16, i16), RuleError> {
    if (0..6).contains(&body) {
        Ok(combat.squad.positions[body as usize])
    } else {
        combat.story_position((body - 10) as usize)
    }
}

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
        .filter(|index| *index < combat.enemies.records.len())
        .ok_or_else(|| RuleError::Invalid("invalid targeting enemy".into()))?;
    let side = combat.enemies.records[actor].record().faction()?;
    let mut target = combat.enemies.records[actor].record().target_body()?;
    if !eligible(combat, warriors, side, target)? {
        let mut selected = None;
        for _ in 0..10000 {
            let candidate = random.bound(combat.enemies.records.len() as i16 + 10) - 1;
            if eligible(combat, warriors, side, candidate)? {
                selected = Some(candidate);
                break;
            }
        }
        target = selected.ok_or_else(|| RuleError::Invalid("no eligible enemy target".into()))?;
    }
    let mut scan = 0;
    loop {
        combat.enemies.records[actor]
            .record_mut()
            .set_target_body(target);
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
            // movemonster consumes this sentinel through its retained-coordinate alias.
            combat.enemies.records[actor]
                .record_mut()
                .set_target_body(-1);
            return Ok(-1);
        };
        target = next;
    }
}
