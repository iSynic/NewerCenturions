//! Tactical playback of one command's NPC steps and result pauses.
//!
//! The core resolves a whole NPC action in one command. The original draws as it
//! goes: movemonster 0x13e83a..0x13e8f0 places the body, draws it (drawbody, or
//! centerstage2 for a -3 definition), plays the tile sound, calls collide and
//! waits delay(WORD439568 / 2). showresults 0x159090..0x15916a then plots its
//! showdamage cicn over the result body and waits delay(30 - 2 * WORD43956a)
//! before centerfield redraws the field. A warrior's step (combatchoice
//! 0x11c9be..0x11cb66) writes the map, then centerfield 0x11cb30 recentres on the
//! warrior and the tile sound plays, with no delay. cast 0x10d4fe draws each
//! target's flight after its launch sound. resolvespell then animates the spell
//! area at once (0x1517c4..0x151828) or flashes each presented target before its
//! showresults (0x151a62, 0x151ba4, 0x151d96). This module replays the emitted
//! events over a copy of the field taken before the command; rule state is untouched.
use crate::combat_flight::{FlightFrame, FlightFrames};
use centurions_core::{
    Event, Flight, Session, SpellArea, SpellInfo,
    combat::{Combat, footprint},
    feedback::NativeFeedback,
};
use std::collections::VecDeque;

/// One showdamage call: the cicn plotted over a body and the number it prints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResultIcon {
    pub body: i16,
    pub icon: i16,
    pub amount: i16,
}

/// What window 128's rect 0x4436fa (64 square) shows: showspellinfo's cicn,
/// showresults' target cicn (0x1584b4..0x1585e2) or killbody's PICT 207
/// (0x12eb50..0x12eb60).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultSlot {
    Spell {
        icon: i16,
        area: bool,
    },
    /// The cicn plotted into the rectangle inset 16 on each axis a size class
    /// does not extend.
    Target {
        icon: i16,
        size: i16,
    },
    Cleared,
}

enum Visual {
    None,
    SpellInfo(SpellInfo),
    Move { body: i16, point: (i16, i16) },
    Icon(ResultIcon),
    Effect { bodies: u128, base: i16, ticks: i16 },
    Flight(FlightFrames),
    Framed { body: i16 },
}

/// The preferences and retained packet bytes the original presentation reads.
pub struct Settings {
    /// WORD43956a, the Speed menu item.
    pub speed: i16,
    /// BYTE443760 after the command, which spelleffect reads.
    pub effect: i8,
    /// BYTE44374f, the retained packet's targeting mode.
    pub targeting: u8,
    /// Preference 9 (WORD439574): cast's short walk.
    pub short_walks: bool,
    /// Preference 10 (WORD439588), read by spelleffect 0x15b4d2 and resolvespell.
    pub batched_effects: bool,
    /// WORD45366c, ITEM work 0x45363c + 0x30 after the command, which showresults
    /// prints with cicns 165 and 166 (0x1590c0, 0x1590e2).
    pub item_value: i16,
}

/// One ray crossing: its first and last event, the walk step that crossed it and
/// the walk (a flight) it belongs to.
struct Crossing {
    start: usize,
    end: usize,
    step: i16,
    walk: usize,
}

/// Each ray walk's flight and its crossings. A result carrying a flight and a
/// crossing opens a walk, results with a crossing continue it, anything else
/// ends it. Results of one crossing share its events.
fn ray_walks(events: &[Event]) -> (Vec<Flight>, Vec<Crossing>) {
    let mut walks = vec![];
    let mut crossings: Vec<Crossing> = vec![];
    let mut open = None;
    for (index, event) in events.iter().enumerate() {
        let flight = match event {
            Event::AttackResolved { flight, .. } | Event::SpellResolved { flight, .. } => flight,
            Event::Launch { .. } => {
                open = None;
                continue;
            }
            _ => continue,
        };
        let Some(crossing) = flight.2 else {
            open = None;
            continue;
        };
        if let Some(first) = flight.0 {
            open = Some(walks.len());
            walks.push(first);
        }
        let Some(walk) = open else { continue };
        let start = index.saturating_sub(usize::from(crossing.lead));
        let end = index + usize::from(crossing.tail);
        match crossings.last_mut() {
            Some(last) if last.walk == walk && last.step == crossing.step => {
                last.start = last.start.min(start);
                last.end = last.end.max(end);
            }
            _ => crossings.push(Crossing {
                start,
                end,
                step: crossing.step,
                walk,
            }),
        }
    }
    (walks, crossings)
}

/// spelleffect's cicns base..base + 7 plotted in turn over each body's restored
/// rectangle: one body held delay(WORD439568 / 30 + 1) a frame (0x15b43c..0x15b5dc),
/// or every body of the area mask held delay(4) a frame (0x15b5e0..0x15b832).
#[derive(Clone, Copy, Debug)]
pub struct EffectFrames {
    /// Bit n set for body n.
    pub bodies: u128,
    pub base: i16,
    pub started: f64,
    pub frame_seconds: f64,
}

impl EffectFrames {
    pub fn icon(&self, now: f64) -> Option<i16> {
        let frame = ((now - self.started) / self.frame_seconds).floor();
        (0. ..8.).contains(&frame).then(|| self.base + frame as i16)
    }

    pub fn bodies(&self) -> impl Iterator<Item = i16> + '_ {
        (0..110).filter(|body| self.bodies >> body & 1 != 0)
    }
}

/// A result of resolvespell's body loop: every spell result, and attack results
/// the core marks with the loop's area (NPC and ally fire through 0x151022).
fn resolvespell_result(event: &Event) -> bool {
    match event {
        Event::SpellResolved { .. } => true,
        Event::AttackResolved { flight, .. } => flight.1.is_some(),
        _ => false,
    }
}

/// spelleffect's cicn base 0x15b314..0x15b32c: 0x2ef8 without an effect, else
/// effect * 8 + 0x2ed8.
fn effect_base(effect: i8) -> i16 {
    if effect == 0 {
        0x2ef8
    } else {
        i16::from(effect) * 8 + 0x2ed8
    }
}

/// spelleffect(body, 0)'s per-body path 0x15b44c: modes 9 and 10, or mode 3 under
/// preference 10, flash drawbody(body, 1) held delay(10) while WORD439588 is clear
/// (0x15b4ba..0x15b522); other modes plot the eight cicns over the body.
fn body_effect(body: i16, mode: i8, effect: i8, settings: &Settings) -> Step {
    if matches!(mode, 9 | 10) || settings.batched_effects && mode == 3 {
        return Step {
            events: vec![],
            visual: Visual::Framed { body },
            hit: None,
            ticks: if settings.batched_effects { 0 } else { 10 },
        };
    }
    let frame_ticks = (settings.speed * 5 - 5) / 30 + 1;
    Step {
        events: vec![],
        visual: Visual::Effect {
            bodies: u32::try_from(body)
                .ok()
                .and_then(|body| 1u128.checked_shl(body))
                .unwrap_or(0),
            base: effect_base(effect),
            ticks: frame_ticks,
        },
        hit: None,
        ticks: frame_ticks * 8,
    }
}

/// The health change a result step presents on the copy.
struct Hit {
    body: i16,
    damage: i16,
    defeated: bool,
}

struct Step {
    /// Events the original produces while this step draws: sounds and messages.
    events: Vec<Event>,
    visual: Visual,
    hit: Option<Hit>,
    /// The original delay after the step, in ticks.
    ticks: i16,
}

pub struct Playback {
    /// The session as presented: field, panels and health lag the resolved session
    /// until each step plays, and the resolved session is shown at the end.
    pub session: Session,
    pub icon: Option<ResultIcon>,
    pub effect: Option<EffectFrames>,
    pub flight: Option<FlightFrames>,
    /// ploticon3 0x10d596: the cast's cicn in window 128 rect 0x4436f2, which
    /// stays until the controls are redrawn after the command. showspellinfo
    /// replaces it for a category-9 packet.
    pub cast_icon: Option<i16>,
    /// Window 128 rect 0x4436fa, kept like `cast_icon`.
    pub result_slot: Option<ResultSlot>,
    /// showspellinfo's name and number, kept like `cast_icon`.
    pub spell_info: Option<SpellInfo>,
    /// spelleffect's flash: drawbody(body, 1) frames the body until drawbody(body, 0).
    pub framed: Option<i16>,
    /// The view origin while playing. movemonster recentres only when a body
    /// leaves view cells 1..8 (0x13e8f2..0x13ea62); showresults' centerfield at
    /// 0x159158 passes the current centre WORD408570/408572, so it only redraws.
    pub view: Option<(i16, i16)>,
    steps: VecDeque<Step>,
    until: f64,
}

/// showresults' d4 for its fixed codes (0x158962..0x158f8e); codes without one
/// keep -1 and draw no cicn unless code 0 resolved damage.
pub fn result_icon(code: i16) -> Option<i16> {
    Some(match code {
        -34 | -32 | -28 | -22 | -16 => 159,
        -19 => 160,
        -20 => 161,
        -21 => 162,
        -23 => 163,
        -33 | -30 | -29 | -27 | -26 | -15 => 164,
        -40 => 165,
        -5 => 166,
        _ => return None,
    })
}

/// The showresults line for a code the core resolves without feedback: code 0
/// (a hit) and -1 (no damage). Presentation only; it never reaches the core,
/// whose rules read real CombatResult feedback.
fn hit_line(body: i16, code: i16, amount: i16) -> Event {
    Event::Feedback(NativeFeedback::CombatResult {
        body,
        code,
        amount,
        audible: false,
    })
}

/// showresults 0x159138: delay(30 - 2 * WORD43956a), the Speed menu item.
fn result_ticks(speed: i16) -> i16 {
    30 - 2 * speed
}

impl Playback {
    /// Returns the playback and the events that are presented at once, or None
    /// when the command produced no tactical step to show.
    pub fn build(
        previous: Session,
        events: &[Event],
        settings: &Settings,
    ) -> Option<(Self, Vec<Event>)> {
        let Settings { speed, effect, .. } = *settings;
        // movemonster 0x13e8d6..0x13e8ea: (d + (d >> 15)) >> 1 of the animation delay.
        let animation_delay = speed * 5 - 5;
        let move_ticks = animation_delay / 2;
        let mut steps: VecDeque<Step> = VecDeque::new();
        let mut immediate = vec![];
        let mut pending = vec![];
        // attack2 plays a hit's impact sound before its showresults call; the core
        // emits it directly after AttackResolved.
        let mut impact = false;
        let mut killbody_delay = false;
        // Whether the attack being read already presented a showresults code.
        let mut presented = false;
        // combatchoice 0x11cb58 plays the tile sound after the warrior's step.
        let mut stepped = false;
        // resolvespell's area for each body loop, placed where the loop's events
        // begin: (first event of the loop, result carrying it, area).
        let areas: Vec<(usize, usize, SpellArea)> = events
            .iter()
            .enumerate()
            .filter_map(|(index, event)| match event {
                Event::AttackResolved { flight, .. } | Event::SpellResolved { flight, .. } => {
                    flight
                        .1
                        .filter(|area| area.opens)
                        .map(|area| (index.saturating_sub(usize::from(area.lead)), index, area))
                }
                _ => None,
            })
            .collect();
        let mut drawn_flights = vec![];
        // The packet of the body loop being read: whether resolvespell set
        // BYTE4358c1 (0x151800), its mode and BYTE443760.
        let mut area_path = false;
        let mut mode = settings.targeting as i8;
        let mut effect = effect;
        let frames = |flight: centurions_core::Flight| {
            let walk = if flight.short && settings.short_walks {
                30
            } else {
                128
            } + animation_delay;
            let frames = FlightFrames::new(flight, walk, 0., 1.);
            // A ray blocked at step b (0x10e110) walks no further.
            let end = flight.stop.map_or(frames.window.1, |stop| {
                FlightFrames::ticks_before(stop).min(frames.window.1)
            });
            frames.part(0, end)
        };
        let part = |frames: FlightFrames, pending: &mut Vec<Event>| Step {
            events: std::mem::take(pending),
            visual: Visual::Flight(frames),
            hit: None,
            ticks: frames.length(),
        };
        // Each ray walk's flight and crossings, and while one plays its walk, its
        // frames and the ticks already shown.
        let (walks, crossings) = ray_walks(events);
        let mut ray: Option<(usize, FlightFrames, i16)> = None;
        // showspellinfo before an NPC's cast or resolvespell, at the event it precedes.
        let infos: Vec<(usize, SpellInfo)> = events
            .iter()
            .enumerate()
            .filter_map(|(index, event)| match event {
                Event::AttackResolved { flight, .. }
                | Event::SpellResolved { flight, .. }
                | Event::Launch { flight, .. } => flight
                    .3
                    .map(|info| (index.saturating_sub(usize::from(info.lead)), info)),
                _ => None,
            })
            .collect();
        for (index, event) in events.iter().enumerate() {
            for (_, info) in infos.iter().filter(|(at, _)| *at == index) {
                steps.push_back(Step {
                    events: std::mem::take(&mut pending),
                    visual: Visual::SpellInfo(*info),
                    hit: None,
                    ticks: 0,
                });
            }
            // cast's default walk resolves each crossed body at its step, before
            // that step's sprite and delay (0x10e22a, 0x10e58c): the walk plays up
            // to the step, then the crossing's events.
            for crossing in crossings.iter().filter(|crossing| crossing.start == index) {
                if ray.as_ref().is_none_or(|(walk, ..)| *walk != crossing.walk) {
                    ray = Some((crossing.walk, frames(walks[crossing.walk]), 0));
                }
                if let Some((_, walk, shown)) = ray.as_mut() {
                    let next = FlightFrames::ticks_before(crossing.step).min(walk.window.1);
                    if next > *shown {
                        steps.push_back(part(walk.part(*shown, next), &mut pending));
                        *shown = next;
                    }
                }
            }
            for (_, result, area) in areas.iter().filter(|(at, ..)| *at == index) {
                // cast draws every flight before it calls resolvespell.
                if let Event::AttackResolved { flight, .. } | Event::SpellResolved { flight, .. } =
                    &events[*result]
                    && let Some(flight) = flight.0.filter(|_| flight.2.is_none())
                {
                    steps.push_back(part(frames(flight), &mut pending));
                    drawn_flights.push(*result);
                }
                mode = area.mode;
                effect = area.effect;
                // 0x1517ca..0x1517fe: preference 10 with mode 3, 4 or above 8.
                area_path = settings.batched_effects && matches!(area.mode, 3 | 4 | 9..);
                if area_path {
                    let mut opening = std::mem::take(&mut pending);
                    opening.push(Event::Sound {
                        resource_id: area.sound,
                    });
                    steps.push_back(Step {
                        events: opening,
                        visual: Visual::Effect {
                            bodies: area.bodies,
                            base: effect_base(area.effect),
                            ticks: 4,
                        },
                        hit: None,
                        // 0x15b7ee..0x15b7fa: delay(4) after each of the eight frames.
                        ticks: 32,
                    });
                }
            }
            let after_hit = std::mem::take(&mut impact);
            let after_step = std::mem::take(&mut stepped);
            if let Event::AttackResolved { flight, .. } | Event::SpellResolved { flight, .. } =
                event
                && let (Some(flight), None) = (flight.0, flight.2)
                && !drawn_flights.contains(&index)
            {
                steps.push_back(part(frames(flight), &mut pending));
            }
            if killbody_delay && matches!(event, Event::Delay { ticks: 15 }) {
                killbody_delay = false;
                continue;
            }
            match event {
                Event::Sound { .. } if after_hit || after_step => {
                    if let Some(step) = steps.back_mut() {
                        step.events.push(event.clone());
                    }
                }
                Event::Sound { .. } | Event::Delay { .. } => pending.push(event.clone()),
                // A flight without a presented result plays after its launch sound.
                Event::Launch { flight, .. } => {
                    pending.push(event.clone());
                    if let Some(flight) = flight.0 {
                        steps.push_back(part(frames(flight), &mut pending));
                    }
                }
                Event::Feedback(feedback) => {
                    if let NativeFeedback::CombatResult {
                        body, amount: 0, ..
                    } = feedback
                        && !area_path
                        && matches!(
                            events.get(index + 1),
                            Some(Event::SpellResolved { target, damage: 0, .. }) if target == body
                        )
                    {
                        // 0x151a3c..0x151a62: a successful zero-damage spelllist
                        // result flashes before its showresults.
                        let mut flash = body_effect(*body, mode, effect, settings);
                        flash.events = std::mem::take(&mut pending);
                        steps.push_back(flash);
                    }
                    pending.push(event.clone());
                    presented |= matches!(feedback, NativeFeedback::CombatResult { .. });
                    if let NativeFeedback::CombatResult {
                        body, code, amount, ..
                    } = feedback
                    {
                        // showresults 0x1590f6: d4 > 0 with a positive WORD442bc8.
                        // 0x1590b4..0x1590ec plot 166 always and 165 after damage;
                        // both print WORD45366c.
                        let visual = result_icon(*code)
                            .filter(|icon| *icon >= 165 || *amount > 0)
                            .map_or(Visual::None, |icon| {
                                Visual::Icon(ResultIcon {
                                    body: *body,
                                    icon,
                                    amount: if icon >= 165 {
                                        settings.item_value
                                    } else {
                                        *amount
                                    },
                                })
                            });
                        steps.push_back(Step {
                            events: std::mem::take(&mut pending),
                            visual,
                            hit: None,
                            ticks: result_ticks(speed),
                        });
                    }
                }
                Event::TacticalMoved { body, x, y } => {
                    stepped = *body < 10;
                    steps.push_back(Step {
                        events: std::mem::take(&mut pending),
                        visual: Visual::Move {
                            body: *body,
                            point: (*x, *y),
                        },
                        hit: None,
                        ticks: if *body < 10 { 0 } else { move_ticks },
                    });
                }
                Event::AttackResolved {
                    target,
                    damage,
                    defeated,
                    icon,
                    ..
                }
                | Event::SpellResolved {
                    target,
                    damage,
                    defeated,
                    icon,
                    ..
                } if *damage > 0 => {
                    immediate.push(event.clone());
                    // showresults 0x159116 (attack 0x101e5e, resolvespell 0x151dba):
                    // code 0 with damage passes icon -1, which showdamage 0x15548c
                    // turns into WORD41f710. A resolved hit always leaves a nonzero
                    // channel cue, so 0 means the caller carries none: no cicn.
                    // attack returns before showresults when the hit kills
                    // (attack 0x101e0e/0x101cc2); killbody waits delay(15) at
                    // 0x12eb32 before bodyground/bodyfield clear the body.
                    let killed = *defeated && *target >= 10;
                    // attack 0x101e5e: showresults code 0 prints the damage line.
                    if matches!(event, Event::AttackResolved { .. })
                        && !killed
                        && !std::mem::take(&mut presented)
                    {
                        pending.push(hit_line(*target, 0, *damage));
                    }
                    if resolvespell_result(event) && !*defeated && !area_path {
                        // 0x151b90..0x151ba4 and 0x151d7c..0x151d96: a surviving
                        // target flashes before its showresults.
                        let mut flash = body_effect(*target, mode, effect, settings);
                        flash.events = std::mem::take(&mut pending);
                        steps.push_back(flash);
                    }
                    let visual = if icon.0 == 0 || killed {
                        Visual::None
                    } else {
                        Visual::Icon(ResultIcon {
                            body: *target,
                            icon: icon.0,
                            amount: *damage,
                        })
                    };
                    steps.push_back(Step {
                        events: std::mem::take(&mut pending),
                        visual,
                        hit: Some(Hit {
                            body: *target,
                            damage: *damage,
                            defeated: false,
                        }),
                        ticks: if killed { 15 } else { result_ticks(speed) },
                    });
                    if *defeated && *target < 10 {
                        // killbody 0x12eb6e..0x12eb7a: a dead warrior's spelleffect(body, 0).
                        steps.push_back(body_effect(*target, mode, effect, settings));
                    }
                    if killed {
                        steps.push_back(Step {
                            events: vec![],
                            visual: Visual::None,
                            hit: Some(Hit {
                                body: *target,
                                damage: 0,
                                defeated: true,
                            }),
                            ticks: 0,
                        });
                        // The core's Delay(15) is killbody's own wait, played above.
                        killbody_delay = true;
                    }
                    impact = matches!(event, Event::AttackResolved { .. }) && !killed;
                }
                Event::AttackResolved {
                    target,
                    defeated: false,
                    ..
                } => {
                    immediate.push(event.clone());
                    // attack 0x101e62..0x101e7e: no damage calls showresults -1.
                    if !std::mem::take(&mut presented) {
                        pending.push(hit_line(*target, -1, 0));
                        steps.push_back(Step {
                            events: std::mem::take(&mut pending),
                            visual: Visual::None,
                            hit: None,
                            ticks: result_ticks(speed),
                        });
                        impact = true;
                    }
                }
                Event::SpellResolved {
                    target,
                    damage,
                    defeated: false,
                    ..
                } if *damage < 0 && !area_path => {
                    // 0x151b90 and 0x151d7c also take a negative WORD4434b8 (a heal).
                    immediate.push(event.clone());
                    let mut flash = body_effect(*target, mode, effect, settings);
                    flash.events = std::mem::take(&mut pending);
                    flash.hit = Some(Hit {
                        body: *target,
                        damage: *damage,
                        defeated: false,
                    });
                    steps.push_back(flash);
                }
                _ => immediate.push(event.clone()),
            }
            if let Some((current, walk, shown)) = ray
                && crossings
                    .iter()
                    .rfind(|crossing| crossing.walk == current)
                    .is_some_and(|last| last.end == index)
            {
                ray = None;
                if walk.window.1 > shown {
                    // The walk continues past the last crossed body.
                    steps.push_back(part(walk.part(shown, walk.window.1), &mut pending));
                }
            }
        }
        if let Some((_, walk, shown)) = ray
            && walk.window.1 > shown
        {
            steps.push_back(part(walk.part(shown, walk.window.1), &mut pending));
        }
        // A warrior's own steps show at once with the resolved session.
        if steps
            .iter()
            .all(|step| matches!(step.visual, Visual::Move { body, .. } if body < 10))
        {
            return None;
        }
        if !pending.is_empty() {
            steps.push_back(Step {
                events: pending,
                visual: Visual::None,
                hit: None,
                ticks: 0,
            });
        }
        Some((
            Self {
                session: previous,
                icon: None,
                effect: None,
                flight: None,
                cast_icon: None,
                result_slot: None,
                spell_info: None,
                framed: None,
                view: None,
                steps,
                until: 0.,
            },
            immediate,
        ))
    }

    /// Starts the next step once the last one's delay and messages are over and
    /// returns the events to present with it.
    pub fn advance(&mut self, now: f64, idle: bool) -> Advance {
        let ticks_per_second = crate::TICKS_PER_SECOND;
        self.recentre_flight(now);
        if now < self.until || !idle {
            return Advance::Waiting;
        }
        let Some(step) = self.steps.pop_front() else {
            return Advance::Finished;
        };
        self.icon = None;
        self.effect = None;
        self.flight = None;
        self.framed = None;
        // showresults plots its target's cicn in rect 0x4436fa.
        if let Some(body) = step.events.iter().rev().find_map(|event| match event {
            Event::Feedback(NativeFeedback::CombatResult { body, .. }) => Some(*body),
            _ => None,
        }) {
            self.result_slot = self.target_slot(body);
        }
        match step.visual {
            Visual::None => {}
            Visual::SpellInfo(info) => {
                self.result_slot = Some(ResultSlot::Spell {
                    icon: info.icon,
                    area: info.area,
                });
                if info.firearm {
                    self.cast_icon = info.weapon_icon;
                }
                self.spell_info = Some(info);
            }
            Visual::Framed { body } => self.framed = Some(body),
            Visual::Flight(frames) => {
                self.cast_icon = frames.flight.cicn;
                self.flight = Some(FlightFrames {
                    started: now,
                    tick_seconds: 1. / ticks_per_second,
                    recentred: false,
                    ..frames
                })
            }
            Visual::Effect {
                bodies,
                base,
                ticks,
            } => {
                self.effect = Some(EffectFrames {
                    bodies,
                    base,
                    started: now,
                    frame_seconds: f64::from(ticks.max(1)) / ticks_per_second,
                })
            }
            Visual::Icon(icon) => self.icon = Some(icon),
            Visual::Move { body, point } => {
                if self.place(body, point).is_err() {
                    // The copy no longer matches the native layout: present the
                    // remaining events at once and show the session's field.
                    let mut events = step.events;
                    events.extend(self.steps.drain(..).flat_map(|step| step.events));
                    self.until = now;
                    return Advance::Step(events);
                }
                if body < 10 {
                    // centerfield 0x11cb30 always recentres on the stepping warrior.
                    if self.view.is_some()
                        && let Some(combat) = &self.session.combat
                    {
                        self.view = Some(crate::combat_controls::centered(combat, point));
                    }
                    self.until = now;
                    return Advance::Step(step.events);
                }
                // movemonster 0x13e840..0x13e868: a -3 definition calls
                // centerstage2(body, 1), whose centerfield always recentres.
                let picture = self.session.combat.as_ref().is_some_and(|combat| {
                    usize::try_from(body - 10)
                        .ok()
                        .and_then(|index| combat.enemies.records.get(index))
                        .is_some_and(|record| record.record().size().ok() == Some(-3))
                });
                if let Some(origin) = self.view {
                    let relative = (point.0 - origin.0, point.1 - origin.1);
                    if (picture || !(1..=8).contains(&relative.0) || !(1..=8).contains(&relative.1))
                        && let Some(combat) = &self.session.combat
                    {
                        self.view = Some(crate::combat_controls::centered(combat, point));
                    }
                }
            }
        }
        if let Some(hit) = step.hit {
            if hit.defeated {
                // killbody draws PICT 207 over rect 0x4436fa (0x12eb50..0x12eb60).
                self.result_slot = Some(ResultSlot::Cleared);
            }
            self.hit(hit);
        }
        self.until = now + f64::from(step.ticks.max(0)) / ticks_per_second;
        Advance::Step(step.events)
    }

    /// showresults 0x1584f0..0x15857c: an NPC plots its definition's cicn +0x50,
    /// plus 308 when its facing BYTE +0x107 is 1, and WORD4435ce is the absolute
    /// size class +0x10; a warrior plots WORD +0x56 with size class 0.
    fn target_slot(&self, body: i16) -> Option<ResultSlot> {
        let combat = self.session.combat.as_ref()?;
        if body < 10 {
            let warrior = self.session.warriors.get(usize::try_from(body).ok()?)?;
            return Some(ResultSlot::Target {
                icon: warrior.record().tactical_icon().ok()?,
                size: 0,
            });
        }
        let monster = combat
            .enemies
            .records
            .get(usize::try_from(body - 10).ok()?)?;
        let record = monster.record();
        let icon = record.tactical_icon().ok()?;
        Some(ResultSlot::Target {
            icon: if record.facing_flag() == 1 {
                icon.wrapping_add(308)
            } else {
                icon
            },
            size: record.size().ok()?.wrapping_abs(),
        })
    }

    /// 0x10e368..0x10e3b0: once a sprite's rectangle misses the clip (33, 33, 287,
    /// 287), cast recentres on the target cell and the walk continues.
    fn recentre_flight(&mut self, now: f64) {
        let (Some(flight), Some(origin), Some(combat)) = (
            self.flight.as_mut(),
            self.view,
            self.session.combat.as_ref(),
        ) else {
            return;
        };
        let Some(FlightFrame::Sprite { point, .. }) = flight.frame(now) else {
            return;
        };
        let left = point.0 - origin.0 * 32;
        let top = point.1 - origin.1 * 32;
        if flight.recentred || (left < 287 && left + 32 > 33 && top < 287 && top + 32 > 33) {
            return;
        }
        flight.recentred = true;
        let target = flight.flight.to;
        self.view = Some(crate::combat_controls::centered(
            combat,
            (
                (target.0 + 16).div_euclid(32),
                (target.1 + 16).div_euclid(32),
            ),
        ));
    }

    /// placemonster 0x149e0e over the presented copy, after bodyground restored
    /// the cells the body left; a warrior's step moves its under tile like
    /// combatchoice 0x11c9be..0x11cad4.
    fn place(&mut self, body: i16, point: (i16, i16)) -> Result<(), centurions_core::RuleError> {
        let combat = self.session.combat.as_mut().ok_or("playback combat")?;
        if body < 10 {
            let member = usize::try_from(body).map_err(|_| "playback warrior")?;
            let shift = combat.map.shift;
            let cell = |(x, y): (i16, i16)| -> Option<usize> {
                let column = usize::try_from(x + shift.0 + 5).ok()?;
                let row = usize::try_from(y + shift.1 + 5).ok()?;
                (column < 90 && row < 90).then_some(column * 90 + row)
            };
            let old = cell(
                *combat
                    .squad
                    .positions
                    .get(member)
                    .ok_or("playback warrior")?,
            );
            let new = cell(point).ok_or("playback warrior cell")?;
            if let Some(old) = old
                && combat.map.tiles[old] == body
            {
                combat.map.tiles[old] = combat.squad.under_tiles[member];
            }
            if combat.map.tiles[new].wrapping_abs() >= 1000 {
                combat.squad.under_tiles[member] = combat.map.tiles[new];
            }
            combat.map.tiles[new] = body;
            combat.squad.positions[member] = point;
            return Ok(());
        }
        let index = usize::try_from(body - 10).map_err(|_| "playback body")?;
        let map = footprint::restore_body(combat, body)?;
        let record = combat.enemies.records.get(index).ok_or("playback record")?;
        let placed = footprint::place(&map, record, point, body)?;
        combat.map = placed.map;
        combat.enemies.records[index] = placed.monster;
        combat.enemies.positions[index] = point;
        Ok(())
    }

    /// The hit's stamina on the copy. A defeated NPC leaves the field, as
    /// killbody's bodyground does; a warrior stays on its cell.
    fn hit(&mut self, hit: Hit) {
        if hit.body < 10 {
            if let Some(warrior) = usize::try_from(hit.body)
                .ok()
                .and_then(|i| self.session.warriors.get_mut(i))
                && let Ok(stamina) = warrior.record().stamina()
            {
                warrior
                    .record_mut()
                    .set_stamina(stamina.wrapping_sub(hit.damage));
            }
            return;
        }
        let Some(combat) = self.session.combat.as_mut() else {
            return;
        };
        let index = (hit.body - 10) as usize;
        if let Some(record) = combat.enemies.records.get_mut(index)
            && let Ok(stamina) = record.record().stamina()
        {
            record
                .record_mut()
                .set_stamina(stamina.wrapping_sub(hit.damage));
        }
        if hit.defeated
            && let Ok(map) = footprint::restore_body(combat, hit.body)
        {
            combat.map = map;
        }
    }

    /// The presented position and showdamage size class of a body.
    pub fn body_cell(combat: &Combat, body: i16) -> Option<((i16, i16), i16)> {
        if body < 10 {
            Some((*combat.squad.positions.get(usize::try_from(body).ok()?)?, 0))
        } else {
            let index = usize::try_from(body - 10).ok()?;
            let size = combat
                .enemies
                .records
                .get(index)?
                .record()
                .size()
                .ok()?
                .wrapping_abs();
            Some((*combat.enemies.positions.get(index)?, size))
        }
    }
}

pub enum Advance {
    Waiting,
    Step(Vec<Event>),
    Finished,
}
