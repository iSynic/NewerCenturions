use crate::{
    Command, Phase, Presentation, RuleError, Session, combat, feedback, shop,
    warrior_management_available,
};

pub(super) fn check(command: &Command, state: &mut Session) -> Result<(), RuleError> {
    if state.retained_walk_choice.is_some() {
        return if matches!(command, Command::AnswerOccupiedChoice { .. }) {
            Ok(())
        } else {
            Err(RuleError::Refused { feedback: vec![] })
        };
    }
    if matches!(command, Command::AnswerOccupiedChoice { .. }) {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state
        .combat
        .as_ref()
        .is_some_and(|combat| combat.selection_pending)
        && !matches!(
            command,
            Command::ResumeCombatSelection
                | Command::ViewWarrior { .. }
                | Command::ViewCreature { .. }
        )
    {
        return Err(RuleError::Refused {
            feedback: vec![feedback::NativeFeedback::Silent],
        });
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| matches!(p.presentation, Some(Presentation::Clinic { .. })))
        && !matches!(
            command,
            Command::ClinicControl { .. }
                | Command::ViewWarrior { .. }
                | Command::UpgradeWarrior { .. }
                | Command::EnterWarriorUpgrade { .. }
                | Command::EditWarriorUpgrade { .. }
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if matches!(
        command,
        Command::SetWarriorAppearance { .. }
            | Command::UpgradeWarrior { .. }
            | Command::EnterWarriorUpgrade { .. }
            | Command::EditWarriorUpgrade { .. }
            | Command::EditCarriedItem { .. }
            | Command::EditWarriorLevel { .. }
            | Command::EditWarriorScans { .. }
    ) && !warrior_management_available(state)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.medical_continuation.is_some()
        && !matches!(
            command,
            Command::SelectMedicalTarget { .. } | Command::AdjustTargetQuantity { .. }
        )
    {
        return Err(RuleError::Invalid(
            "finish choosing the medical target before acting".into(),
        ));
    }
    if state.promotion.is_some()
        && !matches!(
            command,
            Command::PromotionControl { .. } | Command::PromotionAnswer { .. }
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.encounter_actions.is_some()
        && !matches!(command, Command::EncounterActionControl { .. })
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.outdoor_psi.is_some()
        && !matches!(
            command,
            Command::SelectOutdoorPsiTarget { .. }
                | Command::FinishOutdoorPsi
                | Command::CancelOutdoorPsi
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state
        .combat
        .as_ref()
        .is_some_and(|c| c.capture_actor.is_some())
        && !matches!(
            command,
            Command::SelectCaptureTarget { .. } | Command::CancelCapture
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.psi_selection.is_some()
        && !matches!(
            command,
            Command::SelectPsiTarget { .. }
                | Command::FirePsiSelection
                | Command::FirePreparedPsi { .. }
                | Command::FirePreparedPsiOnWarrior { .. }
                | Command::ClearPsiTargets
                | Command::AdjustTargetQuantity { .. }
                | Command::CancelPsiSelection
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state
        .combat
        .as_ref()
        .is_some_and(|c| c.firearm_selection.is_some())
        && !matches!(
            command,
            Command::ShootEnemy { .. }
                | Command::FireSelectedTargets
                | Command::CancelFireTargets
                | Command::BeginFireTargets
                | Command::AdjustTargetQuantity { .. }
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if !matches!(
        command,
        Command::BeginFireTargets
            | Command::AdjustTargetQuantity { .. }
            | Command::ShootEnemy { .. }
            | Command::ShootAt { .. }
            | Command::FireSelectedTargets
            | Command::CancelFireTargets
    ) {
        state.target_quantity = None;
    }
    if combat::runtime::controlled_warrior(state).is_some()
        && matches!(
            command,
            Command::Move { .. }
                | Command::AttackEnemy { .. }
                | Command::AttackFriend { .. }
                | Command::ShootEnemy { .. }
                | Command::ShootAt { .. }
                | Command::Guard
                | Command::DelayTurn
                | Command::UndoCombatMove
                | Command::RequestCombatEscape
                | Command::EscapeCombat
                | Command::SelectCombatWeapon { .. }
                | Command::CastPsi { .. }
                | Command::BeginPsiTargeting { .. }
                | Command::BeginFireTargets
                | Command::FirePreparedPsi { .. }
                | Command::FirePreparedPsiOnWarrior { .. }
                | Command::CastPsiOnWarrior { .. }
        )
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
        && !shop::command_allowed(command)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.phase == Phase::Combat
        && state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_some())
        && !matches!(
            command,
            Command::Continue
                | Command::EndMission
                | Command::SelectSurvivor { .. }
                | Command::Choose { .. }
                | Command::ChooseWarrior { .. }
                | Command::SelectMenuEntry { .. }
                | Command::SubmitText { .. }
                | Command::BeginEncounterActions
                | Command::CastEncounterPsi { .. }
                | Command::EncounterActionControl { .. }
                | Command::UseItem { .. }
                | Command::InspectInventory { .. }
                | Command::OpenInventory { .. }
                | Command::SwitchInventoryOwner { .. }
                | Command::ScrollInventory { .. }
                | Command::HighlightInventoryRow { .. }
                | Command::ShowItemInformation { .. }
                | Command::RetargetItemInformation { .. }
                | Command::ListItemUsers { .. }
                | Command::ShowLootItemInformation { .. }
                | Command::ShowDraggedItemInformation { .. }
                | Command::ChooseAmmunition { .. }
                | Command::CancelMenu
                | Command::DropItem { .. }
                | Command::TakeLoot { .. }
                | Command::StoreLoot { .. }
                | Command::KeepCurrentSpells
                | Command::BeginPromotion
                | Command::PromotionControl { .. }
                | Command::PromotionAnswer { .. }
        )
    {
        return Err(RuleError::Invalid(
            "continue the combat story before acting".into(),
        ));
    }
    Ok(())
}
