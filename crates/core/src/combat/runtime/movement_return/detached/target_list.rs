//! The seven-triple target list at 443722 and the triples past it.
//!
//! combatchoice's firing loop (11c286..11c39e) writes triple `n` at 443722 + 6n without
//! a bound, and cast 10d4fe reads each triple back when it reaches it. Triples 7.. land
//! on the defeated WORD 44374c, then the attack packet 44374e..443788; from 443788 they
//! reach Data S.
use crate::Session;
use crate::combat::spells::SpellTarget;

const LIST: i64 = 0x443722;
const TRIPLES: usize = 7;
const DEFEATED: i64 = 0x44374c;
const PACKET: std::ops::Range<i64> = 0x44374e..0x443788;

fn word_address(index: usize, field: i64) -> i64 {
    LIST + 6 * index as i64 + 2 * field
}

/// Whether triple `index` lies past the seven list triples.
pub(super) fn aliased(index: usize) -> bool {
    index >= TRIPLES
}

fn read(state: &Session, address: i64) -> Option<i16> {
    if address == DEFEATED {
        return Some(state.native_work.victory_work.defeated);
    }
    if !PACKET.contains(&address) {
        return None;
    }
    let offset = (address - PACKET.start) as usize;
    let packet = state.native_work.attack_packet.get(offset..offset + 2)?;
    Some(i16::from_be_bytes([packet[0], packet[1]]))
}

fn write(state: &mut Session, address: i64, value: i16) -> bool {
    if address == DEFEATED {
        state.native_work.victory_work.defeated = value;
        return true;
    }
    if !PACKET.contains(&address) {
        return false;
    }
    let offset = (address - PACKET.start) as usize;
    let Some(packet) = state.native_work.attack_packet.get_mut(offset..offset + 2) else {
        return false;
    };
    packet.copy_from_slice(&value.to_be_bytes());
    true
}

/// Writes an aliased triple's first and kind WORDs in the firing loop's order; the
/// second WORD keeps what the alias holds. False when either WORD lies past the packet.
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
