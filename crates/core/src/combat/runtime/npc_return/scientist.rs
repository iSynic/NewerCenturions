//! The friendly rifle caller over the static records after combat teardown.
use super::{firearm_selection::RetainedSelection, services::FirearmOutcome, work};
use crate::combat::{
    ally_turns::{ScientistWorld, Shot, fire_scientist_in},
    runtime::{FireOutcome, TurnFlow, outdoor_death},
    spells::SelectionView,
    weapon_work::WeaponWork,
};
use crate::{Content, Event, RandomState, RuleError, Session};
use centurions_content::{combat::MonsterRecord, mission::OpeningMission, records::ItemDefinition};

struct Retained<'a> {
    state: &'a mut Session,
    content: &'a Content,
}

impl Retained<'_> {
    fn view(&self) -> Result<RetainedSelection<'_>, RuleError> {
        RetainedSelection::new(
            &self.state.native_work.npc_backing,
            &self.state.native_work.tactical_work,
            &self.state.native_work.warrior_table,
        )
    }
}

impl ScientistWorld for Retained<'_> {
    fn random(&mut self) -> &mut RandomState {
        &mut self.state.random
    }
    fn npc_count(&self) -> Result<usize, RuleError> {
        Ok(self.view()?.npc_count())
    }
    fn npc(&self, index: usize) -> Result<&MonsterRecord, RuleError> {
        Ok(&self.state.native_work.npc_backing.slot(index)?.record)
    }
    fn npc_mut(&mut self, index: usize) -> Result<&mut MonsterRecord, RuleError> {
        Ok(&mut self.state.native_work.npc_backing.slot_mut(index)?.record)
    }
    fn npc_and_random(
        &mut self,
        index: usize,
    ) -> Result<(&mut MonsterRecord, &mut RandomState), RuleError> {
        let state = &mut *self.state;
        Ok((
            &mut state.native_work.npc_backing.slot_mut(index)?.record,
            &mut state.random,
        ))
    }
    fn duel(
        &mut self,
        warrior: usize,
        npc: usize,
        resolve: crate::combat::ally_turns::Duel<'_>,
    ) -> Result<crate::combat::attack::AttackResult, RuleError> {
        // Party targets below 6 past the squad strike the held warrior table record.
        let state = &mut *self.state;
        let mut record = state
            .table_record(warrior)
            .ok_or("retained NPC firearm duel outside the warrior table")?;
        let result = resolve(
            &mut record,
            &mut state.native_work.npc_backing.slot_mut(npc)?.record,
            &mut state.random,
        )?;
        state.store_table_record(warrior, record)?;
        Ok(result)
    }
    fn alive(&self, body: i16) -> Result<bool, RuleError> {
        if (0..6).contains(&body) {
            Ok(self
                .state
                .warriors
                .get(body as usize)
                .is_some_and(|warrior| warrior.record().active_flag() != 0))
        } else if body >= 10 {
            let record = self.npc((body - 10) as usize)?;
            Ok(record.stamina()? > 0 && !record.record().removed())
        } else {
            Ok(false)
        }
    }
    fn faction(&self, body: i16) -> Result<i16, RuleError> {
        self.view()?.faction(&self.state.warriors, body as usize)
    }
    fn position(&self, body: i16) -> Result<(i16, i16), RuleError> {
        self.view()?.position(body as usize)
    }
    fn visible(
        &self,
        mission: &OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError> {
        self.view()?.visible(mission, from, to)
    }
    fn active_body(&self) -> Result<Option<i16>, RuleError> {
        work::body(self.state).map(Some)
    }
    fn difficulty(&self) -> i16 {
        self.state.difficulty
    }
    fn weapon_work(&mut self) -> &mut WeaponWork {
        &mut self.state.native_work.weapon_work
    }
    fn buffers(&mut self) -> &mut crate::ammunition::WorkBuffers {
        &mut self.state.native_work.ammunition_work
    }
    fn publish(&mut self, packet: Vec<u8>, count: i16) {
        self.state.native_work.attack_count = count;
        self.state.native_work.attack_packet = packet;
        self.state.native_work.attack_identity = [3, 0, 0];
    }
    fn refresh_item(&mut self, items: &[Vec<u8>], body: i16) -> Result<(), RuleError> {
        let id = self.npc((body - 10) as usize)?.record().selected_item()?;
        if id != 0 {
            self.state
                .native_work
                .ammunition_work
                .load_item_from(items, id)?;
        }
        Ok(())
    }
    fn kill(&mut self, body: i16, events: &mut Vec<Event>) -> Result<bool, RuleError> {
        outdoor_death::defeat(self.state, self.content, body, events)
    }
}

/// combat1146ee's friendly rifle branch after teardown; killbody takes its
/// combat-zero suffix.
pub(super) fn fire(
    state: &mut Session,
    content: &Content,
    item_context: i16,
) -> Result<FirearmOutcome, RuleError> {
    let body = work::selected(state)? as i16 + 10;
    let mission = work::mission(state, content)?;
    let mut events = vec![];
    let shot = fire_scientist_in(
        &mut Retained { state, content },
        mission,
        &content.items,
        &mut events,
        (item_context, body),
        true,
    )?;
    if shot != Shot::Ended
        && let Some(item) = &state.native_work.ammunition_work.item
    {
        state.native_work.loaded_item_field32 = ItemDefinition(item).retained_field32()?;
    }
    let (outcome, flow) = match shot {
        Shot::Withheld => (FireOutcome::Failed, TurnFlow::Returned),
        Shot::Fired => (FireOutcome::Physical, TurnFlow::Returned),
        Shot::Ended => (FireOutcome::Physical, TurnFlow::Terminated),
    };
    Ok(FirearmOutcome {
        events,
        outcome,
        flow,
    })
}
