//! installitem 0x12c874 adapter for enhanced shop stock. The native memory
//! image and work-buffer addresses are confined here; catalog callers pass
//! game concepts and retain the original RNG commit order.
use crate::{RandomState, RuleError};
use centurions_content::{InventoryRecord, span};

pub(super) fn generate(
    items: &[Vec<u8>],
    id: i16,
    enhancement: i16,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<InventoryRecord, RuleError> {
    use crate::recovered_install::{InstallMemory, InstallRequest};
    let base = 0x4046e6;
    let mut memory = InstallMemory {
        base_address: base,
        bytes: vec![0; 0x4781ee - base as usize],
    };
    memory.bytes[..2].copy_from_slice(&enhancement.to_be_bytes());
    let offset = (0x442576 - base) as usize;
    memory.bytes[offset..offset + 2].copy_from_slice(&difficulty.to_be_bytes());
    let flat = items.iter().flatten().copied().collect::<Vec<_>>();
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
        random,
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
    *random = result.random;
    Ok(InventoryRecord(
        span(&result.memory.bytes, (0x44734e - base) as usize, 22)?
            .try_into()
            .unwrap(),
    ))
}
