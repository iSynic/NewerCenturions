//! Opt-in diagnostic trace of rule execution.
//!
//! A command enables recording through the non-serialized
//! `CommandPreferences::trace`. Recording only observes values the rules have
//! already produced: it never changes session state, events, saves or RNG
//! consumption, and disabled recording returns before building an entry.
use serde::Serialize;
use std::cell::RefCell;
use std::panic::Location;

/// One thing a trace noted: a random draw, a script instruction, the damage channels of an attack,
/// or an original return that produced no events.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TraceEntry {
    /// One quickdraw consumed from a `RandomState`, labelled by its call site.
    Draw {
        site: String,
        call: DrawCall,
        result: i16,
    },
    /// One instruction decoded by a story interpreter. `script` is the map
    /// event slot for exploration and the mission script ID otherwise.
    Story {
        interpreter: &'static str,
        script: i32,
        cursor: usize,
        opcode: i16,
        parameter: i16,
    },
    /// Damage channels after rolling and after every enabled defense.
    Damage {
        rolled: [i16; 9],
        components: [i16; 9],
        total: i16,
    },
    /// An original-behavior return that produced no events.
    Declined {
        site: &'static str,
        owner: &'static str,
        reason: &'static str,
    },
}

/// Which kind of random draw was made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawCall {
    Quickdraw,
    Bound(i16),
    Range(i16, i16),
}

/// The entries recorded so far on this thread, and whether recording is on.
#[derive(Default)]
struct Recorder {
    enabled: bool,
    entries: Vec<TraceEntry>,
}

thread_local! {
    /// The recorder of this thread.
    static RECORDER: RefCell<Recorder> = RefCell::new(Recorder::default());
}

/// Enables recording on this thread until dropped.
pub(crate) struct Scope {
    previous: bool,
}

impl Scope {
    /// Turns recording on for the scope when `enabled`; otherwise recording stays as it was.
    pub(crate) fn enable(enabled: bool) -> Self {
        let previous = RECORDER.with_borrow_mut(|recorder| {
            let previous = recorder.enabled;
            recorder.enabled |= enabled;
            previous
        });
        Self { previous }
    }
}

impl Drop for Scope {
    /// Puts recording back as it was before the scope.
    fn drop(&mut self) {
        RECORDER.with_borrow_mut(|recorder| recorder.enabled = self.previous);
    }
}

/// Removes and returns the entries recorded on this thread.
pub fn take() -> Vec<TraceEntry> {
    RECORDER.with_borrow_mut(|recorder| std::mem::take(&mut recorder.entries))
}

/// Notes an entry when recording is on.
fn record(entry: impl FnOnce() -> TraceEntry) {
    RECORDER.with_borrow_mut(|recorder| {
        if recorder.enabled {
            recorder.entries.push(entry());
        }
    });
}

/// Notes a random draw with where it was made.
#[track_caller]
pub(crate) fn draw(call: DrawCall, result: i16) {
    let site = Location::caller();
    record(|| TraceEntry::Draw {
        site: format!("{}:{}", site.file().replace('\\', "/"), site.line()),
        call,
        result,
    });
}

/// Notes a script instruction a story interpreter decoded.
pub(crate) fn story(
    interpreter: &'static str,
    script: i32,
    cursor: usize,
    opcode: i16,
    parameter: i16,
) {
    record(|| TraceEntry::Story {
        interpreter,
        script,
        cursor,
        opcode,
        parameter,
    });
}

/// Notes the damage of an attack: the nine rolled amounts, the same channels after every enabled
/// defense, and the total.
pub(crate) fn damage(rolled: &[i16; 9], components: &[i16; 9], total: i16) {
    record(|| TraceEntry::Damage {
        rolled: *rolled,
        components: *components,
        total,
    });
}

/// Notes a place where the original game returns without producing events.
pub(crate) fn declined(site: &'static str, owner: &'static str, reason: &'static str) {
    record(|| TraceEntry::Declined {
        site,
        owner,
        reason,
    });
}
