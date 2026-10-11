use crate::{RandomState, RuleError};
use centurions_content::word;

pub fn select(
    region_state: &mut [u8],
    position: (i32, i32),
    random: &mut RandomState,
) -> Result<i16, RuleError> {
    if region_state.len() != 644 {
        return Err(RuleError::Unsupported {
            action: "story.manual_encounter",
            dependency: "regional record must contain 644 bytes",
        });
    }
    let mut regions = region_state.to_vec();
    let mut rng = random.clone();
    let (x, y) = (position.0 as i16, position.1 as i16);
    let mut script = 0;
    for region in (0..20).rev() {
        let bounds = region * 8;
        if y < word(&regions, bounds)?
            || x < word(&regions, bounds + 2)?
            || y >= word(&regions, bounds + 4)?
            || x >= word(&regions, bounds + 6)?
            || word(&regions, 160 + region * 2)? >= 0
        {
            continue;
        }
        for slot in 0..3 {
            let offset = region * 6 + slot * 2;
            let chance = word(&regions, 400 + offset)?;
            if rng.bound(100) <= chance.wrapping_abs() {
                script = word(&regions, 280 + offset)?;
                if chance > 0 {
                    regions[400 + offset..402 + offset].fill(0);
                }
            }
        }
    }
    region_state.copy_from_slice(&regions);
    *random = rng;
    Ok(script)
}
