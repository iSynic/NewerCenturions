//! Borrowed views over original records. These never change their serialized storage.
//!
//! Field names follow their callers in Centurions, not the superficially similar
//! Realmz layouts. Reads retain signed big-endian words; writes touch only the
//! specified bytes. Unknown fields stay opaque.
mod attack;
mod item;
mod monster;
mod psi_memory;
mod shop;
mod story;
mod upgrade;
mod warrior;

pub use attack::{AttackPacket, AttackPacketMut};
pub use item::ItemDefinition;
pub use monster::{EnemyRecord, EnemyRecordMut};
pub use psi_memory::PsiSelectionMemoryMut;
pub use shop::{SHOP_BYTES, SHOP_SLOTS, ShopRecord, ShopRecordMut};
pub use story::{
    FatigueChange, ObjectRectangleExtra, RecoveryRestrictions, StoryRecord, StoryRecordMut,
    TravelExtra,
};
pub use upgrade::UpgradeRecord;
pub use warrior::{WarriorRecord, WarriorRecordMut};

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}
