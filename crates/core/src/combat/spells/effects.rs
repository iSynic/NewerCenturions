//! Spell packet effect numbers, usually taken as the effect's absolute value.
//! Effects 1..=40 add the duration of condition `effect - 1`; spelllist then
//! applies the follow-ups below.

/// Condition index 1 (Helpless): the follow-up clears movement.
pub(crate) const HELPLESS: i16 = 2;
/// Condition index 2 (Entangled): the follow-up halves movement.
pub(crate) const ENTANGLED: i16 = 3;
/// Condition index 6 (MovingSlowly): the follow-up halves movement.
pub(crate) const MOVING_SLOWLY: i16 = 7;
/// Raises a body below -9 stamina on automatic, with a quarter of its maximum stamina.
pub(crate) const QUARTER_REVIVAL: i16 = 26;
/// Party duration count rule, played with sound 601.
pub(crate) const PARTY_DURATION_COUNT: i16 = 50;
/// The healing effect a warrior may cast on a chosen ally.
pub(crate) const HEALING: i16 = 57;
/// The summon arm, which follows the movement arm.
pub(crate) const SUMMON: i16 = 58;
/// Revival: leaves automatic and sets stamina to -9.
pub(crate) const REVIVAL: i16 = 64;
/// Returns unreported after its write and resets fatigue; the damage total is kept.
pub(crate) const FATIGUE_RESET: i16 = 68;
/// Clears condition indices 28 and 3; the mode-7 arm writes the scenario-directory
/// tail (docs/memory-alias-audit.md).
pub(crate) const SCENARIO_TAIL: i16 = 110;

/// Effects 55 and 56 take resolvespell's movement arm in place of the body loop.
pub(crate) fn is_movement(effect: i16) -> bool {
    matches!(effect, 55 | 56)
}

/// Effects 60 and 61 move Psi; with defense category 5 they are drains.
pub(crate) fn is_psi_transfer(effect: i16) -> bool {
    matches!(effect, 60 | 61)
}

/// Effects 27 and 49 read the target's stamina.
pub(crate) fn reads_target_stamina(effect: i16) -> bool {
    matches!(effect, 27 | 49)
}

/// spelllist effects that act on the (possibly reflected) target body, so a reflected
/// one needs its caster.
pub(crate) fn acts_on_target(effect: i16) -> bool {
    matches!(
        effect,
        2 | 3 | 7 | 10 | 26 | 27 | 49 | 51..=54 | 59..=61 | 64 | 110
    )
}

/// The target effects that read the monster record: `acts_on_target` without
/// effects 26 and 64, which address the warrior table even for a monster body
/// (`warrior_alias`).
pub(crate) fn reads_monster_record(effect: i16) -> bool {
    matches!(effect, 2 | 3 | 7 | 10 | 27 | 49 | 51..=54 | 59..=61 | 110)
}
