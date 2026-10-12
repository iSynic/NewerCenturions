use crate::{
    RandomState, RuleError,
    recovered_install::InstallMemory,
    recovered_records::{self, ITEM_TABLE_ADDRESSES, ItemTables, RecordWrite},
    rules,
};
use centurions_content::ITEM_BYTES;

const ITEM: u32 = 0x45363c;
const BOOTY: u32 = 0x41fc74;
const DIFFICULTY: u32 = 0x442576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RandomItemLimits {
    pub maximum_draws: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemRandomDraw {
    pub bound: i16,
    pub quickdraw: i16,
    pub result: i16,
    pub seed_before: u32,
    pub seed_after: u32,
    pub draw_number: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RandomItemEffect {
    Write(RecordWrite),
    Random(ItemRandomDraw),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RandomizedItem {
    pub memory: InstallMemory,
    pub random: RandomState,
    pub effects: Vec<RandomItemEffect>,
    pub native_return: i32,
}

struct Execution {
    memory: InstallMemory,
    random: RandomState,
    effects: Vec<RandomItemEffect>,
    remaining_draws: u64,
}

impl Execution {
    fn span(&self, address: u32, length: usize) -> Result<&[u8], RuleError> {
        let offset = address
            .checked_sub(self.memory.base_address)
            .map(|n| n as usize)
            .ok_or_else(unavailable)?;
        let end = offset.checked_add(length).ok_or_else(unavailable)?;
        self.memory.bytes.get(offset..end).ok_or_else(unavailable)
    }

    fn byte(&self, address: u32) -> Result<u8, RuleError> {
        Ok(self.span(address, 1)?[0])
    }

    fn word(&self, address: u32) -> Result<i16, RuleError> {
        let bytes = self.span(address, 2)?;
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn write(&mut self, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.span(address, bytes.len())?;
        let offset = (address - self.memory.base_address) as usize;
        self.memory.bytes[offset..offset + bytes.len()].copy_from_slice(bytes);
        self.effects.push(RandomItemEffect::Write(RecordWrite {
            address,
            bytes: bytes.to_vec(),
        }));
        Ok(())
    }

    fn put_word(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }

    fn copy(&mut self, source: u32, destination: u32, length: usize) -> Result<(), RuleError> {
        let bytes = self.span(source, length)?.to_vec();
        self.write(destination, &bytes)
    }

    fn add_byte(&mut self, address: u32, increment: u8) -> Result<(), RuleError> {
        let value = self.byte(address)?.wrapping_add(increment);
        self.write(address, &[value])
    }

    fn rand(&mut self, bound: i16) -> Result<i16, RuleError> {
        if self.remaining_draws == 0 {
            return Err(RuleError::Unsupported {
                action: "randomitem",
                dependency: "the supplied random-draw budget is exhausted",
            });
        }
        if self.random.draws == u64::MAX {
            return Err(RuleError::Unsupported {
                action: "randomitem",
                dependency: "the cumulative random-draw counter cannot represent another draw",
            });
        }
        self.remaining_draws -= 1;
        let seed_before = self.random.seed;
        let quickdraw = self.random.quickdraw();
        let result = rules::random_bound(bound, quickdraw);
        self.effects.push(RandomItemEffect::Random(ItemRandomDraw {
            bound,
            quickdraw,
            result,
            seed_before,
            seed_after: self.random.seed,
            draw_number: self.random.draws,
        }));
        Ok(result)
    }

    fn load_item(&mut self, id: i16) -> Result<(), RuleError> {
        let current: [u8; ITEM_BYTES] = self.span(ITEM, ITEM_BYTES)?.try_into().unwrap();
        let absolute = id.wrapping_abs();
        let family = absolute / 200;
        let mut families = [&[][..]; 5];
        if let Ok(family) = usize::try_from(family)
            && family < 5
        {
            let length = (absolute % 200) as usize * ITEM_BYTES + ITEM_BYTES;
            families[family] = self.span(ITEM_TABLE_ADDRESSES[family], length)?;
        }
        let loaded = recovered_records::loaditem(&ItemTables { families }, &current, id)?;
        for write in loaded.writes {
            self.write(write.address, &write.bytes)?;
        }
        Ok(())
    }

    fn finish(self) -> RandomizedItem {
        RandomizedItem {
            memory: self.memory,
            random: self.random,
            effects: self.effects,
            native_return: 0,
        }
    }
}

/// CODE 1 0x12d558. The raw image binds actual table/work/global addresses.
pub fn randomitem(
    current: &InstallMemory,
    random: &RandomState,
    slot: u16,
    mut enhancement: i16,
    limits: RandomItemLimits,
) -> Result<RandomizedItem, RuleError> {
    let mut execution = Execution {
        memory: current.clone(),
        random: random.clone(),
        effects: Vec::new(),
        remaining_draws: limits.maximum_draws,
    };
    let record = BOOTY + u32::from(slot) * 22;
    // This indexed BYTE uses signed D4.W; inventory addressing uses MULU.W.
    let enhancement_byte = (0x42df04i32 + i32::from(slot as i16)) as u32;
    let scale = execution.word(DIFFICULTY)?.wrapping_add(3);
    let id = execution.word(record)?;
    execution.load_item(id)?;
    for (destination, source) in [
        (10, 0x1b),
        (14, 0x4e),
        (15, 0x51),
        (11, 0x52),
        (16, 0x53),
        (17, 0x54),
        (18, 0x55),
        (19, 0x56),
        (21, 0x50),
        (20, 0x4f),
        (13, 0x8b),
    ] {
        execution.copy(ITEM + source, record + destination, 1)?;
    }
    if execution.word(ITEM + 0x5a)? != 0 {
        let maximum = execution
            .word(ITEM + 0x94)?
            .wrapping_add(execution.word(ITEM + 0x5a)?);
        execution.put_word(record + 6, maximum)?;
    } else if execution.word(ITEM + 0x18)? != 0 || execution.word(ITEM + 0x14)? > 0 {
        let maximum = execution
            .word(ITEM + 0x18)?
            .wrapping_add(execution.word(ITEM + 0x14)?);
        execution.put_word(record + 6, maximum)?;
        execution.copy(ITEM + 0x14, record + 2, 2)?;
    }
    if execution.word(record + 6)? < execution.word(record + 2)? {
        execution.copy(record + 2, record + 6, 2)?;
    }
    if execution.word(record + 6)? > 0 && [15, 13, 14].contains(&execution.byte(record + 10)?) {
        let threshold = execution
            .word(DIFFICULTY)?
            .wrapping_mul(10)
            .wrapping_add(50);
        if execution.rand(100)? < threshold {
            let quantity = execution.rand(execution.word(record + 6)?)?;
            execution.put_word(record + 2, quantity)?;
            if execution.word(ITEM + 0x20)? != 14 {
                let mut index = 0i16;
                while index < 200 {
                    if execution.word(ITEM + 0x20)?
                        == execution.word(0x46225e + index as u32 * 150 + 0x22)?
                    {
                        execution.put_word(record + 4, index.wrapping_add(200))?;
                        break;
                    }
                    index = index.wrapping_add(1);
                }
            }
        }
    } else if execution.word(record + 2)? > 0 {
        let quantity = execution.rand(execution.word(record + 6)?)?;
        execution.put_word(record + 2, quantity)?;
    }
    if execution.word(ITEM + 0x22)? != 0 {
        execution.copy(ITEM + 0x14, record + 2, 2)?;
        execution.put_word(record + 4, 0)?;
        execution.put_word(record + 6, 0)?;
    }
    let threshold = 94i16.wrapping_sub(execution.word(DIFFICULTY)?);
    if execution.rand(100)? < threshold && enhancement == 0 {
        return Ok(execution.finish());
    }
    let mut changes = 0i16;
    loop {
        let kind = execution.word(ITEM + 0x1a)?;
        if kind == 26 || kind == 27 || execution.word(ITEM + 0x24)? != 0 {
            return Ok(execution.finish());
        }
        if execution
            .word(ITEM + 0x3e)?
            .wrapping_add(execution.word(ITEM + 0x28)?)
            != 0
            || kind == 2
            || kind == 15
        {
            let mut index = 0i16;
            while index < scale.wrapping_mul(5) {
                if execution.rand(100)? < 7 {
                    let increment = execution.rand(5)? as u8;
                    let destination = if execution.word(ITEM + 0x28)? == 0 {
                        15
                    } else {
                        11
                    };
                    execution.add_byte(record + destination, increment)?;
                    changes = changes.wrapping_add(1);
                }
                index = index.wrapping_add(1);
            }
            let mut index = 0i16;
            while index < scale.wrapping_shl(2) {
                if execution.rand(100)? < 5 {
                    let increment = execution.rand(6)?.wrapping_add(4) as u8;
                    execution.add_byte(record + 18, increment)?;
                    changes = changes.wrapping_add(1);
                }
                index = index.wrapping_add(1);
            }
        } else if execution.byte(ITEM + 0x51)? != 0 {
            let mut index = 0i16;
            while index < scale.wrapping_mul(5) {
                if execution.rand(100)? < 7 {
                    let increment = execution.rand(3)? as u8;
                    execution.add_byte(record + 15, increment)?;
                    changes = changes.wrapping_add(1);
                }
                index = index.wrapping_add(1);
            }
        } else if execution.byte(ITEM + 0x55)? != 0 {
            let mut index = 0i16;
            while index < scale.wrapping_mul(5) {
                if execution.rand(100)? < 5 {
                    let increment = execution.rand(6)?.wrapping_add(4) as u8;
                    execution.add_byte(record + 18, increment)?;
                    changes = changes.wrapping_add(1);
                }
                index = index.wrapping_add(1);
            }
        }
        for (maximum_address, destination) in
            [(ITEM + 0x58, record + 14), (ITEM + 0x5e, record + 19)]
        {
            if execution.word(maximum_address)? > 0 {
                let mut index = 0i16;
                while index < execution.word(maximum_address)?.wrapping_mul(scale) {
                    if execution.rand(100)? < 2 {
                        execution.add_byte(destination, 1)?;
                        changes = changes.wrapping_add(1);
                    }
                    index = index.wrapping_add(1);
                }
                if i16::from(execution.byte(destination)? as i8)
                    > execution.word(maximum_address)?
                {
                    execution.copy(maximum_address + 1, destination, 1)?;
                }
            }
        }
        for (maximum_address, destination) in [(ITEM + 3, record + 16), (ITEM + 4, record + 17)] {
            if execution.byte(maximum_address)? as i8 > 0 {
                let mut index = 0i16;
                while index < scale.wrapping_shl(2) {
                    if execution.rand(100)? < 4 {
                        let increment = execution.rand(3)? as u8;
                        execution.add_byte(destination, increment)?;
                        changes = changes.wrapping_add(1);
                    }
                    index = index.wrapping_add(1);
                }
                if execution.byte(destination)? as i8 > execution.byte(maximum_address)? as i8 {
                    execution.copy(maximum_address, destination, 1)?;
                }
            }
        }
        if execution.word(ITEM + 0x3c)? > 0 {
            let mut index = 0i16;
            while index < scale.wrapping_add(scale) {
                if execution.rand(100)? < 4 {
                    execution.add_byte(record + 20, 1)?;
                    changes = changes.wrapping_add(1);
                }
                index = index.wrapping_add(1);
            }
            if i16::from(execution.byte(record + 20)? as i8) > execution.word(ITEM + 0x3c)? {
                execution.copy(ITEM + 0x4f, record + 20, 1)?;
            }
        }
        if i16::from(execution.byte(enhancement_byte)? as i8) < enhancement && changes != 0 {
            execution.write(enhancement_byte, &[changes as u8])?;
        }
        if execution.byte(enhancement_byte)? == 0 && changes != 0 {
            execution.write(enhancement_byte, &[1])?;
        }
        let modifier = i16::from(execution.byte(enhancement_byte)? as i8)
            .wrapping_add(execution.word(ITEM + 0x8a)?);
        execution.write(record + 13, &[modifier as u8])?;
        enhancement = enhancement.wrapping_sub(1);
        if enhancement <= 0 {
            break;
        }
    }
    Ok(execution.finish())
}

fn unavailable() -> RuleError {
    RuleError::Unsupported {
        action: "randomitem",
        dependency: "a reached native table, work record, or global memory span is unavailable",
    }
}
