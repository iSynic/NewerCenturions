//! The Items window's party portraits: the six framed portraits down the
//! centre column away from a shop, and the strips of small portraits each list
//! chooses its owner from.
use super::{Inventory, InventoryAction};
use crate::{
    assets::Assets,
    portrait,
    presentation::{Action, Control},
};
use centurions_core::Session;
use macroquad::prelude::*;

/// The member buttons each list's strip answers clicks in: the left list's
/// (CNTL 200) and the right list's (CNTL 201).
const STRIPS: [Rect; 2] = [
    Rect::new(342., 408., 46., 92.),
    Rect::new(392., 408., 46., 92.),
];
/// How far right of the left strip the right strip's cells are drawn.
const STRIP_GAP: f32 = 52.;
/// The cicn that marks the member a list shows.
const SHOWN: i16 = 145;

/// The cell member `member` has in a list's strip: the first three down a
/// column 24 apart, the rest in the next column 24 to the right and starting
/// one lower.
///
/// Original: updateport
fn strip_cell(side: usize, member: usize) -> Rect {
    let (across, up) = if member >= 3 { (24., 71.) } else { (0., 0.) };
    Rect::new(
        342. + side as f32 * STRIP_GAP + across,
        408. + member as f32 * 24. - up,
        22.,
        22.,
    )
}

impl Inventory {
    /// Draws each list's strip of small portraits and adds a click target for
    /// every member the lists are not already showing. A list's strip leaves
    /// out the member the other list shows and marks its own.
    ///
    /// Original: updateport, choice
    pub(super) fn draw_strips(
        &self,
        state: &Session,
        assets: &Assets,
        controls: &mut Vec<Control>,
    ) {
        let shop = Self::shop(state);
        let shown = [Some(self.members[0]), (!shop).then_some(self.members[1])];
        for (member, warrior) in state.warriors.iter().enumerate() {
            let image = warrior
                .record()
                .portrait()
                .ok()
                .and_then(|id| assets.portraits.get(&id));
            for side in 0..2 {
                let cell = strip_cell(side, member);
                if shown[1 - side] != Some(member)
                    && let Some(image) = image
                {
                    crate::assets::texture_in(image, cell);
                }
                if shown[side] == Some(member)
                    && let Some(mark) = assets.icons.get(&SHOWN)
                {
                    crate::assets::texture_in(mark, cell);
                }
            }
        }
        // A member button only answers within its own bounds, which the right
        // strip's second column overhangs.
        for (side, strip) in STRIPS.iter().enumerate().take(if shop { 1 } else { 2 }) {
            for member in 0..state.warriors.len() {
                if shown.contains(&Some(member)) {
                    continue;
                }
                if let Some(rect) = strip_cell(side, member).intersect(*strip) {
                    controls.push(Control {
                        rect,
                        enabled: true,
                        action: Action::Inventory(InventoryAction::Warrior { side, member }),
                    });
                }
            }
        }
    }

    /// Away from a shop, draws the centre column: six PICT 154 frames with the
    /// first six members' portraits, then PICT 175 below them.
    ///
    /// Original: Showitems
    pub(super) fn draw_column(state: &Session, assets: &Assets, class_label: bool) {
        for slot in 0..6 {
            let top = 20. + slot as f32 * 50.;
            assets.panel(154, Rect::new(375., top, 50., 50.));
            if let Some(warrior) = state.warriors.get(slot)
                && warrior.record().portrait().unwrap_or(0) != 0
            {
                portrait::plot(
                    assets,
                    warrior,
                    vec2(383., top + 8.),
                    &portrait::Marks {
                        class_label,
                        conditions: None,
                        automatic: false,
                        in_combat: false,
                    },
                );
            }
        }
        assets.panel(175, STOCKPILE);
    }
}

/// PICT 175, the ammunition crate below the centre column.
pub(super) const STOCKPILE: Rect = Rect::new(375., 320., 50., 37.);
