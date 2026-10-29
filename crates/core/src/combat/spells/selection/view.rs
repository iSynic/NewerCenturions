//! Read-only targeting inputs keep the active prefix separate from the caster.
use super::{Combat, RuleError, Warrior, best_choice, faction, position};
use centurions_content::{combat::MonsterRecord, mission::OpeningMission};

pub(crate) trait SelectionView {
    fn npc(&self, index: usize) -> Result<&MonsterRecord, RuleError>;
    fn npc_count(&self) -> usize;
    fn npc_target_bound(&self) -> i16 {
        (self.npc_count() as i16).wrapping_add(10)
    }
    fn position(&self, body: usize) -> Result<(i16, i16), RuleError>;
    fn faction(&self, warriors: &[Warrior], body: usize) -> Result<i16, RuleError>;
    fn map_shift(&self) -> (i16, i16);
    fn visible(
        &self,
        mission: &OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError>;
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

pub(super) struct LiveSelection<'a>(pub &'a Combat, pub crate::combat::map_alias::MapAliases<'a>);

impl SelectionView for LiveSelection<'_> {
    fn npc(&self, index: usize) -> Result<&MonsterRecord, RuleError> {
        self.0
            .enemies
            .records
            .get(index)
            .ok_or_else(|| RuleError::Invalid("spell caster index".into()))
    }
    fn npc_count(&self) -> usize {
        self.0.enemies.records.len()
    }
    fn position(&self, body: usize) -> Result<(i16, i16), RuleError> {
        position(self.0, body).ok_or_else(|| RuleError::Invalid("spell body position".into()))
    }
    fn faction(&self, warriors: &[Warrior], body: usize) -> Result<i16, RuleError> {
        faction(self.0, warriors, body)
    }
    fn map_shift(&self) -> (i16, i16) {
        self.0.map.shift
    }
    fn visible(
        &self,
        mission: &OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError> {
        crate::combat::lifecycle::can_see(self.0, mission, from, to, 0)
    }
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
