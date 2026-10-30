//! Treasure and loot found while exploring.
use super::{continue_with_spells, needs_spell_confirmation};
use crate::{Event, Presentation, RuleError, Session, insert, items, story_collection};
use centurions_content::records::{
    EnemyRecord, EnemyRecordMut, SpellInfoRecord, TreasureRecord, TreasureTable,
};
use centurions_content::{Content, InventoryRecord};
use serde::{Deserialize, Serialize};

/// The credits, per-warrior share and victory points the treasure window shows.
/// booty keeps them only while its window is open.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootyTally {
    pub credits: i32,
    pub share: i32,
    #[serde(rename = "experience")]
    pub victory_points: i32,
}

impl BootyTally {
    /// Whether the tally is still in its starting state.
    pub(crate) fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// Opens the treasure window with 1 victory point and no monsters, so the selected
/// members are left with exactly 1 victory point.
///
/// Original: newland
pub(super) fn reset_experience(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    for (warrior, selected) in state.warriors.iter_mut().zip(state.globals.party_track) {
        if selected != 0 {
            warrior.record_mut().set_remaining_victory_points(1);
        }
    }
    for warrior in &mut state.warriors {
        warrior.record_mut().set_in_battle(1);
    }
    state.store_monsters();
    let health_effect = state
        .combat
        .as_ref()
        .map(|arena| &arena.spell_info)
        .map(|packet| SpellInfoRecord(packet).health_effect());
    state.store_spell_fatigue();
    if let Some(arena) = state.combat.take() {
        state.globals.in_spell = arena.in_spell;
        state.globals.spell_info = arena.spell_info;
        state.globals.cast_identity = arena.cast_identity;
        state.globals.weapon_traits = arena.weapon_traits;
        state.globals.monster_up = arena.monster_up;
        state.globals.macro_monster = arena.macro_monster;
    }
    state.handled_deaths.clear();
    if let Some(story) = state.combat_story.as_mut() {
        // The shared caller resumes after booty with the battle flag cleared.
        story.child_returned = true;
    }
    state.phase = crate::Phase::Exploration;
    let mut effects = crate::combat::runtime::refit_after_combat(state, content)?;
    state.clear_tactical_fields();
    let kill_party = state
        .warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1
                || warrior.record().computer_control().unwrap_or(0) < 0
        })
        .count() as i8;
    // With no monsters, bodycount has no survivors to select.
    for record in state
        .holdover
        .as_chunks_mut::<{ crate::story::allies::ALLY_BYTES }>()
        .0
        .iter_mut()
        .take(state.held_over)
    {
        let weapon = EnemyRecord::from_bytes(record).primary_weapon()?;
        if weapon < 200 {
            EnemyRecordMut::from_bytes(record).set_weapon(weapon);
        }
    }
    let treasure = TreasureRecord::victory_points_award(1);
    effects.push(Event::Sound {
        resource_id: crate::feedback::BOOTY_SOUND,
    });
    effects.extend(present_reward_record(state, content, &treasure)?);
    let mut health = crate::story::health::reselected_context(
        state.char_select_new,
        &state.globals.health_context,
    );
    health.kill_party = kill_party;
    if let Some(effect) = health_effect {
        health.effect = effect;
    }
    state.globals.health_context = health;
    Ok(effects)
}

/// Takes a piece of loot from the treasure window: ammunition goes into the party's reserves, other
/// items to the chosen warrior, or to the crate when no warrior is given.
pub fn take_loot(
    state: &mut Session,
    content: &Content,
    slot: usize,
    warrior: Option<usize>,
) -> Result<Vec<Event>, RuleError> {
    if !state
        .exploration
        .as_ref()
        .is_some_and(|position| matches!(position.presentation, Some(Presentation::Loot)))
    {
        return Err(RuleError::Invalid("no loot choice active".into()));
    }
    let item = state
        .booty_list
        .get(slot)
        .filter(|item| item.id() != 0)
        .ok_or_else(|| RuleError::Invalid("loot slot is empty".into()))?
        .clone();
    state.load_item(content, item.id())?;
    let mut effects = Vec::new();
    if (200..400).contains(&item.id()) && item.category_code() == 26 {
        effects = items::add_ammunition(state, content, &item)?;
    } else if let Some(index) = warrior {
        let recipient = state
            .warriors
            .get_mut(index)
            .ok_or_else(|| RuleError::Invalid("recipient outside squad".into()))?;
        let prior_count = recipient.inventory()?.len();
        let race = recipient.record().race()?.wrapping_sub(1);
        let movement = *crate::Session::land_in(
            content,
            state.mission.as_deref(),
            state.scenario_level,
            state.exploration.as_ref(),
        )
        .unwrap()
        .combat
        .as_ref()
        .unwrap()
        .race_moves
        .get(race as usize)
        .ok_or_else(|| RuleError::Invalid("recipient race unavailable".into()))?;
        let collected = if item.category_code() != 23 {
            crate::equipment::collect_item(
                recipient,
                &item,
                content,
                crate::Session::land_in(
                    content,
                    state.mission.as_deref(),
                    state.scenario_level,
                    state.exploration.as_ref(),
                )
                .unwrap(),
            )?
        } else {
            story_collection::collect_story_item(recipient, &item, content, movement)?
        };
        effects.extend(
            collected
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        effects.extend(
            collected
                .warnings
                .into_iter()
                .map(|warning| Event::CollectionRejected { warning }),
        );
        *recipient = collected.warrior;
        state.booty_list[slot] = collected.item;
        if !collected.collected {
            return Ok(effects);
        }
        if recipient.inventory()?.len() > prior_count
            && let Some(last) = recipient.inventory()?.last()
        {
            let id = last.id();
            state.globals.inventory_record = item.clone();
            state.load_item(content, id)?;
        }
    } else {
        effects = insert(item, &mut state.crate_items, true)?;
        if effects.contains(&Event::StorageFull) {
            return Ok(effects);
        }
    }
    state.booty_list[slot].clear_id();
    if let Some(limit) = &mut state.loot_limit
        && *limit != 0
    {
        *limit = limit.wrapping_sub(1);
        if *limit == 0 {
            state.booty_list.fill(InventoryRecord([0; 22]));
            if !needs_spell_confirmation(state) {
                effects.extend(continue_with_spells(state, content, false)?);
            }
        }
    }
    Ok(effects)
}

/// Opens the treasure window on the scenario's treasure record with the given number.
pub(crate) fn present_treasure(
    state: &mut Session,
    content: &Content,
    id: i16,
) -> Result<Vec<Event>, RuleError> {
    if id < 0 {
        return Err("negative treasure record".into());
    }
    let position = state
        .exploration
        .as_ref()
        .ok_or("treasure mission unavailable")?;
    let map = state
        .map(content, position.map)
        .ok_or("treasure map unavailable")?;
    let record = TreasureTable(&map.treasure).record(id as usize)?.to_vec();
    present_treasure_record(state, content, &record)
}

/// Opens the treasure window on a treasure record already loaded.
///
/// Original: booty
pub(super) fn present_treasure_record(
    state: &mut Session,
    content: &Content,
    record: &[u8],
) -> Result<Vec<Event>, RuleError> {
    let events = present_reward_record(state, content, record)?;
    let mut treasure = TreasureRecord::new(record)?.0.to_vec();
    centurions_content::records::TreasureRecordMut(&mut treasure).consume_items()?;
    state.globals.treasure = treasure;
    Ok(events)
}

/// Prepares a treasure record's money and items and shows them.
fn present_reward_record(
    state: &mut Session,
    content: &Content,
    record: &[u8],
) -> Result<Vec<Event>, RuleError> {
    let prepared = crate::story::treasure::prepare(
        &state.warriors,
        record,
        content,
        &mut state.random,
        state.difficulty,
    )?;
    // Prepared records carry no loot quality of their own.
    state.globals.loot_quality = if prepared.items.is_empty() {
        Some(vec![0; 1000])
    } else {
        None
    };
    state.warriors = prepared.warriors;
    state.party_currency = state.party_currency.wrapping_add(prepared.credits);
    state.booty_tally = BootyTally {
        credits: prepared.credits,
        share: prepared.share,
        victory_points: prepared.victory_points,
    };
    if let Some(record) = prepared.inventory_record {
        state.globals.inventory_record = record;
    }
    state.booty_list.fill(InventoryRecord([0; 22]));
    for (slot, item) in prepared.items.into_iter().enumerate() {
        state.globals.loaded_item_field32 =
            state.globals.loaded_items.load_item(content, item.id())?;
        state.booty_list[slot] = item;
    }
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    Ok(vec![Event::StoryPresented])
}
