//! Noon and midnight food consumption, healing and terminal return.
use crate::{Event, RuleError, Session, feedback::GameFeedback, items, story_health::HealthEvent};
use centurions_content::{
    Content,
    records::{Condition, ItemDefinition, WarriorRecord},
};

/// Heals each warrior whose stamina is above -10 and below the maximum: a quarter of their level
/// (their full level for race 3), at least 1, plus any regeneration. Each eats a ration first, and
/// the healing is halved when the party has none.
pub(super) fn recover(
    state: &mut Session,
    content: &Content,
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    for index in 0..state.warriors.len() {
        let bytes = &state.warriors[index].bytes;
        let stamina = WarriorRecord::from_bytes(bytes).stamina()?;
        let maximum = WarriorRecord::from_bytes(bytes).stamina_max()?;
        if stamina <= -10 || stamina >= maximum {
            continue;
        }
        let level = WarriorRecord::from_bytes(bytes).level()?;
        let mut amount = if WarriorRecord::from_bytes(bytes).race()? == 3 {
            level
        } else {
            level / 4
        } as i8;
        if !consume_food(state, content)? {
            amount /= 2;
        }
        let healing = i16::from(amount.max(1)).wrapping_add(
            state.warriors[index]
                .record()
                .condition(Condition::Regenerating)?,
        );
        let context = crate::story::health::reselected_context(
            state.char_select_new,
            &state.globals.health_context,
        );
        let mut healed = crate::story::health::heal_outdoors(
            &state.warriors,
            index,
            healing,
            false,
            &context,
            &mut state.random,
        )?;
        if healed.events.contains(&HealthEvent::PartyLost) {
            // The last affected mark stands even when partyloss ends the game.
            healed.context.affected[index] = context.affected[index];
        }
        state.warriors = healed.warriors;
        state.globals.health_context = healed.context;
        let mut status = None;
        for effect in healed.events {
            match effect {
                HealthEvent::Defeated { permanent, .. } => status = Some(permanent),
                HealthEvent::Sound(resource_id) => {
                    events.push(Event::Sound { resource_id });
                    if let Some(permanent) = status.take() {
                        events.push(Event::Feedback(GameFeedback::OutdoorBodyStatus {
                            permanent,
                            wait: state.command_preferences.animation_delay,
                        }));
                    }
                }
                HealthEvent::PartyLost => {
                    events.extend(crate::mission::defeat(state, content)?);
                    return Ok(());
                }
                _ => unreachable!("unanimated single-member recovery has no other presentation"),
            }
        }
    }
    Ok(())
}

/// Eats one charge of food (item 800) from the first warrior who carries any, removing the item
/// when its last charge goes and it is discarded when empty. Returns false when there is none.
fn consume_food(state: &mut Session, content: &Content) -> Result<bool, RuleError> {
    for warrior in &mut state.warriors {
        for (slot, item) in warrior.inventory()?.iter().enumerate() {
            if item.id() != 800 {
                continue;
            }
            let quantity = item.charge()?;
            if quantity <= 0 {
                return Ok(false);
            }
            let definition = content
                .items
                .get(800)
                .ok_or_else(|| RuleError::Invalid("food definition unavailable".into()))?;
            let weight = ItemDefinition(definition).charge_weight()?;
            let carried = warrior.record().load()?.wrapping_sub(weight);
            warrior.record_mut().set_load(carried);
            warrior
                .record_mut()
                .set_inventory_quantity(slot, quantity - 1);
            if quantity == 1 && ItemDefinition(definition).discarded_when_empty()? != 0 {
                items::remove_inventory_slot(warrior, slot)?;
            }
            state.load_item(content, 800)?;
            return Ok(true);
        }
    }
    Ok(false)
}
