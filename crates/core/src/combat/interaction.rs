use super::*;

pub fn is_friendly_neighbor(state: &Session, target: i16) -> Result<bool, RuleError> {
    let Some(combat) = &state.combat else {
        return Ok(false);
    };
    friendly_neighbor_parts(combat, &state.warriors, target)
}

pub(crate) fn friendly_neighbor_parts(
    combat: &Combat,
    warriors: &[Warrior],
    target: i16,
) -> Result<bool, RuleError> {
    let Some(actor) = combat
        .active_body()
        .filter(|body| (0..warriors.len() as i16).contains(body))
    else {
        return Ok(false);
    };
    let (position, faction, living) = if (0..warriors.len() as i16).contains(&target) {
        (
            combat.squad.positions[target as usize],
            word(&warriors[target as usize].bytes, 0x10)?,
            warriors[target as usize].bytes[0x9df] != 0,
        )
    } else if target >= 10
        && let Some(enemy) = combat.enemies.records.get((target - 10) as usize)
    {
        (
            combat.enemies.positions[(target - 10) as usize],
            word(&enemy.bytes, 0xe)?,
            enemy.stamina()? > 0 && enemy.bytes[0xb0] != 255,
        )
    } else {
        return Ok(false);
    };
    let origin = combat.squad.positions[actor as usize];
    Ok(actor != target
        && living
        && faction == word(&warriors[actor as usize].bytes, 0x10)?
        && (origin.0 - position.0)
            .abs()
            .max((origin.1 - position.1).abs())
            == 1)
}

pub fn can_exchange(state: &Session, target: i16) -> Result<bool, RuleError> {
    let Some(combat) = &state.combat else {
        return Ok(false);
    };
    can_exchange_parts(combat, &state.warriors, target)
}

pub(crate) fn can_exchange_parts(
    combat: &Combat,
    warriors: &[Warrior],
    target: i16,
) -> Result<bool, RuleError> {
    let Some(actor) = combat
        .active_body()
        .filter(|body| (0..warriors.len() as i16).contains(body))
    else {
        return Ok(false);
    };
    if actor == target || word(&warriors[actor as usize].bytes, 0x3c)? <= 4 {
        return Ok(false);
    }
    let (position, faction, living, size) = if (0..warriors.len() as i16).contains(&target) {
        let warrior = &warriors[target as usize];
        (
            combat.squad.positions[target as usize],
            word(&warrior.bytes, 0x10)?,
            warrior.bytes[0x9df] != 0,
            0,
        )
    } else if target >= 10
        && let Some(monster) = combat.enemies.records.get((target - 10) as usize)
    {
        (
            combat.enemies.positions[(target - 10) as usize],
            word(&monster.bytes, 0xe)?,
            monster.stamina()? > 0 && monster.bytes[0xb0] != 255,
            word(&monster.bytes, 0x10)?,
        )
    } else {
        return Ok(false);
    };
    let from = combat.squad.positions[actor as usize];
    Ok(living
        && size == 0
        && faction == word(&warriors[actor as usize].bytes, 0x10)?
        && (i32::from(from.0) - i32::from(position.0))
            .abs()
            .max((i32::from(from.1) - i32::from(position.1)).abs())
            == 1)
}

/// Confirmed combatchoice exchange block, 0x11c564..0x11c854.
pub fn exchange(state: &mut Session, target: i16) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Combat
        || !can_exchange(state, target)?
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some())
    {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::NativeFeedback::Silent],
        });
    }
    let combat = state.combat.as_mut().unwrap();
    let actor = combat.active_body().unwrap() as usize;
    let from = combat.squad.positions[actor];
    let to;
    if target < 10 {
        to = combat.squad.positions[target as usize];
        combat.squad.positions.swap(actor, target as usize);
        combat.squad.under_tiles.swap(actor, target as usize);
    } else {
        let index = (target - 10) as usize;
        to = combat.enemies.positions[index];
        combat.enemies.positions[index] = from;
        combat.squad.positions[actor] = to;
        let terrain = word(&combat.enemies.records[index].bytes, 0x11e)?;
        put_word(
            &mut combat.enemies.records[index].bytes,
            0x11e,
            combat.squad.under_tiles[actor],
        );
        combat.squad.under_tiles[actor] = terrain;
    }
    let index = |position: (i16, i16)| {
        centurions_content::mission::OpeningMission::index(
            i32::from(position.0) + i32::from(combat.map.shift.0) + 5,
            i32::from(position.1) + i32::from(combat.map.shift.1) + 5,
        )
    };
    let first = index(from)?;
    let second = index(to)?;
    combat.map.tiles[first] = target;
    combat.map.tiles[second] = actor as i16;
    let movement = word(&state.warriors[actor].bytes, 0x3c)?.wrapping_sub(5);
    put_word(&mut state.warriors[actor].bytes, 0x3c, movement);
    Ok(vec![
        Event::Sound { resource_id: 654 },
        Event::TacticalMoved {
            body: actor as i16,
            x: to.0,
            y: to.1,
        },
        Event::TacticalMoved {
            body: target,
            x: from.0,
            y: from.1,
        },
    ])
}
