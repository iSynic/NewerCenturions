//! Access for shop.
use super::RECORD;
use crate::{Command, Event, Phase, Presentation, RuleError, Session};
use centurions_content::mission::OpeningMission;
use centurions_content::records::ShopRecord;
use centurions_content::span;
use centurions_content::{Content, word};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Access {
    pub level: u8,
    pub id: u8,
    #[serde(default)]
    pub enhancement: i16,
    #[serde(default)]
    pub remaining: i16,
}

pub(super) fn refused(warning: i16) -> RuleError {
    RuleError::Refused {
        feedback: vec![crate::feedback::NativeFeedback::Warning { index: warning }],
    }
}

pub(crate) fn validate(state: &Session, content: &Content) -> Result<(), RuleError> {
    state.native_work.shop_work.validate()?;
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

/// newland opcode 6, 0x142fae; positive parameters register later inventory access.
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

fn file_rows<'a>(state: &'a mut Session, map: &OpeningMission, level: u8) -> &'a mut [u8] {
    state.shops.entry(level).or_insert_with(|| {
        state
            .native_import
            .as_ref()
            .and_then(|imported| imported.scenario_records.get(&level))
            .cloned()
            .unwrap_or_else(|| map.shops.clone())
    })
}

/// loadshop 0x12fe54: the level's shop record with each ID family compacted in order.
fn load_shop(state: &mut Session, map: &OpeningMission, id: u8) -> Result<(), RuleError> {
    span(&map.shops, id as usize * RECORD, RECORD)?;
    let level = state.scenario_level;
    let record = span(file_rows(state, map, level), id as usize * RECORD, RECORD)?.to_vec();
    state.native_work.shop_work.load(&record)
}

/// saveshop writes the whole retained buffer, including loadshop's stale tails.
fn save_work(state: &mut Session, content: &Content, level: u8, id: u8) -> Result<(), RuleError> {
    let record = state.native_work.shop_work.record();
    let context = content.context(state.mission.as_deref(), level)?;
    let map = context.map(0).ok_or("shop level unavailable")?;
    span(&map.shops, id as usize * RECORD, RECORD)?;
    let destination = file_rows(state, map, level)
        .get_mut(id as usize * RECORD..(id as usize + 1) * RECORD)
        .ok_or("shop record outside the file")?;
    destination.copy_from_slice(&record);
    Ok(())
}

/// newland51 0x145870: the shop's price percent changes by e2 and every slot holding
/// item e3 changes by e4's low byte as a signed count, floored at zero. Shop zero
/// (loadshop's current shop) and negative shop bytes are refused.
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
        .native_work
        .shop_work
        .alter(word(extra, 2)?, word(extra, 4)?, word(extra, 6)? as i8)?;
    save_work(state, content, state.scenario_level, id as u8)?;
    // loadshop leaves the altered shop current.
    if let Some(access) = state.shop_access.as_mut() {
        access.level = state.scenario_level;
        access.id = id as u8;
    }
    Ok(())
}

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

pub(super) fn show(state: &mut Session) -> Result<(), RuleError> {
    crate::party::pool_credits(&mut state.warriors, &mut state.party_currency)?;
    state
        .exploration
        .as_mut()
        .ok_or("no active mission")?
        .presentation = Some(Presentation::Shop);
    Ok(())
}

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

pub(crate) fn close(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let access = active(state)?.clone();
    save_work(state, content, access.level, access.id)?;
    state.exploration.as_mut().unwrap().presentation = None;
    if state.exploration.as_ref().unwrap().script.is_some() {
        crate::exploration::run_script(state, content)
    } else {
        Ok(vec![])
    }
}

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

/// Access selects the workflow; current stock does not require a file overlay.
pub(super) fn current_work(state: &Session) -> Result<&super::ShopWork, RuleError> {
    state.shop_access.as_ref().ok_or("shop unavailable")?;
    Ok(&state.native_work.shop_work)
}
