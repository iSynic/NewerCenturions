//! Complete CODE 1 battle-order control with explicit native stack memory.

use crate::{
    RuleError,
    recovered_records::RecordWrite,
    recovered_runtime::{self, RuntimeContext, RuntimeEffect, RuntimeMemory, RuntimeResult},
};

const ORDER: u32 = 0x4470b8;
const SORT_INDEX: u32 = 0x4434ac;
const LAST_MEMBER: u32 = 0x44273a;
const EXCLUDED: u32 = 0x442739;
const ENEMY_COUNT: u32 = 0x43874a;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderEffect {
    Write(RecordWrite),
    Runtime(RuntimeEffect),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BattleOrderPlan {
    pub context: RuntimeContext,
    pub effects: Vec<OrderEffect>,
}

struct Execution {
    context: RuntimeContext,
    effects: Vec<OrderEffect>,
    iterations: usize,
    draws: u64,
    maximum_draws: u64,
}

fn invalid(label: &str) -> RuleError {
    RuleError::Invalid(format!("battle order unavailable: {label}"))
}

fn byte(memory: &RuntimeMemory, address: u32) -> Result<i8, RuleError> {
    Ok(memory.read(address, 1)?[0] as i8)
}

fn indexed(base: u32, index: i16, stride: i32) -> u32 {
    base.wrapping_add(i32::from(index).wrapping_mul(stride) as u32)
}

impl Execution {
    fn step(&mut self) -> Result<(), RuleError> {
        if self.iterations >= self.context.step_limit {
            return Err(invalid("loop iteration budget exhausted"));
        }
        self.iterations += 1;
        Ok(())
    }

    fn put(&mut self, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.context.memory.read(address, bytes.len())?;
        self.context.memory.insert_region(address, bytes);
        self.effects.push(OrderEffect::Write(RecordWrite {
            address,
            bytes: bytes.to_vec(),
        }));
        Ok(())
    }

    fn put_word(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.put(address, &value.to_be_bytes())
    }

    fn bound(&self, frame_pointer: u32) -> Result<i16, RuleError> {
        Ok(i16::from(byte(&self.context.memory, LAST_MEMBER)?)
            .wrapping_add(self.context.memory.word(ENEMY_COUNT)?)
            .wrapping_sub(i16::from(byte(&self.context.memory, EXCLUDED)?))
            .wrapping_add(1)
            .wrapping_add(i16::from(byte(
                &self.context.memory,
                frame_pointer.wrapping_sub(7),
            )?)))
    }

    fn random_slot(&mut self, frame_pointer: u32) -> Result<i16, RuleError> {
        if self.draws >= self.maximum_draws {
            return Err(invalid("random placement draw budget exhausted"));
        }
        let bound = self.bound(frame_pointer)?;
        let child = recovered_runtime::rand(&mut self.context, bound)?;
        let RuntimeResult::Word { value, .. } = child.result else {
            unreachable!()
        };
        self.effects
            .extend(child.effects.into_iter().map(OrderEffect::Runtime));
        self.draws += 1;
        Ok(value)
    }

    fn sort_bound(&self) -> Result<i16, RuleError> {
        Ok(i16::from(byte(&self.context.memory, LAST_MEMBER)?)
            .wrapping_add(self.context.memory.word(ENEMY_COUNT)?))
    }

    fn score(&self, body: i16) -> Result<Option<i16>, RuleError> {
        if body <= -1 {
            Ok(None)
        } else if body <= 9 {
            let address = 0x448aa0 + u32::from(body as u16) * 3138 + 0x9e8;
            Ok(Some(self.context.memory.word(address)?))
        } else {
            let index = body.wrapping_sub(10) as u16;
            Ok(Some(
                self.context
                    .memory
                    .word(0x439d3c + u32::from(index) * 288 + 4)?,
            ))
        }
    }
}

/// getbattleorder, 00124d9e. Unavailable stack bytes and exhausted budgets diagnose.
pub fn getbattleorder(
    current: &RuntimeContext,
    frame_pointer: u32,
    mode: i16,
    maximum_draws: u64,
) -> Result<BattleOrderPlan, RuleError> {
    let mode_address = frame_pointer.wrapping_add(8);
    if current.memory.word(mode_address)? != mode {
        return Err(invalid(
            "caller mode WORD disagrees with its native argument slot",
        ));
    }
    let mut x = Execution {
        context: current.clone(),
        effects: vec![],
        iterations: 0,
        draws: 0,
        maximum_draws,
    };
    let excluded = byte(&x.context.memory, EXCLUDED)?;
    let mut remaining = byte(&x.context.memory, LAST_MEMBER)?.wrapping_sub(excluded);
    x.put(frame_pointer.wrapping_sub(7), &[0])?;
    x.put_word(0x44374c, 0)?;
    x.put_word(0x447194, 0)?;
    let mut count = 0_i8;
    x.put_word(frame_pointer.wrapping_sub(10), 0)?;
    let mut member = 0_i16;
    while member <= i16::from(byte(&x.context.memory, LAST_MEMBER)?) {
        x.step()?;
        let address = 0x448aa0 + u32::from(member as u16) * 3138;
        if x.context.memory.word(address + 0x50)? > 0 {
            let index = count;
            count = count.wrapping_add(1);
            x.put(
                indexed(frame_pointer.wrapping_sub(6), i16::from(index), 1),
                &[member as u8],
            )?;
            if x.context.memory.word(address + 0x90)? != 0 {
                remaining = remaining.wrapping_add(1);
                let value = byte(&x.context.memory, frame_pointer.wrapping_sub(7))?.wrapping_add(1);
                x.put(frame_pointer.wrapping_sub(7), &[value as u8])?;
            }
        }
        member = member.wrapping_add(1);
    }
    x.put_word(0x4435c2, 2)?;
    for index in 0..110_i16 {
        x.step()?;
        x.put_word(indexed(ORDER, index, 2), -1)?;
    }
    let placement_mode = x.context.memory.word(mode_address)?;
    if placement_mode == -1 || placement_mode == 1 {
        let mut index = 0_i16;
        while index < i16::from(count) {
            x.step()?;
            let body = i16::from(byte(
                &x.context.memory,
                indexed(frame_pointer.wrapping_sub(6), index, 1),
            )?);
            let destination = if placement_mode == -1 {
                109_i16.wrapping_sub(index)
            } else {
                index.wrapping_add(1)
            };
            x.put_word(indexed(ORDER, destination, 2), body)?;
            index = index.wrapping_add(1);
        }
    } else {
        while remaining != -1 {
            x.step()?;
            let slot = x.random_slot(frame_pointer)?;
            let address = indexed(ORDER, slot, 2);
            if x.context.memory.word(address)? == -1 {
                let old_count = count;
                count = count.wrapping_sub(1);
                let index = i16::from(old_count).wrapping_sub(1);
                let body = i16::from(byte(
                    &x.context.memory,
                    indexed(frame_pointer.wrapping_sub(6), index, 1),
                )?);
                x.put_word(address, body)?;
                remaining = remaining.wrapping_sub(1);
            }
        }
    }
    // The BYTE is from the low half of the enemy-count WORD, not its signed WORD.
    let mut enemy = byte(&x.context.memory, ENEMY_COUNT + 1)?;
    while enemy != 0 {
        x.step()?;
        let slot = x.random_slot(frame_pointer)?;
        let address = indexed(ORDER, slot, 2);
        if x.context.memory.word(address)? == -1 {
            let old_enemy = enemy;
            enemy = enemy.wrapping_sub(1);
            x.put_word(address, i16::from(old_enemy).wrapping_add(9))?;
        }
    }
    if x.context.memory.word(mode_address)? == 0 {
        let mut outer = 1_i16;
        while outer <= x.sort_bound()? {
            x.step()?;
            x.put_word(SORT_INDEX, outer)?;
            while x.context.memory.word(SORT_INDEX)? <= x.sort_bound()? {
                x.step()?;
                let index = x.context.memory.word(SORT_INDEX)?;
                let left_address = indexed(ORDER, index, 2);
                let left = x.context.memory.word(left_address)?;
                if let Some(left_score) = x.score(left)? {
                    let next_index = x.context.memory.word(SORT_INDEX)?.wrapping_add(1);
                    let right_address = indexed(ORDER, next_index, 2);
                    let right = x.context.memory.word(right_address)?;
                    if let Some(right_score) = x.score(right)?
                        && right_score > left_score
                    {
                        // The original swaps exactly these adjacent slots.
                        x.put_word(left_address, right)?;
                        let final_right_address =
                            indexed(ORDER, x.context.memory.word(SORT_INDEX)?.wrapping_add(1), 2);
                        x.put_word(final_right_address, left)?;
                    }
                }
                let next = x.context.memory.word(SORT_INDEX)?.wrapping_add(1);
                x.put_word(SORT_INDEX, next)?;
            }
            outer = outer.wrapping_add(1);
        }
    }
    for index in 1..110_i16 {
        x.step()?;
        let body = x.context.memory.word(indexed(ORDER, index, 2))?;
        if (0..6).contains(&body) {
            x.put_word(0x448aa0 + u32::from(body as u16) * 3138 + 0x20, index)?;
        }
    }
    Ok(BattleOrderPlan {
        context: x.context,
        effects: x.effects,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RandomState;

    fn context(last: i8, excluded: i8, enemies: i16, mode: i16) -> RuntimeContext {
        let mut context = RuntimeContext::new(RandomState::new(12), 10_000);
        context
            .memory
            .insert_region(EXCLUDED, &[excluded as u8, last as u8]);
        context
            .memory
            .insert_region(ENEMY_COUNT, &enemies.to_be_bytes());
        for (at, size) in [
            (0x44374c, 2),
            (0x447194, 2),
            (0x4435c2, 2),
            (SORT_INDEX, 2),
            (ORDER, 220),
            (0x8ff6, 10),
        ] {
            context.memory.insert_region(at, &vec![0xcc; size]);
        }
        context.memory.insert_region(0x9000, &[0xcc; 10]);
        context.memory.insert_region(0x9008, &mode.to_be_bytes());
        for body in 0..=i16::from(last) {
            for (offset, value) in [(0x50, 10), (0x90, 0), (0x9e8, 10 + body), (0x20, -1)] {
                context.memory.insert_region(
                    0x448aa0 + u32::from(body as u16) * 3138 + offset,
                    &value.to_be_bytes(),
                );
            }
        }
        for body in 0..enemies {
            context
                .memory
                .insert_region(0x439d40 + body as u32 * 288, &5_i16.to_be_bytes());
        }
        context
    }

    #[test]
    fn reverse_party_keeps_enemy_draw_and_full_slot_reset_write_order() {
        let current = context(1, 0, 1, -1);
        let plan = getbattleorder(&current, 0x9000, -1, 100).unwrap();
        assert_eq!(current.random.draws, 0);
        assert_eq!(plan.context.random.draws, 1);
        assert_eq!(plan.context.random.seed, 201684);
        assert_eq!(plan.context.memory.word(ORDER + 2).unwrap(), 10);
        assert_eq!(plan.context.memory.word(ORDER + 109 * 2).unwrap(), 0);
        assert_eq!(plan.context.memory.word(ORDER + 108 * 2).unwrap(), 1);
        assert_eq!(plan.context.memory.word(0x448ac0).unwrap(), 109);
        assert_eq!(
            plan.context.memory.word(0x448aa0 + 3138 + 0x20).unwrap(),
            108
        );
        let resets = plan.effects.iter().filter(|e| matches!(e,OrderEffect::Write(w) if w.address>=ORDER && w.address<ORDER+220 && w.bytes==[0xff,0xff])).count();
        assert_eq!(resets, 110);
    }

    #[test]
    fn default_preserves_dead_members_and_atomic_draw_or_memory_diagnostics() {
        let mut current = context(2, 1, 0, 0);
        current
            .memory
            .insert_region(0x448aa0 + 3138 + 0x50, &0_i16.to_be_bytes());
        let plan = getbattleorder(&current, 0x9000, 0, 100).unwrap();
        assert_eq!(plan.context.memory.word(ORDER + 2).unwrap(), 2);
        assert_eq!(plan.context.memory.word(ORDER + 4).unwrap(), 0);
        assert_eq!(
            plan.context.memory.word(0x448aa0 + 3138 + 0x20).unwrap(),
            -1
        );
        let before = current.clone();
        assert!(getbattleorder(&current, 0x9000, 0, 0).is_err());
        assert_eq!(current, before);
        current.memory.bytes.remove(&0x8ffa);
        let before = current.clone();
        assert!(getbattleorder(&current, 0x9000, 0, 100).is_err());
        assert_eq!(current, before);
    }

    #[test]
    fn bonus_placements_read_the_native_local_counter_when_list_index_underflows() {
        let mut current = context(1, 0, 0, 2);
        current
            .memory
            .insert_region(0x448aa0 + 0x90, &1_i16.to_be_bytes());
        let plan = getbattleorder(&current, 0x9000, 2, 100).unwrap();
        let bodies: Vec<_> = (1..=3)
            .map(|i| plan.context.memory.word(ORDER + i * 2).unwrap())
            .collect();
        assert_eq!(bodies.iter().filter(|b| **b == 0).count(), 1);
        assert_eq!(bodies.iter().filter(|b| **b == 1).count(), 2);
        assert_eq!(plan.context.memory.read(0x8ff9, 1).unwrap(), [1]);
        // Index fourteen of the native local list reaches the mode argument.
        let current = context(14, 0, 0, -1);
        let plan = getbattleorder(&current, 0x9000, -1, 100).unwrap();
        assert_eq!(plan.context.memory.word(0x9008).unwrap(), 0x0eff);
        assert!(plan.context.random.draws > 0);
        assert_eq!(plan.context.memory.word(ORDER + 109 * 2).unwrap(), -1);
        assert!(getbattleorder(&current, 0x9000, 1, 100).is_err());
    }
}
