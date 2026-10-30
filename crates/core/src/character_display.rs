//! Character dialog fields, calculated directly from saved records. Control IDs
//! and font sizes follow the original dialog.
use crate::{RuleError, Session};
use centurions_content::{
    game_text::GameText,
    records::{CharacterInformation, CharacterStatistics, Statistic},
};
use std::collections::BTreeMap;

/// Character dialog fields by control id: the text and its font size.
pub type Fields = BTreeMap<i16, (String, u16)>;

/// The detailed record page of the character view.
///
/// Original: viewcharacter
pub fn statistics(state: &Session, member: usize) -> Result<Fields, RuleError> {
    let warrior = state
        .warriors
        .get(member)
        .ok_or_else(|| RuleError::Invalid("statistics warrior is outside the squad".into()))?;
    let mut fields = Fields::new();
    let statistics = CharacterStatistics(&warrior.bytes);
    let value = |statistic| statistics.value(statistic);
    let mut score =
        value(Statistic::DamageDealt)?.wrapping_sub(value(Statistic::DamageReceived)?) / 20;
    for (statistic, multiplier) in [
        (Statistic::Hits, 1_i32),
        (Statistic::HitsReceived, -2),
        (Statistic::Misses, -2),
        (Statistic::AttacksAvoided, 1),
        (Statistic::Kills, 3),
        (Statistic::Deaths, -75),
        (Statistic::Knockouts, -35),
        (Statistic::ScansCast, -3),
        (Statistic::VehiclesDisabled, 4),
        (Statistic::VehiclesCaptured, 6),
        (Statistic::ScorePenalty, -1),
    ] {
        score = score.wrapping_add(value(statistic)?.wrapping_mul(multiplier));
    }
    fields.insert(6, (score.to_string(), 12));
    fields.insert(37, (warrior.name.clone(), 12));
    for (item, statistic) in [
        (26, Statistic::DamageDealt),
        (27, Statistic::Hits),
        (42, Statistic::AttacksAvoided),
        (28, Statistic::Kills),
        (30, Statistic::VehiclesDisabled),
        (35, Statistic::VehiclesCaptured),
        (33, Statistic::ScansCast),
        (31, Statistic::DamageReceived),
        (32, Statistic::HitsReceived),
        (43, Statistic::Misses),
        (29, Statistic::Knockouts),
        (34, Statistic::Deaths),
    ] {
        fields.insert(item, (value(statistic)?.to_string(), 12));
    }
    fields.insert(
        71,
        (
            value(Statistic::ScorePenalty)?.wrapping_neg().to_string(),
            12,
        ),
    );
    Ok(fields)
}

/// The character information page: the fields of the Information dialog for one squad member.
///
/// Original: ShowStats
pub fn information(
    state: &Session,
    member: usize,
    game_text: &GameText,
    strings: &BTreeMap<i16, Vec<u8>>,
) -> Result<Fields, RuleError> {
    let warrior = state
        .warriors
        .get(member)
        .ok_or_else(|| RuleError::Invalid("information warrior is outside the squad".into()))?;
    // The page still requires these original text ranges to be present.
    game_text.bytes(0x400af0, 0x50)?;
    game_text.bytes(0x402f3c, 10)?;
    let record = warrior.record();
    let profile = CharacterInformation(&warrior.bytes);
    let mut fields = Fields::new();
    let stats = statistics(state, member)?;
    for (item, source) in [(103, 28), (104, 26), (101, 6)] {
        fields.insert(item, stats[&source].clone());
    }
    for (index, item) in [19, 20, 21, 22, 23, 24, 52].into_iter().enumerate() {
        number(&mut fields, item, profile.attribute(index)?, 12);
    }
    number(&mut fields, 71, profile.age_days()? / 365, 12);
    let mut credits = state.party_currency;
    // The last member is a signed byte, even for a malformed party.
    let last = state.warriors.len().saturating_sub(1) as u8 as i8;
    for index in 0..=i16::from(last) {
        let carried =
            CharacterInformation(&state.warriors[index as usize].bytes).carried_credits()?;
        credits = credits.wrapping_add(i32::from(carried));
    }
    number(
        &mut fields,
        64,
        credits,
        if credits > 9_999_999 { 10 } else { 12 },
    );
    fields.insert(
        45,
        (decode(indexed(strings, 131, record.profession()?)?), 12),
    );
    fields.insert(46, (decode(indexed(strings, 129, record.race()?)?), 12));
    let gender_text = match profile.gender()? {
        1 => 0x400b12,
        2 => 0x400b0a,
        _ => 0x400b04,
    };
    fields.insert(47, (game_text.text(gender_text, true)?, 12));
    for (item, value) in [
        (48, record.normal_attacks()?),
        (49, record.attack_limit()?),
        (25, record.max_spells_attacks()?),
        (113, record.shield_points()?),
        // The original sets size 10 for shield capacity and resets it to 12 at once.
        (106, profile.shield_capacity()?),
    ] {
        number(&mut fields, item, value, 12);
    }
    number(&mut fields, 81, profile.experience()?, 12);
    number(&mut fields, 83, profile.upgrade_points()?, 12);
    for (item, value) in [
        (34, record.shield_chance()?),
        (35, profile.shield_cost()?),
        (36, record.ac()?),
        (32, record.level()?),
        (50, record.load()?),
        (51, record.load_max()?),
    ] {
        number(&mut fields, item, value, 12);
    }
    if record.psi_points_max()? != 0 {
        number(
            &mut fields,
            33,
            crate::rules::spell_count(record.level()?) as i16,
            12,
        );
    } else {
        fields.insert(33, (game_text.text(0x400afe, true)?, 12));
    }
    fields.insert(67, (decode(profile.name_bytes()?), 12));
    for (item, value) in [
        (108, profile.displayed_damage_minimum()?),
        (
            109,
            record
                .bare_hand_damage()?
                .wrapping_add(record.bare_hand_bonus()?),
        ),
        (110, record.missile_accuracy()?),
        (111, record.melee_accuracy()?),
        (54, record.missile_dodge()?),
        (55, record.melee_dodge()?),
    ] {
        number(&mut fields, item, value, 12);
    }
    if record.psi_school()? != 0 && record.psi_points_max()? != 0 {
        let size = if record.psi_points_max()? > 999 {
            10
        } else {
            12
        };
        number(&mut fields, 112, record.psi_points()?, size);
        number(&mut fields, 58, record.psi_points_max()?, size);
    } else {
        for item in [112, 58] {
            fields.insert(item, (game_text.text(0x400afc, true)?, 12));
        }
    }
    let size = if record.stamina_max()? > 999 { 10 } else { 12 };
    for (item, value) in [
        (56, record.stamina()?),
        (57, record.stamina_max()?),
        (59, profile.psionic_resistance()?),
        (26, profile.movement_rate()?),
    ] {
        number(&mut fields, item, value, size);
    }
    for index in 0..8 {
        number(
            &mut fields,
            37 + index as i16,
            profile.damage_resistance(index)?,
            12,
        );
    }
    for item in 60..64 {
        fields.insert(item, (game_text.text(0x400afc, true)?, 12));
    }
    let mut shown = 0;
    for index in 0..12 {
        let bonus = record.special(index)?;
        if bonus == 0 {
            continue;
        }
        let sign = game_text.text(if bonus > 0 { 0x400afa } else { 0x400af9 }, false)?;
        let mut label = sign.into_bytes();
        label.extend(bonus.to_string().bytes().take(3));
        label.extend(indexed(strings, 132, index as i16 + 1)?.iter().take(45));
        if shown < 4 {
            fields.insert(60 + shown, (decode(&label), 12));
            shown += 1;
        }
    }
    for item in 27..32 {
        fields.insert(item, (game_text.text(0x400afc, true)?, 12));
    }
    let mut shown = 0;
    for index in 0..40 {
        if record.condition_duration(index)? != 0 {
            fields.insert(
                27 + shown,
                (decode(indexed(strings, 133, index as i16 + 1)?), 12),
            );
            shown += 1;
            if shown == 5 {
                break;
            }
        }
    }
    Ok(fields)
}

/// Sets a numeric field of the dialog.
fn number(fields: &mut Fields, item: i16, value: impl ToString, size: u16) {
    fields.insert(item, (value.to_string(), size));
}

/// Decodes Mac Roman text from a resource or record into a string.
fn decode(bytes: &[u8]) -> String {
    encoding_rs::MACINTOSH.decode(bytes).0.into_owned()
}

/// Fetches one string from a character STR# resource by its one-based index.
fn indexed(
    strings: &BTreeMap<i16, Vec<u8>>,
    resource: i16,
    index: i16,
) -> Result<&[u8], RuleError> {
    let table = strings
        .get(&resource)
        .ok_or_else(|| RuleError::Invalid(format!("character STR# {resource} missing")))?;
    if index < 1 || centurions_content::word(table, 0)? < index {
        return Err(RuleError::Invalid(
            "character string index outside resource".into(),
        ));
    }
    let mut offset = 2;
    for _ in 1..index {
        offset += 1 + usize::from(centurions_content::span(table, offset, 1)?[0]);
    }
    let length = usize::from(centurions_content::span(table, offset, 1)?[0]);
    Ok(centurions_content::span(table, offset + 1, length)?)
}
