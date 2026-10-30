//! The script operations by name. The numbers are New Centurions' own, not
//! Realmz's. Scripts are stored as record bytes, so this type is never
//! serialized, and an unknown number still reports a diagnostic. Death and
//! battle-round scripts use the same names; round-only operations run there and
//! the rest run here.
//!
//! One table names each operation, its number and the adapter that runs it.

/// The exploration script adapter that runs an operation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Adapter {
    Battles,
    Party,
    Branches,
    /// Land operations that read their extra before dispatch.
    LandExtra,
    Travel,
    MapMutation,
    Encounters,
    Presentation,
}

macro_rules! operations {
    ($($name:ident = $opcode:literal => $adapter:ident,)*) => {
        /// The script operations by name; the macro below ties each to its number and to the code
        /// that runs it.
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub(crate) enum Operation {
            $($name,)*
            /// A number the game does not handle: the script moves on to the next action.
            Unarmed(i16),
        }

        impl Operation {
            /// The operation a script action number stands for; a number the game does not handle
            /// is unarmed.
            pub(crate) fn from_opcode(opcode: i16) -> Self {
                match opcode {
                    $($opcode => Self::$name,)*
                    other => Self::Unarmed(other),
                }
            }

            /// The script action number of an operation.
            pub(crate) fn opcode(self) -> i16 {
                match self {
                    $(Self::$name => $opcode,)*
                    Self::Unarmed(other) => other,
                }
            }

            /// The group of code that runs the operation.
            pub(super) fn adapter(self) -> Adapter {
                match self {
                    $(Self::$name => Adapter::$adapter,)*
                    Self::Unarmed(_) => Adapter::Presentation,
                }
            }
        }
    };
}

operations! {
    Clinic = -32 => Party,
    ClearObjects = -12 => MapMutation,
    NoAction = 0 => Presentation,
    Dialogue = 1 => Presentation,
    Battle = 2 => Presentation,
    Question = 3 => Presentation,
    Menu = 4 => Encounters,
    TypedMenu = 5 => Encounters,
    Shop = 6 => Party,
    ReplaceMapEvent = 7 => MapMutation,
    CopyMapEvent = 8 => Party,
    Sound = 9 => Presentation,
    Treasure = 10 => Party,
    Experience = 11 => Party,
    ChangeTiles = 12 => MapMutation,
    SetEventChances = 13 => MapMutation,
    ChooseParty = 14 => Presentation,
    SelectedHealth = 15 => Party,
    PartyHealth = 16 => Party,
    SelectedSpell = 17 => Battles,
    PartySpell = 18 => Battles,
    PartyItemBranch = 21 => Encounters,
    ConsumePartyItems = 22 => Party,
    ChangeRegion = 23 => MapMutation,
    EndScript = 24 => Presentation,
    PersistEvent = 25 => Presentation,
    Pause = 26 => Presentation,
    RevealNote = 29 => Party,
    FilterParty = 30 => Party,
    ChooseWarrior = 31 => Presentation,
    PaymentBranch = 33 => Encounters,
    DisableMenuEntry = 35 => Encounters,
    Confiscate = 36 => Party,
    ItemPrerequisiteBranch = 38 => Encounters,
    ReplaceScript = 39 => Party,
    ChanceBranch = 42 => Encounters,
    Travel = 45 => Travel,
    FlagBranch = 46 => Encounters,
    SetMissionFlag = 47 => Party,
    SelectedBattle = 48 => Battles,
    SelectPartyBySave = 52 => Battles,
    EditSchedule = 54 => Battles,
    NpcBattle = 56 => Battles,
    Movie = 62 => Presentation,
    ChangeFatigue = 68 => Party,
    SetRecoveryRestrictions = 69 => Party,
    Delay = 73 => Presentation,
    UpdateVariable = 76 => Encounters,
    VariableBranch = 77 => Encounters,
    RandomBranch = 85 => Encounters,
    AllyClassBranch = 87 => Encounters,
    RemoveAllies = 88 => Party,
    AddAlly = 89 => Party,
    Registration = 98 => Presentation,
    ScenarioRegistration = 99 => Presentation,
    CombatCleanup = 100 => Battles,
    UndoLastStep = 101 => Battles,
    ResetExperience = 102 => Party,
    SuppressAllies = 105 => Presentation,
    GroupBattle = 107 => Battles,
    AdjustStatistics = 108 => Party,
    AddMissionScore = 109 => Party,
    StageTransition = 110 => Battles,
    LimitedShopOrTreasure = 111 => Party,
    SetMapPicture = 113 => Presentation,
    EndCombat = 119 => Battles,
    ReplaceActor = 120 => Battles,
    NeutralizeActors = 122 => Battles,
    SetFactionCondition = 123 => Battles,
    SummonActors = 124 => Battles,
    RemoveActorClass = 125 => Battles,
    ReplaceRoundScript = 126 => Battles,
    MapEffect = 128 => Presentation,
    SetRandomBattles = 104 => Presentation,
    ContinueIfMonster = 127 => Battles,
    DisplayRandomString = 19 => Presentation,
    AdjustPickedPsi = 74 => Presentation,
    CheckRegisteredScenario = 84 => Presentation,
    SetCampingAbility = 66 => Presentation,
    RedrawScreen = 28 => Presentation,
    ShiftParty = 61 => Travel,
    BoatCampingBranch = 103 => Travel,
    SetLandLight = 106 => Travel,
    ChangeLandTiles = 57 => LandExtra,
    PartyPosition = 70 => LandExtra,
    ResizeRegion = 92 => LandExtra,
    PartyConditionBranch = 40 => Branches,
    GameTimeBranch = 64 => Branches,
    ItemChargesBranch = 67 => Branches,
    QuestRangeBranch = 72 => Branches,
    PsiPointsBranch = 75 => Branches,
    CharacterConditionBranch = 81 => Branches,
    TeleportActivate = 20 => Travel,
    BreakEncounter = 34 => Encounters,
    PickedBranch = 55 => Branches,
    MiscellaneousBranch = 86 => Branches,
    GiveCondition = 43 => Party,
    AlterShop = 51 => Party,
    RandomTreasure = 65 => Party,
    CoordinateDisplay = 71 => Presentation,
    DestroyCategories = 121 => Battles,
    DropAllEquipment = 91 => Party,
    PickByTrait = 50 => Party,
    PickByCaste = 53 => Party,
    DifficultyBranch = 58 => Encounters,
    TileBranch = 59 => Encounters,
    DropMemberCredits = 60 => Party,
    AlterGameTime = 63 => Travel,
    TakeVictoryPoints = 90 => Party,
    DrawPicture = 27 => Presentation,
    DisableEncounterChoice = 41 => Encounters,
    ChoosePartyInverted = -14 => Presentation,
    EliminateComplexBranch = 44 => Presentation,
}
