//! Damage rolls and defenses, in the original effect and RNG order.
mod defenses;

use crate::{RandomState, RuleError};
use centurions_content::WARRIOR_BYTES;
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::records::{
    DamageDefenses, DamageDefinition, DamageTargetKind, DamageTargetRecord,
};

/// The damage icon for each channel; the first channel that deals damage picks
/// it. Physical and per-attack damage share icon 157.
const DAMAGE_CHANNEL_CUES: [i16; 9] = [157, 157, 159, 160, 161, 162, 163, 164, 159];

/// The damage record of an attack and the number of attacks it makes.
#[derive(Clone, Debug)]
pub struct DamagePacket {
    pub bytes: Vec<u8>,
    pub attack_count: i16,
}

/// Which defenses apply to a damage roll: shield, armor, saving throws and the armor filter.
#[derive(Clone, Copy, Debug, Default)]
pub struct DamageFlags {
    pub shield: bool,
    pub armor: bool,
    pub saves: bool,
    pub armor_filter: bool,
}

/// The damage dealt, split into its nine channels, with the cue for the first channel that hurt and
/// whether a shield absorbed any of it.
#[derive(Debug, PartialEq, Eq)]
pub struct DamageResult {
    pub total: i16,
    pub components: [i16; 9],
    pub sound: i16,
    pub shield_sound: bool,
}

/// Resolves damage with a previous amount of zero.
pub fn resolve_damage(
    target: &mut [u8],
    monster: bool,
    packet: &DamagePacket,
    flags: DamageFlags,
    random: &mut RandomState,
) -> Result<DamageResult, RuleError> {
    resolve_damage_with_amount(target, monster, packet, flags, 0, random)
}

/// Resolves damage. A monster's negative save works from the caller's previous
/// amount, not the current one (see docs/source-notes/combat.md).
///
/// Original: resolvedamage
pub fn resolve_damage_with_amount(
    target: &mut [u8],
    monster: bool,
    packet: &DamagePacket,
    flags: DamageFlags,
    previous_amount: i16,
    random: &mut RandomState,
) -> Result<DamageResult, RuleError> {
    if packet.bytes.len() != 58
        || target.len()
            != if monster {
                MONSTER_BYTES
            } else {
                WARRIOR_BYTES
            }
    {
        return Err(RuleError::Invalid("damage record size".into()));
    }
    let definition = DamageDefinition::read(&packet.bytes)?;
    let kind = if monster {
        DamageTargetKind::Enemy
    } else {
        DamageTargetKind::Warrior
    };
    let mut target = DamageTargetRecord::new(target, kind);
    let defenses = target.defenses()?;
    let mut components = roll_damage(&definition, packet.attack_count, &defenses, random);
    let rolled = components;
    let armor = defenses.armor.min(95);
    if flags.armor_filter {
        defenses::filter_armor(&mut components, armor, random);
    }
    // Healing still rolls damage and performs the armor-filter pass, but skips
    // shields, ordinary armor, saves and the damaged marker.
    if definition.healing {
        let total = sum(&components).wrapping_neg();
        crate::trace::damage(&rolled, &components, total);
        return Ok(DamageResult {
            total,
            components,
            sound: 0,
            shield_sound: false,
        });
    }
    let mut shield_sound = false;
    if flags.shield && defenses.shield_available {
        let shield = defenses::absorb_shield(&mut components, &defenses, random);
        if let Some(charge) = shield.remaining_charge {
            target.set_shield_charge(charge);
        }
        shield_sound = shield.absorbed;
    }
    if flags.armor {
        defenses::absorb_armor(&mut components, armor, random);
    }
    if flags.saves {
        defenses::apply_resistances(
            &mut components,
            &definition,
            packet.attack_count,
            &defenses,
            previous_amount,
            random,
        );
    }
    let first = components.iter().position(|damage| *damage != 0);
    let sound = first.map_or(0, |channel| DAMAGE_CHANNEL_CUES[channel]);
    if first.is_some() {
        target.mark_been_attacked();
    }
    let total = sum(&components);
    crate::trace::damage(&rolled, &components, total);
    Ok(DamageResult {
        total,
        components,
        sound,
        shield_sound,
    })
}

/// Rolls each damage channel: one flat physical roll, a per-attack roll summed over the attacks,
/// and one roll per element. A monster's damage is multiplied by its size when the attack scales
/// with it.
///
/// Original: resolvedamage
fn roll_damage(
    definition: &DamageDefinition,
    attack_count: i16,
    defenses: &DamageDefenses,
    random: &mut RandomState,
) -> [i16; 9] {
    let mut components = [0_i16; 9];
    let (minimum, maximum) = definition.physical;
    if maximum != 0 {
        components[0] = random.range(minimum, maximum);
    }
    let (minimum, maximum) = definition.per_attack;
    for _ in 0..attack_count {
        if maximum != 0 {
            components[1] = components[1].wrapping_add(random.range(minimum, maximum));
        }
    }
    for (element, maximum) in definition.elemental_maximum.iter().enumerate() {
        if *maximum != 0 {
            components[element + 2] = random.bound(i16::from(*maximum));
        }
    }
    if defenses.kind == DamageTargetKind::Enemy && definition.scale_by_target_size {
        for damage in &mut components {
            *damage = damage.wrapping_mul(defenses.size.wrapping_abs());
        }
    }
    components
}

/// Totals the channels.
fn sum(components: &[i16; 9]) -> i16 {
    components
        .iter()
        .fold(0_i16, |total, damage| total.wrapping_add(*damage))
}
