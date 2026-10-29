use crate::{CommandPreferences, MedicalTarget, creation, field_controls};
use centurions_content::{InventoryRecord, Warrior};
use serde::{Deserialize, Serialize};

/// The window of a click inside DLOG 149 (156988, 156a00, 156a6a).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemInformationList {
    #[default]
    Items,
    Owner,
    Shop,
}

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
    AnswerOccupiedChoice {
        exchange: Option<bool>,
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
    /// upgradewarrior's entry for a warrior, on opening and on each member switch.
    EnterWarriorUpgrade {
        warrior: usize,
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
    OpenInventory {
        warrior: usize,
        #[serde(default)]
        quick_info: bool,
    },
    SwitchInventoryOwner {
        warrior: usize,
        scroll: usize,
        #[serde(default)]
        quick_info: bool,
        /// The native selected display row WORD4471aa, kept across owners; zero
        /// at startup.
        #[serde(default, skip_serializing_if = "is_zero_row")]
        selected_row: i16,
    },
    ScrollInventory {
        warrior: usize,
        scroll: usize,
        delta: i16,
        #[serde(default)]
        quick_info: bool,
    },
    /// Clicking an Items row: the display row is WORD4471aa (0..10).
    HighlightInventoryRow {
        warrior: usize,
        scroll: usize,
        row: usize,
        quick_info: bool,
    },
    /// The Items info button (CNTL 140): showiteminfo on display row+scroll.
    ShowItemInformation {
        warrior: usize,
        scroll: usize,
        row: usize,
        quick_info: bool,
    },
    /// A list click while DLOG 149 is open. `member` and `slot` are the
    /// dialog's member and slot after the click: an Items row keeps the
    /// earlier member, the second list sets it, and a shop cell keeps both.
    RetargetItemInformation {
        warrior: usize,
        member: usize,
        slot: usize,
        quick_info: bool,
        #[serde(default)]
        list: ItemInformationList,
        /// The item ID of a dialog opened with member 0xffff.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        item: Option<i16>,
    },
    /// booty's option-click on a loot cell: showiteminfo on that item ID.
    ShowLootItemInformation {
        slot: usize,
    },
    /// moveicon's option-drag on a shop window row: `stock` marks a shop
    /// work cell, otherwise `slot` is the member's inventory slot.
    /// The Items ammunition button (CNTL 176) and the chosen MENU 1000 item
    /// of loadammoproc's popup on the row+scroll slot; zero when the item has
    /// no caliber and the button only warns.
    ChooseAmmunition {
        warrior: usize,
        scroll: usize,
        row: usize,
        item: u16,
    },
    ShowDraggedItemInformation {
        warrior: usize,
        slot: usize,
        #[serde(default)]
        stock: bool,
        #[serde(default)]
        quick_info: bool,
    },
    /// DLOG 149 items 1 and 2: showcondition154a46 mode 4 (`professions`
    /// false) or 3. `widened` is the shift key that walks the whole squad.
    ListItemUsers {
        warrior: usize,
        professions: bool,
        #[serde(default)]
        widened: bool,
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

fn is_zero_row(row: &i16) -> bool {
    *row == 0
}
