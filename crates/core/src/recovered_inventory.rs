use crate::{
    RuleError,
    recovered_records::{ItemTables, RecordWrite, loaditem},
};
use centurions_content::{INVENTORY_BYTES, ITEM_BYTES, WARRIOR_BYTES, Warrior, word};

pub const BANK_RECORDS: usize = 1000;
pub const BANK_BYTES: usize = BANK_RECORDS * INVENTORY_BYTES;
const PARTY_BASE: u32 = 0x448aa0;
const INVENTORY_OFFSET: usize = 0x132;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InventoryEffect {
    Write(RecordWrite),
    Sound(i16),
    Warning(i16),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BankAddition {
    pub bytes: Vec<u8>,
    pub slot: Option<usize>,
    pub effects: Vec<InventoryEffect>,
    pub native_return: i32,
}

pub fn addtobooty(
    current: &[u8],
    record: &[u8; INVENTORY_BYTES],
) -> Result<BankAddition, RuleError> {
    add_to_bank(current, record, 0x41fc74, false)
}

pub fn addtocrate(
    current: &[u8],
    record: &[u8; INVENTORY_BYTES],
) -> Result<BankAddition, RuleError> {
    add_to_bank(current, record, 0x4285f4, true)
}

fn add_to_bank(
    current: &[u8],
    record: &[u8; INVENTORY_BYTES],
    base: u32,
    sound: bool,
) -> Result<BankAddition, RuleError> {
    if current.len() != BANK_BYTES {
        return Err(unavailable(
            "inventory.bank",
            "bank requires 1000 complete 22-byte records",
        ));
    }
    let mut result = BankAddition {
        bytes: current.to_vec(),
        slot: None,
        effects: Vec::new(),
        native_return: 0,
    };
    for slot in 0..BANK_RECORDS {
        let offset = slot * INVENTORY_BYTES;
        if word(current, offset)? != 0 {
            continue;
        }
        result.bytes[offset..offset + INVENTORY_BYTES].copy_from_slice(record);
        copy_effects(&mut result.effects, base + offset as u32, record);
        if sound {
            result.effects.push(InventoryEffect::Sound(663));
        }
        result.slot = Some(slot);
        result.native_return = 1;
        break;
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactState {
    pub warrior: Warrior,
    pub highest_party_index: i8,
    pub scan: i16,
    pub pass: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactedItems {
    pub state: CompactState,
    pub effects: Vec<InventoryEffect>,
}

/// The supplied warrior is the record selected by the signed BYTE member argument.
pub fn compactitems(state: &CompactState, member: i8) -> Result<CompactedItems, RuleError> {
    let mut result = CompactedItems {
        state: state.clone(),
        effects: Vec::new(),
    };
    if member < 0 || member > state.highest_party_index {
        return Ok(result);
    }
    require_warrior(&state.warrior)?;
    let count = word(&state.warrior.bytes, 0x1a)?;
    if !(0..=100).contains(&count) {
        return Err(unavailable(
            "inventory.compact",
            "inventory count addresses outside 100 records",
        ));
    }
    let base = PARTY_BASE + member as u32 * WARRIOR_BYTES as u32;
    set_pass(&mut result, 1);
    while result.state.pass < count {
        set_scan(&mut result, 1);
        while result.state.scan < count {
            let index = result.state.scan as usize;
            if equipped(&result.state.warrior, index) && !equipped(&result.state.warrior, index - 1)
            {
                swap_records(&mut result, base, index - 1, index);
            }
            let next = result.state.scan.wrapping_add(1);
            set_scan(&mut result, next);
        }
        let next = result.state.pass.wrapping_add(1);
        set_pass(&mut result, next);
    }
    set_scan(&mut result, count);
    while result.state.scan < 100 {
        let offset = inventory_offset(result.state.scan as usize);
        write_warrior(
            &mut result.state.warrior,
            &mut result.effects,
            base,
            offset + 2,
            &[0, 0],
        );
        write_warrior(
            &mut result.state.warrior,
            &mut result.effects,
            base,
            offset + 9,
            &[0],
        );
        write_warrior(
            &mut result.state.warrior,
            &mut result.effects,
            base,
            offset,
            &[0, 0],
        );
        let next = result.state.scan.wrapping_add(1);
        set_scan(&mut result, next);
    }
    // The secondary weapon and armor searches stop at their first equipped match.
    for (equipment, destination, first_only) in [(0xe4, 0, true), (0xca, 1, true), (0xee, 2, false)]
    {
        set_scan(&mut result, 0);
        while result.state.scan < count {
            let index = result.state.scan as usize;
            if word(&result.state.warrior.bytes, inventory_offset(index))?
                == word(&result.state.warrior.bytes, equipment)?
                && equipped(&result.state.warrior, index)
            {
                swap_records(&mut result, base, index, destination);
                if equipment == 0xe4 {
                    let ammo: [u8; 2] =
                        result.state.warrior.bytes[0x136..0x138].try_into().unwrap();
                    write_warrior(
                        &mut result.state.warrior,
                        &mut result.effects,
                        base,
                        0x126,
                        &ammo,
                    );
                }
                if first_only {
                    break;
                }
            }
            let next = result.state.scan.wrapping_add(1);
            set_scan(&mut result, next);
        }
    }
    Ok(result)
}

fn equipped(warrior: &Warrior, slot: usize) -> bool {
    warrior.bytes[inventory_offset(slot) + 9] != 0
}

fn inventory_offset(slot: usize) -> usize {
    INVENTORY_OFFSET + slot * INVENTORY_BYTES
}

fn set_scan(result: &mut CompactedItems, value: i16) {
    result.state.scan = value;
    effect_write(&mut result.effects, 0x4434aa, &value.to_be_bytes());
}

fn set_pass(result: &mut CompactedItems, value: i16) {
    result.state.pass = value;
    effect_write(&mut result.effects, 0x4434ac, &value.to_be_bytes());
}

fn swap_records(result: &mut CompactedItems, base: u32, first: usize, second: usize) {
    let first_offset = inventory_offset(first);
    let second_offset = inventory_offset(second);
    let saved_first: [u8; INVENTORY_BYTES] = result.state.warrior.bytes
        [first_offset..first_offset + INVENTORY_BYTES]
        .try_into()
        .unwrap();
    let saved_second: [u8; INVENTORY_BYTES] = result.state.warrior.bytes
        [second_offset..second_offset + INVENTORY_BYTES]
        .try_into()
        .unwrap();
    result.state.warrior.bytes[first_offset..first_offset + INVENTORY_BYTES]
        .copy_from_slice(&saved_second);
    copy_effects(
        &mut result.effects,
        base + first_offset as u32,
        &saved_second,
    );
    result.state.warrior.bytes[second_offset..second_offset + INVENTORY_BYTES]
        .copy_from_slice(&saved_first);
    copy_effects(
        &mut result.effects,
        base + second_offset as u32,
        &saved_first,
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightState {
    pub warrior: Warrior,
    pub item_work: [u8; ITEM_BYTES],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalculatedWeight {
    pub state: WeightState,
    pub effects: Vec<InventoryEffect>,
    pub native_return: i32,
}

/// calcw's member is an unsigned WORD used by MULU.W; weight is a wrapping WORD.
pub fn calcw(
    state: &WeightState,
    tables: &ItemTables<'_>,
    member: u16,
) -> Result<CalculatedWeight, RuleError> {
    require_warrior(&state.warrior)?;
    let count = word(&state.warrior.bytes, 0x1a)?;
    if count > 100 {
        return Err(unavailable(
            "inventory.weight",
            "inventory count addresses outside 100 records",
        ));
    }
    let base = PARTY_BASE + u32::from(member) * WARRIOR_BYTES as u32;
    let mut result = CalculatedWeight {
        state: state.clone(),
        effects: Vec::new(),
        native_return: 0,
    };
    write_warrior(
        &mut result.state.warrior,
        &mut result.effects,
        base,
        0x9d6,
        &[0, 0],
    );
    for slot in 0..count.max(0) as usize {
        let offset = inventory_offset(slot);
        let id = word(&result.state.warrior.bytes, offset)?;
        let loaded = loaditem(tables, &result.state.item_work, id)?;
        result.state.item_work = loaded.bytes;
        result
            .effects
            .extend(loaded.writes.into_iter().map(InventoryEffect::Write));
        let quantity = word(&result.state.warrior.bytes, offset + 2)?;
        let unit = word(&result.state.item_work, 0x24)?;
        let base_weight = word(&result.state.item_work, 0x10)?;
        let contribution = unit.wrapping_mul(quantity).wrapping_add(base_weight);
        let weight = word(&result.state.warrior.bytes, 0x9d6)?.wrapping_add(contribution);
        write_warrior(
            &mut result.state.warrior,
            &mut result.effects,
            base,
            0x9d6,
            &weight.to_be_bytes(),
        );
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoldState {
    pub party_gold: i32,
    pub highest_party_index: i8,
    pub warriors: Vec<Warrior>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TakenGold {
    pub state: GoldState,
    pub effects: Vec<InventoryEffect>,
    pub native_return: i32,
}

/// takegold reads a signed WORD cost and uses the native doubled WORD token check.
pub fn takegold(state: &GoldState, amount: i16) -> Result<TakenGold, RuleError> {
    let mut result = TakenGold {
        state: state.clone(),
        effects: Vec::new(),
        native_return: 1,
    };
    if i32::from(amount) < state.party_gold {
        result.state.party_gold = state.party_gold.wrapping_sub(i32::from(amount));
        effect_write(
            &mut result.effects,
            0x442788,
            &result.state.party_gold.to_be_bytes(),
        );
        return Ok(result);
    }
    let mut total = 0_u16;
    for member in 0..=i16::from(state.highest_party_index) {
        let warrior = gold_warrior(state, member as usize)?;
        let tokens = word(&warrior.bytes, 0x9da)? as u16;
        total = total.wrapping_add(tokens).wrapping_add(tokens);
    }
    let available = state.party_gold.wrapping_add(i32::from(total));
    if available < i32::from(amount) {
        result.effects.push(InventoryEffect::Warning(50));
        result.native_return = 0;
        return Ok(result);
    }
    let mut remaining = amount.wrapping_sub(state.party_gold as i16);
    result.state.party_gold = 0;
    effect_write(&mut result.effects, 0x442788, &[0; 4]);
    let mut member = 0_i8;
    let mut decremented_this_cycle = false;
    loop {
        let index = usize::try_from(member).map_err(|_| {
            unavailable(
                "inventory.gold",
                "round-robin BYTE addressed before party table",
            )
        })?;
        let warrior = gold_warrior(&result.state, index)?;
        let tokens = word(&warrior.bytes, 0x9da)?;
        if tokens != 0 {
            remaining = remaining.wrapping_sub(1);
            let bytes = tokens.wrapping_sub(1).to_be_bytes();
            write_warrior(
                &mut result.state.warriors[index],
                &mut result.effects,
                PARTY_BASE + index as u32 * WARRIOR_BYTES as u32,
                0x9da,
                &bytes,
            );
            decremented_this_cycle = true;
        }
        member = member.wrapping_add(1);
        if member > state.highest_party_index {
            member = 0;
            if remaining != 0 && !decremented_this_cycle {
                return Err(unavailable(
                    "inventory.gold",
                    "native token loop cannot finish after token exhaustion",
                ));
            }
            decremented_this_cycle = false;
        }
        // This is a do-while: an initially zero remainder still visits member zero once.
        if remaining == 0 {
            break;
        }
    }
    Ok(result)
}

fn gold_warrior(state: &GoldState, member: usize) -> Result<&Warrior, RuleError> {
    let warrior = state
        .warriors
        .get(member)
        .ok_or_else(|| unavailable("inventory.gold", "party record outside supplied table"))?;
    require_warrior(warrior)?;
    Ok(warrior)
}

fn require_warrior(warrior: &Warrior) -> Result<(), RuleError> {
    if warrior.bytes.len() != WARRIOR_BYTES {
        return Err(unavailable(
            "inventory.record",
            "requires a complete warrior record",
        ));
    }
    Ok(())
}

fn write_warrior(
    warrior: &mut Warrior,
    effects: &mut Vec<InventoryEffect>,
    base: u32,
    offset: usize,
    bytes: &[u8],
) {
    warrior.bytes[offset..offset + bytes.len()].copy_from_slice(bytes);
    effect_write(effects, base + offset as u32, bytes);
}

fn copy_effects(effects: &mut Vec<InventoryEffect>, base: u32, record: &[u8; INVENTORY_BYTES]) {
    for (index, bytes) in record[..20].as_chunks::<4>().0.iter().enumerate() {
        effect_write(effects, base + index as u32 * 4, bytes);
    }
    effect_write(effects, base + 20, &record[20..]);
}

fn effect_write(effects: &mut Vec<InventoryEffect>, address: u32, bytes: &[u8]) {
    effects.push(InventoryEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
}

fn unavailable(action: &'static str, dependency: &'static str) -> RuleError {
    RuleError::Unsupported { action, dependency }
}
