use crate::{RuleError, combat};
use centurions_content::{Content, Warrior, span, word};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioItemUse {
    pub warriors: Vec<Warrior>,
    pub return_code: i16,
    pub script: Option<i16>,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScriptItemContext {
    Exploration,
    Encounter,
}

/// Items12ac14 warns for non-activating samples;12ad9a skips their effects.
pub(crate) fn reject_inert_item(definition: &[u8]) -> Result<(), RuleError> {
    let class = word(definition, 26)?.wrapping_abs();
    let effect = word(definition, 44)?;
    if class != 23 && effect <= 1100 {
        return Err(RuleError::Refused {
            feedback: if matches!(class, 24 | 25) && effect < 1100 {
                vec![
                    crate::feedback::NativeFeedback::Warning { index: 88 },
                    crate::feedback::NativeFeedback::Warning { index: 89 },
                ]
            } else {
                vec![]
            },
        });
    }
    Ok(())
}

/// items 12ac90 uses class 23 and buttonchoice 10a9f4 decodes its script return.
pub fn use_scenario_item(
    warriors: &[Warrior],
    member: usize,
    slot: usize,
    content: &Content,
    race_movements: &[i8],
    context: ScriptItemContext,
) -> Result<ScenarioItemUse, RuleError> {
    let warrior = warriors
        .get(member)
        .ok_or_else(|| RuleError::Invalid("selected item warrior unavailable".into()))?;
    let bytes = span(&warrior.bytes, 0, 3138)?;
    let count = usize::try_from(word(bytes, 0x1a)?)
        .map_err(|_| RuleError::Invalid("selected inventory count is negative".into()))?;
    if slot >= count || count > 100 {
        return Err(RuleError::Invalid(
            "selected inventory item unavailable".into(),
        ));
    }
    let offset = 0x132 + slot * 22;
    let id = word(bytes, offset)?;
    let definition = content
        .items
        .get(id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("selected item definition unavailable".into()))?;
    reject_inert_item(definition)?;
    if word(definition, 0x1a)?.wrapping_abs() != 23 {
        return Err(RuleError::Unsupported {
            action: "item.use",
            dependency: "non-script item use",
        });
    }
    let quantity = word(bytes, offset + 2)?;
    let quantity = if quantity > 0 { quantity - 1 } else { quantity };
    let remove = quantity == 0 && word(definition, 0x26)? != 0;
    if remove && (count > 30 || bytes[offset + 9] != 0) {
        return Err(RuleError::Unsupported {
            action: "item.use",
            dependency: "equipped item removal or extended inventory",
        });
    }
    let return_code = word(definition, 0x32)?.wrapping_add(100).wrapping_neg();
    let mut result = ScenarioItemUse {
        warriors: warriors.to_vec(),
        return_code,
        script: (return_code <= -100).then(|| return_code.wrapping_abs().wrapping_sub(100)),
        sounds: vec![],
        warnings: if context == ScriptItemContext::Exploration && word(definition, 0x2c)? < 1100 {
            vec![88, 89]
        } else {
            vec![]
        },
    };
    let bytes = &mut result.warriors[member].bytes;
    bytes[offset + 2..offset + 4].copy_from_slice(&quantity.to_be_bytes());
    let carried = word(bytes, 0x9d6)?.wrapping_sub(word(definition, 0x24)?);
    bytes[0x9d6..0x9d8].copy_from_slice(&carried.to_be_bytes());
    if remove {
        let weight =
            word(definition, 0x10)?.wrapping_add(word(definition, 0x24)?.wrapping_mul(quantity));
        let carried = word(bytes, 0x9d6)?.wrapping_sub(weight);
        bytes[0x9d6..0x9d8].copy_from_slice(&carried.to_be_bytes());
        for destination in slot..29 {
            let start = 0x132 + (destination + 1) * 22;
            bytes.copy_within(start..start + 22, 0x132 + destination * 22);
        }
        bytes[0x1a..0x1c].copy_from_slice(&((count - 1) as i16).to_be_bytes());
        bytes[0x132 + (count - 1) * 22..0x132 + 100 * 22].fill(0);
        let race = word(bytes, 0x32)?.wrapping_sub(1);
        let movement = race_movements
            .get(race as usize)
            .ok_or_else(|| RuleError::Invalid("selected item race movement unavailable".into()))?;
        combat::calculate_movement(&mut result.warriors[member], *movement)?;
        result.sounds.push(655);
    }
    Ok(result)
}
