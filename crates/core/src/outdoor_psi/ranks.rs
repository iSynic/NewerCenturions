//! Target ranks count down as targets are chosen, and which choices are valid.

/// Whether the target ranks are a legal state for a choice with this limit and number of picks
/// remaining: only party members hold ranks, and the ranks already given are distinct and count
/// down from the limit (from -1 downward when the limit is zero).
pub(super) fn valid(mask: &[i16; 6], members: usize, limit: i16, remaining: i16) -> bool {
    if limit >= members as i16 || mask[members..].iter().any(|rank| *rank != 0) {
        return false;
    }
    if limit == 0 {
        if remaining == 0 {
            return mask.iter().all(|rank| *rank == 0);
        }
        return (-(members as i16 + 1)..=-1).contains(&remaining)
            && mask.iter().all(|rank| *rank <= 0 && *rank > remaining)
            && (remaining + 1..0).all(|rank| mask.iter().filter(|r| **r == rank).count() == 1);
    }
    (1..=limit).contains(&remaining)
        && mask.iter().all(|rank| *rank >= 0 && *rank <= limit)
        && (remaining + 1..=limit).all(|rank| mask.iter().filter(|r| **r == rank).count() == 1)
        && mask.iter().all(|rank| *rank == 0 || *rank > remaining)
}

/// Picks or unpicks a warrior as a target. A pick takes the next rank and returns the sound 141;
/// unpicking closes the gap in the ranks and returns the sound 144.
pub(super) fn toggle(mask: &mut [i16], remaining: &mut i16, warrior: usize) -> i16 {
    let old = mask[warrior];
    if old == 0 {
        mask[warrior] = *remaining;
        *remaining = remaining.wrapping_sub(1);
        141
    } else {
        mask[warrior] = 0;
        for rank in mask {
            if *rank != 0 && *rank < old {
                *rank = rank.wrapping_add(1);
            }
        }
        *remaining = remaining.wrapping_add(1);
        144
    }
}
