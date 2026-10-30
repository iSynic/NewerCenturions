//! Party-only public adapter for the shared spell resolver.
use super::{DirectSpell, RandomState, RuleError, WarriorSpellResult, resolve_support_inner};
use centurions_content::Warrior;

/// A spell cast on warriors alone.
pub fn resolve_warriors(
    warriors: &[Warrior],
    packet: &[u8],
    level: i16,
    count: i16,
    mask: &[i16],
    armor_filter: bool,
    random: &mut RandomState,
) -> Result<WarriorSpellResult, RuleError> {
    if mask.len() < warriors.len() {
        return Err(RuleError::Invalid("spell resolution inputs".into()));
    }
    let mut mask = mask.to_vec();
    mask.resize(mask.len().max(10), 0);
    let result = resolve_support_inner(
        warriors,
        &[],
        &mask,
        &DirectSpell {
            packet,
            level,
            count,
            armor_filter,
            weapon_sound: 0,
        },
        (None, None),
        None,
        random,
    )?;
    Ok(WarriorSpellResult {
        warriors: result.warriors,
        targets: result.targets,
        race_excluded: result.race_excluded,
    })
}
