use crate::{Command, Event, Phase, Presentation, RuleError, Session, equipment};
use centurions_content::{Content, InventoryRecord, mission::long, span, word};
use serde::{Deserialize, Serialize};

const RECORD: usize = 3002;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Access {
    pub level: u8,
    pub id: u8,
    #[serde(default)]
    pub enhancement: i16,
    #[serde(default)]
    pub remaining: i16,
}

fn refused(warning: i16) -> RuleError {
    RuleError::Refused {
        feedback: vec![crate::feedback::NativeFeedback::Warning { index: warning }],
    }
}

pub(crate) fn validate(state: &Session, content: &Content) -> Result<(), RuleError> {
    for (level, records) in &state.shops {
        let context = content.context(state.mission.as_deref(), *level)?;
        let authored = context.map(0).ok_or("shop level unavailable")?;
        if records.len() != authored.shops.len() || !records.len().is_multiple_of(RECORD) {
            return Err("invalid persistent shop records".into());
        }
        for shop in records.as_chunks::<RECORD>().0 {
            for slot in 0..1000 {
                if shop[2000 + slot] != 0 && !(0..1000).contains(&word(shop, slot * 2)?) {
                    return Err("invalid shop item identifier".into());
                }
            }
        }
    }
    if let Some(access) = &state.shop_access {
        let context = content.context(state.mission.as_deref(), access.level)?;
        let map = context.map(0).ok_or("shop level unavailable")?;
        span(&map.shops, access.id as usize * RECORD, RECORD)?;
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
        && (state.phase != Phase::Exploration
            || state
                .shop_access
                .as_ref()
                .is_none_or(|a| a.level != state.scenario_level))
    {
        return Err("invalid active shop".into());
    }
    Ok(())
}

/// newland opcode 6, 0x142fae; positive parameters register later inventory access.
pub(crate) fn visit(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<bool, RuleError> {
    let id = parameter.wrapping_abs() as u8;
    state.loot_limit = None;
    let map = state.map(content, 0).ok_or("shop scenario unavailable")?;
    span(&map.shops, id as usize * RECORD, RECORD)?;
    let records = state.shops.entry(state.scenario_level).or_insert_with(|| {
        state
            .native_import
            .as_ref()
            .and_then(|i| i.scenario_records.get(&state.scenario_level))
            .cloned()
            .unwrap_or_else(|| map.shops.clone())
    });
    let shop = &mut records[id as usize * RECORD..(id as usize + 1) * RECORD];
    // loadshop 0x12fe54 compacts each ID family independently and keeps its order.
    for family in 0..5 {
        let mut next = family * 200;
        for slot in family * 200..(family + 1) * 200 {
            if shop[2000 + slot] != 0 {
                let item = word(shop, slot * 2)?;
                let count = shop[2000 + slot];
                shop[next * 2..next * 2 + 2].copy_from_slice(&item.to_be_bytes());
                shop[2000 + next] = count;
                next += 1;
            }
        }
        for slot in next..(family + 1) * 200 {
            shop[slot * 2..slot * 2 + 2].fill(0);
            shop[2000 + slot] = 0;
        }
    }
    state.shop_access = Some(Access {
        level: state.scenario_level,
        id,
        enhancement: 0,
        remaining: 0,
    });
    if parameter <= 0 {
        show(state)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub(crate) fn visit_restricted(
    state: &mut Session,
    content: &Content,
    extra: &[u8],
) -> Result<(), RuleError> {
    visit(state, content, word(extra, 0)?)?;
    let access = state.shop_access.as_mut().unwrap();
    access.enhancement = word(extra, 4)?.wrapping_add(1);
    access.remaining = word(extra, 2)?;
    Ok(())
}

fn show(state: &mut Session) -> Result<(), RuleError> {
    crate::party::pool_credits(&mut state.warriors, &mut state.party_currency)?;
    state
        .exploration
        .as_mut()
        .ok_or("no active mission")?
        .presentation = Some(Presentation::Shop);
    Ok(())
}

pub(crate) fn open(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || state
            .shop_access
            .as_ref()
            .is_none_or(|a| a.level != state.scenario_level)
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some())
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    show(state)?;
    Ok(vec![Event::StoryPresented])
}

pub(crate) fn close(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    active(state)?;
    state.exploration.as_mut().unwrap().presentation = None;
    if state.exploration.as_ref().unwrap().script.is_some() {
        crate::exploration::run_script(state, content)
    } else {
        Ok(vec![])
    }
}

fn active(state: &Session) -> Result<&Access, RuleError> {
    if state.phase != Phase::Exploration
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation != Some(Presentation::Shop))
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    state
        .shop_access
        .as_ref()
        .filter(|a| a.level == state.scenario_level)
        .ok_or_else(|| "no active shop records".into())
}

fn records(state: &Session) -> Result<&[u8], RuleError> {
    let access = state.shop_access.as_ref().ok_or("shop unavailable")?;
    span(
        state
            .shops
            .get(&access.level)
            .ok_or("shop records unavailable")?,
        access.id as usize * RECORD,
        RECORD,
    )
    .map_err(Into::into)
}

pub fn stock(state: &Session) -> Result<Vec<(usize, i16, u8)>, RuleError> {
    let records = records(state)?;
    (0..1000)
        .filter(|i| records[2000 + i] != 0)
        .map(|i| Ok((i, word(records, i * 2)?, records[2000 + i])))
        .collect()
}

pub fn price(state: &Session, content: &Content, id: i16, selling: bool) -> Result<i32, RuleError> {
    let definition = content
        .items
        .get(id.unsigned_abs() as usize)
        .ok_or("shop item unavailable")?;
    let mut cost = long(definition, 142)?.wrapping_abs();
    if selling {
        cost = cost
            .checked_div(i32::from(state.difficulty.wrapping_mul(2).wrapping_add(10)))
            .ok_or("native shop division traps")?;
    }
    Ok(
        crate::recovered_economy::itemcost(crate::recovered_economy::ItemCostInput {
            cost,
            percent: word(records(state)?, 3000)?,
            mode: i16::from(selling),
        })?
        .cost,
    )
}

/// installitem 0x12c874 fresh-stock branch, without its randomized-loot branch.
pub fn preview(content: &Content, id: i16) -> Result<InventoryRecord, RuleError> {
    let item = crate::ammunition::definition(content, id)?;
    let mut record = InventoryRecord([0; 22]);
    let quantity = word(&item, 148)?.wrapping_add(word(&item, 20)?);
    let capacity = match word(&item, 90)? {
        0 => match word(&item, 24)? {
            0 => quantity,
            n => n,
        },
        n => n,
    };
    for (offset, value) in [(0, word(&item, 102)?), (2, quantity), (6, capacity)] {
        record.0[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    for (destination, source) in [
        (8, 35),
        (10, 27),
        (13, 139),
        (14, 78),
        (15, 81),
        (16, 83),
        (17, 84),
        (18, 85),
        (19, 86),
        (21, 80),
        (20, 79),
    ] {
        record.0[destination] = item[source];
    }
    Ok(record)
}

fn mutate_stock(state: &mut Session) -> &mut [u8] {
    let access = state.shop_access.as_ref().unwrap();
    &mut state.shops.get_mut(&access.level).unwrap()
        [access.id as usize * RECORD..(access.id as usize + 1) * RECORD]
}

pub(crate) fn buy(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    active(state)?;
    let access = state.shop_access.as_ref().unwrap();
    if access.enhancement != 0 && access.remaining == 0 {
        return Err(refused(159));
    }
    let records = records(state)?;
    if slot >= 1000 || records[2000 + slot] == 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let id = word(records, slot * 2)?;
    let cost = price(state, content, id, false)?;
    let item = preview(content, id)?;
    let recipient = state
        .warriors
        .get(warrior)
        .ok_or("shop recipient unavailable")?;
    let definition = &content.items[id.unsigned_abs() as usize];
    let weight =
        word(definition, 16)?.wrapping_add(word(definition, 36)?.wrapping_mul(word(&item.0, 2)?));
    if recipient.inventory()?.len() >= 30
        || word(&recipient.bytes, 0x9d6)?.wrapping_add(weight) as u16
            > word(&recipient.bytes, 0x9d8)? as u16
    {
        return Err(refused(19));
    }
    if cost > state.party_currency {
        return Err(refused(37));
    }
    state.load_item_work(content, id)?;
    let mut events = vec![Event::Sound { resource_id: 83 }];
    if word(definition, 34)? != 0 {
        let reserve = id
            .checked_sub(200)
            .filter(|i| (0..200).contains(i))
            .ok_or("shop ammunition identifier outside reserves")? as usize;
        state.ammunition[reserve] =
            state.ammunition[reserve].wrapping_add(i32::from(word(definition, 20)?));
        let sound = if word(definition, 12)? != 0 {
            word(definition, 12)?
        } else {
            state.random.bound(2).wrapping_add(403)
        };
        events.push(Event::Sound { resource_id: sound });
        events.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
            address: 0x401d9e,
            x: 50,
            y: 50,
            wait: 30,
            sound: 0,
        }));
    } else {
        // moveicon calculates movement before installitem updates the load field.
        calculate_movement(state, content, warrior)?;
        let inventory = state.warriors[warrior].inventory()?;
        let count = inventory.len();
        let merged = if word(definition, 36)? != 0 {
            inventory.iter().position(|i| i.id() == id)
        } else {
            None
        };
        let item = if merged.is_none() && state.shop_access.as_ref().unwrap().enhancement != 0 {
            enhanced_stock(state, content, id)?
        } else {
            item
        };
        let record = &mut state.warriors[warrior];
        if let Some(slot) = merged {
            let offset = 0x132 + slot * 22 + 2;
            let quantity = word(&record.bytes, offset)?.wrapping_add(word(definition, 20)?);
            record.bytes[offset..offset + 2].copy_from_slice(&quantity.to_be_bytes());
        } else {
            record.bytes[0x132 + count * 22..0x132 + (count + 1) * 22].copy_from_slice(&item.0);
            record.bytes[0x1a..0x1c].copy_from_slice(&((count + 1) as i16).to_be_bytes());
        }
        equipment::calculate_weight(&mut state.warriors[warrior], content)?;
    }
    state.party_currency = state.party_currency.wrapping_sub(cost);
    if let Some(access) = &mut state.shop_access
        && access.remaining != 0
    {
        access.remaining = access.remaining.wrapping_sub(1);
    }
    let records = mutate_stock(state);
    records[2000 + slot] = records[2000 + slot].wrapping_sub(1);
    if records[2000 + slot] == 0 {
        records[slot * 2..slot * 2 + 2].fill(0);
    }
    Ok(events)
}

fn calculate_movement(
    state: &mut Session,
    content: &Content,
    warrior: usize,
) -> Result<(), RuleError> {
    let record = &mut state.warriors[warrior];
    let race = word(&record.bytes, 0x32)?.wrapping_sub(1) as usize;
    let movement = content
        .map(0)
        .and_then(|m| m.combat.as_ref())
        .and_then(|c| c.race_moves.get(race))
        .copied()
        .ok_or("shop race movement unavailable")?;
    crate::combat::calculate_movement(record, movement)
}

pub(crate) fn sell(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    active(state)?;
    let access = state.shop_access.as_ref().unwrap();
    if access.enhancement != 0 || access.remaining != 0 {
        return Err(refused(158));
    }
    let item = state
        .warriors
        .get(warrior)
        .ok_or("shop warrior unavailable")?
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("shop item unavailable")?;
    let cost = price(state, content, item.id(), true)?;
    let mission = content.map(0).ok_or("shop scenario unavailable")?;
    equipment::drop(&mut state.warriors[warrior], slot, content, mission)?;
    let stock = mutate_stock(state);
    let family = usize::from(item.id().unsigned_abs()) / 200 * 200;
    let existing = (family..family + 200).find(|i| word(stock, i * 2).ok() == Some(item.id()));
    if let Some(slot) = existing.or_else(|| (family..family + 200).find(|i| stock[2000 + i] == 0)) {
        stock[slot * 2..slot * 2 + 2].copy_from_slice(&item.id().to_be_bytes());
        stock[2000 + slot] = stock[2000 + slot].wrapping_add(1);
    }
    state.party_currency = state.party_currency.wrapping_add(cost);
    Ok(vec![Event::Sound { resource_id: 83 }])
}

fn enhanced_stock(
    state: &mut Session,
    content: &Content,
    id: i16,
) -> Result<InventoryRecord, RuleError> {
    use crate::recovered_install::{InstallMemory, InstallRequest};
    let base = 0x4046e6;
    let mut memory = InstallMemory {
        base_address: base,
        bytes: vec![0; 0x4781ee - base as usize],
    };
    memory.bytes[..2].copy_from_slice(
        &state
            .shop_access
            .as_ref()
            .unwrap()
            .enhancement
            .to_be_bytes(),
    );
    let offset = (0x442576 - base) as usize;
    memory.bytes[offset..offset + 2].copy_from_slice(&state.difficulty.to_be_bytes());
    let flat = content.items.iter().flatten().copied().collect::<Vec<_>>();
    let tables = crate::recovered_records::ItemTables {
        families: std::array::from_fn(|i| &flat[i * 30000..(i + 1) * 30000]),
    };
    for (address, family) in crate::recovered_records::ITEM_TABLE_ADDRESSES
        .into_iter()
        .zip(tables.families)
    {
        let offset = (address - base) as usize;
        memory.bytes[offset..offset + family.len()].copy_from_slice(family);
    }
    let result = crate::recovered_install::installitem_complete(
        &memory,
        &state.random,
        &tables,
        InstallRequest {
            source_member: 65535,
            destination_member: 65535,
            source_slot: 0,
            destination_slot: 0,
            item_id: id,
        },
        crate::recovered_random_item::RandomItemLimits {
            maximum_draws: 1024,
        },
    )?;
    state.random = result.random;
    Ok(InventoryRecord(
        span(&result.memory.bytes, (0x44734e - base) as usize, 22)?
            .try_into()
            .unwrap(),
    ))
}

pub(crate) fn command_allowed(command: &Command) -> bool {
    matches!(
        command,
        Command::CloseShop
            | Command::InspectInventory { .. }
            | Command::BuyItem { .. }
            | Command::SellItem { .. }
            | Command::Continue
            | Command::Reload { .. }
            | Command::ViewWarrior { .. }
    )
}

pub(crate) fn reload(
    state: &mut Session,
    content: &Content,
    warrior: usize,
) -> Result<Vec<Event>, RuleError> {
    active(state)?;
    state.exploration.as_mut().unwrap().presentation = None;
    let result = crate::combat::firearm::reload_command(state, content, warrior);
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Shop);
    result
}
