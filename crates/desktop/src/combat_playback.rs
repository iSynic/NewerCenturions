//! Plays one combat command's monster steps and result pauses back over the field.
//!
//! The rules resolve a whole monster action in one command, but the game draws
//! as it goes. movemonster places and draws the body (centerstage2 for a -3
//! definition), plays the tile sound, calls collide and waits half the animation
//! delay. showresults plots the damage cicn over the result body and waits
//! 30 - 2 * speed ticks before centerfield redraws the field. A warrior's own
//! step writes the map, recentres on the warrior and plays the tile sound with no
//! wait. cast draws each target's flight after its launch sound; resolvespell
//! then animates the whole area at once or flashes each target before its
//! showresults. Playback runs over a copy of the field taken before the command,
//! so rule state is never touched.
use crate::combat_flight::{FlightFrame, FlightFrames};
use centurions_core::{
    Event, Flight, Session, SpellArea, SpellInfo,
    combat::{Combat, footprint},
    feedback::GameFeedback,
};
use std::collections::VecDeque;

/// One showdamage call: the cicn plotted over a body and the number it prints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResultIcon {
    pub body: i16,
    pub icon: i16,
    pub amount: i16,
}

/// What the 64-pixel result rectangle of the combat panel shows: showspellinfo's
/// cicn, showresults' target cicn or killbody's PICT 207.
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

/// What a step shows: nothing, the spell's name, a body's move, a result icon, a
/// spell effect, a missile's flight, or a framed body.
enum Visual {
    None,
    SpellInfo(SpellInfo),
    Move { body: i16, point: (i16, i16) },
    Icon(ResultIcon),
    Effect { bodies: u128, base: i16, ticks: i16 },
    Flight(FlightFrames),
    Framed { body: i16 },
}

/// The preferences and spell values the playback reads.
pub struct Settings {
    /// The Speed menu item.
    pub speed: i16,
    /// The spell's effect after the command, which picks spelleffect's cicns.
    pub effect: i8,
    /// The last spell's targeting mode.
    pub targeting: u8,
    /// Faster Scan Casting (preference 9): cast's short walk.
    pub short_walks: bool,
    /// Faster Scan Resolution (preference 10), read by spelleffect and resolvespell.
    pub batched_effects: bool,
    /// The item value after the command, which showresults prints with cicns 165
    /// and 166.
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
/// rectangle: one body holds each frame for a thirtieth of the animation delay
/// plus one tick, or every body of the area holds each frame for four ticks.
#[derive(Clone, Copy, Debug)]
pub struct EffectFrames {
    /// Bit n set for body n.
    pub bodies: u128,
    pub base: i16,
    pub started: f64,
    pub frame_seconds: f64,
}

impl EffectFrames {
    /// The cicn of the effect at time `now`, while its eight frames last.
    pub fn icon(&self, now: f64) -> Option<i16> {
        let frame = ((now - self.started) / self.frame_seconds).floor();
        (0. ..8.).contains(&frame).then(|| self.base + frame as i16)
    }

    /// The bodies the effect covers.
    pub fn bodies(&self) -> impl Iterator<Item = i16> + '_ {
        (0..110).filter(|body| self.bodies >> body & 1 != 0)
    }
}

/// A result of resolvespell's body loop: every spell result, and the attack
/// results marked with the loop's area (monster and ally fire through resolvespell).
fn resolvespell_result(event: &Event) -> bool {
    match event {
        Event::SpellResolved { .. } => true,
        Event::AttackResolved { flight, .. } => flight.1.is_some(),
        _ => false,
    }
}

/// spelleffect's first cicn: 0x2ef8 without an effect, else effect * 8 + 0x2ed8.
///
/// Original: spelleffect
fn effect_base(effect: i8) -> i16 {
    if effect == 0 {
        0x2ef8
    } else {
        i16::from(effect) * 8 + 0x2ed8
    }
}

/// spelleffect for one body: modes 9 and 10, or mode 3 under Faster Scan
/// Resolution, frame the body with drawbody(body, 1), held ten ticks unless Faster
/// Scan Resolution is on; other modes plot the eight cicns over the body.
///
/// Original: spelleffect
fn body_effect(body: i16, mode: i8, effect: i8, settings: &Settings) -> Step {
    if matches!(mode, 9 | 10) || settings.batched_effects && mode == 3 {
        return Step {
            events: vec![],
            visual: Visual::Framed { body },
            hit: None,
            ticks: if settings.batched_effects { 0 } else { 10 },
        };
    }
    let frame_ticks = animation_delay(settings.speed) / 30 + 1;
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

/// One beat of the playback: its sounds and messages, what it shows, the health
/// change it applies and the wait after it.
struct Step {
    /// The sounds and messages presented while this step draws.
    events: Vec<Event>,
    visual: Visual,
    hit: Option<Hit>,
    /// The wait after the step, in ticks.
    ticks: i16,
}

/// A combat command's results played one step at a time, over the session as it
/// was before the command.
pub struct Playback {
    /// The session as presented: field, panels and health lag the resolved session
    /// until each step plays, and the resolved session is shown at the end.
    pub session: Session,
    pub icon: Option<ResultIcon>,
    pub effect: Option<EffectFrames>,
    pub flight: Option<FlightFrames>,
    /// ploticon3's cast cicn in the combat panel's cast rectangle, which stays
    /// until the controls are redrawn after the command. showspellinfo replaces
    /// it for a category-9 spell.
    pub cast_icon: Option<i16>,
    /// The result rectangle, kept like `cast_icon`.
    pub result_slot: Option<ResultSlot>,
    /// showspellinfo's name and number, kept like `cast_icon`.
    pub spell_info: Option<SpellInfo>,
    /// spelleffect's flash: drawbody(body, 1) frames the body until drawbody(body, 0).
    pub framed: Option<i16>,
    /// The view origin while playing. movemonster recentres only when a body
    /// leaves view cells 1..8; showresults' centerfield passes the current
    /// centre, so it only redraws.
    pub view: Option<(i16, i16)>,
    steps: VecDeque<Step>,
    until: f64,
}

/// The cicn showresults draws for each fixed result code; other codes draw none
/// unless code 0 resolved damage.
///
/// Original: showresults
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

/// The showresults line for a code the rules resolve without feedback: code 0
/// (a hit) and -1 (no damage). Presentation only; it never reaches the rules,
/// which read real CombatResult feedback.
fn hit_line(body: i16, code: i16, amount: i16) -> Event {
    Event::Feedback(GameFeedback::CombatResult {
        body,
        code,
        amount,
        audible: false,
    })
}

/// The animation delay for the Speed menu item: 5 * speed - 5, so the default
/// speed 5 gives 20.
///
/// Original: createpref
pub fn animation_delay(speed: i16) -> i16 {
    speed * 5 - 5
}

/// showresults' pause after a result: 30 - 2 * speed ticks.
fn result_ticks(speed: i16) -> i16 {
    30 - 2 * speed
}

/// The command's events read ahead of playback: resolvespell's areas, the
/// showspellinfo calls and the ray walks.
struct Plan {
    /// resolvespell's area for each body loop, placed where the loop's events
    /// begin: (first event of the loop, result carrying it, area).
    areas: Vec<(usize, usize, SpellArea)>,
    /// Each ray walk's flight and crossings.
    walks: Vec<Flight>,
    crossings: Vec<Crossing>,
    /// showspellinfo before an NPC's cast or resolvespell, at the event it precedes.
    infos: Vec<(usize, SpellInfo)>,
}

impl Plan {
    /// Finds the missile flights, area effects and spell names in the command's
    /// events.
    fn new(events: &[Event]) -> Self {
        let areas = events
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
        let (walks, crossings) = ray_walks(events);
        let infos = events
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
        Self {
            areas,
            walks,
            crossings,
            infos,
        }
    }
}

/// The steps read so far and the presentation state carried between events.
struct Builder<'a> {
    events: &'a [Event],
    settings: &'a Settings,
    plan: &'a Plan,
    animation_delay: i16,
    /// movemonster's wait after a step: half the animation delay, rounded toward
    /// zero.
    move_ticks: i16,
    steps: VecDeque<Step>,
    immediate: Vec<Event>,
    pending: Vec<Event>,
    /// attack2 plays a hit's impact sound before its showresults call; the rules
    /// emit it directly after AttackResolved.
    impact: bool,
    killbody_delay: bool,
    /// Whether the attack being read already presented a showresults code.
    presented: bool,
    /// combatchoice plays the tile sound after the warrior's step.
    stepped: bool,
    drawn_flights: Vec<usize>,
    /// The body loop being read: whether resolvespell animates its whole area
    /// at once, its mode and its effect.
    area_path: bool,
    mode: i8,
    effect: i8,
    /// While a ray walk plays: its walk, frames and the ticks already shown.
    ray: Option<(usize, FlightFrames, i16)>,
}

impl<'a> Builder<'a> {
    /// Starts a build of the steps for `events` with the animation delay for the
    /// speed setting.
    fn new(events: &'a [Event], settings: &'a Settings, plan: &'a Plan) -> Self {
        let animation_delay = animation_delay(settings.speed);
        Self {
            events,
            settings,
            plan,
            animation_delay,
            move_ticks: animation_delay / 2,
            steps: VecDeque::new(),
            immediate: vec![],
            pending: vec![],
            impact: false,
            killbody_delay: false,
            presented: false,
            stepped: false,
            drawn_flights: vec![],
            area_path: false,
            mode: settings.targeting as i8,
            effect: settings.effect,
            ray: None,
        }
    }

    /// Reads one event into steps, in the order cast, resolvespell and
    /// showresults present it.
    fn read(&mut self, index: usize, event: &Event) {
        self.show_spell_info(index);
        self.advance_ray(index);
        self.open_areas(index);
        let after_hit = std::mem::take(&mut self.impact);
        let after_step = std::mem::take(&mut self.stepped);
        self.draw_result_flight(index, event);
        if self.killbody_delay && matches!(event, Event::Delay { ticks: 15 }) {
            self.killbody_delay = false;
            return;
        }
        match event {
            Event::Sound { .. } if after_hit || after_step => {
                if let Some(step) = self.steps.back_mut() {
                    step.events.push(event.clone());
                }
            }
            Event::Sound { .. } | Event::Delay { .. } => self.pending.push(event.clone()),
            // A flight without a presented result plays after its launch sound.
            Event::Launch { flight, .. } => {
                self.pending.push(event.clone());
                if let Some(flight) = flight.0 {
                    self.push_flight(self.frames(flight));
                }
            }
            Event::Feedback(feedback) => self.on_feedback(index, event, feedback),
            Event::TacticalMoved { body, x, y } => self.on_move(*body, (*x, *y)),
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
            } if *damage > 0 => self.on_damage_result(event, *target, *damage, *defeated, icon.0),
            Event::AttackResolved {
                target,
                defeated: false,
                ..
            } => self.on_miss(event, *target),
            Event::SpellResolved {
                target,
                damage,
                defeated: false,
                ..
            } if *damage < 0 && !self.area_path => self.on_heal(event, *target, *damage),
            _ => self.immediate.push(event.clone()),
        }
        self.end_ray(index);
    }

    /// The flight frames cast draws for one flight.
    fn frames(&self, flight: Flight) -> FlightFrames {
        let walk = if flight.short && self.settings.short_walks {
            30
        } else {
            128
        } + self.animation_delay;
        let frames = FlightFrames::new(flight, walk, 0., 1.);
        // A ray blocked at a step walks no further.
        let end = flight.stop.map_or(frames.window.1, |stop| {
            FlightFrames::ticks_before(stop).min(frames.window.1)
        });
        frames.part(0, end)
    }

    /// A step carrying the pending events.
    fn push(&mut self, visual: Visual, hit: Option<Hit>, ticks: i16) {
        self.steps.push_back(Step {
            events: std::mem::take(&mut self.pending),
            visual,
            hit,
            ticks,
        });
    }

    /// Adds a step that plays a missile flight.
    fn push_flight(&mut self, frames: FlightFrames) {
        self.push(Visual::Flight(frames), None, frames.length());
    }

    /// spelleffect(body, 0) carrying the pending events.
    fn flash(&mut self, body: i16) -> Step {
        let mut flash = body_effect(body, self.mode, self.effect, self.settings);
        flash.events = std::mem::take(&mut self.pending);
        flash
    }

    /// Adds the spell name and number steps that come before event `index`.
    fn show_spell_info(&mut self, index: usize) {
        let plan = self.plan;
        for (_, info) in plan.infos.iter().filter(|(at, _)| *at == index) {
            self.push(Visual::SpellInfo(*info), None, 0);
        }
    }

    /// cast's default walk resolves each crossed body at its step, before
    /// that step's sprite and delay: the walk plays up to the step, then the
    /// crossing's events.
    fn advance_ray(&mut self, index: usize) {
        let plan = self.plan;
        for crossing in plan
            .crossings
            .iter()
            .filter(|crossing| crossing.start == index)
        {
            if self
                .ray
                .as_ref()
                .is_none_or(|(walk, ..)| *walk != crossing.walk)
            {
                self.ray = Some((crossing.walk, self.frames(plan.walks[crossing.walk]), 0));
            }
            if let Some((current, walk, shown)) = self.ray {
                let next = FlightFrames::ticks_before(crossing.step).min(walk.window.1);
                if next > shown {
                    self.push_flight(walk.part(shown, next));
                    self.ray = Some((current, walk, next));
                }
            }
        }
    }

    /// Enters each resolvespell body loop that begins at this event.
    fn open_areas(&mut self, index: usize) {
        let (plan, events) = (self.plan, self.events);
        for (_, result, area) in plan.areas.iter().filter(|(at, ..)| *at == index) {
            // cast draws every flight before it calls resolvespell.
            if let Event::AttackResolved { flight, .. } | Event::SpellResolved { flight, .. } =
                &events[*result]
                && let Some(flight) = flight.0.filter(|_| flight.2.is_none())
            {
                self.push_flight(self.frames(flight));
                self.drawn_flights.push(*result);
            }
            self.mode = area.mode;
            self.effect = area.effect;
            // Faster Scan Resolution animates the area at once for modes 3, 4
            // and above 8.
            self.area_path = self.settings.batched_effects && matches!(area.mode, 3 | 4 | 9..);
            if self.area_path {
                self.pending.push(Event::Sound {
                    resource_id: area.sound,
                });
                self.push(
                    Visual::Effect {
                        bodies: area.bodies,
                        base: effect_base(area.effect),
                        ticks: 4,
                    },
                    None,
                    // Four ticks after each of the eight frames.
                    32,
                );
            }
        }
    }

    /// A result's own flight, unless a ray walk or its area already drew it.
    fn draw_result_flight(&mut self, index: usize, event: &Event) {
        if let Event::AttackResolved { flight, .. } | Event::SpellResolved { flight, .. } = event
            && let (Some(flight), None) = (flight.0, flight.2)
            && !self.drawn_flights.contains(&index)
        {
            self.push_flight(self.frames(flight));
        }
    }

    /// Handles a combat result message: a zero-damage spell result flashes its
    /// target first.
    fn on_feedback(&mut self, index: usize, event: &Event, feedback: &GameFeedback) {
        if let GameFeedback::CombatResult {
            body, amount: 0, ..
        } = feedback
            && !self.area_path
            && matches!(
                self.events.get(index + 1),
                Some(Event::SpellResolved { target, damage: 0, .. }) if target == body
            )
        {
            // A successful zero-damage spell result flashes before its
            // showresults.
            let flash = self.flash(*body);
            self.steps.push_back(flash);
        }
        self.pending.push(event.clone());
        self.presented |= matches!(feedback, GameFeedback::CombatResult { .. });
        if let GameFeedback::CombatResult {
            body, code, amount, ..
        } = feedback
        {
            // A damage cicn needs a positive amount. showresults plots 166
            // always and 165 after damage; both print the item value.
            let visual = result_icon(*code)
                .filter(|icon| *icon >= 165 || *amount > 0)
                .map_or(Visual::None, |icon| {
                    Visual::Icon(ResultIcon {
                        body: *body,
                        icon,
                        amount: if icon >= 165 {
                            self.settings.item_value
                        } else {
                            *amount
                        },
                    })
                });
            self.push(visual, None, result_ticks(self.settings.speed));
        }
    }

    /// Adds a body's step to its new cell; a warrior's step has no wait of its own.
    fn on_move(&mut self, body: i16, point: (i16, i16)) {
        self.stepped = crate::acting::is_warrior_body(body);
        self.push(
            Visual::Move { body, point },
            None,
            if crate::acting::is_warrior_body(body) {
                0
            } else {
                self.move_ticks
            },
        );
    }

    /// Adds the steps for a hit or spell result: the damage message, the result
    /// icon, the health change, and the defeated body leaving the field.
    fn on_damage_result(
        &mut self,
        event: &Event,
        target: i16,
        damage: i16,
        defeated: bool,
        icon: i16,
    ) {
        self.immediate.push(event.clone());
        // A resolved hit always carries a nonzero cicn, so 0 means the caller
        // shows none. A killing hit skips showresults: killbody waits 15 ticks
        // before bodyground and bodyfield clear the body.
        let killed = defeated && !crate::acting::is_warrior_body(target);
        // attack's showresults code 0 prints the damage line.
        if matches!(event, Event::AttackResolved { .. })
            && !killed
            && !std::mem::take(&mut self.presented)
        {
            self.pending.push(hit_line(target, 0, damage));
        }
        if resolvespell_result(event) && !defeated && !self.area_path {
            // A surviving target flashes before its showresults.
            let flash = self.flash(target);
            self.steps.push_back(flash);
        }
        let visual = if icon == 0 || killed {
            Visual::None
        } else {
            Visual::Icon(ResultIcon {
                body: target,
                icon,
                amount: damage,
            })
        };
        self.push(
            visual,
            Some(Hit {
                body: target,
                damage,
                defeated: false,
            }),
            if killed {
                15
            } else {
                result_ticks(self.settings.speed)
            },
        );
        if defeated && crate::acting::is_warrior_body(target) {
            // killbody gives a dead warrior its spelleffect.
            self.steps
                .push_back(body_effect(target, self.mode, self.effect, self.settings));
        }
        if killed {
            self.steps.push_back(Step {
                events: vec![],
                visual: Visual::None,
                hit: Some(Hit {
                    body: target,
                    damage: 0,
                    defeated: true,
                }),
                ticks: 0,
            });
            // The rules' 15-tick delay is killbody's own wait, played above.
            self.killbody_delay = true;
        }
        self.impact = matches!(event, Event::AttackResolved { .. }) && !killed;
    }

    /// Adds the step for a miss.
    fn on_miss(&mut self, event: &Event, target: i16) {
        self.immediate.push(event.clone());
        // A miss calls showresults with -1.
        if !std::mem::take(&mut self.presented) {
            self.pending.push(hit_line(target, -1, 0));
            self.push(Visual::None, None, result_ticks(self.settings.speed));
            self.impact = true;
        }
    }

    /// Adds the step for a heal: the target flashes and its health rises.
    fn on_heal(&mut self, event: &Event, target: i16, damage: i16) {
        // A heal (negative damage) takes the surviving target's flash.
        self.immediate.push(event.clone());
        let mut flash = self.flash(target);
        flash.hit = Some(Hit {
            body: target,
            damage,
            defeated: false,
        });
        self.steps.push_back(flash);
    }

    /// Ends the ray walk after its last crossing's events.
    fn end_ray(&mut self, index: usize) {
        let plan = self.plan;
        if let Some((current, walk, shown)) = self.ray
            && plan
                .crossings
                .iter()
                .rfind(|crossing| crossing.walk == current)
                .is_some_and(|last| last.end == index)
        {
            self.ray = None;
            if walk.window.1 > shown {
                // The walk continues past the last crossed body.
                self.push_flight(walk.part(shown, walk.window.1));
            }
        }
    }

    /// Completes the steps into a playback, with the events to present at once.
    fn finish(mut self, previous: Session) -> Option<(Playback, Vec<Event>)> {
        if let Some((_, walk, shown)) = self.ray
            && walk.window.1 > shown
        {
            self.push_flight(walk.part(shown, walk.window.1));
        }
        // A warrior's own steps show at once with the resolved session.
        if self.steps.iter().all(|step| {
            matches!(step.visual, Visual::Move { body, .. } if crate::acting::is_warrior_body(body))
        }) {
            return None;
        }
        if !self.pending.is_empty() {
            self.push(Visual::None, None, 0);
        }
        Some((
            Playback {
                session: previous,
                icon: None,
                effect: None,
                flight: None,
                cast_icon: None,
                result_slot: None,
                spell_info: None,
                framed: None,
                view: None,
                steps: self.steps,
                until: 0.,
            },
            self.immediate,
        ))
    }
}

impl Playback {
    /// Returns the playback and the events that are presented at once, or None
    /// when the command produced no tactical step to show.
    pub fn build(
        previous: Session,
        events: &[Event],
        settings: &Settings,
    ) -> Option<(Self, Vec<Event>)> {
        let plan = Plan::new(events);
        let mut builder = Builder::new(events, settings, &plan);
        for (index, event) in events.iter().enumerate() {
            builder.read(index, event);
        }
        builder.finish(previous)
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
        // showresults plots its target's cicn in the result rectangle.
        if let Some(body) = step.events.iter().rev().find_map(|event| match event {
            Event::Feedback(GameFeedback::CombatResult { body, .. }) => Some(*body),
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
                    // The copy no longer matches the field the rules saw:
                    // present the remaining events at once and show the
                    // session's field.
                    let mut events = step.events;
                    events.extend(self.steps.drain(..).flat_map(|step| step.events));
                    self.until = now;
                    return Advance::Step(events);
                }
                if crate::acting::is_warrior_body(body) {
                    // centerfield always recentres on the stepping warrior.
                    if self.view.is_some()
                        && let Some(combat) = &self.session.combat
                    {
                        self.view = Some(crate::combat_controls::centered(combat, point));
                    }
                    self.until = now;
                    return Advance::Step(step.events);
                }
                // A -3 definition draws through centerstage2(body, 1), whose
                // centerfield always recentres.
                let picture = self.session.combat.as_ref().is_some_and(|combat| {
                    usize::try_from(body - 10)
                        .ok()
                        .and_then(|index| combat.roster.records.get(index))
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
                // killbody draws PICT 207 over the result rectangle.
                self.result_slot = Some(ResultSlot::Cleared);
            }
            self.hit(hit);
        }
        self.until = now + f64::from(step.ticks.max(0)) / ticks_per_second;
        Advance::Step(step.events)
    }

    /// The target cicn for the result rectangle: a monster's definition cicn,
    /// plus 308 when it faces left, with its absolute size class; a warrior's
    /// tactical icon with size class 0.
    ///
    /// Original: showresults
    fn target_slot(&self, body: i16) -> Option<ResultSlot> {
        let combat = self.session.combat.as_ref()?;
        if crate::acting::is_warrior_body(body) {
            let warrior = self.session.warriors.get(usize::try_from(body).ok()?)?;
            return Some(ResultSlot::Target {
                icon: warrior.record().icon().ok()?,
                size: 0,
            });
        }
        let monster = combat
            .roster
            .records
            .get(usize::try_from(body - 10).ok()?)?;
        let record = monster.record();
        let icon = record.icon().ok()?;
        Some(ResultSlot::Target {
            icon: if record.facing_left() == 1 {
                icon.wrapping_add(308)
            } else {
                icon
            },
            size: record.size().ok()?.wrapping_abs(),
        })
    }

    /// Once a sprite's rectangle misses the clip (33, 33, 287, 287), cast
    /// recentres on the target cell and the walk continues.
    ///
    /// Original: cast
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

    /// Places a body on the presented copy after bodyground restores the cells
    /// it left; a warrior's step moves its under tile as combatchoice does.
    ///
    /// Original: placemonster
    fn place(&mut self, body: i16, point: (i16, i16)) -> Result<(), centurions_core::RuleError> {
        let combat = self.session.combat.as_mut().ok_or("playback combat")?;
        if crate::acting::is_warrior_body(body) {
            let member = usize::try_from(body).map_err(|_| "playback warrior")?;
            let shift = combat.field.map_shift;
            let cell = |(x, y): (i16, i16)| -> Option<usize> {
                let column = usize::try_from(x + shift.0 + 5).ok()?;
                let row = usize::try_from(y + shift.1 + 5).ok()?;
                (column < 90 && row < 90).then_some(column * 90 + row)
            };
            let old = cell(*combat.squad.pos.get(member).ok_or("playback warrior")?);
            let new = cell(point).ok_or("playback warrior cell")?;
            if let Some(old) = old
                && combat.field.tiles[old] == body
            {
                combat.field.tiles[old] = combat.squad.char_under[member];
            }
            if combat.field.tiles[new].wrapping_abs() >= 1000 {
                combat.squad.char_under[member] = combat.field.tiles[new];
            }
            combat.field.tiles[new] = body;
            combat.squad.pos[member] = point;
            return Ok(());
        }
        let index = usize::try_from(body - 10).map_err(|_| "playback body")?;
        let map = footprint::restore_body(combat, body)?;
        let record = combat.roster.records.get(index).ok_or("playback record")?;
        let placed = footprint::place(&map, record, point, body)?;
        combat.field = placed.map;
        combat.roster.records[index] = placed.monster;
        combat.roster.mon_pos[index] = point;
        Ok(())
    }

    /// The hit's stamina on the copy. A defeated NPC leaves the field, as
    /// killbody's bodyground does; a warrior stays on its cell.
    fn hit(&mut self, hit: Hit) {
        if crate::acting::is_warrior_body(hit.body) {
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
        if let Some(record) = combat.roster.records.get_mut(index)
            && let Ok(stamina) = record.record().stamina()
        {
            record
                .record_mut()
                .set_stamina(stamina.wrapping_sub(hit.damage));
        }
        if hit.defeated
            && let Ok(map) = footprint::restore_body(combat, hit.body)
        {
            combat.field = map;
        }
    }

    /// The presented position and showdamage size class of a body.
    pub fn body_cell(combat: &Combat, body: i16) -> Option<((i16, i16), i16)> {
        if crate::acting::is_warrior_body(body) {
            Some((*combat.squad.pos.get(usize::try_from(body).ok()?)?, 0))
        } else {
            let index = usize::try_from(body - 10).ok()?;
            let size = combat
                .roster
                .records
                .get(index)?
                .record()
                .size()
                .ok()?
                .wrapping_abs();
            Some((*combat.roster.mon_pos.get(index)?, size))
        }
    }
}

/// What the playback gives back on each frame: still waiting, the events of the
/// next step, or finished.
pub enum Advance {
    Waiting,
    Step(Vec<Event>),
    Finished,
}
