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
        Command::OpenInventory {
            warrior,
            quick_info,
        } => items::open_inventory(state, content, warrior, quick_info),
        Command::SwitchInventoryOwner {
            warrior,
            scroll,
            quick_info,
            selected_row,
        } => items::switch_inventory_owner(
            state,
            content,
            warrior,
            scroll,
            quick_info.then_some(selected_row),
        ),
        Command::ScrollInventory {
            warrior,
            scroll,
            delta,
            quick_info,
        } => items::scroll_inventory(state, content, warrior, scroll, delta, quick_info),
        Command::HighlightInventoryRow {
            warrior,
            scroll,
            row,
            quick_info,
        } => items::highlight_inventory_row(state, content, warrior, scroll, row, quick_info),
        Command::ShowItemInformation {
            warrior,
            scroll,
            row,
            quick_info,
        } => items::show_item_information(state, content, warrior, scroll, row, quick_info),
        Command::RetargetItemInformation {
            warrior,
            member,
            slot,
            quick_info,
            list,
            item,
        } => items::retarget_item_information(
            state,
            content,
            warrior,
            (member, slot, list, item),
            quick_info,
        ),
        Command::ShowLootItemInformation { slot } => {
            items::show_loot_item_information(state, content, slot)
        }
        Command::ChooseAmmunition {
            warrior,
            scroll,
            row,
            item,
        } => items::choose_ammunition(state, content, warrior, (scroll, row), item),
        Command::ShowDraggedItemInformation {
            warrior,
            slot,
            stock,
            quick_info,
        } => {
            items::show_dragged_item_information(state, content, warrior, (slot, stock), quick_info)
        }
        Command::ListItemUsers {
            warrior,
            professions,
            widened,
        } => items::list_item_users(state, content, warrior, professions, widened),
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
                    &state.native_work.ammunition_work,
                    &mut state.random,
                    Some(&state.native_work.inventory_record),
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
            state.native_work.inventory_record = record
                .inventory()?
                .get(slot)
                .cloned()
                .ok_or("drop slot missing")?;
            let globals = equipment::drop(record, slot, content, mission)?;
            state.apply_equipment_globals(&globals)?;
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
