//! Original takegold 0x15d210. Credits in the shared pool and carried tokens
//! are intentionally different: the admission sum doubles tokens in a WORD,
//! then payment removes one token per visit in party order.
use crate::RuleError;
use centurions_content::{WARRIOR_BYTES, Warrior};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentState {
    pub party_gold: i32,
    pub highest_party_index: i8,
    pub warriors: Vec<Warrior>,
}

pub enum PaymentEffect {
    PartyCredits(i32),
    WarriorCredits { index: usize, credits: i16 },
    InsufficientCredits,
}

pub struct Payment {
    pub state: PaymentState,
    pub effects: Vec<PaymentEffect>,
    pub paid: bool,
}

/// takegold reads a signed WORD cost and uses the native doubled WORD token check.
pub fn pay(state: &PaymentState, amount: i16) -> Result<Payment, RuleError> {
    let mut result = Payment {
        state: state.clone(),
        effects: Vec::new(),
        paid: true,
    };
    if i32::from(amount) < state.party_gold {
        result.state.party_gold = state.party_gold.wrapping_sub(i32::from(amount));
        result
            .effects
            .push(PaymentEffect::PartyCredits(result.state.party_gold));
        return Ok(result);
    }
    let mut total = 0_u16;
    for member in 0..=i16::from(state.highest_party_index) {
        let warrior = payment_warrior(state, member as usize)?;
        let tokens = warrior.record().carried_credits()? as u16;
        total = total.wrapping_add(tokens).wrapping_add(tokens);
    }
    let available = state.party_gold.wrapping_add(i32::from(total));
    if available < i32::from(amount) {
        result.effects.push(PaymentEffect::InsufficientCredits);
        result.paid = false;
        return Ok(result);
    }
    let mut remaining = amount.wrapping_sub(state.party_gold as i16);
    result.state.party_gold = 0;
    result.effects.push(PaymentEffect::PartyCredits(0));
    let mut member = 0_i8;
    let mut decremented_this_cycle = false;
    loop {
        let index = usize::try_from(member).map_err(|_| {
            unavailable(
                "inventory.gold",
                "round-robin BYTE addressed before party table",
            )
        })?;
        let warrior = payment_warrior(&result.state, index)?;
        let tokens = warrior.record().carried_credits()?;
        if tokens != 0 {
            remaining = remaining.wrapping_sub(1);
            let remaining_tokens = tokens.wrapping_sub(1);
            result.state.warriors[index]
                .record_mut()
                .set_carried_credits(remaining_tokens);
            result.effects.push(PaymentEffect::WarriorCredits {
                index,
                credits: remaining_tokens,
            });
            decremented_this_cycle = true;
        }
        member = member.wrapping_add(1);
        if member > state.highest_party_index {
            member = 0;
            if remaining != 0 && !decremented_this_cycle {
                return Err(unavailable(
                    "inventory.gold",
                    "native token loop cannot finish after token exhaustion",
                ));
            }
            decremented_this_cycle = false;
        }
        // This is a do-while: an initially zero remainder still visits member zero once.
        if remaining == 0 {
            break;
        }
    }
    Ok(result)
}

fn payment_warrior(state: &PaymentState, member: usize) -> Result<&Warrior, RuleError> {
    let warrior = state
        .warriors
        .get(member)
        .ok_or_else(|| unavailable("inventory.gold", "party record outside supplied table"))?;
    if warrior.bytes.len() != WARRIOR_BYTES {
        return Err(unavailable(
            "inventory.record",
            "requires a complete warrior record",
        ));
    }
    Ok(warrior)
}

fn unavailable(action: &'static str, dependency: &'static str) -> RuleError {
    RuleError::Unsupported { action, dependency }
}
