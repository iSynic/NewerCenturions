use crate::feedback;
use serde::{Deserialize, Serialize};

/// A cicn ID the desktop draws for an event. It is presentation data only: it is
/// not serialized and never distinguishes two events, so replays, recorded event
/// fixtures and event hashes are unchanged by it.
#[derive(Clone, Copy, Debug, Default)]
pub struct PresentationIcon(pub i16);

impl PartialEq for PresentationIcon {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for PresentationIcon {}

/// The flight cast 0x10d4fe draws for one target triple after its launch sound
/// (0x10db1a) and before that target resolves, the cast's spell area that
/// resolvespell 0x1517c4 reads before its body loop, for a body a ray crosses the
/// walk step whose resolvespell (0x10e22a) presented it, and the showspellinfo
/// drawing that precedes an NPC's cast. Like `PresentationIcon` it is presentation
/// data only: not serialized and never distinguishing two events.
#[derive(Clone, Copy, Debug, Default)]
pub struct PresentationFlight(
    pub Option<Flight>,
    pub Option<SpellArea>,
    pub Option<RayCrossing>,
    pub Option<SpellInfo>,
);

impl PartialEq for PresentationFlight {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for PresentationFlight {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flight {
    /// Packet byte 17 (BYTE44375f), cast's switch at 0x10dca2.
    pub kind: i8,
    /// The caster's and target's pixels in tactical positions times 32
    /// (0x10d654..0x10d71e and 0x10d870..0x10dada).
    pub from: (i16, i16),
    pub to: (i16, i16),
    /// cast 0x10d5aa..0x10d5d0 shortens the walk to 30 + WORD439568 steps under
    /// preference 9 for more than one target outside a ray or physical packet.
    pub short: bool,
    /// A type-24 flight's sixteen Rand(10) - 5 draws (0x10de8a, 0x10de9e): pass n adds
    /// `jitter[2n]` to the target's h pixel and `jitter[2n + 1]` to its v pixel.
    pub jitter: [i8; 16],
    /// ploticon3 0x10d596: cicn 12000 + 8 * byte 17 that cast plots in window 128
    /// at rect 0x4436f2 before its triple loop; none for a category-9 packet
    /// (BYTE443761, 0x10d56e).
    pub cicn: Option<i16>,
    /// The walk step at which a ray stopped on a blocking cell (0x10e110..0x10e198).
    pub stop: Option<i16>,
}

/// One body a ray walk crosses: cast resolves it at walk step `step` (0x10e22a),
/// before that step's sprite and delay (0x10e58c). `lead` events of the crossing
/// precede the result carrying it and `tail` follow it; all of them happen at
/// that step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RayCrossing {
    pub step: i16,
    pub lead: u16,
    pub tail: u16,
}

/// showspellinfo 0x159192 in window 128, which combat calls on an NPC's turn
/// before resolvespell (0x11495e, 0x114a4c, 0x114b50) or cast (0x11509e). It
/// plots `icon` in rect 0x4436fa (0x15929c..0x1592fa) and, for a packet whose
/// bytes 2 and 3 sum above zero, cicn 14022 in that rectangle's top-left 32
/// square raised 5 (0x159302..0x15936e). A category-9 packet first erases rect
/// 0x4436f2 and plots WEAPON work WORD 0x453772 there (0x1591a0..0x1591e6). It
/// then draws GetIndString(`name`) in yellowColor at (WORD40856c + 369,
/// WORD40856e + 402 - 32) and, after TextMode srcOr, the number in redColor at
/// (WORD40856c + 548, WORD40856e + 386 - 32) (0x159370..0x159462), TextSize 12.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellInfo {
    pub icon: i16,
    pub area: bool,
    /// A category-9 packet, which erases rect 0x4436f2 and plots `weapon_icon`.
    pub firearm: bool,
    pub weapon_icon: Option<i16>,
    /// STR# ID and index; none when the work record it reads is not represented.
    pub name: Option<(i16, i16)>,
    /// WORD4435cc, or for category 9 WORD4435be; none when not represented.
    pub number: Option<i16>,
    /// How many events precede the event carrying it.
    pub lead: u16,
}

impl SpellInfo {
    /// showspellinfo's drawing for `packet`, the spell identity WORDs 0x4435c6..
    /// 0x4435ca and WORD4435cc. A category-9 packet reads WEAPON and ITEM work.
    pub fn new(
        packet: &[u8],
        identity: [i16; 3],
        count: i16,
        number: Option<i16>,
        (weapon, item): (Option<&[u8]>, Option<&[u8]>),
    ) -> Self {
        let byte = |offset: usize| packet.get(offset).copied().unwrap_or(0);
        let word = |record: &[u8], offset: usize| {
            record
                .get(offset..offset + 2)
                .map(|word| i16::from_be_bytes([word[0], word[1]]))
        };
        // 0x159202..0x159282: the jump on BYTE44374f, then WORD443776 == 0x3a.
        let detail = i16::from(byte(0x1f));
        let mut icon = match byte(1) {
            3 => detail + 9999,
            4 => count.wrapping_add(9999),
            5 if detail == 0 => 10215,
            5 => detail + 9999,
            6 => 10213,
            7 => 10214,
            9 => 10211,
            10 => 10210,
            11 => 10212,
            _ => 10000,
        };
        if packet.get(0x28..0x2a) == Some(&[0, 0x3a]) {
            icon = detail + 10013;
        }
        let firearm = byte(19) == 9;
        let name = if firearm {
            // getselection 0x125a08 stores the base in WORD4471e8 before the index reads it.
            weapon.and_then(|weapon| word(weapon, 0x66)).and_then(|id| {
                let base = crate::rules::selection_base(id);
                item.and_then(|item| word(item, 0x66)).map(|item| {
                    (
                        base.wrapping_add(1),
                        item.wrapping_sub(base).wrapping_add(1),
                    )
                })
            })
        } else {
            Some((
                identity[0]
                    .wrapping_add(1)
                    .wrapping_mul(1000)
                    .wrapping_add(identity[1]),
                identity[2].wrapping_add(1),
            ))
        };
        Self {
            icon,
            area: i16::from(byte(2) as i8) + i16::from(byte(3) as i8) > 0,
            firearm,
            weapon_icon: weapon
                .and_then(|weapon| word(weapon, 10))
                .filter(|_| firearm),
            name,
            number: if firearm { number } else { Some(count) },
            lead: 0,
        }
    }

    /// Puts the drawing on the first result or launch at or after `start`.
    pub(crate) fn attach(events: &mut [Event], start: usize, info: SpellInfo) {
        for (lead, event) in events.iter_mut().skip(start).enumerate() {
            if let Event::AttackResolved { flight, .. }
            | Event::SpellResolved { flight, .. }
            | Event::Launch { flight, .. } = event
            {
                flight.3 = Some(SpellInfo {
                    lead: u16::try_from(lead).unwrap_or(u16::MAX),
                    ..info
                });
                return;
            }
        }
    }
}

/// resolvespell's state at 0x1517c4, before the body loop: preference 10 sends a
/// mode 3, 4 or above-8 packet through spelleffect's area path 0x15b5e0, which
/// plays the impact sound WORD443774 (0x151808) and animates every body of mask
/// 0x4434e0 at once; otherwise each presented target flashes in turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellArea {
    /// The nonzero WORDs of mask 0x4434e0, bit n for body n (0x15b670).
    pub bodies: u128,
    /// Packet byte 1 (BYTE44374f), compared as a signed BYTE at 0x1517d2..0x1517ee.
    pub mode: i8,
    /// Packet WORD 0x26 (WORD443774).
    pub sound: i16,
    /// Packet byte 18 (BYTE443760) after the body loop, spelleffect's cicn base.
    pub effect: i8,
    /// How many events of the body loop precede the result carrying the area.
    pub lead: u16,
    /// Set on the loop's first result only; every result of the loop carries the
    /// area, which marks it as a resolvespell result rather than attack2's.
    pub opens: bool,
}

impl SpellArea {
    /// Puts the area of a body loop whose events run from `start` to the end on
    /// each of its results, opening at the first.
    pub(crate) fn attach(events: &mut [Event], start: usize, mask: &[i16; 110], packet: &[u8]) {
        let bodies = mask
            .iter()
            .enumerate()
            .filter(|(_, flag)| **flag != 0)
            .fold(0u128, |bodies, (body, _)| bodies | 1 << body);
        let area = |lead: usize, opens: bool| SpellArea {
            bodies,
            mode: packet.get(1).map_or(0, |mode| *mode as i8),
            sound: packet
                .get(0x26..0x28)
                .map_or(0, |word| i16::from_be_bytes([word[0], word[1]])),
            effect: packet.get(18).map_or(0, |effect| *effect as i8),
            lead: u16::try_from(lead).unwrap_or(u16::MAX),
            opens,
        };
        let mut opens = true;
        for (lead, event) in events.iter_mut().skip(start).enumerate() {
            if let Event::AttackResolved { flight, .. } | Event::SpellResolved { flight, .. } =
                event
            {
                flight.1 = Some(area(lead, std::mem::take(&mut opens)));
            }
        }
    }

    /// resolvespell's area for a body loop over one target (0x151022 from the
    /// NPC firearm caller), carried on that target's result.
    pub(crate) fn single(target: i16, packet: &[u8]) -> PresentationFlight {
        let mut mask = [0; 110];
        if let Some(flag) = usize::try_from(target).ok().and_then(|t| mask.get_mut(t)) {
            *flag = 1;
        }
        let mut events = [Event::AttackResolved {
            attacker: 0,
            target,
            damage: 0,
            defeated: false,
            icon: PresentationIcon(0),
            flight: PresentationFlight::default(),
        }];
        Self::attach(&mut events, 0, &mask, packet);
        match events {
            [Event::AttackResolved { flight, .. }] => flight,
            _ => PresentationFlight::default(),
        }
    }
}

impl Flight {
    /// Puts each flight, recorded with the event count at its launch sound, on the
    /// first result presented before the next launch. A flight without a result is
    /// not shown.
    pub(crate) fn attach(events: &mut [Event], flights: &[(usize, Option<Flight>)]) {
        for (index, (start, flight)) in flights.iter().enumerate() {
            let end = flights.get(index + 1).map_or(events.len(), |next| next.0);
            let found = events.get_mut(*start..end).and_then(|events| {
                events.iter_mut().find_map(|event| match event {
                    Event::AttackResolved { flight, .. } | Event::SpellResolved { flight, .. } => {
                        Some(flight)
                    }
                    _ => None,
                })
            });
            if let Some(slot) = found {
                slot.0 = *flight;
            } else if let Some(event) = events.get_mut(*start)
                && let Event::Sound { resource_id } = *event
                && flight.is_some()
            {
                // A ray that walks nothing, a duration or global cast and a mask
                // without a presented body carry their flight on the launch sound.
                *event = Event::Launch {
                    resource_id,
                    flight: PresentationFlight(*flight, None, None, None),
                };
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// cast's launch sound 0x10db1a for a flight that presents no result. It
    /// serializes and compares as `Sound`; the flight is presentation data only.
    #[serde(rename = "Sound", skip_deserializing)]
    Launch {
        resource_id: i16,
        #[serde(skip)]
        flight: PresentationFlight,
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
        /// The showdamage icon for this hit: WORD 0x41f710 after resolvedamage,
        /// which showresults 0x159124 draws over the target; 0 without damage.
        #[serde(skip)]
        icon: PresentationIcon,
        #[serde(skip)]
        flight: PresentationFlight,
    },
    SpellResolved {
        caster: Option<i16>,
        spell: i16,
        target: i16,
        damage: i16,
        defeated: bool,
        /// WORD 0x41f710 for resolvespell's code-0 showresults 0x151dba; 0 when the
        /// caller does not resolve damage through a selected-body result.
        #[serde(skip)]
        icon: PresentationIcon,
        #[serde(skip)]
        flight: PresentationFlight,
    },
    CombatantAdded {
        body: i16,
        definition: i16,
    },
    /// Effect58 placed a copy of MONSTER work, which has no definition number.
    CombatantSummoned {
        body: i16,
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

impl Event {
    /// The resource a `Sound` or a launch `Launch` plays.
    pub fn sound(&self) -> Option<i16> {
        match self {
            Event::Sound { resource_id } | Event::Launch { resource_id, .. } => Some(*resource_id),
            _ => None,
        }
    }
}

/// Events compare as they serialize: presentation fields are skipped and a
/// `Launch` is its `Sound`.
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        match (self.sound(), other.sound()) {
            (Some(a), Some(b)) => a == b,
            (None, None) => serde_json::to_value(self).ok() == serde_json::to_value(other).ok(),
            _ => false,
        }
    }
}

impl Eq for Event {}
