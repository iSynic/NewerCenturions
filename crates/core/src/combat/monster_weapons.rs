//! The weapons monsters carry. Battle setup takes the first table match; placement
//! keeps the last.
use crate::{RandomState, RuleError};
use centurions_content::records::{EnemyRecord, EnemyRecordMut};

/// Whether a weapon roll that matches several table entries takes the first or the last.
#[derive(Clone, Copy)]
pub(crate) enum MatchOrder {
    First,
    Last,
}

/// A weapon table: five bands of a hundred-sided roll, each with the weapon it gives.
type WeaponTable = [(i16, i16, i16); 5];

/// Nine five-entry weapon tables.
const TABLES: [WeaponTable; 9] = [
    [
        (0, 50, 10),
        (51, 60, 20),
        (61, 70, 71),
        (71, 95, 75),
        (96, 100, 24),
    ],
    [
        (0, 35, 65),
        (36, 70, 37),
        (71, 85, 125),
        (85, 94, 137),
        (95, 100, 138),
    ],
    [
        (0, 40, 120),
        (41, 60, 37),
        (61, 90, 125),
        (91, 95, 65),
        (96, 100, 138),
    ],
    [
        (0, 40, 81),
        (41, 80, 92),
        (81, 85, 81),
        (86, 93, 92),
        (94, 100, 136),
    ],
    [
        (0, 35, 81),
        (36, 70, 75),
        (71, 90, 81),
        (91, 95, 75),
        (96, 100, 136),
    ],
    [
        (0, 20, 37),
        (21, 40, 65),
        (41, 60, 75),
        (61, 80, 120),
        (81, 100, 120),
    ],
    [
        (0, 15, 140),
        (16, 30, 142),
        (31, 45, 120),
        (46, 75, 140),
        (76, 100, 120),
    ],
    [
        (0, 25, 92),
        (26, 50, 81),
        (51, 75, 120),
        (76, 90, 120),
        (91, 100, 120),
    ],
    [
        (0, 20, 1),
        (21, 40, 31),
        (41, 65, 75),
        (66, 90, 44),
        (91, 100, 31),
    ],
];

/// The weapon table a monster's negative weapon number selects, or none when the number names a
/// weapon directly.
fn table(selected: i16) -> Result<Option<&'static WeaponTable>, RuleError> {
    if selected >= 0 {
        return Ok(None);
    }
    TABLES
        .get(usize::from(selected.unsigned_abs()) - 1)
        .map(Some)
        .ok_or(RuleError::Unsupported {
            action: "combat.random_weapon",
            dependency: "monster weapon table past the nine tables combatsetup copies to its stack",
        })
}

/// Checks that the monster's weapon number names a table the game has.
pub(crate) fn validate(bytes: &[u8]) -> Result<(), RuleError> {
    table(EnemyRecord::from_bytes(bytes).weapon()?)?;
    Ok(())
}

/// Rolls the weapon for a monster whose weapon number selects a table, and gives it that weapon.
pub(crate) fn initialize(
    bytes: &mut [u8],
    random: &mut RandomState,
    order: MatchOrder,
) -> Result<(), RuleError> {
    let Some(table) = table(EnemyRecord::from_bytes(bytes).weapon()?)? else {
        return Ok(());
    };
    let roll = random.bound(100);
    let mut matches = table
        .iter()
        .filter(|&&(low, high, _)| low <= roll && roll <= high);
    let selected = match order {
        MatchOrder::First => matches.next(),
        MatchOrder::Last => matches.next_back(),
    }
    .map_or(0, |&(_, _, item)| item);
    let mut record = EnemyRecordMut::from_bytes(bytes);
    record.set_primary_weapon(selected);
    record.set_weapon(selected);
    Ok(())
}
