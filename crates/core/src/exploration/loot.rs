//! Loot for maintained exploration.
use super::{continue_with_spells, needs_spell_confirmation};
use crate::{Event, Presentation, RuleError, Session, insert, items, story_collection};
use centurions_content::records::{
    AttackPacket, EnemyRecord, EnemyRecordMut, TreasureRecord, TreasureTable,
};
use centurions_content::{Content, InventoryRecord};
use serde::{Deserialize, Serialize};

/// What booty105748 draws from its own locals: register a3's credits (added to
/// LONG 0x442788 at 0x1060dc), the per-warrior share -$96 and experience total
/// LONG 0x443392. booty keeps them only while its window is open.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootyTally {
    pub credits: i32,
    pub share: i32,
    pub experience: i32,
}

impl BootyTally {
    pub(crate) fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// newland102, 0x141358..0x1413e2, enters ordinary booty with experience 1
/// and no NPCs. Selected members' remaining experience is literally 1.
pub(super) fn reset_experience(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    for (warrior, selected) in state.warriors.iter_mut().zip(state.native_work.party_mask) {
        if selected != 0 {
            warrior.record_mut().set_remaining_experience(1);
        }
    }
    for warrior in &mut state.warriors {
        warrior.record_mut().set_active_flag(1);
    }
    state.retain_npcs();
    let health_effect = state
        .combat
        .as_ref()
        .map(|arena| &arena.attack_packet)
        .map(|packet| AttackPacket(packet).health_effect());
    state.store_spell_fatigue();
    if let Some(arena) = state.combat.take() {
        state.native_work.ready = arena.ready;
        state.native_work.attack_packet = arena.attack_packet;
        state.native_work.attack_identity = arena.attack_identity;
        state.native_work.weapon_work = arena.weapon_work;
        state.native_work.selected_enemy = arena.selected_enemy;
        state.native_work.script_enemy = arena.script_enemy;
    }
    state.handled_deaths.clear();
    if let Some(story) = state.combat_story.as_mut() {
        // The shared caller resumes after booty with the battle flag cleared.
        story.child_returned = true;
    }
    state.phase = crate::Phase::Exploration;
    let mut effects = crate::combat::runtime::refit_after_combat(state, content)?;
    state.clear_tactical_fields();
    let unavailable = state
        .warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1 || warrior.record().automatic_mode().unwrap_or(0) < 0
        })
        .count() as i8;
    // With count43874a cleared, bodycount has no surviving NPCs to select.
    for record in state
        .allies
        .as_chunks_mut::<{ crate::story_allies::ALLY_BYTES }>()
        .0
        .iter_mut()
        .take(state.ally_count)
    {
        let weapon = EnemyRecord::from_bytes(record).primary_weapon()?;
        if weapon < 200 {
            EnemyRecordMut::from_bytes(record).set_selected_item(weapon);
        }
    }
    let treasure = TreasureRecord::experience_award(1);
    effects.push(Event::Sound {
        resource_id: crate::feedback::BOOTY_SOUND,
    });
    effects.extend(present_reward_record(state, content, &treasure)?);
    let mut health = crate::story_health::retained_context(
        state.selected_warrior,
        &state.native_work.health_context,
    );
    health.unavailable = unavailable;
    if let Some(effect) = health_effect {
        health.effect = effect;
    }
    state.native_work.health_context = health;
    Ok(effects)
}

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
        .booty_items
        .get(slot)
        .filter(|item| item.id() != 0)
        .ok_or_else(|| RuleError::Invalid("loot slot is empty".into()))?
        .clone();
    state.load_item_work(content, item.id())?;
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
        state.booty_items[slot] = collected.item;
        if !collected.collected {
            return Ok(effects);
        }
        if recipient.inventory()?.len() > prior_count
            && let Some(last) = recipient.inventory()?.last()
        {
            let id = last.id();
            state.native_work.inventory_record = item.clone();
            state.load_item_work(content, id)?;
        }
    } else {
        effects = insert(item, &mut state.crate_items, true)?;
        if effects.contains(&Event::StorageFull) {
            return Ok(effects);
        }
    }
    state.booty_items[slot].clear_id();
    if let Some(limit) = &mut state.loot_limit
        && *limit != 0
    {
        *limit = limit.wrapping_sub(1);
        if *limit == 0 {
            state.booty_items.fill(InventoryRecord([0; 22]));
            if !needs_spell_confirmation(state) {
                effects.extend(continue_with_spells(state, content, false)?);
            }
        }
    }
    Ok(effects)
}

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

/// booty(1, 0) on a treasure record already in the booty buffer at 0x44331c.
pub(super) fn present_treasure_record(
    state: &mut Session,
    content: &Content,
    record: &[u8],
) -> Result<Vec<Event>, RuleError> {
    let events = present_reward_record(state, content, record)?;
    let mut retained = TreasureRecord::new(record)?.0.to_vec();
    centurions_content::records::TreasureRecordMut(&mut retained).consume_items()?;
    state.native_work.treasure_record = retained;
    Ok(events)
}

fn present_reward_record(
    state: &mut Session,
    content: &Content,
    record: &[u8],
) -> Result<Vec<Event>, RuleError> {
    let prepared = crate::story_treasure::prepare(
        &state.warriors,
        record,
        content,
        &mut state.random,
        state.difficulty,
    )?;
    // The prepared records do not retain randomitem's parallel quality bytes.
    state.native_work.loot_quality = if prepared.items.is_empty() {
        Some(vec![0; 1000])
    } else {
        None
    };
    state.warriors = prepared.warriors;
    state.party_currency = state.party_currency.wrapping_add(prepared.credits);
    state.booty_tally = BootyTally {
        credits: prepared.credits,
        share: prepared.share,
        experience: prepared.experience,
    };
    if let Some(record) = prepared.inventory_record {
        state.native_work.inventory_record = record;
    }
    state.booty_items.fill(InventoryRecord([0; 22]));
    for (slot, item) in prepared.items.into_iter().enumerate() {
        state.native_work.loaded_item_field32 = state
            .native_work
            .ammunition_work
            .load_item(content, item.id())?;
        state.booty_items[slot] = item;
    }
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    Ok(vec![Event::StoryPresented])
}
