//! The fast spell: Command with a digit casts the scan kept in a preset slot.
use super::party_psi::{begin_cast, require_field_turn};
use crate::combat::player_psi::{
    PartyCastRequest, charge_party, cost, mode_allowed, packet, require_outdoor_effect,
    require_outdoor_spell,
};
use crate::combat::spells::spell_descriptor;
use crate::feedback::GameFeedback;
use crate::{Event, RuleError, Session};
use centurions_content::{Content, records::PsiSelectionMemory};

/// The text shown when the selected warrior lacks the psi points for the preset scan.
const NOT_ENOUGH_SPELL_POINTS: u32 = 0x403ede;

/// The selected warrior casts the scan kept in preset `slot` (1 to 10) outside combat. The scan is
/// loaded first. A cost above the warrior's psi points shows the not-enough-spell-points text with
/// its sound and changes nothing more. A scan that cannot be cast in camp, or a preset with no
/// power, only sounds the refusal. Otherwise the psi points are spent, the warrior is marked as
/// having acted and the cast goes on as if Scan had been chosen with those settings.
///
/// Original: checkkeypad
pub fn fast_spell(
    state: &mut Session,
    content: &Content,
    slot: u8,
) -> Result<Vec<Event>, RuleError> {
    require_field_turn(state)?;
    let warrior = state.char_select_new;
    if !(1..=10).contains(&slot) || warrior >= state.warriors.len() {
        return Err(RuleError::Invalid("fast spell slot or warrior".into()));
    }
    let [school, level, index, power] =
        PsiSelectionMemory(&state.globals.define_spells).preset(warrior, usize::from(slot))?;
    let identity = [school, level, index];
    let mission = state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("mission missing".into()))?;
    let spell = spell_descriptor(identity);
    let loaded =
        if (0..=30).contains(&school) && (0..=8).contains(&level) && (0..=98).contains(&index) {
            packet(mission, spell)
        } else {
            Err(RuleError::Invalid("fast spell preset".into()))
        };
    let packet = match loaded {
        Ok(packet) => Some(packet),
        Err(_) if power == 0 => None,
        Err(error) => return Err(error),
    };
    if let Some(packet) = &packet {
        state.globals.spell_info = packet.clone();
        state.globals.cast_identity = identity;
    }
    state.globals.attack_count = power;
    let charge = match &packet {
        Some(packet) => cost(packet, power)?,
        None => 0,
    };
    if charge > state.warriors[warrior].psi_points()? {
        return Ok(vec![Event::Feedback(GameFeedback::Flash {
            address: NOT_ENOUGH_SPELL_POINTS,
            x: 30,
            y: 350,
            wait: 0,
            sound: -143,
        })]);
    }
    let Some(packet) =
        packet.filter(|packet| power != 0 && matches!(mode_allowed(packet, false), Ok(true)))
    else {
        return Ok(vec![Event::Sound { resource_id: 143 }]);
    };
    let request = PartyCastRequest {
        warrior,
        spell,
        power,
    };
    require_outdoor_spell(request)?;
    require_outdoor_effect(&packet)?;
    let result = charge_party(
        &state.warriors,
        &state.globals.define_spells,
        &state.party_condition,
        packet,
        request,
    )?;
    state.globals.health_context.affected[warrior] = 1;
    begin_cast(state, content, warrior, spell, power, result)
}
