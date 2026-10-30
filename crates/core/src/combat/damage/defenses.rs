use crate::RandomState;
use centurions_content::records::{DamageDefenses, DamageDefinition, DamageTargetKind};

/// The psionic channel, which armor does not reduce.
const PSIONIC: usize = 6;
/// The electrical channel, doubled for warriors of race 3.
const ELECTRICAL: usize = 5;
/// The biological channel, nullified for warriors of race 3 and for monsters immune to it.
const BIOLOGICAL: usize = 8;

/// What a shield did to a hit: the charge it has left, if it could act, and whether it absorbed
/// anything.
pub(super) struct ShieldAbsorption {
    pub remaining_charge: Option<i16>,
    pub absorbed: bool,
}

/// Armor filter: each point of every channel survives only if a draw of 100 beats the armor.
///
/// Original: resolvedamage
pub(super) fn filter_armor(components: &mut [i16; 9], armor: i16, random: &mut RandomState) {
    for damage in components {
        let mut kept = 0_i16;
        for _ in 0..*damage {
            if random.bound(100) > armor {
                kept = kept.wrapping_add(1);
            }
        }
        *damage = kept;
    }
}

/// A shield takes a point off any channel with more than one point, by its chance, paying its cost
/// from its charge. A monster's spent shield keeps drawing; a warrior's stops every remaining
/// channel.
///
/// Original: resolvedamage
pub(super) fn absorb_shield(
    components: &mut [i16; 9],
    defenses: &DamageDefenses,
    random: &mut RandomState,
) -> ShieldAbsorption {
    let enemy = defenses.kind == DamageTargetKind::Enemy;
    let chance = defenses.shield_chance.min(95);
    let mut charge = defenses.shield_charge;
    let mut cost = defenses.shield_cost;
    if enemy && chance != 0 && charge != 0 && cost == 0 {
        cost = 1;
    }
    if !enemy && (charge == 0 || charge < cost) {
        return ShieldAbsorption {
            remaining_charge: None,
            absorbed: false,
        };
    }
    let mut absorbed = false;
    // A monster's shields keep drawing after they are spent; a warrior's spent
    // shields stop every remaining channel. Each channel keeps at least one point.
    'shield: for damage in components {
        let initial = *damage;
        for _ in 0..initial {
            if random.bound(100) < chance && charge >= cost && *damage > 1 {
                *damage = damage.wrapping_sub(1);
                charge = charge.wrapping_sub(cost);
                absorbed = true;
            } else if !enemy && charge < cost {
                break 'shield;
            }
        }
    }
    ShieldAbsorption {
        remaining_charge: Some(charge),
        absorbed,
    }
}

/// Armor takes a point off any channel with more than one point, by its rating. Psionic damage
/// ignores it.
///
/// Original: resolvedamage
pub(super) fn absorb_armor(components: &mut [i16; 9], armor: i16, random: &mut RandomState) {
    for (channel, damage) in components.iter_mut().enumerate() {
        let initial = if channel == PSIONIC { 0 } else { *damage };
        for _ in 0..initial {
            if random.bound(100) < armor && *damage > 1 {
                *damage = damage.wrapping_sub(1);
            }
        }
    }
}

/// Halves each elemental channel the target saves against (unless the attack allows no save) or is
/// protected from. A monster's negative resistance replaces the channel with the earlier amount
/// times 1 plus the resistance's size in hundredths, and a resistance over 100 cancels it. Then a
/// monster immune to biological damage takes none of it, and a warrior of race 3 takes none of it
/// and double electrical damage.
///
/// Original: resolvedamage
pub(super) fn apply_resistances(
    components: &mut [i16; 9],
    definition: &DamageDefinition,
    attack_count: i16,
    defenses: &DamageDefenses,
    previous_amount: i16,
    random: &mut RandomState,
) {
    let enemy = defenses.kind == DamageTargetKind::Enemy;
    if definition.cannot < 2 {
        let bonus = i16::from(definition.saving_throw_bonus_per_attack)
            .wrapping_mul(attack_count)
            .wrapping_add(i16::from(definition.saving_throw_bonus));
        for (element, damage) in components[2..].iter_mut().enumerate() {
            if *damage != 0 {
                let save = defenses.saves[element];
                // The draw is taken even when a monster's save over 99 is certain.
                if random.bound(100) <= save.wrapping_add(bonus) || enemy && save > 99 {
                    *damage /= 2;
                }
            }
        }
    }
    for (element, damage) in components[2..].iter_mut().enumerate() {
        if defenses.protected[element] {
            *damage /= 2;
        }
    }
    if enemy {
        for (element, damage) in components[2..].iter_mut().enumerate() {
            let resistance = defenses.saves[element];
            if *damage != 0 && resistance < 0 {
                let multiplier = f32::from(resistance.wrapping_abs()) / 100. + 1.;
                *damage = ((f32::from(previous_amount) * multiplier) as i32) as i16;
            }
            if resistance > 100 {
                *damage = 0;
            }
        }
        if defenses.biological_immunity {
            components[BIOLOGICAL] = 0;
        }
    } else if defenses.race == Some(3) {
        components[BIOLOGICAL] = 0;
        components[ELECTRICAL] = components[ELECTRICAL].wrapping_mul(2);
    }
}
