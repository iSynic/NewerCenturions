use crate::{Event, Phase, RuleError, Session, feedback::GameFeedback};
use centurions_content::records::{
    EquipmentStat as Stat, InventoryModifier as Modifier, ItemUpgrade,
};
use centurions_content::{Content, InventoryRecord, Warrior, word};

/// Using an upgrade item (class 27) upgrades the first matching equipped item.
///
/// Original: items
pub(crate) fn use_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    crate::exploration::check_item_actor(state, warrior)?;
    if !matches!(state.phase, Phase::Exploration | Phase::Combat)
        || state.exploration.as_ref().is_none_or(|position| {
            position.presentation.as_ref().is_some_and(|presentation| {
                !matches!(presentation, crate::Presentation::Typed { .. })
            })
        })
    {
        return Err("modifier use requires inventory access".into());
    }
    if state.combat.is_none() {
        state.set_warrior_selector(warrior);
    }
    let records = state.warriors[warrior].inventory()?;
    let selected = records.get(slot).ok_or("modifier slot unavailable")?;
    let item = selected.id();
    let definition = crate::ammunition::definition(content, item)?;
    if word(&definition, 26)? != 27 {
        return Err("selected item is not a modifier".into());
    }
    state.load_item(content, item)?;
    let refusal = |feedback| Err(RuleError::RefusedAfterEffects { feedback });
    if state.phase == Phase::Combat {
        return refusal(vec![GameFeedback::Warning { index: 24 }]);
    }
    let Some(target) = records
        .iter()
        .position(|record| record.0[9] != 0 && record.0[10] == definition[87])
    else {
        return refusal(vec![
            GameFeedback::Warning { index: 138 },
            GameFeedback::Warning { index: 139 },
        ]);
    };
    let sound = word(&definition, 12)?.wrapping_neg();
    let upgrade = word(&definition, 102)?;
    let upgrade_definition = crate::ammunition::definition(content, upgrade)?;
    let complete = upgrade_record(
        &mut state.warriors[warrior],
        target,
        &upgrade_definition,
        content,
    )?;
    state.load_item(content, upgrade)?;
    if !complete {
        let mut feedback = vec![GameFeedback::Sound { resource_id: sound }];
        feedback.push(GameFeedback::Warning { index: 148 });
        return refusal(feedback);
    }
    let map = crate::exploration::active_map(state, content)?;
    let globals = crate::equipment::drop(&mut state.warriors[warrior], slot, content, map)?;
    state.apply_equipment_globals(&globals)?;
    state.load_item(content, item)?;
    Ok(vec![
        Event::Sound { resource_id: sound },
        Event::Sound { resource_id: -217 },
        Event::ItemUpgraded {
            warrior,
            slot: target - usize::from(slot < target),
        },
    ])
}

/// Each stage of the upgrade takes effect before the shield-efficiency refusal.
///
/// Original: upgrade
fn upgrade_record(
    warrior: &mut Warrior,
    slot: usize,
    definition: &[u8],
    content: &Content,
) -> Result<bool, RuleError> {
    let upgrade = ItemUpgrade::read(definition)?;
    let mut item = warrior.record().inventory_slot(slot);
    if upgrade.capacity_percent != 0 {
        let increase = f32::from(upgrade.capacity_percent);
        let factor = (f64::from((f64::from(increase) + 100.0) as f32) / 100.0) as f32;
        let previous = item.capacity()?;
        let value = ((f64::from(previous) * f64::from(factor)) as f32).trunc();
        let value = (value as i32) as i16;
        item.set_capacity(if value == previous {
            previous.wrapping_add(1)
        } else {
            value
        });
        // The battery's shield capacity gets the calculated value, before the
        // same-value +1.
        if item.category_code() == 20 {
            Stat::ShieldCapacity.write(warrior, value)?;
        }
    }
    for (stat, modifier, amount) in [
        (
            Stat::ShieldChance,
            Modifier::ShieldChance,
            upgrade.shield_chance,
        ),
        (Stat::MissileAccuracy, Modifier::Accuracy, upgrade.accuracy),
        (
            Stat::MissileDodge,
            Modifier::MissileDodge,
            upgrade.missile_dodge,
        ),
    ] {
        add_modifier(warrior, &mut item, stat, modifier, amount)?;
    }
    if matches!(upgrade.equipment_kind, 13 | 14) {
        Modifier::ShieldCost.add(&mut item, upgrade.shield_cost);
        let worn = Stat::Equipment(i16::from(upgrade.equipment_kind)).read(warrior)?;
        let comparison = crate::ammunition::definition(content, worn)?;
        let cost = i16::from(Modifier::ShieldCost.read(&item)).wrapping_add(
            centurions_content::records::EquipmentModifiers::read(&comparison)?.shield_cost,
        );
        if cost < 1 {
            Modifier::ShieldCost.add(&mut item, 1);
            warrior.record_mut().set_inventory_slot(slot, &item);
            return Ok(false);
        }
        Stat::ShieldCost.write(
            warrior,
            Stat::ShieldCost
                .read(warrior)?
                .wrapping_add(i16::from(upgrade.shield_cost)),
        )?;
    }
    Modifier::Range.add(&mut item, upgrade.range);
    add_modifier(
        warrior,
        &mut item,
        Stat::MeleeDodge,
        Modifier::MeleeDodge,
        upgrade.melee_dodge,
    )?;
    Modifier::Damage.add(&mut item, upgrade.damage);
    add_modifier(
        warrior,
        &mut item,
        Stat::Armor,
        Modifier::Armor,
        upgrade.armor,
    )?;
    warrior.record_mut().set_inventory_slot(slot, &item);
    Ok(true)
}

/// Adds a modifier to the item record and applies the same amount to the warrior.
fn add_modifier(
    warrior: &mut Warrior,
    item: &mut InventoryRecord,
    stat: Stat,
    modifier: Modifier,
    amount: i8,
) -> Result<(), RuleError> {
    stat.write(warrior, stat.read(warrior)?.wrapping_add(i16::from(amount)))?;
    modifier.add(item, amount);
    Ok(())
}
