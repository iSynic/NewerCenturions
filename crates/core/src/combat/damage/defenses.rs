use crate::RandomState;
use centurions_content::records::{DamageDefenses, DamageDefinition, DamageTargetKind};

const PSIONIC: usize = 6;
const ELECTRICAL: usize = 5;
const BIOLOGICAL: usize = 8;

pub(super) struct ShieldAbsorption {
    pub remaining_charge: Option<i16>,
    pub absorbed: bool,
}

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
    // NPC shields continue drawing after exhaustion; warrior shields terminate
    // all remaining channels. Each channel retains at least one damage point.
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

pub(super) fn apply_resistances(
    components: &mut [i16; 9],
    definition: &DamageDefinition,
    attack_count: i16,
    defenses: &DamageDefenses,
    previous_amount: i16,
    random: &mut RandomState,
) {
    let enemy = defenses.kind == DamageTargetKind::Enemy;
    if definition.saving_throw_mode < 2 {
        let bonus = i16::from(definition.saving_throw_bonus_per_attack)
            .wrapping_mul(attack_count)
            .wrapping_add(i16::from(definition.saving_throw_bonus));
        for (element, damage) in components[2..].iter_mut().enumerate() {
            if *damage != 0 {
                let save = defenses.saves[element];
                // Preserve the draw even when an NPC save over 99 guarantees it.
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
