//! Ammunition transactions shared by command, shop and occupied-cell callers.
mod context;
mod fitting;
mod refill;
mod selection;

pub use context::LoadedItems;
pub use context::definition;
pub use fitting::fit;
pub use refill::refit;
pub(crate) use refill::refit_with_record;
pub use selection::compatible_ammunition;

use crate::{RandomState, RuleError};
use centurions_content::{InventoryRecord, Warrior, word};

/// How a refit is run: in a battle or not, forced, without sounds, and with or without the warning
/// when it cannot complete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefitFlags {
    pub in_combat: bool,
    pub force: bool,
    pub silent: bool,
    pub warnings: bool,
}

/// A request to load a reserve of ammunition into the weapon in a slot. When present, the
/// loot-quality bytes stand for the stock of reserve 200.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FitRequest {
    pub slot: usize,
    pub reserve_index: usize,
    pub in_combat: bool,
    pub loot_quality: Option<[u8; 4]>,
}

/// What fitting or refitting ammunition did: the warrior and reserves afterward, the items loaded,
/// the sounds and warnings, and whether the ammunition changed.
#[derive(Debug, PartialEq, Eq)]
pub struct AmmunitionResult {
    pub warrior: Warrior,
    pub reserves: Vec<i32>,
    pub loaded: LoadedItems,
    /// The value refit returns, kept for callers that compare it.
    pub return_code: i16,
    outcome: AmmunitionOutcome,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
    pub changed_ammo: bool,
    pub(crate) inventory_record: Option<InventoryRecord>,
    feedback: Vec<(usize, crate::Event)>,
    /// Party-wide writes from battery removal and re-wear, applied on commit.
    global_writes: Vec<crate::equipment::GlobalWrite>,
}

/// How a refit ended: not complete, complete, or one reserve was fitted.
#[derive(Debug, PartialEq, Eq)]
enum AmmunitionOutcome {
    RefitIncomplete,
    RefitCompleted,
    Fitted { reserve_index: usize },
}

impl AmmunitionResult {
    /// Whether the refit filled every weapon it set out to.
    pub(crate) fn completed_refit(&self) -> bool {
        self.outcome == AmmunitionOutcome::RefitCompleted
    }

    /// Settles the result: the return value follows the outcome and the random numbers drawn are
    /// kept.
    fn finish(mut self, random: &mut RandomState, next_random: RandomState) -> Self {
        self.return_code = match self.outcome {
            AmmunitionOutcome::RefitIncomplete => 0,
            AmmunitionOutcome::RefitCompleted => 1,
            AmmunitionOutcome::Fitted { reserve_index } => reserve_index as i16,
        };
        *random = next_random;
        self
    }
}

/// The refusal given when ammunition handling needs something the reconstruction does not carry.
fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "ammunition",
        dependency,
    }
}

/// Starts a result for a warrior with the reserves and loaded items as they stand; the reserves
/// must number 200.
fn initial(
    warrior: &Warrior,
    reserves: &[i32],
    loaded: &LoadedItems,
) -> Result<AmmunitionResult, RuleError> {
    if reserves.len() != 200 {
        return Err(RuleError::Invalid(
            "ammunition reserve vector length".into(),
        ));
    }
    warrior.inventory()?;
    loaded.validate()?;
    Ok(AmmunitionResult {
        warrior: warrior.clone(),
        reserves: reserves.to_vec(),
        loaded: loaded.clone(),
        return_code: 0,
        outcome: AmmunitionOutcome::RefitIncomplete,
        sounds: vec![],
        warnings: vec![],
        changed_ammo: false,
        inventory_record: None,
        feedback: vec![],
        global_writes: vec![],
    })
}

/// Applies a finished result to the session: the party's reserves, the warrior, and the sounds and
/// warnings.
pub(crate) fn commit_result(
    state: &mut crate::Session,
    member: usize,
    result: AmmunitionResult,
) -> Result<Vec<crate::Event>, RuleError> {
    state.apply_equipment_globals(&result.global_writes)?;
    let show_level_refusals =
        state.combat.is_none() || result.warrior.record().computer_control()? == 0;
    if let Some(item) = result.loaded.item.as_deref() {
        state.globals.loaded_item_field32 = word(item, 32)?;
    }
    let projected = if let Some(combat) = state.combat.as_mut() {
        &mut combat.weapon_traits
    } else {
        &mut state.globals.weapon_traits
    };
    if let Some(weapon) = result.loaded.weapon.as_deref() {
        projected.load_weapon(weapon)?;
    }
    if let Some(ammo) = result.loaded.ammo.as_deref() {
        projected.load_ammunition(ammo)?;
    }
    state.warriors[member] = result.warrior;
    state.ammunition = result.reserves;
    state.globals.loaded_items = result.loaded;
    if let Some(record) = result.inventory_record {
        state.globals.inventory_record = record;
    }
    let mut feedback = result.feedback.into_iter().peekable();
    let mut events = Vec::new();
    for (index, sound) in result.sounds.iter().map(Some).chain([None]).enumerate() {
        while feedback.peek().is_some_and(|(at, _)| *at == index) {
            let (_, event) = feedback.next().unwrap();
            if show_level_refusals
                || !matches!(
                    event,
                    crate::Event::Feedback(crate::feedback::GameFeedback::EquipmentLevel { .. })
                )
            {
                events.push(event);
            }
        }
        if let Some(&resource_id) = sound {
            events.push(crate::Event::Sound { resource_id });
        }
    }
    Ok(events)
}

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/native/ammunition_work_reference.rs"]
mod reference_tests;
