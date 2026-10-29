//! cast10e0c8..10e642: a ray crosses cells in fixed pixel steps and yields each new body.
use crate::{RandomState, RuleError, combat::Combat, event::Flight};
use centurions_content::{mission::OpeningMission, records::AttackPacket};

pub(crate) enum RayStep {
    Body(usize),
    Blocked,
}

/// One ray from pixel `start` to pixel `end`; the caller resolves each body before stepping again.
pub(crate) struct RayWalk {
    point: (f32, f32),
    increment: (f32, f32),
    steps: i16,
    remaining: i16,
    tested: i16,
    seen: [bool; 110],
    actor: usize,
}

impl RayWalk {
    pub(crate) fn new(start: (i16, i16), end: (i16, i16), steps: i16, actor: usize) -> Self {
        Self {
            point: (f32::from(start.0), f32::from(start.1)),
            increment: (
                f32::from(end.0.wrapping_sub(start.0)) / steps as f32,
                f32::from(end.1.wrapping_sub(start.1)) / steps as f32,
            ),
            steps,
            remaining: steps,
            tested: -1,
            seen: [false; 110],
            actor,
        }
    }

    /// The walk step (cast's d3) the last `next` tested.
    pub(crate) fn step(&self) -> i16 {
        self.tested
    }

    pub(crate) fn next(
        &mut self,
        tiles: &[i16],
        shift: (i16, i16),
        mission: &OpeningMission,
        packet: &[u8],
    ) -> Result<Option<RayStep>, RuleError> {
        while self.remaining > 0 {
            self.remaining -= 1;
            self.tested = self.steps - self.remaining - 1;
            let x = (self.point.0.trunc() as i16).wrapping_add(16) / 32;
            let y = (self.point.1.trunc() as i16).wrapping_add(16) / 32;
            self.point.0 += self.increment.0;
            self.point.1 += self.increment.1;
            let cell = OpeningMission::index(
                i32::from(x) + i32::from(shift.0) + 5,
                i32::from(y) + i32::from(shift.1) + 5,
            )?;
            let body = tiles[cell].wrapping_abs();
            // 0x10e0d8..0x10e0ec: packet bytes 3 and 2, each sign-extended, summed as a WORD.
            if AttackPacket(packet).range_per_power() + AttackPacket(packet).base_range() > 0
                && body > 1000
                && mission.tile_word(body - 1000, 12)? != 0
            {
                self.remaining = 0;
                return Ok(Some(RayStep::Blocked));
            }
            if (0..110).contains(&body) && body as usize != self.actor && !self.seen[body as usize]
            {
                self.seen[body as usize] = true;
                return Ok(Some(RayStep::Body(body as usize)));
            }
        }
        Ok(None)
    }
}

/// cast10d654/10da62: a body's ray pixel; an NPC footprint of size 1 or 3 moves it up and
/// a size above 1 moves it left by half a cell.
pub(crate) fn body_pixel(cell: (i16, i16), npc_size: Option<i16>) -> (i16, i16) {
    let mut pixel = (cell.0.wrapping_mul(32), cell.1.wrapping_mul(32));
    if let Some(size) = npc_size {
        if matches!(size, 1 | 3) {
            pixel.1 = pixel.1.wrapping_sub(16);
        }
        if size > 1 {
            pixel.0 = pixel.0.wrapping_sub(16);
        }
    }
    pixel
}

/// cast's switch on packet byte 17 (BYTE44375f, 0x10dca2) after a target's launch
/// sound: type 24 (0x10de70..0x10df1e) makes eight passes, each adding Rand(10) - 5
/// to the target's h pixel (0x10de8a) and then its v pixel (0x10de9e). No other type
/// draws. Returns the offsets in draw order, zero when nothing was drawn.
pub(crate) fn flight_jitter(packet: &[u8], random: &mut RandomState) -> [i8; 16] {
    let mut jitter = [0; 16];
    if packet.get(17) == Some(&24) {
        for offset in &mut jitter {
            *offset = (random.bound(10) - 5) as i8;
        }
    }
    jitter
}

/// cast's arms for byte-17 types 13 and 23..=27 end at 0x10e5dc, past the default walk
/// (0x10e07a..0x10e5d6) that alone resolves a ray's crossed bodies, and a mode-6 packet
/// then skips spelltargets (0x10e61c). Such a ray resolves no body and makes no block test.
pub(crate) fn ray_walks(packet: &[u8]) -> bool {
    !matches!(packet.get(17).map(|kind| *kind as i8), Some(13 | 23..=27))
}

/// cast's flight for one target triple: the caster's pixel (0x10d654..0x10d71e), the
/// target's (a warrior, an NPC footprint or a map cell, 0x10d870..0x10dada), the
/// short-walk test at 0x10d5aa..0x10d5d0, which also needs preference 9, and the
/// offsets `flight_jitter` drew for this target.
pub(crate) fn flight(
    combat: &Combat,
    caster: i16,
    target: &super::SpellTarget,
    packet: &[u8],
    targets: usize,
    jitter: [i8; 16],
) -> Option<Flight> {
    let pixel = |body: i16| -> Option<(i16, i16)> {
        if body < 10 {
            Some(body_pixel(
                *combat.squad.positions.get(usize::try_from(body).ok()?)?,
                None,
            ))
        } else {
            let index = usize::try_from(body - 10).ok()?;
            let size = combat.enemies.records.get(index)?.record().size().ok()?;
            Some(body_pixel(
                *combat.enemies.positions.get(index)?,
                Some(size.wrapping_abs()),
            ))
        }
    };
    let to = match target.kind {
        1 | 2 => pixel(target.first)?,
        _ => (
            target
                .first
                .wrapping_sub(combat.map.shift.0 + 5)
                .wrapping_mul(32),
            target
                .second
                .wrapping_sub(combat.map.shift.1 + 5)
                .wrapping_mul(32),
        ),
    };
    flight_between(packet, pixel(caster)?, to, targets, jitter)
}

/// cast's flight between two pixels already placed like `flight`'s.
pub(crate) fn flight_between(
    packet: &[u8],
    from: (i16, i16),
    to: (i16, i16),
    targets: usize,
    jitter: [i8; 16],
) -> Option<Flight> {
    let attack = AttackPacket(packet);
    let kind = packet.get(17).map(|kind| *kind as i8)?;
    Some(Flight {
        kind,
        from,
        to,
        short: attack.targeting_mode() != 6 && attack.defense_category() != 9 && targets > 1,
        jitter,
        cicn: (attack.defense_category() != 9).then(|| 12000 + i16::from(kind) * 8),
        stop: None,
    })
}

/// Each crossed body with its walk step, and the step of a blocking stop.
pub(crate) type Crossings = (Vec<(usize, i16)>, Option<i16>);

/// The ray's crossed bodies and blocking stop by walk step, read before the walk
/// resolves anything: a resolved body leaves the map only after the walk passed
/// it, so later steps meet the same bodies.
pub(crate) fn crossings(
    tiles: &[i16],
    shift: (i16, i16),
    mission: &OpeningMission,
    packet: &[u8],
    (start, end, steps, actor): ((i16, i16), (i16, i16), i16, usize),
) -> Result<Crossings, RuleError> {
    let mut walk = RayWalk::new(start, end, steps, actor);
    let mut bodies = vec![];
    while let Some(step) = walk.next(tiles, shift, mission, packet)? {
        match step {
            RayStep::Body(body) => bodies.push((body, walk.step())),
            RayStep::Blocked => return Ok((bodies, Some(walk.step()))),
        }
    }
    Ok((bodies, None))
}

/// Marks each result a ray walk presented with the step that crossed its body.
/// The walk's events between two crossings belong to the later one, and after
/// the last crossing to it, except the blocking stop's sound when `blocked`.
pub(crate) fn mark_crossings(events: &mut [crate::Event], bodies: &[(usize, i16)], blocked: bool) {
    let steps: Vec<Option<i16>> = events
        .iter()
        .map(|event| match event {
            crate::Event::AttackResolved { target, .. }
            | crate::Event::SpellResolved { target, .. } => bodies
                .iter()
                .find(|(body, _)| *body as i16 == *target)
                .map(|(_, step)| *step),
            _ => None,
        })
        .collect();
    let end = events.len().saturating_sub(usize::from(blocked));
    let (mut start, mut index) = (0, 0);
    while index < end {
        let Some(step) = steps[index] else {
            index += 1;
            continue;
        };
        let mut last = index;
        let mut next = index + 1;
        while next < end && steps[next].is_none_or(|other| other == step) {
            if steps[next].is_some() {
                last = next;
            }
            next += 1;
        }
        let stop = if next >= end { end } else { last + 1 };
        mark(&mut events[start..stop], step, |at| {
            steps[start + at] == Some(step)
        });
        (start, index) = (stop, stop);
    }
}

/// Marks the results one crossed body presented with the walk step that crossed it.
pub(crate) fn mark_step(events: &mut [crate::Event], step: i16) {
    mark(events, step, |_| true);
}

/// Marks the chosen results of one crossing's events with its step and their place.
fn mark(events: &mut [crate::Event], step: i16, chosen: impl Fn(usize) -> bool) {
    let length = events.len();
    for (at, event) in events.iter_mut().enumerate() {
        if let crate::Event::AttackResolved { flight, .. }
        | crate::Event::SpellResolved { flight, .. } = event
            && chosen(at)
        {
            flight.2 = Some(crate::event::RayCrossing {
                step,
                lead: u16::try_from(at).unwrap_or(u16::MAX),
                tail: u16::try_from(length - at - 1).unwrap_or(u16::MAX),
            });
        }
    }
}

/// Records where a ray stopped on the flight launched last.
pub(crate) fn stop_flight(flights: &mut [(usize, Option<Flight>)], stop: Option<i16>) {
    if let Some((_, Some(flight))) = flights.last_mut() {
        flight.stop = stop;
    }
}
