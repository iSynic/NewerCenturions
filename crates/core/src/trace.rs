//! Opt-in diagnostic trace of rule execution.
//!
//! A command enables recording through the non-serialized
//! `CommandPreferences::trace`. Recording only observes values the rules have
//! already produced: it never changes session state, events, saves or RNG
//! consumption, and disabled recording returns before building an entry.
use serde::Serialize;
use std::cell::RefCell;
use std::panic::Location;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawCall {
    Quickdraw,
    Bound(i16),
    Range(i16, i16),
}

#[derive(Default)]
struct Recorder {
    enabled: bool,
    entries: Vec<TraceEntry>,
}

thread_local! {
    static RECORDER: RefCell<Recorder> = RefCell::new(Recorder::default());
}

/// Enables recording on this thread until dropped.
pub(crate) struct Scope {
    previous: bool,
}

impl Scope {
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
    fn drop(&mut self) {
        RECORDER.with_borrow_mut(|recorder| recorder.enabled = self.previous);
    }
}

/// Removes and returns the entries recorded on this thread.
pub fn take() -> Vec<TraceEntry> {
    RECORDER.with_borrow_mut(|recorder| std::mem::take(&mut recorder.entries))
}

fn record(entry: impl FnOnce() -> TraceEntry) {
    RECORDER.with_borrow_mut(|recorder| {
        if recorder.enabled {
            recorder.entries.push(entry());
        }
    });
}

#[track_caller]
pub(crate) fn draw(call: DrawCall, result: i16) {
    let site = Location::caller();
    record(|| TraceEntry::Draw {
        site: format!("{}:{}", site.file().replace('\\', "/"), site.line()),
        call,
        result,
    });
}

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

pub(crate) fn damage(rolled: &[i16; 9], components: &[i16; 9], total: i16) {
    record(|| TraceEntry::Damage {
        rolled: *rolled,
        components: *components,
        total,
    });
}

pub(crate) fn declined(site: &'static str, owner: &'static str, reason: &'static str) {
    record(|| TraceEntry::Declined {
        site,
        owner,
        reason,
    });
}
