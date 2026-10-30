use crate::fixture_hex as hex;
use centurions_content::combat::{MONSTER_BYTES, MonsterRecord};
use centurions_content::{Content, InventoryRecord, WARRIOR_BYTES, Warrior};
use centurions_core::{Command, Event, Session, apply};
use serde_json::Value;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
mod native_context;
pub mod psi_frame;
pub use native_context::{
    captured_session, source_frame, source_victory_context, source_victory_context_recorded,
};
mod command_replay;
pub use command_replay::{assert_no_turn_started, current_combat, saved_command};

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
    records::<_, 2>(value, |pair| i16::from_be_bytes(*pair))
}
pub fn positions<T: BinaryInput + ?Sized>(value: &T) -> Vec<(i16, i16)> {
    records::<_, 4>(value, |pair| {
        (
            i16::from_be_bytes([pair[0], pair[1]]),
            i16::from_be_bytes([pair[2], pair[3]]),
        )
    })
}
/// The checkout root, from the maintained crates and the archive's reference tests alike.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|path| path.join("crates/core/Cargo.toml").is_file())
        .unwrap()
        .to_path_buf()
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
    for command in [
        Command::ImportStarters,
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
    ] {
        apply(command, &mut state, content).unwrap();
    }
    state
}

pub fn data(name: &str) -> Vec<u8> {
    let files = root().join("NewCenturions/TheNewCenturions/Data Files");
    std::fs::read(files.join(name)).unwrap()
}
pub fn items() -> Vec<Vec<u8>> {
    data("Data ID")
        .as_chunks::<150>()
        .0
        .iter()
        .map(|record| record.to_vec())
        .collect()
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
scalar!(as_bool; bool);
impl<T: NativeScalar> NativeScalar for Vec<T> {
    fn from_native(value: &Value) -> Self {
        let values = value.as_array().unwrap();
        values.iter().map(T::from_native).collect()
    }
}
impl<T: NativeScalar, const N: usize> NativeScalar for [T; N] {
    fn from_native(value: &Value) -> Self {
        Vec::<T>::from_native(value).try_into().unwrap()
    }
}

pub trait NativeRecord {
    fn record_bytes(&self) -> &[u8];
}
macro_rules! record_bytes {
    ($($ty:ty => $field:tt),+) => {
        $(impl NativeRecord for $ty {
            fn record_bytes(&self) -> &[u8] { &self.$field }
        })+
    };
}
record_bytes!(Warrior => bytes, MonsterRecord => bytes, InventoryRecord => 0);
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
    use centurions_core::combat::{Combat, CombatField, EnemySetup, SquadPlacement};
    Combat::from_records(
        CombatField {
            tiles: vec![0; 8100],
            map_shift: (0, 0),
            edge_word: 0,
        },
        SquadPlacement {
            pos: vec![(-1, -1); 6],
            char_under: vec![1155; 6],
        },
        EnemySetup {
            records: vec![],
            mon_pos: vec![],
        },
    )
}

fn records<T, const N: usize>(
    value: &(impl BinaryInput + ?Sized),
    read: impl Fn(&[u8; N]) -> T,
) -> Vec<T> {
    let raw = value.binary();
    assert_eq!(raw.len() % N, 0);
    raw.as_chunks::<N>().0.iter().map(read).collect()
}

pub fn warriors<T: BinaryInput + ?Sized>(value: &T) -> Vec<Warrior> {
    records::<_, WARRIOR_BYTES>(value, |record| Warrior::read(record).unwrap())
}
/// The warrior record at `index` in a captured party buffer.
pub fn warrior_at(party: &[u8], index: usize) -> Warrior {
    Warrior::read(&party[index * WARRIOR_BYTES..(index + 1) * WARRIOR_BYTES]).unwrap()
}
/// Writes one warrior back into its slot of a captured party buffer.
pub fn put_warrior(party: &mut [u8], index: usize, warrior: &Warrior) {
    party[index * WARRIOR_BYTES..(index + 1) * WARRIOR_BYTES].copy_from_slice(&warrior.bytes);
}
pub fn monsters<T: BinaryInput + ?Sized>(value: &T) -> Vec<MonsterRecord> {
    records::<_, MONSTER_BYTES>(value, |record| MonsterRecord {
        bytes: record.to_vec(),
    })
}

pub fn longs<T: BinaryInput + ?Sized>(value: &T) -> Vec<i32> {
    records::<_, 4>(value, |record| i32::from_be_bytes(*record))
}

pub fn inventory<T: BinaryInput + ?Sized>(value: &T) -> Vec<InventoryRecord> {
    records::<_, 22>(value, |record| InventoryRecord(*record))
}

pub fn terrain_mission(fixture: &Value) -> centurions_content::mission::OpeningMission {
    centurions_content::mission::OpeningMission {
        map_stats: fixture.bytes("properties")[64..12064].to_vec(),
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
        map_stats: vec![],
        terrain: 2,
        default_tile: 155,
        minutes_per_tick: 5,
        shared: std::sync::Arc::new(Default::default()),
    }
}

pub fn record_bytes<T: NativeRecord>(records: &[T]) -> Vec<u8> {
    records
        .iter()
        .flat_map(|record| record.record_bytes().iter().copied())
        .collect()
}
pub fn event_sounds(events: &[Event]) -> Vec<i16> {
    events.iter().filter_map(Event::sound).collect()
}
pub fn word_bytes(values: &[i16]) -> Vec<u8> {
    values.iter().copied().flat_map(i16::to_be_bytes).collect()
}

pub fn long_bytes(values: &[i32]) -> Vec<u8> {
    values.iter().copied().flat_map(i32::to_be_bytes).collect()
}

#[track_caller]
pub fn same_session(actual: &Session, expected: &Session) {
    if actual != expected {
        let actual = serde_json::to_value(actual).unwrap();
        let expected = serde_json::to_value(expected).unwrap();
        let difference = crate::native_common::first_difference(&actual, &expected, "session")
            .unwrap_or_else(|| "session differs outside serialized fields".into());
        panic!("{difference}");
    }
}

pub fn checkpoint(state: &Session, content: &Content, label: &str) -> Session {
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
) -> Vec<Event> {
    let events = apply(command.clone(), state, content).unwrap();
    assert_eq!(apply(command, resumed, content).unwrap(), events);
    same_session(state, resumed);
    events
}

pub fn reject_command(state: &Session, content: &Content, command: Command) {
    let mut candidate = state.clone();
    assert!(apply(command, &mut candidate, content).is_err());
    same_session(&candidate, state);
}

pub fn refuse_command(
    state: &Session,
    content: &Content,
    command: Command,
    feedback: Vec<centurions_core::feedback::GameFeedback>,
) {
    let mut candidate = state.clone();
    let events = match apply(command, &mut candidate, content) {
        Err(centurions_core::RuleError::Refused { feedback }) => {
            feedback.into_iter().map(Event::Feedback).collect()
        }
        result => result.unwrap(),
    };
    let expected: Vec<_> = feedback.into_iter().map(Event::Feedback).collect();
    assert_eq!(events, expected);
    same_session(&candidate, state);
}

pub fn compare_record_bytes<T: NativeRecord>(actual: &[T], expected: &[T]) {
    assert_eq!(record_bytes(actual), record_bytes(expected));
}

pub fn same_captured_session(actual: &Session, expected: &Session) {
    same_session(&captured_session(actual), &captured_session(expected));
}

pub fn sound_events(sounds: impl IntoIterator<Item = i16>) -> Vec<Event> {
    sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect()
}

#[path = "../native/comparison.rs"]
mod native_comparison;
pub(crate) use native_comparison::native_expect;
pub use native_comparison::{
    advance_rng, assert_bytes, assert_rng, attack_calls, buffer, compare_party,
    compare_party_named, expect, expect_array, native_value, patch_bytes, replay_rng,
};

/// Pause before a scheduled record, so its first public Continue owns execution.
pub fn pause_script(state: &mut Session, record: Vec<u8>, cursor: u8) {
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(centurions_core::ScriptState::scheduled(record, cursor));
    position.presentation = Some(centurions_core::Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
}

pub fn script_record(actions: &[(i16, i16)]) -> Vec<u8> {
    let mut record = vec![0; 42];
    for (slot, (op, parameter)) in actions.iter().enumerate() {
        record[8 + slot * 2..10 + slot * 2].copy_from_slice(&op.to_be_bytes());
        record[24 + slot * 2..26 + slot * 2].copy_from_slice(&parameter.to_be_bytes());
    }
    record
}

pub fn script(state: &Session) -> &centurions_core::ScriptState {
    state.exploration.as_ref().unwrap().script.as_ref().unwrap()
}
