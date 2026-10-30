use crate::{RandomState, RuleError};
use centurions_content::records::{RegionTable, RegionTableMut};

/// Picks the script of a manual encounter. Each region holding the party's square and having a
/// negative encounter chance rolls its events, and the last event to pass gives the script. A
/// passing event with a positive chance is used up.
pub fn select(
    region_state: &mut [u8],
    position: (i32, i32),
    random: &mut RandomState,
) -> Result<i16, RuleError> {
    if region_state.len() != RegionTable::BYTES {
        return Err(RuleError::Unsupported {
            action: "story.manual_encounter",
            dependency: "regional record must contain 644 bytes",
        });
    }
    let mut regions = region_state.to_vec();
    let mut rng = random.clone();
    let (x, y) = (position.0 as i16, position.1 as i16);
    let mut script = 0;
    for region in (0..RegionTable::REGIONS).rev() {
        let view = RegionTable(&regions);
        if y < view.first_y(region)?
            || x < view.first_x(region)?
            || y >= view.last_y(region)?
            || x >= view.last_x(region)?
            // The manual search admits only regions whose encounter chance is negative.
            || view.encounter_chance(region)? >= 0
        {
            continue;
        }
        for slot in 0..RegionTable::EVENTS {
            let chance = RegionTable(&regions).event_chance(region, slot)?;
            if rng.bound(100) <= chance.wrapping_abs() {
                script = RegionTable(&regions).event_script(region, slot)?;
                if chance > 0 {
                    RegionTableMut(&mut regions).clear_event_chance(region, slot);
                }
            }
        }
    }
    region_state.copy_from_slice(&regions);
    *random = rng;
    Ok(script)
}
