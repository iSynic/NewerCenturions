//! NewCharacter 0x13f684 record initialization, separated from its dialogs.
use crate::{Content, RandomState, RuleError, character, combat, story_promotion};
use centurions_content::{WARRIOR_BYTES, Warrior, native_text::NativeText, span, word};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub race: i16,
    pub mos: i16,
    pub gender: i16,
    pub level: i16,
    pub name: String,
    pub portrait: i16,
    pub icon: i16,
}

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
    let race = span(&map.races, (request.race as usize - 1) * 408, 408)?;
    let mos = span(&map.profiles, (request.mos as usize - 1) * 576, 576)?;
    if span(race, 208 + request.mos as usize - 1, 1)?[0] == 0 {
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
    let native = NativeText::load(content, std::path::Path::new("."))?;
    let mut b = vec![0; WARRIOR_BYTES];
    put(&mut b, 0x32, request.race);
    put(&mut b, 0x34, request.mos);
    put(&mut b, 0x38, request.gender);
    put(&mut b, 0x3a, 1);
    put(&mut b, 0x54, request.portrait);
    put(&mut b, 0x56, request.icon);
    b[0xa21..0xa21 + name.len()].copy_from_slice(&name);
    for i in 0..40 {
        let value = if word(mos, 138 + i * 2)? == 1 {
            -1
        } else {
            word(race, 112 + i * 2)?
        };
        put(&mut b, 0x5e + i * 2, value);
    }
    for i in 0..100 {
        b[0x132 + i * 22..0x132 + (i + 1) * 22].copy_from_slice(native.bytes(0x41fc32, 22)?);
    }
    put(
        &mut b,
        0x128,
        if request.gender == 3 {
            38
        } else {
            request.gender * 8 + 30
        },
    );
    put_long(
        &mut b,
        0xa72,
        (i32::from(request.level - 1))
            .wrapping_mul(i32::from(request.level - 1))
            .wrapping_mul(10),
    );
    for i in 0..8 {
        put(
            &mut b,
            0x112 + i * 2,
            word(race, 44 + i * 2)?
                .wrapping_add(word(mos, 56 + i * 2)?)
                .clamp(-99, 300),
        );
    }
    for i in 0..8 {
        put(&mut b, 0xae + i * 2, word(race, i * 2)?);
    }
    b[0x9df] = 1;
    let power = word(race, 204)?
        .wrapping_add(2)
        .wrapping_add(word(mos, 266)?);
    put(&mut b, 0x40, power);
    put(&mut b, 0x12, power);
    let experience = story_promotion::experience_threshold(request.level)?;
    put_long(&mut b, 0x9ce, experience);
    put_long(&mut b, 0x9d2, experience);
    put(&mut b, 12, word(mos, 242)?);
    put(&mut b, 0xac6, word(mos, 452)?);
    put(&mut b, 0xaca, word(mos, 268)?);
    for i in 0..7 {
        let row = if i == 6 { 5 } else { i };
        let lower = word(mos, 110 + row * 4)?.max(word(race, 74 + row * 4)?);
        let upper = word(mos, 112 + row * 4)?.min(word(race, 76 + row * 4)?);
        let value = random
            .range(lower, upper)
            .wrapping_add(word(race, 60 + row * 2)?.wrapping_add(word(mos, 72 + row * 2)?));
        put(&mut b, 0x9e2 + i * 2, value);
    }
    for (offset, delta) in match request.gender {
        1 => &[(0x9e2, 5), (0x9e8, 5), (0x9e6, -2)][..],
        2 => &[(0x9e2, -3), (0x9e8, -7), (0x9ea, 5), (0x9e6, 5)][..],
        _ => &[(0x9e2, -5), (0x9ea, 5), (0x9e6, 5), (0x9ec, -5), (0x9ee, 5)][..],
    } {
        add(&mut b, *offset, *delta)?;
    }
    for i in 0..7 {
        let offset = 0x9e2 + i * 2;
        let value = word(&b, offset)?.clamp(3, 99);
        put(&mut b, offset, value);
    }
    for threshold in [80, 90, 95] {
        if random.bound(100) > threshold {
            let choice = random.range(0, 7) as usize;
            add(&mut b, 0xae + choice * 2, 5)?;
        }
    }
    put(&mut b, 0, 1);
    let stamina = random.bound(word(mos, 222)?);
    put(&mut b, 0xb42, stamina);
    put(&mut b, 4, word(mos, 238)?.wrapping_add(word(race, 202)?));
    let strength = word(&b, 0x9e2)?;
    let strength_adjust = character::strength_adjustments(strength);
    put(
        &mut b,
        6,
        word(mos, 234)?
            .wrapping_add(word(race, 200)?)
            .wrapping_add(strength_adjust[0]),
    );
    add(&mut b, 12, strength_adjust[1])?;
    put(&mut b, 10, word(mos, 250)?);
    put(&mut b, 8, word(mos, 246)?);
    put(&mut b, 0x22, word(mos, 260)?.wrapping_add(word(race, 198)?));
    let age_index = word(mos, 256)?.wrapping_sub(1) as usize;
    let years = random.range(
        word(race, 238 + age_index * 4)?,
        word(race, 240 + age_index * 4)?,
    );
    put_long(&mut b, 0x9ca, i32::from(years).wrapping_mul(365));
    let base_psi = word(&b, 0x9e6)?.wrapping_add(word(&b, 0x9e4)?) / 6 + word(&b, 0x9ee)? / 3;
    put(&mut b, 0xb40, base_psi);
    for (flag, school) in [(88, 1), (94, 2), (100, 3)] {
        if word(mos, flag)? != 0 {
            put(&mut b, 0x36, school);
        }
    }
    put(&mut b, 0x30, word(mos, 258)?);
    specialties(&mut b, race, mos, &native)?;
    let mut warrior = Warrior::read(&b)?;
    combat::calculate_movement(&mut warrior, race[197] as i8)?;
    character::calculate_statistics(&mut warrior)?;
    let agility = character::agility_adjustment(&warrior)?;
    put(&mut warrior.bytes, 0x2c, agility);
    let stamina = word(&warrior.bytes, 0xb42)?.wrapping_add(word(&warrior.bytes, 0xb46)?);
    put(&mut warrior.bytes, 0x50, stamina);
    put(&mut warrior.bytes, 0x52, stamina);
    let psi = word(&warrior.bytes, 0xb40)?.wrapping_add(word(&warrior.bytes, 0xb44)?);
    put(&mut warrior.bytes, 0x58, psi);
    put(&mut warrior.bytes, 0x5a, psi);
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
/// addinitialitems 0x13f52c uses MOS items, native installation, eligibility and equipment.
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
    let mos = span(
        &mission.profiles,
        (warrior.record().profession()? as usize - 1) * 576,
        576,
    )?;
    use crate::recovered_install::{InstallMemory, InstallRequest, installitem_complete};
    let backing: [Vec<u8>; 5] =
        std::array::from_fn(|family| content.items[family * 200..(family + 1) * 200].concat());
    let tables = crate::recovered_records::ItemTables {
        families: std::array::from_fn(|i| backing[i].as_slice()),
    };
    let base = 0x4046e6;
    let record = (0x44721c - base) as usize;
    let mut memory = InstallMemory {
        base_address: base,
        bytes: vec![0; (0x478288 - base) as usize],
    };
    memory.bytes[record..record + WARRIOR_BYTES].copy_from_slice(&warrior.bytes);
    let mut count = 0_u16;
    for index in 0..20 {
        let id = word(mos, 392 + index * 2)?;
        if id == 0 {
            continue;
        }
        let definition = content
            .items
            .get(id.unsigned_abs() as usize)
            .ok_or("creation item definition missing")?;
        let weight = word(definition, 20)?
            .wrapping_mul(word(definition, 36)?)
            .wrapping_add(warrior.record().carried_weight()?)
            .wrapping_add(word(definition, 16)?);
        if weight as u16 > warrior.record().weight_capacity()? as u16
            || !crate::equipment::can_use(warrior, definition, mission)?
        {
            continue;
        }
        count += 1;
        memory.bytes[record + 0x1a..record + 0x1c].copy_from_slice(&count.to_be_bytes());
        let installed = installitem_complete(
            &memory,
            &state.random,
            &tables,
            InstallRequest {
                source_member: u16::MAX,
                destination_member: u16::MAX,
                source_slot: 0,
                destination_slot: count - 1,
                item_id: id,
            },
            crate::recovered_random_item::RandomItemLimits {
                maximum_draws: 1000,
            },
        )?;
        memory = installed.memory;
        state.random = installed.random;
    }
    warrior
        .bytes
        .copy_from_slice(&memory.bytes[record..record + WARRIOR_BYTES]);
    crate::equipment::calculate_weight(warrior, content)?;
    for slot in 0..warrior.inventory()?.len() {
        if warrior.inventory()?[slot].id() == 1 {
            let result = crate::equipment::wear(warrior, content, mission, slot, false)?;
            *warrior = result.warrior;
        }
    }
    put(&mut warrior.bytes, 0x9da, word(mos, 390)?);
    // This caller supplies wearitem's zero feedback gate.
    Ok(vec![])
}

fn specialties(
    b: &mut [u8],
    race: &[u8],
    mos: &[u8],
    native: &NativeText,
) -> Result<(), RuleError> {
    let mut tables = native.bytes(0x400634, 576)?.to_vec();
    tables.extend(native.bytes(0x4008a4, 456)?);
    // NewCharacter's reachable specialty overread WORDs are zero caller locals.
    // The nonzero return-address WORD occurs only at quickness 3; supplied profiles start at 4.
    tables.extend([0; 96]);
    for i in 0..15 {
        put(
            b,
            0xf4 + i * 2,
            if word(mos, i * 2)? != 0 {
                word(race, 16 + i * 2)?.wrapping_add(word(mos, i * 2)?)
            } else {
                0
            },
        );
    }
    let thresholds = native.bytes(0x400874, 48)?;
    let strength = word(b, 0x9e2)? as u16;
    let quickness = (word(b, 0x9e8)? as u16).wrapping_add(word(b, 0x9ea)? as u16) / 2;
    for (attribute, rows, address, stride) in
        [(strength, 24, 0x400634, 48), (quickness, 19, 0x4008a4, 38)]
    {
        let table = &tables[if address == 0x400634 { 0 } else { 576 }..];
        for row in 0..rows {
            if attribute == word(thresholds, row * 2)? as u16 {
                for i in 0..14 {
                    if word(mos, i * 2)? != 0 {
                        let value = word(table, i * stride + row * 2)?;
                        add(b, 0xf4 + i * 2, value)?;
                    }
                }
            }
        }
    }
    for i in 0..12 {
        let o = 0xf4 + i * 2;
        put(b, o, word(b, o)?.clamp(-99, 999));
    }
    for o in [4, 6, 8, 10, 0x22, 12, 14] {
        put(b, o, word(b, o)?.clamp(0, 999));
    }
    Ok(())
}
fn put(b: &mut [u8], o: usize, v: i16) {
    b[o..o + 2].copy_from_slice(&v.to_be_bytes());
}
fn put_long(b: &mut [u8], o: usize, v: i32) {
    b[o..o + 4].copy_from_slice(&v.to_be_bytes());
}
fn add(b: &mut [u8], o: usize, v: i16) -> Result<(), RuleError> {
    put(b, o, word(b, o)?.wrapping_add(v));
    Ok(())
}
