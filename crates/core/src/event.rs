use crate::feedback;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event {
    ItemUpgraded {
        warrior: usize,
        slot: usize,
    },
    ItemTransferred {
        from: usize,
        to: usize,
        item: i16,
    },
    ExchangeOffered {
        body: i16,
    },
    FriendAttackOffered {
        body: i16,
    },
    MissionEnded {
        defeated: bool,
    },
    CreatureViewed {
        index: usize,
        ally: bool,
        record: Vec<u8>,
        description: String,
    },
    NoteViewed {
        index: usize,
        record: Vec<u8>,
        text: String,
    },
    Feedback(feedback::NativeFeedback),
    CreditsPooled {
        total: i32,
    },
    CreditsShared {
        each: i16,
    },
    StatisticsCalculated {
        warrior: usize,
    },
    StartersImported {
        count: usize,
    },
    ItemAdded {
        slot: usize,
        crate_storage: bool,
    },
    Sound {
        resource_id: i16,
    },
    CombatEscapeOffered,
    Delay {
        ticks: u32,
    },
    StorageFull,
    CollectionRejected {
        warning: i16,
    },
    MissionEntered {
        title: String,
        x: i32,
        y: i32,
    },
    Moved {
        x: i32,
        y: i32,
    },
    StoryPresented,
    /// newland27 draws the PICT centred in a 320-pixel square of the map window,
    /// where it stays until the map is drawn again.
    PictureDrawn {
        resource_id: i16,
    },
    /// newland28 redraws the screen over a drawn picture.
    ScreenRedrawn,
    Message {
        id: i16,
        text: String,
    },
    AllyAdded {
        definition: i16,
    },
    OutdoorEffect {
        effect: i16,
        x: i16,
        y: i16,
        size: i16,
        delay: i16,
    },
    CombatStarted {
        battle: i16,
        enemies: usize,
    },
    TacticalMoved {
        body: i16,
        x: i16,
        y: i16,
    },
    AttackResolved {
        attacker: i16,
        target: i16,
        damage: i16,
        defeated: bool,
    },
    SpellResolved {
        caster: Option<i16>,
        spell: i16,
        target: i16,
        damage: i16,
        defeated: bool,
    },
    CombatantAdded {
        body: i16,
        definition: i16,
    },
    CombatantRemoved {
        body: i16,
    },
    TurnStarted {
        body: i16,
        round: i16,
    },
    BattleEnded {
        victory: bool,
    },
    TimeAdvanced {
        day: i16,
        hour: i16,
        minute: i16,
    },
    ScheduleRescheduled {
        slot: usize,
        next_day: i16,
        dispatched: bool,
    },
    AmmunitionAdded {
        ammunition: i16,
        quantity: i16,
        reserve: i32,
    },
    LevelAdvanced {
        warrior: usize,
        level: i16,
    },
    EditorLevelAdvanced {
        warrior: usize,
        level: i16,
        information: bool,
    },
}
