//! Ammunition transactions shared by command, shop and occupied-cell callers.
mod context;
mod fitting;
mod refill;
mod selection;

pub use context::WorkBuffers;
pub(crate) use context::definition;
pub use fitting::fit;
pub use refill::refit;
pub(crate) use refill::refit_with_record;
pub use selection::compatible_ammunition;

use crate::{RandomState, RuleError};
use centurions_content::{InventoryRecord, Warrior, word};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefitFlags {
    pub in_combat: bool,
    pub force: bool,
    pub silent: bool,
    pub warnings: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FitRequest {
    pub slot: usize,
    pub reserve_index: usize,
    pub in_combat: bool,
    pub loot_quality: Option<[u8; 4]>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AmmunitionResult {
    pub warrior: Warrior,
    pub reserves: Vec<i32>,
    pub work: WorkBuffers,
    /// Compatibility return for preserved primitive callers and native receipts.
    pub native_return: i16,
    outcome: AmmunitionOutcome,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
    pub changed_ammo: bool,
    pub(crate) retained_record: Option<InventoryRecord>,
}

#[derive(Debug, PartialEq, Eq)]
enum AmmunitionOutcome {
    RefitIncomplete,
    RefitCompleted,
    Fitted { reserve_index: usize },
}

impl AmmunitionResult {
    pub(crate) fn completed_refit(&self) -> bool {
        self.outcome == AmmunitionOutcome::RefitCompleted
    }

    fn finish(mut self, random: &mut RandomState, next_random: RandomState) -> Self {
        self.native_return = match self.outcome {
            AmmunitionOutcome::RefitIncomplete => 0,
            AmmunitionOutcome::RefitCompleted => 1,
            AmmunitionOutcome::Fitted { reserve_index } => reserve_index as i16,
        };
        *random = next_random;
        self
    }
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "ammunition",
        dependency,
    }
}

fn initial(
    warrior: &Warrior,
    reserves: &[i32],
    work: &WorkBuffers,
) -> Result<AmmunitionResult, RuleError> {
    if reserves.len() != 200 {
        return Err(RuleError::Invalid(
            "ammunition reserve vector length".into(),
        ));
    }
    warrior.inventory()?;
    work.validate()?;
    Ok(AmmunitionResult {
        warrior: warrior.clone(),
        reserves: reserves.to_vec(),
        work: work.clone(),
        native_return: 0,
        outcome: AmmunitionOutcome::RefitIncomplete,
        sounds: vec![],
        warnings: vec![],
        changed_ammo: false,
        retained_record: None,
    })
}

pub(crate) fn commit_result(
    state: &mut crate::Session,
    member: usize,
    result: AmmunitionResult,
) -> Result<Vec<crate::Event>, RuleError> {
    if let Some(item) = result.work.item.as_deref() {
        state.loaded_item_field32 = Some(word(item, 32)?);
    }
    let projected = if let Some(combat) = state.combat.as_mut() {
        &mut combat.weapon_work
    } else {
        &mut state.weapon_work
    };
    if let Some(weapon) = result.work.weapon.as_deref() {
        projected.load_weapon(weapon)?;
    }
    if let Some(ammo) = result.work.ammo.as_deref() {
        projected.load_ammunition(ammo)?;
    }
    state.warriors[member] = result.warrior;
    state.ammunition = result.reserves;
    state.ammunition_work = result.work;
    if let Some(record) = result.retained_record {
        state.inventory_record = Some(record);
    }
    Ok(result
        .sounds
        .into_iter()
        .map(|resource_id| crate::Event::Sound { resource_id })
        .chain(
            result
                .warnings
                .into_iter()
                .map(|warning| crate::Event::CollectionRejected { warning }),
        )
        .collect())
}

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/ammunition_work_tests.rs"]
mod work_tests;
