//! Enhanced stock generation (installitem 0x12c874, randomitem 0x12d558).
//! Random quantity selection precedes modifier rolls even though this caller
//! installs fresh quantities afterward. No scratch address space is required.
use crate::{RandomState, RuleError};
use centurions_content::{
    InventoryRecord,
    records::{EquipmentModifiers, InventoryModifier as Modifier, ItemDefinition, ItemUpgrade},
};

enum Increment {
    Fixed(i16),
    Random { bound: i16, base: i16 },
}

/// Every draw belongs to a finite attempt count, so stock generation always ends.
struct Rolls {
    state: RandomState,
}
impl Rolls {
    fn draw(&mut self, bound: i16) -> Result<i16, RuleError> {
        if self.state.draws == u64::MAX {
            return Err(RuleError::Invalid("random-draw counter overflow".into()));
        }
        Ok(self.state.bound(bound))
    }
    fn improve(
        &mut self,
        record: &mut InventoryRecord,
        modifier: Modifier,
        attempts: i16,
        threshold: i16,
        increment: Increment,
        changes: &mut i16,
    ) -> Result<(), RuleError> {
        for _ in 0..attempts {
            if self.draw(100)? < threshold {
                let amount = match increment {
                    Increment::Random { bound, base } => self.draw(bound)?.wrapping_add(base),
                    Increment::Fixed(amount) => amount,
                };
                modifier.add(record, amount as i8);
                *changes = changes.wrapping_add(1);
            }
        }
        Ok(())
    }
}

pub(super) fn generate(
    items: &[Vec<u8>],
    id: i16,
    enhancement: i16,
    difficulty: i16,
    random: &mut RandomState,
) -> Result<InventoryRecord, RuleError> {
    let bytes = items
        .get(usize::from(id.unsigned_abs()))
        .ok_or_else(|| RuleError::Invalid("generated stock item outside the item table".into()))?;
    let definition = ItemDefinition(bytes);
    let mut installed = InventoryRecord([0; 22]);
    definition.install_fresh_into(&mut installed)?;
    if enhancement == 0 {
        return Ok(installed);
    }
    let mut rolls = Rolls {
        state: random.clone(),
    };
    let generated = randomize(items, definition, enhancement, difficulty, &mut rolls)?;
    // The caller copies level and these eight modifiers, but not melee accuracy,
    // generated ammunition, charge or capacity. Those retain fresh-stock values.
    for modifier in [
        Modifier::RequiredLevel,
        Modifier::ShieldChance,
        Modifier::Accuracy,
        Modifier::MissileDodge,
        Modifier::MeleeDodge,
        Modifier::Damage,
        Modifier::Armor,
        Modifier::ShieldCost,
        Modifier::Range,
    ] {
        modifier.write(&mut installed, modifier.read(&generated));
    }
    *random = rolls.state;
    Ok(installed)
}

fn randomize(
    items: &[Vec<u8>],
    definition: ItemDefinition<'_>,
    mut enhancement: i16,
    difficulty: i16,
    rolls: &mut Rolls,
) -> Result<InventoryRecord, RuleError> {
    let base = ItemUpgrade::read(definition.0)?;
    let equipment = EquipmentModifiers::read(definition.0)?;
    let kind = definition.category()?.wrapping_abs();
    let mut item = InventoryRecord([0; 22]);
    for (modifier, value) in [
        (Modifier::RequiredLevel, definition.required_level()? as i8),
        (Modifier::ShieldChance, base.shield_chance),
        (Modifier::Accuracy, base.accuracy),
        (Modifier::MeleeAccuracy, definition.base_melee_accuracy()),
        (Modifier::MissileDodge, base.missile_dodge),
        (Modifier::MeleeDodge, base.melee_dodge),
        (Modifier::Damage, base.damage),
        (Modifier::Armor, base.armor),
        (Modifier::ShieldCost, base.shield_cost),
        (Modifier::Range, base.range),
    ] {
        modifier.write(&mut item, value);
    }
    let mut quantity = 0;
    let mut capacity = 0;
    if definition.battery_capacity()? != 0 {
        capacity = definition
            .capacity_bonus()?
            .wrapping_add(definition.battery_capacity()?);
    } else if definition.magazine_capacity()? != 0 || definition.supplied_quantity()? > 0 {
        capacity = definition
            .magazine_capacity()?
            .wrapping_add(definition.supplied_quantity()?);
        quantity = definition.supplied_quantity()?;
    }
    capacity = capacity.max(quantity);
    if capacity > 0 && [15, 13, 14].contains(&(kind as u8)) {
        if rolls.draw(100)? < difficulty.wrapping_mul(10).wrapping_add(50) {
            rolls.draw(capacity)?;
            // Original selection reads the ammunition table before enhancements.
            if definition.ammunition_caliber()? != 14 {
                for index in 200..400 {
                    let ammo = items.get(index).ok_or(RuleError::Invalid(
                        "ammunition table entry outside the item table".into(),
                    ))?;
                    if definition.ammunition_caliber()?
                        == ItemDefinition(ammo).reserve_ammunition_kind()?
                    {
                        break;
                    }
                }
            }
        }
    } else if quantity > 0 {
        rolls.draw(capacity)?;
    }
    let threshold = 94_i16.wrapping_sub(difficulty);
    if rolls.draw(100)? < threshold && enhancement == 0 {
        return Ok(item);
    }
    let scale = difficulty.wrapping_add(3);
    let mut changes = 0_i16;
    let mut enhancement_level = enhancement as i8;
    loop {
        if kind == 26 || kind == 27 || definition.unit_weight()? != 0 {
            return Ok(item);
        }
        if definition
            .missile_damage()?
            .wrapping_add(definition.melee_damage()?)
            != 0
            || kind == 2
            || kind == 15
        {
            let accuracy = if definition.melee_damage()? == 0 {
                Modifier::Accuracy
            } else {
                Modifier::MeleeAccuracy
            };
            rolls.improve(
                &mut item,
                accuracy,
                scale.wrapping_mul(5),
                7,
                Increment::Random { bound: 5, base: 0 },
                &mut changes,
            )?;
            rolls.improve(
                &mut item,
                Modifier::Damage,
                scale.wrapping_shl(2),
                5,
                Increment::Random { bound: 6, base: 4 },
                &mut changes,
            )?;
        } else if base.accuracy != 0 {
            rolls.improve(
                &mut item,
                Modifier::Accuracy,
                scale.wrapping_mul(5),
                7,
                Increment::Random { bound: 3, base: 0 },
                &mut changes,
            )?;
        } else if base.damage != 0 {
            rolls.improve(
                &mut item,
                Modifier::Damage,
                scale.wrapping_mul(5),
                5,
                Increment::Random { bound: 6, base: 4 },
                &mut changes,
            )?;
        }
        for (maximum, modifier) in [
            (equipment.shield_chance, Modifier::ShieldChance),
            (equipment.armor, Modifier::Armor),
        ] {
            if maximum > 0 {
                rolls.improve(
                    &mut item,
                    modifier,
                    maximum.wrapping_mul(scale),
                    2,
                    Increment::Fixed(1),
                    &mut changes,
                )?;
                if i16::from(modifier.read(&item)) > maximum {
                    modifier.write(&mut item, maximum as i8);
                }
            }
        }
        for (maximum, modifier) in [
            (equipment.missile_dodge, Modifier::MissileDodge),
            (equipment.melee_dodge, Modifier::MeleeDodge),
        ] {
            if maximum > 0 {
                rolls.improve(
                    &mut item,
                    modifier,
                    scale.wrapping_shl(2),
                    4,
                    Increment::Random { bound: 3, base: 0 },
                    &mut changes,
                )?;
                if i16::from(modifier.read(&item)) > maximum {
                    modifier.write(&mut item, maximum as i8);
                }
            }
        }
        if equipment.range > 0 {
            rolls.improve(
                &mut item,
                Modifier::Range,
                scale.wrapping_add(scale),
                4,
                Increment::Fixed(1),
                &mut changes,
            )?;
            if i16::from(Modifier::Range.read(&item)) > equipment.range {
                Modifier::Range.write(&mut item, base.range);
            }
        }
        if i16::from(enhancement_level) < enhancement && changes != 0 {
            enhancement_level = changes as i8;
        }
        if enhancement_level == 0 && changes != 0 {
            enhancement_level = 1;
        }
        Modifier::RequiredLevel.write(
            &mut item,
            i16::from(enhancement_level).wrapping_add(definition.required_level()?) as i8,
        );
        enhancement = enhancement.wrapping_sub(1);
        if enhancement <= 0 {
            break;
        }
    }
    Ok(item)
}
