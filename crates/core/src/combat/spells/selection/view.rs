//! Read-only targeting inputs keep the active prefix separate from the caster.
use super::{Combat, RuleError, Warrior, best_choice, faction};
use crate::combat::map_alias::MapAliases;
use centurions_content::{combat::MonsterRecord, mission::OpeningMission};

/// What target selection reads of the battle: the monsters, where bodies stand, which side they are
/// on, the map shift and the line of sight.
pub(crate) trait SelectionView {
    /// The monster at `index`.
    fn monster(&self, index: usize) -> Result<&MonsterRecord, RuleError>;
    /// How many monsters are in the battle.
    fn monster_count(&self) -> usize;
    /// The bound of the draw for a random target: the monsters after the ten body slots.
    fn monster_target_bound(&self) -> i16 {
        (self.monster_count() as i16).wrapping_add(10)
    }
    /// Where a body stands.
    fn position(&self, body: usize) -> Result<(i16, i16), RuleError>;
    /// Which side a body is on.
    fn faction(&self, warriors: &[Warrior], body: usize) -> Result<i16, RuleError>;
    /// How far the map is shifted on the battlefield.
    fn map_shift(&self) -> (i16, i16);
    /// Whether the line between two cells is clear.
    fn visible(
        &self,
        mission: &OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError>;
    /// The body whose cell the area is best aimed at.
    fn best_choice(
        &self,
        warriors: &[Warrior],
        areas: &[u8],
        shape: usize,
        side: i16,
        distances: &[i16; 112],
        range: i16,
    ) -> Result<Option<usize>, RuleError>;
}

/// Target selection over the live battle.
pub(super) struct LiveSelection<'a>(pub &'a Combat, pub MapAliases<'a>);

impl SelectionView for LiveSelection<'_> {
    /// The monster at `index` in the live battle.
    fn monster(&self, index: usize) -> Result<&MonsterRecord, RuleError> {
        self.0
            .roster
            .records
            .get(index)
            .ok_or_else(|| RuleError::Invalid("spell caster index".into()))
    }
    /// The live battle's monster count.
    fn monster_count(&self) -> usize {
        self.0.roster.records.len()
    }
    /// Where the body stands on the live battlefield.
    fn position(&self, body: usize) -> Result<(i16, i16), RuleError> {
        self.0
            .get_body_position(body)
            .ok_or_else(|| RuleError::Invalid("spell body position".into()))
    }
    /// Which side the body is on.
    fn faction(&self, warriors: &[Warrior], body: usize) -> Result<i16, RuleError> {
        faction(self.0, warriors, body)
    }
    /// The live battlefield's map shift.
    fn map_shift(&self) -> (i16, i16) {
        self.0.field.map_shift
    }
    /// Whether the line of sight between two cells is clear on the live battlefield.
    fn visible(
        &self,
        mission: &OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError> {
        crate::combat::lifecycle::can_see(self.0, mission, from, to, 0)
    }
    /// The body whose cell the area is best aimed at, over the live battlefield.
    fn best_choice(
        &self,
        warriors: &[Warrior],
        areas: &[u8],
        shape: usize,
        side: i16,
        distances: &[i16; 112],
        range: i16,
    ) -> Result<Option<usize>, RuleError> {
        best_choice(
            (self.0, self.1),
            warriors,
            areas,
            shape,
            side,
            distances,
            range,
        )
    }
}
