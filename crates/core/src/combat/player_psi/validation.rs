//! Checking a psi command before it is cast.
use super::COMBAT_SPELLS;
use super::{
    CastRequest, Combat, RuleError, can_cast, cost, in_range, lifecycle, mode_allowed, packet,
    range,
};
use crate::combat::map_alias::MapAliases;
use centurions_content::mission::OpeningMission;
use centurions_content::{Warrior, records::SpellInfoRecord};

/// Checks a psi command before it is cast.
pub fn validate(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &OpeningMission,
    disabled: u8,
    request: CastRequest,
) -> Result<(), RuleError> {
    validate_target(
        combat,
        warriors,
        mission,
        disabled,
        request,
        (None, MapAliases::of_mission(mission)),
    )
}

/// Checks a psi command aimed at a warrior.
pub fn validate_warrior(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &OpeningMission,
    disabled: u8,
    spell: i16,
    power: i16,
    warrior: usize,
) -> Result<(), RuleError> {
    let target = *combat
        .squad
        .pos
        .get(warrior)
        .ok_or("Psi warrior target missing")?;
    validate_target(
        combat,
        warriors,
        mission,
        disabled,
        CastRequest {
            rotation: 0,
            spell,
            power,
            target: Some(target),
        },
        (Some(warrior as i16), MapAliases::of_mission(mission)),
    )
}

/// Checks a psi command: a power battle offers at power level 1 to 7, cast by the active warrior
/// who may cast, knows it and has the psi points, in range and in sight, and aimed at the kind of
/// target the power takes.
pub(super) fn validate_target(
    combat: &Combat,
    warriors: &[Warrior],
    mission: &OpeningMission,
    disabled: u8,
    request: CastRequest,
    (body, aliases): (Option<i16>, MapAliases<'_>),
) -> Result<(), RuleError> {
    if !COMBAT_SPELLS.contains(&request.spell) || !(1..=7).contains(&request.power) {
        return Err(RuleError::Unsupported {
            action: "combat.player_psi",
            dependency: "other learned spell or power",
        });
    }
    if request.rotation > 3 {
        return Err(RuleError::Invalid("psi rotation".into()));
    }
    let actor = combat
        .active_body()
        .filter(|body| (0..6).contains(body))
        .ok_or_else(|| RuleError::Invalid("psi casting requires an active warrior".into()))?;
    if !can_cast(warriors, actor, disabled)? {
        return Err(RuleError::Invalid("warrior cannot cast psi".into()));
    }
    if !warriors[actor as usize].knows_spell(request.spell)? {
        return Err(RuleError::Invalid(
            "warrior has not learned this psi spell".into(),
        ));
    }
    let packet = packet(mission, request.spell)?;
    if !mode_allowed(&packet, true)? {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Warning { index: 23 }],
        });
    }
    if warriors[actor as usize].psi_points()? < cost(&packet, request.power)? {
        return Err(RuleError::Invalid("insufficient psi".into()));
    }
    if SpellInfoRecord(&packet).is_party_duration() {
        if range(&packet, request.power)? != 0
            || !(0..10).contains(&SpellInfoRecord(&packet).effect()?)
            || request.target.is_some()
        {
            return Err(RuleError::Invalid(
                "party spell uses automatic caster targeting".into(),
            ));
        }
        return Ok(());
    }
    if SpellInfoRecord(&packet).is_global() {
        if request.target.is_some() {
            return Err(RuleError::Invalid(
                "global psi spell does not take a point target".into(),
            ));
        }
        return Ok(());
    }
    let target = request
        .target
        .ok_or_else(|| RuleError::Invalid("psi point target missing".into()))?;
    let from = *combat
        .squad
        .pos
        .get(actor as usize)
        .ok_or_else(|| RuleError::Invalid("psi caster position missing".into()))?;
    if !in_range(from, target, range(&packet, request.power)?) {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Combat {
                kind: crate::feedback::CombatRefusal::OutOfRange,
            }],
        });
    }
    let center = crate::combat::records::grid_point(target, combat.field.map_shift);
    if SpellInfoRecord(&packet).range_base() + SpellInfoRecord(&packet).range_per_power() > 0
        && !lifecycle::can_see(combat, mission, from, target, 0)?
    {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Combat {
                kind: crate::feedback::CombatRefusal::Blocked,
            }],
        });
    }
    if SpellInfoRecord(&packet).target_type() < 2 {
        if let Some(body) = body {
            let warrior = usize::try_from(body)
                .ok()
                .and_then(|index| warriors.get(index))
                .ok_or("Psi warrior target outside party")?;
            if SpellInfoRecord(&packet).target_type() != 1
                || warrior.record().in_battle() == 0 && SpellInfoRecord(&packet).effect()? != 64
            {
                return Err(RuleError::Refused { feedback: vec![] });
            }
        } else {
            selected_body(combat, warriors, target)?;
        }
        return Ok(());
    }
    let shape = match SpellInfoRecord(&packet).target_type() {
        3 => {
            usize::from(SpellInfoRecord(&packet).area_size())
                + if SpellInfoRecord(&packet).can_rotate() {
                    usize::from(request.rotation)
                } else {
                    0
                }
        }
        4 => request.power as usize,
        _ => 1,
    };
    if shape == 0 {
        return Err(RuleError::Invalid("psi area missing".into()));
    }
    let area = centurions_content::span(&mission.areas, (shape - 1) * 49, 49)?;
    for y in 0..7 {
        for x in 0..7 {
            // The map is read as an unsigned column × 90 plus a signed row, so a row
            // past either edge reads the next column or what lies beside the map.
            let column = i32::from((center.0 - 3 + x as i32) as i16 as u16);
            let index = column * 90 + (center.1 - 3 + y as i32);
            if request.spell == 1305 && area[y * 7 + x] != 0 {
                let body = aliases
                    .read(&combat.field, index)?
                    .ok_or(RuleError::Unsupported {
                        action: "combat.player_psi",
                        dependency: "spell area outside the map and the represented memory beside it",
                    })?
                    .wrapping_abs();
                if (6..10).contains(&body) {
                    return Err(RuleError::Invalid("empty support body".into()));
                }
            }
        }
    }
    Ok(())
}

/// The body in a cell, if it is a warrior in the battle or a monster still standing.
pub(crate) fn selected_body(
    combat: &Combat,
    warriors: &[Warrior],
    target: (i16, i16),
) -> Result<i16, RuleError> {
    let cell = OpeningMission::index(
        i32::from(target.0 + combat.field.map_shift.0 + 5),
        i32::from(target.1 + combat.field.map_shift.1 + 5),
    )?;
    let body = combat.field.tiles[cell].wrapping_abs();
    if (0..warriors.len() as i16).contains(&body)
        && warriors[body as usize].record().in_battle() != 0
        || (10..10 + combat.roster.records.len() as i16).contains(&body)
            && combat.roster.records[(body - 10) as usize]
                .record()
                .stamina()?
                > 0
    {
        Ok(body)
    } else {
        Err(RuleError::Refused { feedback: vec![] })
    }
}
