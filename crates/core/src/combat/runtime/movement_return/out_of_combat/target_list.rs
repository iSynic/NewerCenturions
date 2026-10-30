//! The seven-entry target list and the entries past it.
//!
//! The firing loop writes targets without a bound, and the cast reads each back when
//! it reaches it. Targets from 7 land on killmon, then the attack packet, then the
//! spell data (see docs/source-notes/combat.md).
use crate::Session;
use crate::combat::spells::SpellTarget;

const LIST: i64 = 0x443722;
const TRIPLES: usize = 7;
const DEFEATED: i64 = 0x44374c;
const PACKET: std::ops::Range<i64> = 0x44374e..0x443788;

/// The word `field` of target triple `index` in the firing loop's unbounded list.
fn word_address(index: usize, field: i64) -> i64 {
    LIST + 6 * index as i64 + 2 * field
}

/// Whether triple `index` lies past the seven list triples.
pub(super) fn aliased(index: usize) -> bool {
    index >= TRIPLES
}

/// Reads a word that the list reaches beyond its seven entries: the defeated count or the attack
/// packet. Other words are not kept.
fn read(state: &Session, address: i64) -> Option<i16> {
    if address == DEFEATED {
        return Some(state.globals.kill_counts.kill_mon);
    }
    if !PACKET.contains(&address) {
        return None;
    }
    let offset = (address - PACKET.start) as usize;
    let packet = state.globals.spell_info.get(offset..offset + 2)?;
    Some(i16::from_be_bytes([packet[0], packet[1]]))
}

/// Writes such a word; false when it lies beyond the defeated count and the attack packet.
fn write(state: &mut Session, address: i64, value: i16) -> bool {
    if address == DEFEATED {
        state.globals.kill_counts.kill_mon = value;
        return true;
    }
    if !PACKET.contains(&address) {
        return false;
    }
    let offset = (address - PACKET.start) as usize;
    let Some(packet) = state.globals.spell_info.get_mut(offset..offset + 2) else {
        return false;
    };
    packet.copy_from_slice(&value.to_be_bytes());
    true
}

/// Writes an aliased target's first coordinate and kind in the firing loop's order;
/// the second coordinate keeps what lies there. False when either lies past the packet.
pub(super) fn store(state: &mut Session, index: usize, first: i16, kind: i16) -> bool {
    write(state, word_address(index, 0), first) && write(state, word_address(index, 2), kind)
}

/// Reads an aliased triple through its owners when the cast reaches it.
pub(super) fn load(state: &Session, index: usize) -> Option<SpellTarget> {
    Some(SpellTarget {
        first: read(state, word_address(index, 0))?,
        second: read(state, word_address(index, 1))?,
        kind: read(state, word_address(index, 2))?,
    })
}
