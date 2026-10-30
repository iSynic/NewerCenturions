//! A new character's record, apart from the creation dialogs.
use crate::{Content, RandomState, RuleError, character, combat, story_promotion};
use centurions_content::{
    WARRIOR_BYTES, Warrior,
    game_text::GameText,
    records::{
        ATTRIBUTES, Attribute, CharacterIdentityMut, CharacterStatisticsMut,
        DevelopmentField as Field, InventorySlotsMut, ItemDefinition, ProfessionProfile,
        ProfileTable, RaceProfile, Statistic, WarriorRecordMut,
    },
    word,
};
use serde::{Deserialize, Serialize};

/// The choices made in the creation dialog for a new warrior.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    /// The race chosen.
    pub race: i16,
    /// The profession chosen.
    pub mos: i16,
    /// The gender chosen.
    pub gender: i16,
    /// The starting level chosen.
    pub level: i16,
    /// The warrior's name.
    pub name: String,
    /// The portrait chosen.
    pub portrait: i16,
    /// The icon chosen.
    pub icon: i16,
}

/// Builds a new warrior's record from the dialog's choices: race and profession profiles, rolled
/// attributes, starting skills, resistances and conditions. Starting items are given separately
/// by `initial_items`.
pub fn generate(
    request: &Request,
    content: &Content,
    random: &mut RandomState,
    library_seed: &mut u32,
    library_draws: &mut u64,
) -> Result<Warrior, RuleError> {
    let map = content
        .map(0)
        .ok_or_else(|| RuleError::Invalid("creation profiles missing".into()))?;
    if !(1..=30).contains(&request.race)
        || !(1..=30).contains(&request.mos)
        || !(0..=3).contains(&request.gender)
        || ![1, 3, 5, 10, 15, 20, 30].contains(&request.level)
    {
        return Err(RuleError::Invalid(
            "invalid warrior creation selection".into(),
        ));
    }
    let race = RaceProfile(ProfileTable::races(&map.races).row(request.race as usize - 1)?);
    let mos =
        ProfessionProfile(ProfileTable::professions(&map.profiles).row(request.mos as usize - 1)?);
    if !race.accepts_profession(request.mos)? {
        return Err(RuleError::Invalid("race and MOS are incompatible".into()));
    }
    let (name, _, invalid) = encoding_rs::MACINTOSH.encode(&request.name);
    if invalid || name.is_empty() || name.len() > 30 || name.iter().any(|b| *b == 0 || *b < 32) {
        return Err(RuleError::Invalid(
            "warrior name must fit the original 30-byte field".into(),
        ));
    }
    for (owner, id) in [
        ("/Portraits", request.portrait),
        ("/Tacticals", request.icon),
    ] {
        if !content
            .resources
            .iter()
            .any(|r| r.original.ends_with(owner) && r.r#type == "cicn" && r.id == id)
        {
            return Err(RuleError::Invalid(
                "creation appearance resource missing".into(),
            ));
        }
    }
    let game_text = GameText::load(content, std::path::Path::new("."))?;
    let mut b = vec![0; WARRIOR_BYTES];
    CharacterIdentityMut(&mut b).initialize(
        request.race,
        request.mos,
        request.gender,
        request.portrait,
        request.icon,
        &name,
    );
    for i in 0..40 {
        let value = if mos.condition_level(i)? == 1 {
            -1
        } else {
            race.condition(i)?
        };
        WarriorRecordMut::from_bytes(&mut b).set_condition_duration(i, value);
    }
    for i in 0..100 {
        WarriorRecordMut::from_bytes(&mut b).set_inventory_slot(
            i,
            &centurions_content::InventoryRecord(
                game_text.bytes(0x41fc32, 22)?.try_into().unwrap(),
            ),
        );
    }
    Field::MeleeSound.write(
        &mut b,
        if request.gender == 3 {
            38
        } else {
            request.gender * 8 + 30
        },
    );
    CharacterStatisticsMut(&mut b).set(
        Statistic::ScorePenalty,
        (i32::from(request.level - 1))
            .wrapping_mul(i32::from(request.level - 1))
            .wrapping_mul(10),
    );
    for i in 0..8 {
        Field::Resistance(i).write(
            &mut b,
            race.resistance(i)?
                .wrapping_add(mos.resistance(i)?)
                .clamp(-99, 300),
        );
    }
    for i in 0..8 {
        Field::CreatureBonus(i).write(&mut b, race.creature_bonus(i)?);
    }
    WarriorRecordMut::from_bytes(&mut b).set_in_battle(1);
    let power = race
        .action_points()?
        .wrapping_add(2)
        .wrapping_add(mos.action_points()?);
    WarriorRecordMut::from_bytes(&mut b).set_attacks(power);
    Field::ActionsPerRound.write(&mut b, power);
    let experience = story_promotion::experience_threshold(request.level)?;
    WarriorRecordMut::from_bytes(&mut b).set_remaining_experience(experience);
    WarriorRecordMut::from_bytes(&mut b).set_upgrade_points(experience);
    Field::BareHandMaximum.write(&mut b, mos.bare_hand_maximum()?);
    Field::ScanCapacity.write(&mut b, mos.scan_capacity()?);
    Field::AttackLimit.write(&mut b, mos.attack_limit()?);
    for attribute in ATTRIBUTES {
        let (mos_low, mos_high) = mos.attribute_bounds(attribute)?;
        let (race_low, race_high) = race.attribute_bounds(attribute)?;
        let value = random
            .range(mos_low.max(race_low), mos_high.min(race_high))
            .wrapping_add(
                race.attribute_bonus(attribute)?
                    .wrapping_add(mos.attribute_bonus(attribute)?),
            );
        Field::Attribute(attribute).write(&mut b, value);
    }
    use Attribute::{Brawn, Coordination, Judgement, Psionics, Quickness, Vitality};
    for (attribute, delta) in match request.gender {
        1 => &[(Brawn, 5), (Quickness, 5), (Judgement, -2)][..],
        2 => &[
            (Brawn, -3),
            (Quickness, -7),
            (Coordination, 5),
            (Judgement, 5),
        ][..],
        _ => &[
            (Brawn, -5),
            (Coordination, 5),
            (Judgement, 5),
            (Vitality, -5),
            (Psionics, 5),
        ][..],
    } {
        Field::Attribute(*attribute).add(&mut b, *delta)?;
    }
    for attribute in ATTRIBUTES {
        let field = Field::Attribute(attribute);
        let value = field.read(&b)?.clamp(3, 99);
        field.write(&mut b, value);
    }
    for threshold in [80, 90, 95] {
        if random.bound(100) > threshold {
            let choice = random.range(0, 7) as usize;
            Field::CreatureBonus(choice).add(&mut b, 5)?;
        }
    }
    CharacterIdentityMut(&mut b).set_creation_marker();
    let stamina = random.bound(mos.initial_stamina_bound()?);
    Field::BaseStamina.write(&mut b, stamina);
    Field::MissileAccuracy.write(
        &mut b,
        mos.missile_accuracy()?
            .wrapping_add(race.missile_accuracy()?),
    );
    let strength = Field::Attribute(Brawn).read(&b)?;
    let strength_adjust = character::strength_adjustments(strength);
    Field::MeleeAccuracy.write(
        &mut b,
        mos.melee_accuracy()?
            .wrapping_add(race.melee_accuracy()?)
            .wrapping_add(strength_adjust[0]),
    );
    Field::BareHandMaximum.add(&mut b, strength_adjust[1])?;
    Field::MeleeDodge.write(&mut b, mos.melee_dodge()?);
    Field::MissileDodge.write(&mut b, mos.missile_dodge()?);
    Field::PsiResistance.write(
        &mut b,
        mos.psi_resistance()?.wrapping_add(race.psi_resistance()?),
    );
    let age_index = mos.age_category()?.wrapping_sub(1) as usize;
    let (youngest, oldest) = race.age_bounds(age_index)?;
    let years = random.range(youngest, oldest);
    CharacterIdentityMut(&mut b).set_age_days(i32::from(years).wrapping_mul(365));
    let base_psi = Field::Attribute(Judgement)
        .read(&b)?
        .wrapping_add(Field::Attribute(Attribute::Knowledge).read(&b)?)
        / 6
        + Field::Attribute(Psionics).read(&b)? / 3;
    Field::BasePsi.write(&mut b, base_psi);
    for school in 0..3 {
        if mos.psi_school_enabled(school)? {
            Field::PsiSchool.write(&mut b, school as i16 + 1);
        }
    }
    Field::MovementBonus.write(&mut b, mos.movement_bonus()?);
    specialties(&mut b, &race, &mos, &game_text)?;
    let mut warrior = Warrior::read(&b)?;
    combat::calculate_movement(&mut warrior, race.movement_adjustment())?;
    character::calculate_statistics(&mut warrior)?;
    let agility = character::agility_adjustment(&warrior)?;
    Field::Armor.write(&mut warrior.bytes, agility);
    let stamina = Field::BaseStamina
        .read(&warrior.bytes)?
        .wrapping_add(Field::StaminaBonus.read(&warrior.bytes)?);
    warrior.record_mut().set_stamina(stamina);
    warrior.record_mut().set_stamina_max(stamina);
    let psi = Field::BasePsi
        .read(&warrior.bytes)?
        .wrapping_add(Field::PsiBonus.read(&warrior.bytes)?);
    warrior.record_mut().set_psi_points(psi);
    warrior.record_mut().set_psi_points_max(psi);
    let mut library = story_promotion::LibraryRandom {
        seed: *library_seed,
        draws: *library_draws,
    };
    for _ in 1..request.level {
        story_promotion::editor_level_up(&mut warrior, &map.races, &map.profiles, &mut library)?;
    }
    *library_seed = library.seed;
    *library_draws = library.draws;
    warrior.original_sha256 = centurions_content::digest(&warrior.bytes);
    Ok(warrior)
}
/// Gives a new character its profession's starting items, installs and checks
/// each one, and wears what it can.
///
/// Original: addinitialitems
pub fn initial_items(
    state: &mut crate::Session,
    content: &Content,
) -> Result<Vec<crate::Event>, RuleError> {
    let mission = content.map(0).ok_or("creation item profiles missing")?;
    let warrior = state
        .warriors
        .first_mut()
        .ok_or("created warrior missing")?;
    if !warrior.inventory()?.is_empty() {
        return Err("creation equipment already installed".into());
    }
    let mos = ProfessionProfile(
        ProfileTable::professions(&mission.profiles)
            .row(warrior.record().profession()? as usize - 1)?,
    );
    let mut candidate = warrior.clone();
    let mut count = 0_u16;
    for index in 0..20 {
        let id = mos.starting_item(index)?;
        if id == 0 {
            continue;
        }
        let definition = content
            .items
            .get(id.unsigned_abs() as usize)
            .ok_or("creation item definition missing")?;
        let weight = ItemDefinition(definition)
            .supplied_quantity()?
            .wrapping_mul(ItemDefinition(definition).charge_weight()?)
            .wrapping_add(warrior.record().load()?)
            .wrapping_add(ItemDefinition(definition).weight()?);
        if weight as u16 > warrior.record().load_max()? as u16
            || !crate::equipment::can_use(warrior, definition, mission)?
        {
            continue;
        }
        count += 1;
        let mut item = candidate.record().inventory_slot(usize::from(count - 1));
        centurions_content::records::ItemDefinition(definition).install_fresh_into(&mut item)?;
        candidate
            .record_mut()
            .set_inventory_slot(usize::from(count - 1), &item);
        InventorySlotsMut(&mut candidate.bytes).set_count(count as i16);
    }
    *warrior = candidate;
    crate::equipment::calculate_weight(warrior, content)?;
    for slot in 0..warrior.inventory()?.len() {
        if warrior.inventory()?[slot].id() == 1 {
            let result = crate::equipment::wear(warrior, content, mission, slot, false)?;
            // A new character has no Session yet to receive party-wide writes.
            crate::equipment::require_no_globals(&result.global_writes)?;
            *warrior = result.warrior;
        }
    }
    warrior
        .record_mut()
        .set_carried_credits(mos.starting_credits()?);
    // Starting items are worn without feedback.
    Ok(vec![])
}

/// Sets the starting special skills: race plus profession for each skill the profession has,
/// then adjustments from the brawn and quickness-and-coordination tables.
fn specialties(
    b: &mut [u8],
    race: &RaceProfile<'_>,
    mos: &ProfessionProfile<'_>,
    game_text: &GameText,
) -> Result<(), RuleError> {
    let mut tables = game_text.bytes(0x400634, 576)?.to_vec();
    tables.extend(game_text.bytes(0x4008a4, 456)?);
    // The specialty table is read past its end into values that are zero for
    // every supplied profile: an original overread (see docs/source-notes/rules.md).
    tables.extend([0; 96]);
    for i in 0..15 {
        Field::SpecialSkill(i).write(
            b,
            if mos.special_skill(i)? != 0 {
                race.special_skill(i)?.wrapping_add(mos.special_skill(i)?)
            } else {
                0
            },
        );
    }
    let thresholds = game_text.bytes(0x400874, 48)?;
    let strength = Field::Attribute(Attribute::Brawn).read(b)? as u16;
    let quickness = (Field::Attribute(Attribute::Quickness).read(b)? as u16)
        .wrapping_add(Field::Attribute(Attribute::Coordination).read(b)? as u16)
        / 2;
    for (attribute, rows, address, stride) in
        [(strength, 24, 0x400634, 48), (quickness, 19, 0x4008a4, 38)]
    {
        let table = &tables[if address == 0x400634 { 0 } else { 576 }..];
        for row in 0..rows {
            if attribute == word(thresholds, row * 2)? as u16 {
                for i in 0..14 {
                    if mos.special_skill(i)? != 0 {
                        let value = word(table, i * stride + row * 2)?;
                        Field::SpecialSkill(i).add(b, value)?;
                    }
                }
            }
        }
    }
    clamp_development_fields(b)?;
    Ok(())
}

/// Holds special skills to -99..999, and accuracy, dodge, psi resistance and bare-hand damage to
/// 0..999, after creation or development.
pub(crate) fn clamp_development_fields(bytes: &mut [u8]) -> Result<(), RuleError> {
    for index in 0..12 {
        let field = Field::SpecialSkill(index);
        field.write(bytes, field.read(bytes)?.clamp(-99, 999));
    }
    for field in [
        Field::MissileAccuracy,
        Field::MeleeAccuracy,
        Field::MissileDodge,
        Field::MeleeDodge,
        Field::PsiResistance,
        Field::BareHandMaximum,
        Field::BareHandMinimum,
    ] {
        field.write(bytes, field.read(bytes)?.clamp(0, 999));
    }
    Ok(())
}
