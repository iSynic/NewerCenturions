use crate::fixture_hex as hex;
use centurions_content::Content;
use centurions_core::{Command, Event, Session, apply};
use serde_json::Value;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
mod native_context;
pub mod psi_frame;
pub use native_context::{captured_session, legacy_save};
mod command_replay;
pub use command_replay::{current_combat, saved_command};

pub fn fixture_content(identity: &str) -> Content {
    Content {
        tutorials: vec![],
        identity: identity.into(),
        warriors: vec![],
        items: vec![],
        resources: vec![],
        original_services: None,
        scenario: None,
    }
}

pub trait BinaryInput {
    fn binary(&self) -> Cow<'_, [u8]>;
}
impl BinaryInput for str {
    fn binary(&self) -> Cow<'_, [u8]> {
        Cow::Owned(hex::decode(self))
    }
}
impl BinaryInput for String {
    fn binary(&self) -> Cow<'_, [u8]> {
        self.as_str().binary()
    }
}
impl BinaryInput for Value {
    fn binary(&self) -> Cow<'_, [u8]> {
        self.as_str().unwrap().binary()
    }
}
impl BinaryInput for [u8] {
    fn binary(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self)
    }
}
impl BinaryInput for Vec<u8> {
    fn binary(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self)
    }
}
pub fn bytes<T: BinaryInput + ?Sized>(value: &T) -> Vec<u8> {
    value.binary().into_owned()
}
pub fn words<T: BinaryInput + ?Sized>(value: &T) -> Vec<i16> {
    value
        .binary()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_be_bytes(*pair))
        .collect()
}
pub fn positions<T: BinaryInput + ?Sized>(value: &T) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| (pair[0], pair[1]))
        .collect()
}
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
pub fn fixture(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root().join("evidence/fixtures").join(name)).unwrap())
        .unwrap()
}
pub fn content() -> Content {
    static CONTENT: std::sync::OnceLock<Content> = std::sync::OnceLock::new();
    CONTENT
        .get_or_init(|| {
            let root = root();
            Content::load(
                &root.join("NewCenturions"),
                &root.join("analysis/extracted"),
            )
            .unwrap()
        })
        .clone()
}

pub fn mission_session(content: &Content) -> Session {
    let mut state = Session::new(content);
    apply(Command::ImportStarters, &mut state, content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        content,
    )
    .unwrap();
    state
}

pub fn script_state(record: Vec<u8>, cursor: u8) -> centurions_core::ScriptState {
    centurions_core::ScriptState {
        record,
        cursor,
        map_slot: 0,
        item_return: false,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    }
}

pub fn data(name: &str) -> Vec<u8> {
    std::fs::read(
        root()
            .join("NewCenturions/TheNewCenturions/Data Files")
            .join(name),
    )
    .unwrap()
}
pub fn items() -> Vec<Vec<u8>> {
    data("Data ID")
        .as_chunks::<150>()
        .0
        .iter()
        .map(|record| record.to_vec())
        .collect()
}
pub fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context}: length");
    let differing = actual.iter().zip(expected).position(|(a, b)| a != b);
    assert_eq!(
        differing.map(|i| (i, actual[i], expected[i])),
        None,
        "{context}: first differing byte"
    );
}

pub trait NativeScalar: PartialEq + std::fmt::Debug {
    fn from_native(value: &Value) -> Self;
}
macro_rules! scalar {
    ($method:ident; $($ty:ty),+) => {
        $(impl NativeScalar for $ty {
            fn from_native(value: &Value) -> Self {
                value.$method().unwrap() as Self
            }
        })+
    };
}
scalar!(as_i64; i8, i16, i32, i64);
scalar!(as_u64; u8, u16, u32, u64, usize);
scalar!(as_f64; f32, f64);
impl NativeScalar for bool {
    fn from_native(value: &Value) -> Self {
        value.as_bool().unwrap()
    }
}
impl<T: NativeScalar> NativeScalar for Vec<T> {
    fn from_native(value: &Value) -> Self {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(T::from_native)
            .collect()
    }
}
impl<T: NativeScalar, const N: usize> NativeScalar for [T; N] {
    fn from_native(value: &Value) -> Self {
        Vec::<T>::from_native(value).try_into().unwrap()
    }
}
#[track_caller]
pub fn expect<T: NativeScalar>(actual: T, expected: &Value) {
    assert_eq!(actual, T::from_native(expected));
}
#[track_caller]
pub fn expect_array<T: NativeScalar>(actual: &T, expected: &Value) {
    assert_eq!(actual, &T::from_native(expected));
}
pub trait NativeRecord {
    fn record_bytes(&self) -> &[u8];
}
impl NativeRecord for centurions_content::Warrior {
    fn record_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
impl NativeRecord for centurions_content::combat::MonsterRecord {
    fn record_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
impl NativeRecord for centurions_content::InventoryRecord {
    fn record_bytes(&self) -> &[u8] {
        &self.0
    }
}
pub trait NativeCase {
    fn n<T: NativeScalar>(&self, key: &str) -> T;
    fn text(&self, key: &str) -> &str;
    fn array(&self, key: &str) -> &[Value];
    fn bytes(&self, key: &str) -> Vec<u8>;
    fn words(&self, key: &str) -> Vec<i16>;
}
impl NativeCase for Value {
    fn n<T: NativeScalar>(&self, key: &str) -> T {
        T::from_native(&self[key])
    }
    fn text(&self, key: &str) -> &str {
        self[key].as_str().unwrap()
    }
    fn array(&self, key: &str) -> &[Value] {
        self[key].as_array().unwrap()
    }
    fn bytes(&self, key: &str) -> Vec<u8> {
        bytes(&self[key])
    }
    fn words(&self, key: &str) -> Vec<i16> {
        words(&self[key])
    }
}

pub fn battle() -> centurions_core::combat::Combat {
    use centurions_core::combat::{Combat, EnemySetup, SquadPlacement, TacticalMap};
    Combat {
        difficulty: 0,
        battle: 0,
        map: TacticalMap {
            tiles: vec![0; 8100],
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: vec![(-1, -1); 6],
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: vec![],
            positions: vec![],
        },
        order: vec![-1; 110],
        cursor: 1,
        round: 0,
        inactive_rounds: 0,
        selection_pending: false,
        script_enemy: 0,
        selected_enemy: 0,
        round_scripts: Default::default(),
        death_queue: Default::default(),
        spell_fields: vec![0; 1200],
        field_used: vec![0; 128],
        attack_packet: Some(vec![0; 58]),
        attack_identity: Some([0; 3]),
        weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
        unavailable: 0,
        pending_last_defeated: None,
        retained_npcs: Default::default(),
        direct_deaths: false,
        direct_death: None,
        firearm_selection: None,
        capture_actor: None,
        automatic: [false; 6],
        turn_origin: None,
    }
}

pub fn warriors<T: BinaryInput + ?Sized>(value: &T) -> Vec<centurions_content::Warrior> {
    let raw = bytes(value);
    let (records, tail) = raw.as_chunks::<3138>();
    assert!(tail.is_empty());
    records
        .iter()
        .map(|record| centurions_content::Warrior::read(record).unwrap())
        .collect()
}
pub fn monsters<T: BinaryInput + ?Sized>(
    value: &T,
) -> Vec<centurions_content::combat::MonsterRecord> {
    let raw = bytes(value);
    let (records, tail) = raw.as_chunks::<288>();
    assert!(tail.is_empty());
    records
        .iter()
        .map(|record| centurions_content::combat::MonsterRecord {
            bytes: record.to_vec(),
        })
        .collect()
}

pub fn longs<T: BinaryInput + ?Sized>(value: &T) -> Vec<i32> {
    let raw = bytes(value);
    let (records, tail) = raw.as_chunks::<4>();
    assert!(tail.is_empty());
    records
        .iter()
        .map(|record| i32::from_be_bytes(*record))
        .collect()
}

pub fn inventory<T: BinaryInput + ?Sized>(value: &T) -> Vec<centurions_content::InventoryRecord> {
    let raw = bytes(value);
    let (records, tail) = raw.as_chunks::<22>();
    assert!(tail.is_empty());
    records
        .iter()
        .map(|record| centurions_content::InventoryRecord(*record))
        .collect()
}

macro_rules! native_expect {
    ($($kind:ident $actual:expr => $expected:expr, $context:expr;)*) => {
        $(crate::support::native_expect!(@$kind $actual, $expected, $context);)*
    };
    (@rng $actual:expr, $expected:expr, $context:expr) => {
        {
            let (seed, draws) = $expected;
            crate::support::native_expect!(@scalar $actual.seed, seed, $context);
            crate::support::native_expect!(@scalar $actual.draws, draws, $context);
        }
    };
    (@scalar $actual:expr, $expected:expr, $context:expr) => {
        { let actual = &$actual; assert_eq!(actual, &crate::support::native_value(actual, $expected), "{}", $context); }
    };
    (@records $actual:expr, $expected:expr, $context:expr) => {
        crate::support::native_expect!(@bytes crate::support::record_bytes($actual), $expected, $context)
    };
    (@bytes $actual:expr, $expected:expr, $context:expr) => {
        crate::support::assert_bytes(&$actual[..], &crate::support::bytes($expected), &$context)
    };
    (@positions $actual:expr, $expected:expr, $context:expr) => {
        assert_eq!(&$actual[..], &crate::support::positions($expected)[..], "{}", $context)
    };
    (@words $actual:expr, $expected:expr, $context:expr) => {
        assert_eq!(&$actual[..], &crate::support::words($expected)[..], "{}", $context)
    };
}
pub(crate) use native_expect;
pub fn native_value<T: NativeScalar>(_: &T, value: &Value) -> T {
    T::from_native(value)
}

pub fn stories() -> centurions_content::mission::StoryTables {
    centurions_content::mission::StoryTables::default()
}
pub fn terrain_mission(fixture: &Value) -> centurions_content::mission::OpeningMission {
    centurions_content::mission::OpeningMission {
        tile_properties: fixture.bytes("properties")[64..12064].to_vec(),
        ..mission()
    }
}
pub fn mission() -> centurions_content::mission::OpeningMission {
    centurions_content::mission::OpeningMission {
        name: "fixture".into(),
        start: (2, 2),
        header: vec![],
        regions: vec![],
        floor: vec![],
        objects: vec![],
        events: vec![],
        tile_properties: vec![],
        terrain: 2,
        default_tile: 155,
        minutes_per_tick: 5,
        shared: std::sync::Arc::new(stories()),
    }
}

pub fn record_bytes<T: NativeRecord>(records: &[T]) -> Vec<u8> {
    records
        .iter()
        .flat_map(|record| record.record_bytes().iter().copied())
        .collect()
}
pub fn event_sounds(events: &[centurions_core::Event]) -> Vec<i16> {
    events
        .iter()
        .filter_map(|event| match event {
            centurions_core::Event::Sound { resource_id } => Some(*resource_id),
            _ => None,
        })
        .collect()
}
pub fn word_bytes(values: &[i16]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

pub fn long_bytes(values: &[i32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

#[track_caller]
pub fn same_session(actual: &centurions_core::Session, expected: &centurions_core::Session) {
    if actual == expected {
        return;
    }
    let actual = serde_json::to_value(actual).unwrap();
    let expected = serde_json::to_value(expected).unwrap();
    if let Some(difference) = crate::native_common::first_difference(&actual, &expected, "session")
    {
        panic!("{difference}");
    }
    panic!("session differs outside serialized fields");
}

pub fn buffer(fixture: &Value, value: &Value) -> Vec<u8> {
    let encoded = value.as_str().unwrap_or_else(|| {
        fixture["shared_rule_buffers"][value.text("shared_buffer_sha256")]
            .as_str()
            .unwrap()
    });
    let decoded = bytes(encoded);
    if value.is_object() {
        expect(decoded.len(), &value["byte_length"]);
        assert_eq!(
            centurions_content::digest(&decoded),
            value.text("shared_buffer_sha256")
        );
    }
    decoded
}

pub fn checkpoint(
    state: &centurions_core::Session,
    content: &centurions_content::Content,
    label: &str,
) -> centurions_core::Session {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    // Parallel tests may share a label; each checkpoint needs its own file.
    let path = std::env::temp_dir().join(format!(
        "centurions-reference-{}-{label}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    centurions_core::save(&path, state, content).unwrap();
    let restored = centurions_core::load(&path, content).unwrap();
    same_session(&restored, state);
    std::fs::remove_file(path).unwrap();
    restored
}

#[track_caller]
pub fn replay_command(
    command: Command,
    state: &mut Session,
    resumed: &mut Session,
    content: &Content,
) -> Vec<centurions_core::Event> {
    let events = apply(command.clone(), state, content).unwrap();
    assert_eq!(apply(command, resumed, content).unwrap(), events);
    same_session(state, resumed);
    events
}

pub fn replay_rng(random: &mut centurions_core::RandomState, receipts: &[Value]) {
    for receipt in receipts {
        expect(random.bound(receipt.n::<i16>("bound")), &receipt["result"]);
    }
}

pub fn advance_rng<'a>(
    random: &mut centurions_core::RandomState,
    receipts: &mut impl Iterator<Item = &'a Value>,
    draws: u64,
) {
    while random.draws < draws {
        let receipt = receipts.next().unwrap();
        expect(random.bound(receipt.n::<i16>("bound")), &receipt["result"]);
    }
    assert_eq!(random.draws, draws);
}

pub fn reject_command(
    state: &centurions_core::Session,
    content: &centurions_content::Content,
    command: centurions_core::Command,
) {
    let mut candidate = state.clone();
    assert!(centurions_core::apply(command, &mut candidate, content).is_err());
    same_session(&candidate, state);
}

pub fn refuse_command(
    state: &Session,
    content: &Content,
    command: Command,
    feedback: Vec<centurions_core::feedback::NativeFeedback>,
) {
    let mut candidate = state.clone();
    let events = match apply(command, &mut candidate, content) {
        Err(centurions_core::RuleError::Refused { feedback }) => {
            same_session(&candidate, state);
            feedback.into_iter().map(Event::Feedback).collect()
        }
        result => result.unwrap(),
    };
    assert_eq!(
        events,
        feedback
            .into_iter()
            .map(centurions_core::Event::Feedback)
            .collect::<Vec<_>>()
    );
    assert_eq!(candidate.warriors, state.warriors);
    assert_eq!(candidate.random, state.random);
    let (after, before) = (
        candidate.combat.as_ref().unwrap(),
        state.combat.as_ref().unwrap(),
    );
    assert_eq!(after.map, before.map);
    assert_eq!(after.squad, before.squad);
    assert_eq!(after.enemies, before.enemies);
    assert_eq!(after.cursor, before.cursor);
    assert_eq!(after.order, before.order);
}

pub fn compare_party(actual: &[centurions_content::Warrior], snapshot: &Value) {
    compare_party_named(actual, snapshot, "party");
}

pub fn compare_party_named(
    actual: &[centurions_content::Warrior],
    snapshot: &Value,
    context: &str,
) {
    assert_bytes(&record_bytes(actual), &bytes(&snapshot["party"]), context);
}

pub fn compare_record_bytes<T: NativeRecord>(actual: &[T], expected: &[T]) {
    assert_eq!(record_bytes(actual), record_bytes(expected));
}

#[track_caller]
pub fn assert_rng(actual: &centurions_core::RandomState, seed: &Value, draws: &Value) {
    assert_eq!(
        actual.seed,
        seed.as_u64().unwrap() as u32,
        "native RNG seed"
    );
    assert_eq!(actual.draws, draws.as_u64().unwrap(), "native RNG draws");
}

/// Native attack receipts may append caller metadata after the two body IDs.
pub fn attack_calls(value: &Value) -> Vec<(i16, i16)> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|call| {
            (
                call[0].as_i64().unwrap() as i16,
                call[1].as_i64().unwrap() as i16,
            )
        })
        .collect()
}

pub fn same_captured_session(actual: &Session, expected: &Session) {
    same_session(&captured_session(actual), &captured_session(expected));
}

/// Apply the sparse byte changes recorded by original-instruction receipts.
pub fn patch_bytes(raw: &mut [u8], changes: &Value) {
    for change in changes.as_array().unwrap() {
        raw[change[0].as_u64().unwrap() as usize] = change[1].as_u64().unwrap() as u8;
    }
}

/// Remove one saved field when constructing a historical-version specimen.
pub fn remove_field(value: &mut Value, name: &str) {
    value.as_object_mut().unwrap().remove(name);
}

pub fn sound_events(sounds: impl IntoIterator<Item = i16>) -> Vec<Event> {
    sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect()
}
