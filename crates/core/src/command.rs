use crate::{CommandPreferences, MedicalTarget, creation, field_controls};
use centurions_content::{InventoryRecord, Warrior};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    FieldControl {
        action: field_controls::Action,
    },
    ClinicControl {
        item: u16,
    },
    OpenShop,
    CloseShop,
    BuyItem {
        warrior: usize,
        stock: usize,
    },
    SellItem {
        warrior: usize,
        slot: usize,
    },
    SwapCombatants {
        body: i16,
    },
    AttackFriend {
        body: i16,
    },
    ViewCreature {
        index: usize,
        ally: bool,
    },
    ViewNote {
        index: usize,
    },
    WithPreferences {
        command: Box<Command>,
        preferences: CommandPreferences,
    },
    AutomaticBandage,
    CreationDialog,
    CreationEquipment,
    CreationScans,
    GenerateWarrior {
        request: creation::Request,
    },
    SelectWarriors {
        warriors: Vec<Warrior>,
    },
    ReplaceSquad {
        warriors: Vec<Warrior>,
    },
    ReorderSquad {
        destinations: Vec<usize>,
    },
    EndMission,
    RefitParty,
    DropItem {
        warrior: usize,
        slot: usize,
    },
    TransferItem {
        from: usize,
        to: usize,
        slot: usize,
    },
    SetWarriorAppearance {
        warrior: usize,
        portrait: bool,
        resource: i16,
    },
    ViewWarrior {
        warrior: usize,
    },
    CombatClick {
        x: i16,
        y: i16,
    },
    UpgradeWarrior {
        warrior: usize,
        selector: u16,
    },
    SetDifficulty {
        level: i16,
    },
    PoolCredits,
    ShareCredits,
    RecalculateStatistics {
        warrior: usize,
    },
    EditWarriorUpgrade {
        warrior: usize,
        selector: u16,
    },
    EditCarriedItem {
        warrior: usize,
        slot: usize,
        control: u16,
    },
    EditWarriorLevel {
        warrior: usize,
    },
    EditWarriorScans {
        warrior: usize,
        controls: Vec<u16>,
    },
    ImportStarters,
    SelectStarters {
        indices: Vec<usize>,
    },
    EnterMission {
        name: String,
    },
    Encounter,
    Move {
        dx: i8,
        dy: i8,
    },
    Continue,
    KeepCurrentSpells,
    BeginPromotion,
    PromotionControl {
        item: u16,
    },
    PromotionAnswer {
        yes: bool,
    },
    Choose {
        answer: bool,
    },
    CancelMenu,
    SelectMenuEntry {
        index: usize,
    },
    BeginEncounterActions,
    CastEncounterPsi {
        warrior: usize,
        spell: i16,
        power: i16,
    },
    EncounterActionControl {
        item: u16,
    },
    SubmitText {
        text: String,
    },
    ChooseWarrior {
        index: usize,
    },
    TakeLoot {
        slot: usize,
        warrior: usize,
    },
    StoreLoot {
        slot: usize,
    },
    TakeCrate {
        slot: usize,
        warrior: usize,
    },
    EquipItem {
        warrior: usize,
        slot: usize,
    },
    InspectInventory {
        members: [usize; 2],
        scroll: [usize; 2],
        selected: Option<(usize, usize)>,
        #[serde(default)]
        shop_family: usize,
    },
    UnequipItem {
        warrior: usize,
        slot: usize,
    },
    BeginAutomaticTurn,
    DelayTurn,
    UndoCombatMove,
    ToggleCombatAuto {
        warrior: usize,
    },
    RequestCombatEscape,
    EscapeCombat,
    SelectCombatWeapon {
        slot: usize,
    },
    ViewCombatBody {
        body: i16,
    },
    EndTurn,
    Guard,
    BeginCapture,
    SelectCaptureTarget {
        x: i16,
        y: i16,
    },
    CancelCapture,
    SelectSurvivor {
        row: usize,
    },
    RunEnemyTurn,
    ResumeCombatSelection,
    RunControlledTurn,
    AttackEnemy {
        body: i16,
    },
    ShootEnemy {
        body: i16,
    },
    ShootAt {
        x: i16,
        y: i16,
    },
    FireSelectedTargets,
    CancelFireTargets,
    CastPsi {
        #[serde(default)]
        rotation: u8,
        spell: i16,
        power: i16,
        target: Option<(i16, i16)>,
    },
    BeginPsiSelection {
        spell: i16,
        power: i16,
    },
    CastPsiOnWarrior {
        warrior: usize,
        spell: i16,
        power: i16,
    },
    SelectPsiTarget {
        x: i16,
        y: i16,
    },
    FirePsiSelection,
    ClearPsiTargets,
    BeginPsiTargeting {
        spell: i16,
        power: i16,
    },
    FirePreparedPsi {
        x: i16,
        y: i16,
        rotation: u8,
    },
    FirePreparedPsiOnWarrior {
        warrior: usize,
    },
    BeginFireTargets,
    AdjustTargetQuantity {
        increase: bool,
    },
    CancelPsiSelection,
    SelectOutdoorPsiTarget {
        warrior: usize,
    },
    FinishOutdoorPsi,
    CancelOutdoorPsi,
    CastPartyPsi {
        warrior: usize,
        spell: i16,
        power: i16,
    },
    Reload {
        warrior: usize,
    },
    FitAmmunition {
        warrior: usize,
        slot: usize,
        ammunition: i16,
    },
    SetFirearmMode {
        warrior: usize,
        enabled: bool,
    },
    UseAmmunitionPack {
        warrior: usize,
        slot: usize,
    },
    UseItem {
        warrior: usize,
        slot: usize,
    },
    BeginMedicalItem {
        warrior: usize,
        slot: usize,
    },
    BeginLearnedHeal {
        power: i16,
    },
    SelectMedicalTarget {
        selection: MedicalTarget,
    },
    StopBleeding {
        warrior: Option<usize>,
    },
    AddToCrate {
        item: InventoryRecord,
    },
    AddToBooty {
        item: InventoryRecord,
    },
}
