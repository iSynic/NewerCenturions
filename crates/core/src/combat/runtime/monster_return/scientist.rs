//! The friendly scientist's rifle, fired after the battle has ended.
use super::{firearm_selection::OutOfCombatSelection, services::FirearmOutcome, turn_state};
use crate::combat::ally_turns::Duel;
use crate::combat::attack::AttackResult;
use crate::combat::{
    ally_turns::{ScientistWorld, Shot, fire_scientist_in},
    firearm::weapon_traits::WeaponTraits,
    runtime::{FireOutcome, TurnFlow, outdoor_death},
    spells::SelectionView,
};
use crate::{Content, Event, RandomState, RuleError, Session};
use centurions_content::{combat::MonsterRecord, mission::OpeningMission, records::ItemDefinition};

/// The friendly scientist's rifle shot seen over the party's state after the battle has ended.
struct OutOfCombat<'a> {
    state: &'a mut Session,
    content: &'a Content,
}

impl OutOfCombat<'_> {
    /// The monsters and map as the shot sees them.
    fn view(&self) -> Result<OutOfCombatSelection<'_>, RuleError> {
        OutOfCombatSelection::new(
            &self.state.globals.monsters,
            &self.state.globals.battlefield,
            &self.state.globals.warrior_table,
        )
    }
}

impl ScientistWorld for OutOfCombat<'_> {
    /// The random numbers the shot draws.
    fn random(&mut self) -> &mut RandomState {
        &mut self.state.random
    }
    /// How many monsters the shot may pick from.
    fn monster_count(&self) -> Result<usize, RuleError> {
        Ok(self.view()?.monster_count())
    }
    /// The monster in a slot.
    fn monster(&self, index: usize) -> Result<&MonsterRecord, RuleError> {
        Ok(&self.state.globals.monsters.slot(index)?.record)
    }
    /// The monster in a slot, to change it.
    fn monster_mut(&mut self, index: usize) -> Result<&mut MonsterRecord, RuleError> {
        Ok(&mut self.state.globals.monsters.slot_mut(index)?.record)
    }
    /// The monster in a slot together with the random numbers, for a roll that changes it.
    fn monster_and_random(
        &mut self,
        index: usize,
    ) -> Result<(&mut MonsterRecord, &mut RandomState), RuleError> {
        let state = &mut *self.state;
        Ok((
            &mut state.globals.monsters.slot_mut(index)?.record,
            &mut state.random,
        ))
    }
    /// The rifle's duel with a monster: the warrior's record, even one beyond the squad, is struck
    /// or spared by the same roll as in a battle.
    fn duel(
        &mut self,
        warrior: usize,
        monster: usize,
        resolve: Duel<'_>,
    ) -> Result<AttackResult, RuleError> {
        // Party targets below 6 past the squad strike the held warrior table record.
        let state = &mut *self.state;
        let mut record = state
            .table_record(warrior)
            .ok_or("retained NPC firearm duel outside the warrior table")?;
        let result = resolve(
            &mut record,
            &mut state.globals.monsters.slot_mut(monster)?.record,
            &mut state.random,
        )?;
        state.store_table_record(warrior, record)?;
        Ok(result)
    }
    /// Whether a body can be shot at: a warrior in the battle with stamina left, or a monster with
    /// stamina that has not been removed.
    fn targetable(&self, body: i16) -> Result<bool, RuleError> {
        if (0..6).contains(&body) {
            match self.state.warriors.get(body as usize) {
                Some(warrior) => {
                    Ok(warrior.record().in_battle() != 0 && warrior.record().stamina()? > 0)
                }
                None => Ok(false),
            }
        } else if body >= 10 {
            let record = self.monster((body - 10) as usize)?;
            Ok(record.stamina()? > 0 && !record.record().removed())
        } else {
            Ok(false)
        }
    }
    /// The side a body fights on.
    fn faction(&self, body: i16) -> Result<i16, RuleError> {
        self.view()?.faction(&self.state.warriors, body as usize)
    }
    /// The cell a body stands in.
    fn position(&self, body: i16) -> Result<(i16, i16), RuleError> {
        self.view()?.position(body as usize)
    }
    /// Whether one cell sees another.
    fn visible(
        &self,
        mission: &OpeningMission,
        from: (i16, i16),
        to: (i16, i16),
    ) -> Result<bool, RuleError> {
        self.view()?.visible(mission, from, to)
    }
    /// The body whose turn it is.
    fn active_body(&self) -> Result<Option<i16>, RuleError> {
        turn_state::body(self.state).map(Some)
    }
    /// The difficulty of the game.
    fn difficulty(&self) -> i16 {
        self.state.difficulty
    }
    /// The traits of the weapon being used.
    fn weapon_traits(&mut self) -> &mut WeaponTraits {
        &mut self.state.globals.weapon_traits
    }
    /// The loaded item buffers.
    fn buffers(&mut self) -> &mut crate::ammunition::LoadedItems {
        &mut self.state.globals.loaded_items
    }
    /// Makes the packet the current spell, sets the attack count and gives the cast the identity
    /// of a firearm shot.
    fn publish(&mut self, packet: Vec<u8>, count: i16) {
        self.state.globals.attack_count = count;
        self.state.globals.spell_info = packet;
        self.state.globals.cast_identity = [3, 0, 0];
    }
    /// Loads the item the shooting monster holds, if any, as the item being worked on.
    fn refresh_item(&mut self, items: &[Vec<u8>], body: i16) -> Result<(), RuleError> {
        let id = self.monster((body - 10) as usize)?.record().weapon()?;
        if id != 0 {
            self.state.globals.loaded_items.load_item_from(items, id)?;
        }
        Ok(())
    }
    /// Defeats a body the rifle killed, with its death story.
    fn kill(&mut self, body: i16, events: &mut Vec<Event>) -> Result<bool, RuleError> {
        outdoor_death::defeat(self.state, self.content, body, events)
    }
}

/// The friendly scientist fires its rifle after the battle has ended.
///
/// Original: combat
pub(super) fn fire(
    state: &mut Session,
    content: &Content,
    item_context: i16,
) -> Result<FirearmOutcome, RuleError> {
    let body = turn_state::selected(state)? as i16 + 10;
    let mission = turn_state::mission(state, content)?;
    let mut events = vec![];
    let shot = fire_scientist_in(
        &mut OutOfCombat { state, content },
        mission,
        &content.items,
        &mut events,
        (item_context, body),
        true,
    )?;
    if shot != Shot::Ended
        && let Some(item) = &state.globals.loaded_items.item
    {
        state.globals.loaded_item_field32 = ItemDefinition(item).ammunition_caliber()?;
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
