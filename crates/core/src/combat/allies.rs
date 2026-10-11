use super::*;
use centurions_content::mission::OpeningMission;

pub fn place_allies(
    menu: &[u8],
    count: usize,
    map: &mut TacticalMap,
    mission: &OpeningMission,
    first: (i16, i16),
    bounds: (i16, i16, i16, i16),
    _random: &mut RandomState,
) -> Result<EnemySetup, RuleError> {
    if menu.len() != 20 * 288 || count > 20 || map.tiles.len() != 8100 {
        return Err(RuleError::Invalid("invalid ally placement input".into()));
    }
    let mut staged = map.clone();
    let origin = (map.shift.0.wrapping_add(5), map.shift.1.wrapping_add(5));
    let mut result = EnemySetup {
        records: vec![],
        positions: vec![],
    };
    for index in 0..count {
        let mut bytes = menu[index * 288..(index + 1) * 288].to_vec();
        if word(&bytes, 0x10)? != 0 {
            return Err(RuleError::Unsupported {
                action: "combat.allies",
                dependency: "large ally placement",
            });
        }
        put_word(&mut bytes, 0x10c, -1);
        bytes[0x82] = 0;
        bytes[0x104] = 0;
        let mut found = None;
        'search: for ring in 1..=180_i16 {
            for dy in -ring..ring {
                for dx in -ring..ring {
                    let position = (
                        first.0.wrapping_add(dx.wrapping_mul(2)),
                        first.1.wrapping_add(dy.wrapping_mul(2)),
                    );
                    if position.1 < bounds.0
                        || position.0 < bounds.1
                        || position.1 >= bounds.2
                        || position.0 >= bounds.3
                    {
                        continue;
                    }
                    let cell = OpeningMission::index(
                        i32::from(position.0.wrapping_add(origin.0)),
                        i32::from(position.1.wrapping_add(origin.1)),
                    )?;
                    let tile = staged.tiles[cell];
                    if tile > 999 && mission.tile_word(tile - 1000, 4)? == 0 {
                        found = Some((position, cell, tile));
                        break 'search;
                    }
                }
            }
        }
        let (position, cell, tile) =
            found.ok_or_else(|| RuleError::Invalid("no free ally placement cell".into()))?;
        staged.tiles[cell] = index as i16 + 10;
        put_word(&mut bytes, 0x11e, tile);
        for offset in [0x118, 0x11a, 0x11c] {
            put_word(&mut bytes, offset, 0);
        }
        result.records.push(MonsterRecord { bytes });
        result.positions.push(position);
    }
    *map = staged;
    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllySynchronization {
    pub menu: Vec<u8>,
    pub count: usize,
    pub modal: bool,
    pub sounds: Vec<i16>,
}

pub fn synchronize(
    menu: &[u8],
    count: usize,
    records: &mut [MonsterRecord],
    random: &mut RandomState,
) -> Result<AllySynchronization, RuleError> {
    if menu.len() != 20 * 288
        || count > 20
        || records.len() > 100
        || records.iter().any(|record| record.bytes.len() != 288)
    {
        return Err(RuleError::Invalid(
            "invalid ally synchronization input".into(),
        ));
    }
    let mut menu = menu.to_vec();
    let mut eligible = vec![];
    for (index, record) in records.iter_mut().enumerate() {
        let bytes = &mut record.bytes;
        if bytes[0xb0] == 255 && bytes[0xb1] == 1 {
            bytes[0xb0] = 1;
            put_word(bytes, 0xe, 0);
        }
        for offset in [0xd8, 0xcf, 0xb7, 0xb4, 0xb3, 0xb2, 0x105] {
            bytes[offset] = 0;
        }
        if word(bytes, 0x5a)? < 0 {
            put_word(bytes, 0xe, 0);
        }
        if word(bytes, 0x110)? > 0 && word(bytes, 0xe)? == 0 && word(bytes, 0x5a)? != 0 {
            eligible.push(index);
        }
    }
    if eligible.is_empty() {
        normalize_menu(&mut menu, count);
        return Ok(AllySynchronization {
            menu,
            count,
            modal: false,
            sounds: vec![],
        });
    }
    for _ in 0..eligible.len() - 1 {
        for position in 0..eligible.len() - 1 {
            let left = &records[eligible[position]].bytes;
            let right = &records[eligible[position + 1]].bytes;
            if word(left, 0x110)? < word(right, 0x110)? || word(right, 0x5a)? == -1 {
                eligible.swap(position, position + 1);
            }
        }
    }
    eligible.truncate(32);
    let mut selected = vec![false; eligible.len()];
    let mut selected_count = 0;
    for (position, &index) in eligible.iter().enumerate() {
        if word(&records[index].bytes, 0x5a)? == -1 && selected_count < 18 {
            selected[position] = true;
            selected_count += 1;
        }
    }
    let limit = (4 + selected_count).min(18);
    for selected in selected.iter_mut().take(10) {
        if !*selected && selected_count < limit {
            *selected = true;
            selected_count += 1;
        }
    }
    let count = selected_count;
    let sounds = vec![29999_i16.wrapping_add(random.bound(4))];
    for (index, selected) in eligible.into_iter().zip(selected) {
        if !selected {
            continue;
        }
        selected_count -= 1;
        let bytes = &mut records[index].bytes;
        let destination = &mut menu[selected_count * 288..(selected_count + 1) * 288];
        destination.copy_from_slice(bytes);
        for value in &mut destination[0xb2..0xb6] {
            if (*value as i8) > 0 {
                *value = 0;
            }
        }
        put_word(bytes, 0, 0);
        put_word(bytes, 0x110, 0);
        for offset in (0x26..0x32).step_by(2) {
            put_word(bytes, offset, 0);
        }
        put_word(bytes, 0x3a, 0);
    }
    for record in records {
        put_word(&mut record.bytes, 0x110, 0);
    }
    normalize_menu(&mut menu, count);
    Ok(AllySynchronization {
        menu,
        count,
        modal: true,
        sounds,
    })
}

fn normalize_menu(menu: &mut [u8], count: usize) {
    for index in 0..count {
        let bytes = &mut menu[index * 288..(index + 1) * 288];
        put_word(bytes, 0x10c, 0);
        bytes[0x105] = 0;
    }
}
