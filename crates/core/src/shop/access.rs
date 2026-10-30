//! Entering, loading and saving shops.
use super::RECORD;
use crate::{Command, Event, Phase, Presentation, RuleError, Session};
use centurions_content::mission::OpeningMission;
use centurions_content::records::ShopRecord;
use centurions_content::span;
use centurions_content::{Content, word};
use serde::{Deserialize, Serialize};

/// The shop the party is visiting: its level and number, and for a shop that stocks enhanced items
/// the enhancement and how many items the party may still take.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Access {
    /// The scenario level of the shop.
    pub level: u8,
    /// The shop's number on its level.
    pub id: u8,
    /// The enhancement level of the stock the shop generates, the number of improvement rounds
    /// each item gets; zero for an ordinary shop.
    #[serde(default)]
    pub enhancement: i16,
    /// How many items the party may still take from a limited shop; an ordinary shop leaves it
    /// zero.
    #[serde(default)]
    pub remaining: i16,
}

/// The refusal given with a shop warning.
pub(super) fn refused(warning: i16) -> RuleError {
    RuleError::Refused {
        feedback: vec![crate::feedback::GameFeedback::Warning { index: warning }],
    }
}

/// Checks the shop being visited and every shop's saved record against the scenario's shops.
pub(crate) fn validate(state: &Session, content: &Content) -> Result<(), RuleError> {
    state.globals.the_shop.validate()?;
    for (level, records) in &state.shops {
        let context = content.context(state.mission.as_deref(), *level)?;
        let authored = context.map(0).ok_or("shop level unavailable")?;
        if records.len() != authored.shops.len() || !records.len().is_multiple_of(RECORD) {
            return Err("invalid persistent shop records".into());
        }
        for shop in records.as_chunks::<RECORD>().0 {
            for slot in 0..1000 {
                if ShopRecord(shop).quantity(slot) != 0
                    && !(0..1000).contains(&ShopRecord(shop).item(slot)?)
                {
                    return Err("invalid shop item identifier".into());
                }
            }
        }
    }
    if let Some(access) = &state.shop_access {
        let context = content.context(state.mission.as_deref(), access.level)?;
        let map = context.map(0).ok_or("shop level unavailable")?;
        span(&map.shops, access.id as usize * RECORD, RECORD)?;
    }
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
        && (state.phase != Phase::Exploration
            || state
                .shop_access
                .as_ref()
                .is_none_or(|a| a.level != state.scenario_level))
    {
        return Err("invalid active shop".into());
    }
    Ok(())
}

/// The shop script operation opens a shop; a positive parameter also lets the
/// party reach its inventory later.
///
/// Original: newland
pub(crate) fn visit(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<bool, RuleError> {
    let id = parameter.wrapping_abs() as u8;
    state.loot_limit = None;
    let map = state.map(content, 0).ok_or("shop scenario unavailable")?;
    load_shop(state, map, id)?;
    state.shop_access = Some(Access {
        level: state.scenario_level,
        id,
        enhancement: 0,
        remaining: 0,
    });
    if parameter <= 0 {
        show(state)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// The saved shop records of a level, taken from an imported game or the scenario when the party
/// has not visited them.
fn file_rows<'a>(state: &'a mut Session, map: &OpeningMission, level: u8) -> &'a mut [u8] {
    state.shops.entry(level).or_insert_with(|| {
        state
            .imported
            .as_ref()
            .and_then(|imported| imported.scenario_records.get(&level))
            .cloned()
            .unwrap_or_else(|| map.shops.clone())
    })
}

/// Loads the level's shop record, compacting each ID family in order.
///
/// Original: loadshop
fn load_shop(state: &mut Session, map: &OpeningMission, id: u8) -> Result<(), RuleError> {
    span(&map.shops, id as usize * RECORD, RECORD)?;
    let level = state.scenario_level;
    let record = span(file_rows(state, map, level), id as usize * RECORD, RECORD)?.to_vec();
    state.globals.the_shop.load(&record)
}

/// Saves the whole shop record, including rows a load left stale.
///
/// Original: saveshop
fn save_shop(state: &mut Session, content: &Content, level: u8, id: u8) -> Result<(), RuleError> {
    let record = state.globals.the_shop.record();
    let context = content.context(state.mission.as_deref(), level)?;
    let map = context.map(0).ok_or("shop level unavailable")?;
    span(&map.shops, id as usize * RECORD, RECORD)?;
    let destination = file_rows(state, map, level)
        .get_mut(id as usize * RECORD..(id as usize + 1) * RECORD)
        .ok_or("shop record outside the file")?;
    destination.copy_from_slice(&record);
    Ok(())
}

/// The alter-shop script operation changes the shop's price percent by e2, and
/// every slot holding item e3 by e4's low byte as a signed count, floored at
/// zero. Shop zero (the current shop) and negative shops are refused.
///
/// Original: newland
pub(crate) fn alter(state: &mut Session, content: &Content, extra: &[u8]) -> Result<(), RuleError> {
    let id = word(extra, 0)? as i8;
    if id <= 0 {
        return Err(RuleError::Unsupported {
            action: "story.opcode",
            dependency: "shop alteration outside the scenario shops",
        });
    }
    let map = state.map(content, 0).ok_or("shop scenario unavailable")?;
    load_shop(state, map, id as u8)?;
    state
        .globals
        .the_shop
        .alter(word(extra, 2)?, word(extra, 4)?, word(extra, 6)? as i8)?;
    save_shop(state, content, state.scenario_level, id as u8)?;
    // The altered shop becomes the current shop.
    if let Some(access) = state.shop_access.as_mut() {
        access.level = state.scenario_level;
        access.id = id as u8;
    }
    Ok(())
}

/// Visits a shop that sells only a limited number of enhanced items: the extra gives the
/// enhancement level (stored one higher) and how many items the party may take.
pub(crate) fn visit_restricted(
    state: &mut Session,
    content: &Content,
    extra: &[u8],
) -> Result<(), RuleError> {
    visit(state, content, word(extra, 0)?)?;
    let access = state.shop_access.as_mut().unwrap();
    access.enhancement = word(extra, 4)?.wrapping_add(1);
    access.remaining = word(extra, 2)?;
    Ok(())
}

/// Puts the shop window up after pooling the party's money.
pub(super) fn show(state: &mut Session) -> Result<(), RuleError> {
    crate::party::pool_credits(&mut state.warriors, &mut state.party_currency)?;
    state
        .exploration
        .as_mut()
        .ok_or("no active mission")?
        .presentation = Some(Presentation::Shop);
    Ok(())
}

/// Opens the shop window for the shop the party is visiting.
pub(crate) fn open(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || state
            .shop_access
            .as_ref()
            .is_none_or(|a| a.level != state.scenario_level)
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some())
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    show(state)?;
    Ok(vec![Event::StoryPresented])
}

/// Closes the shop: its stock is saved and the script that sent the party there carries on.
pub(crate) fn close(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let access = active(state)?.clone();
    save_shop(state, content, access.level, access.id)?;
    state.exploration.as_mut().unwrap().presentation = None;
    if state.exploration.as_ref().unwrap().script.is_some() {
        crate::exploration::run_script(state, content)
    } else {
        Ok(vec![])
    }
}

/// The shop being visited; refused unless the shop window is showing.
pub(super) fn active(state: &Session) -> Result<&Access, RuleError> {
    if state.phase != Phase::Exploration
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation != Some(Presentation::Shop))
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    state
        .shop_access
        .as_ref()
        .filter(|a| a.level == state.scenario_level)
        .ok_or_else(|| "no active shop records".into())
}

/// Whether a command may be given while the shop window is open.
pub(crate) fn command_allowed(command: &Command) -> bool {
    matches!(
        command,
        Command::CloseShop
            | Command::InspectInventory { .. }
            | Command::BuyItem { .. }
            | Command::SellItem { .. }
            | Command::Continue
            | Command::Reload { .. }
            | Command::ViewWarrior { .. }
            | Command::RetargetItemInformation { .. }
            | Command::ListItemUsers { .. }
            | Command::ShowDraggedItemInformation { .. }
    )
}

/// The open shop's stock, which needs no file overlay.
pub(super) fn current_shop(state: &Session) -> Result<&super::TheShop, RuleError> {
    state.shop_access.as_ref().ok_or("shop unavailable")?;
    Ok(&state.globals.the_shop)
}
