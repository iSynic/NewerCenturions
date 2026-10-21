use crate::{
    Command, Event, Phase, Presentation, RuleError, Session, ammunition, combat, equipment,
    exploration, items, party, shop,
};
use centurions_content::{Content, InventoryRecord};

pub(super) fn execute(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    match command {
        Command::OpenShop => shop::open(state),
        Command::CloseShop => shop::close(state, content),
        Command::BuyItem { warrior, stock } => shop::buy(state, content, warrior, stock),
        Command::SellItem { warrior, slot } => shop::sell(state, content, warrior, slot),
        Command::TransferItem { from, to, slot } => {
            equipment::transfer(state, content, from, to, slot)
        }
        Command::RefitParty => {
            if state.phase != Phase::Exploration {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let mut events = vec![];
            for member in 0..state.warriors.len() {
                let result = ammunition::refit_with_record(
                    &state.warriors[member],
                    &state.ammunition,
                    content,
                    ammunition::RefitFlags {
                        in_combat: false,
                        force: false,
                        silent: true,
                        warnings: false,
                    },
                    &state.ammunition_work,
                    &mut state.random,
                    state.inventory_record.as_ref(),
                )?;
                events.extend(ammunition::commit_result(state, member, result)?);
            }
            Ok(events)
        }
        Command::DropItem { warrior, slot } => {
            let collecting = state
                .exploration
                .as_ref()
                .is_some_and(|p| p.presentation == Some(Presentation::Loot));
            if state.phase != Phase::Exploration && !(state.phase == Phase::Combat && collecting) {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let mission = state
                .exploration
                .as_ref()
                .and_then(|p| content.map(p.map))
                .ok_or("drop profiles missing")?;
            let record = state
                .warriors
                .get_mut(warrior)
                .ok_or("drop warrior outside squad")?;
            if state.inventory_record.is_some() {
                state.inventory_record = Some(
                    record
                        .inventory()?
                        .get(slot)
                        .cloned()
                        .ok_or("drop slot missing")?,
                );
            }
            equipment::drop(record, slot, content, mission)?;
            Ok(vec![Event::Sound { resource_id: 655 }])
        }
        Command::PoolCredits => {
            party::pool_credits(&mut state.warriors, &mut state.party_currency)?;
            Ok(vec![Event::CreditsPooled {
                total: state.party_currency,
            }])
        }
        Command::ShareCredits => {
            let each = party::share_credits(&mut state.warriors, &mut state.party_currency)?;
            Ok(vec![Event::CreditsShared { each }])
        }
        Command::TakeLoot { slot, warrior } => {
            exploration::take_loot(state, content, slot, Some(warrior))
        }
        Command::StoreLoot { slot } => exploration::take_loot(state, content, slot, None),
        Command::TakeCrate { slot, warrior } => {
            equipment::take_crate(state, content, slot, warrior)
        }
        Command::EquipItem { warrior, slot } => equipment::equip(state, content, warrior, slot),
        Command::InspectInventory {
            members,
            scroll,
            selected,
            shop_family,
        } => equipment::inspect(state, content, members, scroll, selected, shop_family),
        Command::UnequipItem { warrior, slot } => equipment::unequip(state, content, warrior, slot),
        Command::Reload { warrior }
            if state
                .exploration
                .as_ref()
                .is_some_and(|p| p.presentation == Some(Presentation::Shop)) =>
        {
            shop::reload(state, content, warrior)
        }
        Command::Reload { warrior } => combat::firearm::reload_command(state, content, warrior),
        Command::FitAmmunition {
            warrior,
            slot,
            ammunition,
        } => combat::runtime::fit_ammunition(state, content, warrior, slot, ammunition),
        Command::UseAmmunitionPack { warrior, slot } => {
            items::use_ammunition_pack(state, content, warrior, slot)
        }
        Command::UseItem { warrior, slot } => exploration::use_item(state, content, warrior, slot),
        Command::AddToCrate { item } => insert(item, &mut state.crate_items, true),
        Command::AddToBooty { item } => insert(item, &mut state.booty_items, false),
        _ => unreachable!("command routed to wrong subsystem"),
    }
}

pub(crate) fn insert(
    item: InventoryRecord,
    records: &mut [InventoryRecord],
    crate_storage: bool,
) -> Result<Vec<Event>, RuleError> {
    if let Some(slot) = records.iter().position(|entry| entry.id() == 0) {
        records[slot] = item;
        let mut events = vec![Event::ItemAdded {
            slot,
            crate_storage,
        }];
        if crate_storage {
            events.push(Event::Sound { resource_id: 663 });
        }
        Ok(events)
    } else {
        Ok(vec![Event::StorageFull])
    }
}
