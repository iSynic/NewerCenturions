use crate::{Command, Event, RuleError, Session};
use centurions_content::Content;

mod admission;
mod boundary;
mod characters;
mod combat;
mod inventory;
mod transaction;
mod world;
pub use transaction::apply;

fn apply_inner(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    admission::check(&command, state)?;
    match command {
        command @ (Command::FieldControl { .. }
        | Command::ClinicControl { .. }
        | Command::ViewCreature { .. }
        | Command::ViewNote { .. }
        | Command::EndMission
        | Command::SetDifficulty { .. }
        | Command::EnterMission { .. }
        | Command::Move { .. }
        | Command::Continue
        | Command::KeepCurrentSpells
        | Command::BeginPromotion
        | Command::PromotionControl { .. }
        | Command::PromotionAnswer { .. }
        | Command::Choose { .. }
        | Command::CancelMenu
        | Command::SelectMenuEntry { .. }
        | Command::SubmitText { .. }
        | Command::BeginEncounterActions
        | Command::CastEncounterPsi { .. }
        | Command::EncounterActionControl { .. }
        | Command::ChooseWarrior { .. }
        | Command::Encounter) => world::execute(command, state, content),
        command @ (Command::OpenShop
        | Command::CloseShop
        | Command::BuyItem { .. }
        | Command::SellItem { .. }
        | Command::TransferItem { .. }
        | Command::RefitParty
        | Command::DropItem { .. }
        | Command::PoolCredits
        | Command::ShareCredits
        | Command::TakeLoot { .. }
        | Command::StoreLoot { .. }
        | Command::TakeCrate { .. }
        | Command::EquipItem { .. }
        | Command::InspectInventory { .. }
        | Command::UnequipItem { .. }
        | Command::Reload { .. }
        | Command::FitAmmunition { .. }
        | Command::UseAmmunitionPack { .. }
        | Command::UseItem { .. }
        | Command::AddToCrate { .. }
        | Command::AddToBooty { .. }) => inventory::execute(command, state, content),
        command @ (Command::SwapCombatants { .. }
        | Command::AutomaticBandage
        | Command::BeginAutomaticTurn
        | Command::DelayTurn
        | Command::UndoCombatMove
        | Command::ToggleCombatAuto { .. }
        | Command::RequestCombatEscape
        | Command::EscapeCombat
        | Command::SelectCombatWeapon { .. }
        | Command::ViewCombatBody { .. }
        | Command::Guard
        | Command::SelectSurvivor { .. }
        | Command::EndTurn
        | Command::RunEnemyTurn
        | Command::ResumeCombatSelection
        | Command::RunControlledTurn
        | Command::BeginCapture
        | Command::SelectCaptureTarget { .. }
        | Command::CancelCapture
        | Command::AttackEnemy { .. }
        | Command::AttackFriend { .. }
        | Command::ShootEnemy { .. }
        | Command::ShootAt { .. }
        | Command::FireSelectedTargets
        | Command::CancelFireTargets
        | Command::CastPsi { .. }
        | Command::CastPartyPsi { .. }
        | Command::CastPsiOnWarrior { .. }
        | Command::BeginMedicalItem { .. }
        | Command::BeginLearnedHeal { .. }
        | Command::BeginPsiSelection { .. }
        | Command::SelectPsiTarget { .. }
        | Command::FirePsiSelection
        | Command::BeginPsiTargeting { .. }
        | Command::FirePreparedPsi { .. }
        | Command::FirePreparedPsiOnWarrior { .. }
        | Command::BeginFireTargets
        | Command::AdjustTargetQuantity { .. }
        | Command::ClearPsiTargets
        | Command::CancelPsiSelection
        | Command::SelectOutdoorPsiTarget { .. }
        | Command::FinishOutdoorPsi
        | Command::CancelOutdoorPsi
        | Command::SelectMedicalTarget { .. }
        | Command::StopBleeding { .. }
        | Command::SetFirearmMode { .. }) => combat::execute(command, state, content),
        command @ (Command::WithPreferences { .. } | Command::CombatClick { .. }) => {
            boundary::execute(command, state, content)
        }
        command @ (Command::CreationDialog
        | Command::CreationEquipment
        | Command::CreationScans
        | Command::GenerateWarrior { .. }
        | Command::SelectWarriors { .. }
        | Command::ReplaceSquad { .. }
        | Command::ReorderSquad { .. }
        | Command::EditWarriorScans { .. }
        | Command::EditWarriorLevel { .. }
        | Command::EditCarriedItem { .. }
        | Command::SetWarriorAppearance { .. }
        | Command::EditWarriorUpgrade { .. }
        | Command::UpgradeWarrior { .. }
        | Command::ViewWarrior { .. }
        | Command::RecalculateStatistics { .. }
        | Command::ImportStarters
        | Command::SelectStarters { .. }) => characters::execute(command, state, content),
    }
}

pub(crate) use inventory::insert;
