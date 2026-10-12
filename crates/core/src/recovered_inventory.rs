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

#[cfg(test)]
mod tests {
    use super::*;

    fn warrior() -> Warrior {
        Warrior {
            name: "test".into(),
            original_sha256: "original".into(),
            bytes: vec![0x5a; WARRIOR_BYTES],
        }
    }

    fn put(warrior: &mut Warrior, offset: usize, value: i16) {
        warrior.bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }

    fn record(id: i16, equipped: u8, fill: u8) -> [u8; INVENTORY_BYTES] {
        let mut record = [fill; INVENTORY_BYTES];
        record[..2].copy_from_slice(&id.to_be_bytes());
        record[9] = equipped;
        record
    }

    #[test]
    fn banks_use_only_the_id_word_and_copy_all_bytes_into_the_first_available_slot() {
        let mut bank = vec![0x5a; BANK_BYTES];
        bank[22..24].fill(0); // The quantity WORD remains nonzero.
        let before = bank.clone();
        let item = record(-804, 0, 0x31);
        let booty = addtobooty(&bank, &item).unwrap();
        let crate_result = addtocrate(&bank, &item).unwrap();
        let mut expected = before.clone();
        expected[22..44].copy_from_slice(&item);
        assert_eq!(booty.bytes, expected);
        assert_eq!(crate_result.bytes, expected);
        assert_eq!(booty.slot, Some(1));
        assert_eq!(booty.effects.len(), 6);
        assert_eq!(crate_result.effects.len(), 7);
        assert_eq!(
            crate_result.effects.last(),
            Some(&InventoryEffect::Sound(663))
        );
        assert_eq!(crate_result.native_return, 1);
        assert_eq!(bank, before);
        bank[22..24].copy_from_slice(&1_i16.to_be_bytes());
        let full = addtocrate(&bank, &item).unwrap();
        assert_eq!(full.bytes, bank);
        assert_eq!(full.native_return, 0);
        assert!(full.effects.is_empty());
        assert!(addtobooty(&bank[..BANK_BYTES - 1], &item).is_err());
    }

    #[test]
    fn compact_preserves_whole_records_and_native_priority_swaps_and_tail_fields() {
        let mut original = warrior();
        put(&mut original, 0x1a, 5);
        put(&mut original, 0xe4, 165);
        put(&mut original, 0xca, 403);
        put(&mut original, 0xee, 460);
        let records = [
            record(1, 0, 11),
            record(460, 1, 12),
            record(403, 2, 13),
            record(700, 0, 14),
            record(165, 255, 15),
        ];
        for (slot, bytes) in records.iter().enumerate() {
            original.bytes[inventory_offset(slot)..inventory_offset(slot) + 22]
                .copy_from_slice(bytes);
        }
        let state = CompactState {
            warrior: original.clone(),
            highest_party_index: 5,
            scan: -7,
            pass: -8,
        };
        let result = compactitems(&state, 4).unwrap();
        let mut expected = original.clone();
        for (slot, source) in [4, 2, 1, 0, 3].into_iter().enumerate() {
            expected.bytes[inventory_offset(slot)..inventory_offset(slot) + 22]
                .copy_from_slice(&records[source]);
        }
        for slot in 5..100 {
            let offset = inventory_offset(slot);
            expected.bytes[offset..offset + 4].fill(0);
            expected.bytes[offset + 9] = 0;
        }
        expected.bytes[0x126..0x128].copy_from_slice(&records[4][4..6]);
        assert_eq!(result.state.warrior, expected);
        assert_eq!(result.state.scan, 5);
        assert_eq!(result.state.pass, 5);
        assert_eq!(state.warrior, original);
        assert_eq!(compactitems(&state, -1).unwrap().state, state);
        assert_eq!(compactitems(&state, 6).unwrap().state, state);
        let mut invalid = state.clone();
        put(&mut invalid.warrior, 0x1a, -1);
        assert!(compactitems(&invalid, 4).is_err());
    }

    #[test]
    fn calcw_loads_every_id_in_order_and_accumulates_word_products_without_widening() {
        let mut table = vec![0; 200 * ITEM_BYTES];
        for (id, base, unit) in [(1, 30000_i16, -3_i16), (2, 20000, 4)] {
            table[id * 150 + 16..id * 150 + 18].copy_from_slice(&base.to_be_bytes());
            table[id * 150 + 36..id * 150 + 38].copy_from_slice(&unit.to_be_bytes());
        }
        let tables = ItemTables {
            families: [&table, &[], &[], &[], &[]],
        };
        let mut party = warrior();
        put(&mut party, 0x1a, 2);
        put(&mut party, inventory_offset(0), 1);
        put(&mut party, inventory_offset(0) + 2, -5);
        put(&mut party, inventory_offset(1), 2);
        put(&mut party, inventory_offset(1) + 2, 10000);
        let state = WeightState {
            warrior: party.clone(),
            item_work: [0x6a; 150],
        };
        let result = calcw(&state, &tables, 4).unwrap();
        let weight = 30015_i16.wrapping_add(20000_i16.wrapping_add(4_i16.wrapping_mul(10000)));
        put(&mut party, 0x9d6, weight);
        assert_eq!(result.state.warrior, party);
        assert_eq!(result.state.item_work.as_slice(), &table[300..450]);
        assert_eq!(result.effects.len(), 81);
        assert_eq!(result.native_return, 0);
        let mut empty = state.clone();
        put(&mut empty.warrior, 0x1a, -1);
        let result = calcw(&empty, &tables, 4).unwrap();
        assert_eq!(word(&result.state.warrior.bytes, 0x9d6).unwrap(), 0);
        assert_eq!(result.state.item_work, empty.item_work);
        assert_eq!(result.effects.len(), 1);
        let bad = ItemTables { families: [&[]; 5] };
        assert!(calcw(&state, &bad, 4).is_err());
        assert_eq!(state.item_work, [0x6a; 150]);
    }

    #[test]
    fn gold_uses_word_availability_round_robin_and_reports_nonterminating_native_paths() {
        let mut warriors = vec![warrior(); 3];
        for (warrior, tokens) in warriors.iter_mut().zip([2, 0, 3]) {
            put(warrior, 0x9da, tokens);
        }
        let state = GoldState {
            party_gold: 3,
            highest_party_index: 2,
            warriors,
        };
        let direct = takegold(&state, -2).unwrap();
        assert_eq!(direct.state.party_gold, 5);
        assert_eq!(direct.state.warriors, state.warriors);
        let result = takegold(&state, 7).unwrap();
        assert_eq!(result.state.party_gold, 0);
        assert_eq!(
            result
                .state
                .warriors
                .iter()
                .map(|w| word(&w.bytes, 0x9da).unwrap())
                .collect::<Vec<_>>(),
            vec![0, 0, 1]
        );
        assert_eq!(result.effects.len(), 5);
        assert_eq!(result.native_return, 1);
        let denied = takegold(&state, 14).unwrap();
        assert_eq!(denied.state, state);
        assert_eq!(denied.effects, vec![InventoryEffect::Warning(50)]);
        assert_eq!(denied.native_return, 0);
        // The doubled precheck passes but only five tokens exist for a six-token remainder.
        assert!(takegold(&state, 9).is_err());
        // Equality still executes the do-while once and underflows when member zero has tokens.
        assert!(takegold(&state, 3).is_err());
        let mut zero = state.clone();
        put(&mut zero.warriors[0], 0x9da, 0);
        let equality = takegold(&zero, 3).unwrap();
        assert_eq!(equality.state.party_gold, 0);
        assert_eq!(equality.state.warriors, zero.warriors);
        assert_eq!(equality.effects.len(), 1);
    }
}
