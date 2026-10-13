use crate::RuleError;
use centurions_content::{Warrior, mission::OpeningMission, span, word};

use super::Combat;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionDeath {
    pub text: i16,
    pub ended_early: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeathContext {
    pub script: i16,
    pub actor: usize,
    pub deferred_faction: Option<i16>,
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.mission_death",
        dependency,
    }
}

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

/// Authored death336/337 after killbody cleanup; queued stories retain global actor zero.
pub fn apply(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    extras: &[u8],
    context: DeathContext,
    unavailable: &mut i8,
) -> Result<MissionDeath, RuleError> {
    let DeathContext {
        script,
        actor,
        deferred_faction,
    } = context;
    if combat.enemies.records.len() > 100
        || combat
            .enemies
            .records
            .iter()
            .any(|record| record.bytes.len() != 288)
        || actor >= combat.enemies.records.len()
        || !(1..=6).contains(&warriors.len())
        || warriors.iter().any(|warrior| warrior.bytes.len() != 3138)
    {
        return Err(RuleError::Invalid(
            "invalid mission death story input".into(),
        ));
    }
    let expected: &[(i16, i16)] = match script {
        336 => &[(1, 1247), (0, 0), (123, 619)],
        337 => &[(1, 1248), (119, 0)],
        _ => return Err(unsupported("unverified mission death script")),
    };
    let record = span(&mission.door_scripts, script as usize * 42, 42)?;
    for index in 0..8 {
        let actual = (word(record, 8 + index * 2)?, word(record, 24 + index * 2)?);
        if actual != expected.get(index).copied().unwrap_or((0, 0)) {
            return Err(unsupported(
                "mission death script differs from verified actions",
            ));
        }
    }
    let mut next = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut next_unavailable = *unavailable;
    let ended_early = if script == 336 {
        let extra = span(extras, 619 * 10, 10)?;
        for (index, value) in [113, 114, 115, 0, 0].into_iter().enumerate() {
            if word(extra, index * 2)? != value {
                return Err(unsupported(
                    "mission death extra differs from verified fields",
                ));
            }
        }
        let faction = deferred_faction.unwrap_or(word(&next.enemies.records[actor].bytes, 0xe)?);
        for class in [113, 114, 115] {
            for record in &mut next.enemies.records {
                if i16::from(record.bytes[0x103] as i8) == class
                    && word(&record.bytes, 0xe)? == faction
                {
                    record.bytes[0xb2] = 255;
                    put_word(&mut record.bytes, 0x46, 70);
                }
            }
        }
        false
    } else if 5 < i16::from(next_unavailable) {
        for warrior in &mut next_warriors {
            put_word(&mut warrior.bytes, 0x50, 1);
            put_word(&mut warrior.bytes, 0x90, 0);
            next_unavailable = 0;
        }
        true
    } else {
        let record = &mut next.enemies.records[actor].bytes;
        put_word(record, 0x110, 1);
        put_word(record, 0xe, 0);
        record[0x82] = 1;
        false
    };
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *unavailable = next_unavailable;
    Ok(MissionDeath {
        text: expected[0].1,
        ended_early,
    })
}
