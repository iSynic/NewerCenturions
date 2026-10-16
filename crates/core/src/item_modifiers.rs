use crate::recovered_feedback::{
    FeedbackMemory, FeedbackObservation, FeedbackRequest, FeedbackServices,
};
use crate::recovered_inventory_rules::{self as inventory, InventoryServices, SaneRequest};
use crate::recovered_records::{ITEM_TABLE_ADDRESSES, RecordWrite};
use crate::recovered_rendering::RenderingMemory;
use crate::{Event, Phase, RuleError, Session, feedback::NativeFeedback};
use centurions_content::{Content, WARRIOR_BYTES, word};

struct Services;

impl FeedbackServices for Services {
    fn observe(
        &self,
        _: &FeedbackRequest,
        _: &FeedbackMemory,
    ) -> Result<FeedbackObservation, RuleError> {
        Err("unexpected modifier presentation service".into())
    }

    fn sound_observations(
        &self,
        _: i16,
        _: &crate::recovered_audio::AudioContext,
        _: &FeedbackMemory,
    ) -> Result<Vec<crate::recovered_audio::AudioObservation>, RuleError> {
        Err("unexpected modifier audio service".into())
    }
}

impl InventoryServices for Services {
    fn sane(
        &self,
        request: &SaneRequest,
        memory: &RenderingMemory,
    ) -> Result<Vec<RecordWrite>, RuleError> {
        let destination = request.destination.unwrap_or(request.source);
        Ok(vec![RecordWrite {
            address: destination,
            bytes: crate::item_display::finite_arithmetic(
                memory,
                request.operation,
                Some(request.source),
                destination,
            )?,
        }])
    }
}

/// Items12aae4..12ac10: class27 upgrades the first matching equipped record.
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
    let records = state.warriors[warrior].inventory()?;
    let selected = records.get(slot).ok_or("modifier slot unavailable")?;
    let item = selected.id();
    let definition = crate::ammunition::definition(content, item)?;
    if word(&definition, 26)? != 27 {
        return Err("selected item is not a modifier".into());
    }
    state.load_item_work(content, item)?;
    let refusal = |feedback| Err(RuleError::RefusedAfterEffects { feedback });
    if state.phase == Phase::Combat {
        return refusal(vec![NativeFeedback::Warning { index: 24 }]);
    }
    let Some(target) = records
        .iter()
        .position(|record| record.0[9] != 0 && record.0[10] == definition[87])
    else {
        return refusal(vec![
            NativeFeedback::Warning { index: 138 },
            NativeFeedback::Warning { index: 139 },
        ]);
    };
    let sound = word(&definition, 12)?.wrapping_neg();
    const FRAME: u32 = 0x6ffe00;
    let address = 0x448aa0 + warrior as u32 * WARRIOR_BYTES as u32;
    let mut memory = RenderingMemory::default();
    memory.insert_region(address, &state.warriors[warrior].bytes);
    memory.insert_region(FRAME - 256, &[0; 512]);
    memory.insert_region(FRAME + 8, &(warrior as u16).to_be_bytes());
    memory.insert_region(FRAME + 10, &(target as u16).to_be_bytes());
    let upgrade = word(&definition, 102)?;
    let upgrade_definition = crate::ammunition::definition(content, upgrade)?;
    memory.insert_region(FRAME + 12, &upgrade.to_be_bytes());
    memory.insert_region(0x45363c, &definition);
    memory.insert_region(0x4781ee, &[0; 150]);
    let comparison = if matches!(upgrade_definition[87], 13 | 14) {
        word(
            &state.warriors[warrior].bytes,
            0xc6 + usize::from(upgrade_definition[87]) * 2,
        )?
    } else {
        0
    };
    for id in [upgrade, comparison] {
        let index = usize::from(id.unsigned_abs());
        let record = content
            .items
            .get(index)
            .ok_or("modifier definition unavailable")?;
        let family = *ITEM_TABLE_ADDRESSES
            .get(index / 200)
            .ok_or("modifier definition band unavailable")?;
        memory.insert_region(family + (index % 200 * 150) as u32, record);
    }
    let result = inventory::upgrade_rules(&memory, FRAME, &Services)?;
    state.warriors[warrior].bytes =
        crate::recovered_equipment::read(&result.memory, address, WARRIOR_BYTES)?;
    state.load_item_work(content, upgrade)?;
    if result.native_return == u32::MAX {
        let mut feedback = vec![NativeFeedback::Sound { resource_id: sound }];
        feedback.extend(
            result
                .warnings
                .into_iter()
                .map(|index| NativeFeedback::Warning { index }),
        );
        return refusal(feedback);
    }
    let map = crate::exploration::active_map(state, content)?;
    crate::equipment::drop(&mut state.warriors[warrior], slot, content, map)?;
    state.load_item_work(content, item)?;
    Ok(vec![
        Event::Sound { resource_id: sound },
        Event::Sound { resource_id: -217 },
        Event::ItemUpgraded {
            warrior,
            slot: target - usize::from(slot < target),
        },
    ])
}
