use crate::RuleError;
use centurions_content::WARRIOR_BYTES;
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::{
    Warrior,
    mission::OpeningMission,
    records::{Condition, EnemyRecordMut, StoryRecord},
    span, word,
};

use super::Combat;

/// What a scenario's death story did: the text it shows and whether the battle ended at once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionDeath {
    pub text: i16,
    pub ended_early: bool,
}

/// The death story that is running, the monster that ran it and the side that monster was on when
/// it was queued.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeathContext {
    pub script: i16,
    pub actor: usize,
    pub deferred_faction: Option<i16>,
}

/// Reports a death story this build cannot yet run, without changing anything.
fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.mission_death",
        dependency,
    }
}

/// The scenario's death stories 336 and 337, after killbody's cleanup; queued
/// stories run as actor zero.
pub fn apply(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &OpeningMission,
    extras: &[u8],
    context: DeathContext,
    kill_party: &mut i8,
) -> Result<MissionDeath, RuleError> {
    let DeathContext {
        script,
        actor,
        deferred_faction,
    } = context;
    if combat.roster.records.len() > 100
        || combat
            .roster
            .records
            .iter()
            .any(|record| record.bytes.len() != MONSTER_BYTES)
        || actor >= combat.roster.records.len()
        || !(1..=6).contains(&warriors.len())
        || warriors
            .iter()
            .any(|warrior| warrior.bytes.len() != WARRIOR_BYTES)
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
        let actual = StoryRecord(record).instruction(index)?;
        if actual != expected.get(index).copied().unwrap_or((0, 0)) {
            return Err(unsupported(
                "mission death script differs from verified actions",
            ));
        }
    }
    let mut next = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut next_unavailable = *kill_party;
    let ended_early = if script == 336 {
        let extra = span(extras, 619 * 10, 10)?;
        for (index, value) in [113, 114, 115, 0, 0].into_iter().enumerate() {
            if word(extra, index * 2)? != value {
                return Err(unsupported(
                    "mission death extra differs from verified fields",
                ));
            }
        }
        let faction = deferred_faction.unwrap_or(next.roster.records[actor].record().faction()?);
        for class in [113, 114, 115] {
            for record in &mut next.roster.records {
                if i16::from(record.record().monster_id()) == class
                    && record.record().faction()? == faction
                {
                    record.record_mut().set_condition(Condition::Retreating, -1);
                    record.record_mut().set_surrender_percent(70);
                }
            }
        }
        false
    } else if 5 < i16::from(next_unavailable) {
        for warrior in &mut next_warriors {
            warrior.record_mut().set_stamina(1);
            warrior.record_mut().set_computer_control(0);
            next_unavailable = 0;
        }
        true
    } else {
        let record = &mut next.roster.records[actor].bytes;
        EnemyRecordMut::from_bytes(record).set_stamina(1);
        EnemyRecordMut::from_bytes(record).set_faction(0);
        EnemyRecordMut::from_bytes(record).set_target_scan_excluded(true);
        false
    };
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *kill_party = next_unavailable;
    Ok(MissionDeath {
        text: expected[0].1,
        ended_early,
    })
}
