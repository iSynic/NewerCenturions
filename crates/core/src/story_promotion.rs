use crate::{Content, Event, Presentation, RuleError, Session, story_inventory};
mod scans;
use centurions_content::{
    Warrior,
    records::{
        AttackPacket, DevelopmentField as Field, LearnedScans, ProfessionProfile,
        PsiSelectionMemoryMut, RaceProfile,
    },
};
use scans::keep_spells;
pub use scans::{SCAN_COSTS, ScanSelection};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryRandom {
    pub seed: u32,
    pub draws: u64,
}
impl LibraryRandom {
    pub fn next_value(&mut self) -> i16 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        self.draws += 1;
        ((self.seed >> 16) & 32767) as i16
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Promotion {
    pub warriors: Vec<Warrior>,
    pub promoted: Vec<usize>,
    pub spell_confirmation: Vec<usize>,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
}

pub fn experience_threshold(level: i16) -> Result<i32, RuleError> {
    let mut running = 714.0_f32;
    let mut total = 0_i32;
    for index in 0..level {
        let factor = ((1.45_f64 - 0.01 * f64::from(index)) as f32).max(1.15_f32);
        // Past level 64 the SINGLE product exceeds a LONG and later becomes infinite.
        running = (f64::from(running) * f64::from(factor)) as f32;
        total = total.wrapping_add(crate::rules::sane_long(f64::from(running)));
    }
    Ok(total)
}

/// levelup 0x12ed42: editor mode leaves experience and existing scans unchanged.
pub fn editor_level_up(
    warrior: &mut Warrior,
    races: &[u8],
    profiles: &[u8],
    random: &mut LibraryRandom,
) -> Result<bool, RuleError> {
    let mut candidate = warrior.clone();
    let level = candidate.record().level()?.wrapping_add(1);
    Field::Level.write(&mut candidate.bytes, level);
    let bonus = advance_statistics(&mut candidate, races, profiles)?;
    random.next_value();
    *warrior = candidate;
    Ok(bonus)
}

fn advance_statistics(
    warrior: &mut Warrior,
    races: &[u8],
    profiles: &[u8],
) -> Result<bool, RuleError> {
    let bytes = &mut warrior.bytes;
    let record = centurions_content::records::WarriorRecord::from_bytes(bytes);
    let level = record.level()?;
    let race_index = record.race()?;
    let profile_index = record.profession()?;
    if race_index < 1 || profile_index < 1 {
        return Err(RuleError::Unsupported {
            action: "story.promotion",
            dependency: "zero profile retains external service context",
        });
    }
    let race = RaceProfile::from_table(races, race_index)?;
    let profile = ProfessionProfile::from_table(profiles, profile_index)?;
    let mut points = race
        .action_points()?
        .wrapping_add(profile.action_points()?)
        .wrapping_add(2);
    let mut bonus = false;
    for unlock in profile.action_unlock_levels() {
        if *unlock != 0 && i16::from(*unlock as i8) <= level {
            bonus |= i16::from(*unlock as i8) == level;
            points = points.wrapping_add(1);
        }
    }
    for index in 0..40 {
        if profile.condition_level(index)? == level {
            let value = centurions_content::records::WarriorRecord::from_bytes(bytes)
                .condition_duration(index)?;
            centurions_content::records::WarriorRecordMut::from_bytes(bytes)
                .set_condition_duration(
                    index,
                    if value > -1 {
                        -1
                    } else {
                        value.wrapping_sub(1)
                    },
                );
        }
    }
    let cap = race.action_limit()?.wrapping_mul(2);
    if points > cap {
        points = cap;
    }
    Field::ActionsPerRound.write(bytes, points);
    crate::creation::clamp_development_fields(bytes)?;
    Ok(bonus)
}

/// Ordinary Close with no new spell selections; native Done/Yes preserves valid selections.
pub fn close(
    warriors: &[Warrior],
    races: &[u8],
    profiles: &[u8],
    attacks: &[u8],
    registered: bool,
    library_random: &mut LibraryRandom,
) -> Result<Promotion, RuleError> {
    close_with_preparation(
        warriors,
        races,
        profiles,
        attacks,
        registered,
        library_random,
        true,
    )
}

pub(crate) fn close_recovery(
    warriors: &[Warrior],
    races: &[u8],
    profiles: &[u8],
    attacks: &[u8],
    registered: bool,
    library_random: &mut LibraryRandom,
) -> Result<Promotion, RuleError> {
    close_with_preparation(
        warriors,
        races,
        profiles,
        attacks,
        registered,
        library_random,
        false,
    )
}

fn close_with_preparation(
    warriors: &[Warrior],
    races: &[u8],
    profiles: &[u8],
    attacks: &[u8],
    registered: bool,
    library_random: &mut LibraryRandom,
    normalize: bool,
) -> Result<Promotion, RuleError> {
    if !(1..=6).contains(&warriors.len()) {
        return Err(RuleError::Invalid(
            "promotion requires one to six warrior records".into(),
        ));
    }
    let mut warriors = if normalize {
        story_inventory::normalize_story_loot(warriors)?
    } else {
        warriors.to_vec()
    };
    let mut next_library = library_random.clone();
    let mut promoted = vec![];
    let mut spell_confirmation = vec![];
    let mut sounds = vec![];
    let mut warnings = vec![];
    for (member, warrior) in warriors.iter_mut().enumerate() {
        let result = promote_member(warrior, races, profiles, registered, &mut next_library)?;
        sounds.extend(result.sounds);
        warnings.extend(result.warnings);
        if result.promoted {
            promoted.push(member);
        }
        if result.select_spells {
            spell_confirmation.push(member);
        }
    }
    for &member in &spell_confirmation {
        keep_spells(&mut warriors[member], attacks)?;
    }
    for warrior in &mut warriors {
        warrior.record_mut().set_active_flag(1);
    }
    *library_random = next_library;
    Ok(Promotion {
        warriors,
        promoted,
        spell_confirmation,
        sounds,
        warnings,
    })
}

struct MemberPromotion {
    promoted: bool,
    select_spells: bool,
    sounds: Vec<i16>,
    warnings: Vec<i16>,
}

fn promote_member(
    warrior: &mut Warrior,
    races: &[u8],
    profiles: &[u8],
    registered: bool,
    library: &mut LibraryRandom,
) -> Result<MemberPromotion, RuleError> {
    let mut result = MemberPromotion {
        promoted: false,
        select_spells: false,
        sounds: vec![],
        warnings: vec![],
    };
    if warrior.stamina()? <= 0 || warrior.record().remaining_experience()? >= 0 {
        return Ok(result);
    }
    let level = warrior.record().level()?;
    let remaining = warrior
        .record()
        .remaining_experience()?
        .wrapping_add(experience_threshold(level)?);
    warrior.record_mut().set_remaining_experience(remaining);
    result.sounds.push(3002);
    if level >= 7 && !registered {
        result.warnings.push(40);
        return Ok(result);
    }
    Field::Level.write(&mut warrior.bytes, level.wrapping_add(1));
    if library.next_value() < 15 && !registered {
        result.warnings.push(452);
    }
    advance_statistics(warrior, races, profiles)?;
    result.promoted = true;
    result.select_spells =
        warrior.record().maximum_psi()? != 0 && warrior.record().psi_school()? != 0;
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    pub member: usize,
    pub selection: ScanSelection,
    pub question: bool,
    pub description: Option<(usize, usize)>,
    pub warning: bool,
}

impl Progress {
    pub fn validate(&self, state: &Session, content: &Content) -> Result<(), RuleError> {
        if !state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation == Some(Presentation::Loot))
            || state.booty_items.iter().any(|item| item.id() != 0)
            || self
                .description
                .is_some_and(|(row, column)| row >= 7 || column >= 7)
            || (self.question && self.warning)
        {
            return Err(RuleError::Invalid("invalid promotion continuation".into()));
        }
        let warrior = state
            .warriors
            .get(self.member)
            .ok_or_else(|| RuleError::Invalid("promotion member outside squad".into()))?;
        let map = crate::exploration::active_map(state, content)?;
        let mut expected = ScanSelection::new(warrior, &map.profiles)?;
        for adding in [false, true] {
            for row in 0..7 {
                for column in 0..7 {
                    let actual = LearnedScans(&self.selection.warrior.bytes)
                        .scan_marker(row, column)
                        .unwrap_or(255);
                    if actual > 1 {
                        return Err(RuleError::Invalid("invalid learned scan marker".into()));
                    }
                    // ScanSelection::new has already indexed every expected marker.
                    if (actual != 0) == adding
                        && LearnedScans(&expected.warrior.bytes).scan_marker(row, column)
                            != Some(actual)
                    {
                        expected.select_tier(row)?;
                        if expected.toggle(column)?.is_none() {
                            return Err(RuleError::Invalid(
                                "promotion selection exceeds budget".into(),
                            ));
                        }
                    }
                }
            }
        }
        expected.select_tier(self.selection.tier)?;
        if expected != self.selection {
            return Err(RuleError::Invalid(
                "promotion selection differs from its warrior".into(),
            ));
        }
        Ok(())
    }
}

pub fn begin(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.promotion.is_some()
        || !state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation == Some(Presentation::Loot))
        || state.booty_items.iter().any(|item| item.id() != 0)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if state.recovery_return.is_none() {
        state.warriors = story_inventory::normalize_story_loot(&state.warriors)?;
    }
    let mut events = vec![Event::Sound { resource_id: 20 }];
    events.extend(advance(state, content, 0)?);
    Ok(events)
}

fn advance(state: &mut Session, content: &Content, first: usize) -> Result<Vec<Event>, RuleError> {
    let map = crate::exploration::active_map(state, content)?;
    let mut library = LibraryRandom {
        seed: state.library_random_seed,
        draws: state.library_random_draws,
    };
    let mut events = vec![];
    for member in first..state.warriors.len() {
        let result = promote_member(
            &mut state.warriors[member],
            &map.races,
            &map.profiles,
            true,
            &mut library,
        )?;
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        events.extend(
            result
                .warnings
                .into_iter()
                .map(|warning| Event::CollectionRejected { warning }),
        );
        if result.promoted {
            state.native_work.character_bare_hand_bonus =
                state.warriors[member].record().bare_hand_bonus()?;
            events.push(Event::LevelAdvanced {
                warrior: member,
                level: state.warriors[member].record().level()?,
            });
        }
        if result.select_spells {
            state.promotion = Some(Progress {
                member,
                selection: ScanSelection::new(&state.warriors[member], &map.profiles)?,
                question: false,
                description: None,
                warning: false,
            });
            events.push(Event::Sound { resource_id: 3000 });
            state.library_random_seed = library.seed;
            state.library_random_draws = library.draws;
            return Ok(events);
        }
    }
    state.library_random_seed = library.seed;
    state.library_random_draws = library.draws;
    for warrior in &mut state.warriors {
        warrior.record_mut().set_active_flag(1);
    }
    events.extend(crate::exploration::finish_story_page(state, content)?);
    Ok(events)
}

pub fn control(state: &mut Session, content: &Content, item: u16) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .promotion
        .as_mut()
        .ok_or(RuleError::Refused { feedback: vec![] })?;
    if pending.warning {
        pending.warning = false;
        return Ok(vec![]);
    }
    if pending.question {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    match item {
        1 | 32 => {
            if pending.selection.remaining != 0 {
                pending.question = true;
                Ok(vec![])
            } else {
                finish(state, content)
            }
        }
        2..=8 => {
            if pending
                .selection
                .select_tier(usize::from(item - 2))
                .is_err()
            {
                pending.warning = true;
                Ok(vec![])
            } else {
                Ok(vec![Event::Sound { resource_id: 137 }])
            }
        }
        10..=16 => {
            let column = usize::from(item - 10);
            pending.description = Some((pending.selection.tier, column));
            Ok(pending
                .selection
                .toggle(column)?
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id })
                .collect())
        }
        _ => Err(RuleError::Refused { feedback: vec![] }),
    }
}

pub fn answer(state: &mut Session, content: &Content, yes: bool) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .promotion
        .as_mut()
        .ok_or(RuleError::Refused { feedback: vec![] })?;
    if !pending.question {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    pending.question = false;
    if yes {
        finish(state, content)
    } else {
        Ok(vec![])
    }
}

fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state.promotion.take().unwrap();
    let map = crate::exploration::active_map(state, content)?;
    let attacks = &map
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("promotion scan records unavailable".into()))?
        .attacks;
    state.warriors[pending.member] = pending.selection.finish(attacks)?;
    cache_scans(state, pending.member, attacks)?;
    state.native_work.character_bare_hand_bonus =
        state.warriors[pending.member].record().bare_hand_bonus()?;
    advance(state, content, pending.member + 1)
}

pub(crate) fn cache_scans(
    state: &mut Session,
    member: usize,
    attacks: &[u8],
) -> Result<(), RuleError> {
    let record = &state.warriors[member];
    PsiSelectionMemoryMut(&mut state.native_work.psi_memory)
        .store_shortcuts(member, LearnedScans(&record.bytes).shortcuts());
    if let Some(index) = LearnedScans(&record.bytes).last_selected() {
        let school = record.record().psi_school()? as usize - 1;
        state.native_work.attack_packet =
            AttackPacket::scan_entry(attacks, school, index)?.0.to_vec();
    }
    Ok(())
}
