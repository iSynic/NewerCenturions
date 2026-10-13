use crate::{RuleError, story_inventory};
use centurions_content::{Warrior, mission::long, span, word};

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
    if level > 64 {
        return Err(RuleError::Unsupported {
            action: "story.promotion",
            dependency: "high-level SANE experience conversion",
        });
    }
    let mut running = 714.0_f32;
    let mut total = 0_i32;
    for index in 0..level {
        let factor = ((1.45_f64 - 0.01 * f64::from(index)) as f32).max(1.15_f32);
        running = (f64::from(running) * f64::from(factor)) as f32;
        if f64::from(running) > f64::from(i32::MAX) {
            return Err(RuleError::Unsupported {
                action: "story.promotion",
                dependency: "SANE threshold overflow",
            });
        }
        total = total.wrapping_add(running as i32);
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
    let level = word(&candidate.bytes, 0x3a)?.wrapping_add(1);
    put_word(&mut candidate.bytes, 0x3a, level);
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
    let level = word(bytes, 0x3a)?;
    let race_index = word(bytes, 0x32)?;
    let profile_index = word(bytes, 0x34)?;
    if race_index < 1 || profile_index < 1 {
        return Err(RuleError::Unsupported {
            action: "story.promotion",
            dependency: "zero profile retains external service context",
        });
    }
    let race = span(races, (race_index as usize - 1) * 408, 408)?;
    let profile = span(profiles, (profile_index as usize - 1) * 576, 576)?;
    let mut points = word(race, 204)?
        .wrapping_add(word(profile, 266)?)
        .wrapping_add(2);
    let mut bonus = false;
    for unlock in &profile[432..442] {
        if *unlock != 0 && i16::from(*unlock as i8) <= level {
            bonus |= i16::from(*unlock as i8) == level;
            points = points.wrapping_add(1);
        }
    }
    for index in 0..40 {
        if word(profile, 138 + index * 2)? == level {
            let offset = 0x5e + index * 2;
            let value = word(bytes, offset)?;
            put_word(
                bytes,
                offset,
                if value > -1 {
                    -1
                } else {
                    value.wrapping_sub(1)
                },
            );
        }
    }
    let cap = word(race, 206)?.wrapping_mul(2);
    if points > cap {
        points = cap;
    }
    put_word(bytes, 0x12, points);
    for index in 0..12 {
        let offset = 0xf4 + index * 2;
        let value = word(bytes, offset)?.clamp(-99, 999);
        put_word(bytes, offset, value);
    }
    for offset in [4, 6, 8, 10, 0x22, 12, 14] {
        let value = word(bytes, offset)?.clamp(0, 999);
        put_word(bytes, offset, value);
    }
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
    if !(1..=6).contains(&warriors.len()) {
        return Err(RuleError::Invalid(
            "promotion requires one to six warrior records".into(),
        ));
    }
    let mut warriors = story_inventory::normalize_story_loot(warriors)?;
    let mut next_library = library_random.clone();
    let mut promoted = vec![];
    let mut spell_confirmation = vec![];
    let mut sounds = vec![];
    let mut warnings = vec![];
    for (member, warrior) in warriors.iter_mut().enumerate() {
        if warrior.stamina()? <= 0 || long(&warrior.bytes, 0x9ce)? >= 0 {
            continue;
        }
        let level = word(&warrior.bytes, 0x3a)?;
        let remaining = long(&warrior.bytes, 0x9ce)?.wrapping_add(experience_threshold(level)?);
        warrior.bytes[0x9ce..0x9d2].copy_from_slice(&remaining.to_be_bytes());
        sounds.push(3002);
        if level >= 7 && !registered {
            warnings.push(40);
            continue;
        }
        let bytes = &mut warrior.bytes;
        let level = level.wrapping_add(1);
        put_word(bytes, 0x3a, level);
        let library_value = next_library.next_value();
        if library_value < 15 && !registered {
            warnings.push(452);
        }
        advance_statistics(warrior, races, profiles)?;
        let bytes = &warrior.bytes;
        promoted.push(member);
        if word(bytes, 0x5a)? != 0 && word(bytes, 0x36)? != 0 {
            spell_confirmation.push(member);
        }
    }
    for &member in &spell_confirmation {
        keep_spells(&mut warriors[member], attacks)?;
    }
    for warrior in &mut warriors {
        warrior.bytes[0x9df] = 1;
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

pub const SCAN_COSTS: [i16; 7] = [5, 7, 10, 15, 25, 40, 60];

/// scanselect 0x15a9cc: normalized selection draft and original signed-WORD costs.
pub struct ScanSelection {
    pub warrior: Warrior,
    pub tier: usize,
    pub remaining: i16,
    highest_tier: i16,
}

impl ScanSelection {
    pub fn new(warrior: &Warrior, profiles: &[u8]) -> Result<Self, RuleError> {
        if word(&warrior.bytes, 0xac6)? == 0 || !(1..=4).contains(&word(&warrior.bytes, 0x36)?) {
            return Err(RuleError::Unsupported {
                action: "editor.scans",
                dependency: "warrior cannot perform scans",
            });
        }
        let profile = word(&warrior.bytes, 0x34)?;
        if profile < 1 {
            return Err(RuleError::Invalid("scan MOS profile is unavailable".into()));
        }
        let profile = span(profiles, (profile as usize - 1) * 576, 576)?;
        let highest_tier = word(profile, 90)?
            .wrapping_add(word(profile, 96)?)
            .wrapping_add(word(profile, 102)?);
        let mut warrior = warrior.clone();
        let remaining = normalize_scans(&mut warrior)?;
        Ok(Self {
            warrior,
            tier: 0,
            remaining,
            highest_tier,
        })
    }

    pub fn select_tier(&mut self, tier: usize) -> Result<(), RuleError> {
        if tier >= 7 || tier as i16 + 1 > self.highest_tier {
            return Err(RuleError::Unsupported {
                action: "editor.scans",
                dependency: "selected tier exceeds the warrior's MOS scan limit",
            });
        }
        self.tier = tier;
        Ok(())
    }

    pub fn toggle(&mut self, column: usize) -> Result<Option<i16>, RuleError> {
        if column >= 7 {
            return Err(RuleError::Invalid(
                "scan column outside original matrix".into(),
            ));
        }
        let cost = SCAN_COSTS[self.tier];
        let bytes = &mut self.warrior.bytes;
        let selected = bytes[0x9f0 + self.tier * 7 + column] != 0;
        if !selected && self.remaining < cost {
            return Ok(None);
        }
        let offset = 0x42 + self.tier * 2;
        let count = word(bytes, offset)?;
        bytes[0x9f0 + self.tier * 7 + column] = u8::from(!selected);
        self.remaining = self
            .remaining
            .wrapping_add(if selected { cost } else { -cost });
        put_word(
            bytes,
            offset,
            count.wrapping_add(if selected { 1 } else { -1 }),
        );
        Ok(Some(if selected { 144 } else { 145 }))
    }

    pub fn finish(mut self, attacks: &[u8]) -> Result<Warrior, RuleError> {
        finish_scans(&mut self.warrior, attacks)?;
        Ok(self.warrior)
    }
}

fn keep_spells(warrior: &mut Warrior, attacks: &[u8]) -> Result<(), RuleError> {
    normalize_scans(warrior)?;
    finish_scans(warrior, attacks)
}

fn normalize_scans(warrior: &mut Warrior) -> Result<i16, RuleError> {
    let bytes = &mut warrior.bytes;
    let level = word(bytes, 0x3a)?;
    let mut available = level.wrapping_mul(level).wrapping_add(16);
    for (row, cost) in SCAN_COSTS.into_iter().enumerate() {
        for column in 0..7 {
            let index = 0x9f0 + row * 7 + column;
            if bytes[index] != 0 {
                let next = available.wrapping_sub(cost);
                if next > -1 {
                    available = next;
                } else {
                    bytes[index] = 0;
                }
            }
        }
    }
    Ok(available)
}

fn finish_scans(warrior: &mut Warrior, attacks: &[u8]) -> Result<(), RuleError> {
    let bytes = &mut warrior.bytes;
    bytes[0x9dc] = 0;
    let school = word(bytes, 0x36)? - 1;
    if !(0..4).contains(&school) {
        return Err(RuleError::Unsupported {
            action: "story.promotion",
            dependency: "spell school outside loaded bands",
        });
    }
    for row in 0..7 {
        for column in 0..7 {
            if bytes[0x9f0 + row * 7 + column] != 0 {
                let spell = span(
                    attacks,
                    school as usize * 2842 + row * 406 + column * 58,
                    58,
                )?;
                if word(spell, 40)? == 57 {
                    bytes[0x9dc] = 1;
                }
            }
        }
    }
    for index in 0..10 {
        let offset = 0xa76 + index * 8;
        let row = word(bytes, offset + 2)?;
        let column = word(bytes, offset + 4)?;
        if !(0..7).contains(&row) || !(0..7).contains(&column) {
            return Err(RuleError::Unsupported {
                action: "story.promotion",
                dependency: "spell shortcut outside original selection matrix",
            });
        }
        if bytes[0x9f0 + row as usize * 7 + column as usize] == 0 {
            bytes[offset..offset + 8].fill(0);
        }
    }
    Ok(())
}

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}
