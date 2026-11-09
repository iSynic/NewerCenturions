//! The range readout shown while a target is chosen: how far the cursor cell is from the acting
//! warrior, how far the power reaches, and the cursor that counts the targets still to pick.
use super::castspell_ceiling;
use crate::Session;
use crate::combat::{Combat, player_psi, shooting, spells};
use centurions_content::Content;

/// The first of the colour cursors that show how many targets remain to be picked.
const FIRST_COUNT_CURSOR: i16 = 147;

/// What the targeting bar shows for a cursor cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeReadout {
    /// The straight-line distance from the acting warrior to the cell, in whole cells.
    pub distance: i16,
    /// How far the power reaches.
    pub maximum: i16,
    /// The colour cursor: 147 plus the targets the selection admits less those picked.
    pub cursor: i16,
}

impl RangeReadout {
    /// Whether the cell lies beyond the reach of the power.
    pub fn beyond_maximum(&self) -> bool {
        self.distance > self.maximum
    }
}

/// The readout for `cell` while the active warrior chooses targets. `choice` is the Psi power
/// picked from the menu and not yet pending; a pending Psi selection, a medical item or a firearm
/// shot is read from the session. Nothing is shown when no battle target is being chosen.
///
/// Original: updatespell
pub fn range_readout(
    state: &Session,
    content: &Content,
    choice: Option<(i16, i16)>,
    cell: (i16, i16),
) -> Option<RangeReadout> {
    let combat = state.combat.as_ref()?;
    let (actor, maximum, total, picked) = if let Some(pending) = &state.psi_selection {
        let packet = &combat.spell_info;
        (
            pending.actor,
            player_psi::range(packet, pending.power).ok()?,
            castspell_ceiling(packet, pending.power),
            pending.targets.len(),
        )
    } else if let Some(pending) = &state.medical_continuation {
        let packet = &combat.spell_info;
        (
            pending.actor,
            player_psi::range(packet, pending.power).ok()?,
            castspell_ceiling(packet, pending.power),
            0,
        )
    } else if let Some((spell, power)) = choice {
        let packet = spells::load_packet(
            &content
                .map(state.exploration.as_ref()?.map)?
                .combat
                .as_ref()?
                .attacks,
            spell,
        )
        .ok()?;
        (
            active_warrior(state, combat)?,
            player_psi::range(&packet, power).ok()?,
            castspell_ceiling(&packet, power),
            0,
        )
    } else {
        let prepared = shooting::prepare(state, content).ok()?;
        (
            active_warrior(state, combat)?,
            player_psi::range(&prepared.packet, prepared.shots).ok()?,
            shooting::ceiling(state, content).ok()?,
            combat
                .firearm_selection
                .as_ref()
                .map_or(0, |selection| selection.count()),
        )
    };
    let from = *combat.squad.pos.get(actor)?;
    Some(RangeReadout {
        distance: player_psi::cell_distance(from, cell),
        maximum,
        cursor: FIRST_COUNT_CURSOR
            .wrapping_add(total)
            .wrapping_sub(picked as i16),
    })
}

/// The warrior whose turn it is.
fn active_warrior(state: &Session, combat: &Combat) -> Option<usize> {
    combat
        .active_body()
        .and_then(|body| usize::try_from(body).ok())
        .filter(|body| *body < state.warriors.len())
}
