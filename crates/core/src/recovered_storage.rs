//! Scenario storage control with observed stdio boundaries.

use crate::{
    RuleError,
    recovered_files::{
        self, FileBuffer, FileContext, FileEffect, FileObservation, FileReply, FileRequest,
    },
    recovered_load_control::{
        self, LoadArg, LoadEffect, LoadFlow, LoadObservation, LoadRequest, LoadServices,
    },
    recovered_records::RecordWrite,
    recovered_rendering::RenderingMemory,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageRequest {
    MoveWindow {
        window: u32,
        horizontal: i16,
        vertical: i16,
        front: u8,
    },
    Write {
        handle: u32,
        size: u32,
        count: u32,
        source: u32,
    },
    StringLength(u32),
    CompareCString {
        left: u32,
        right: u32,
    },
    GetFileInfo {
        name: u32,
        volume: i16,
        info: u32,
    },
    SetFileInfo {
        name: u32,
        volume: i16,
        info: u32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageObservation {
    pub memory: RenderingMemory,
    pub value: u32,
    pub d0: u32,
}
pub trait StorageServices: LoadServices {
    fn observe_storage(
        &self,
        request: &StorageRequest,
        memory: &RenderingMemory,
    ) -> Result<StorageObservation, RuleError>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageLimits {
    pub filename_capacity: usize,
    pub maximum_iterations: usize,
    pub maximum_polls: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageEffect {
    Write(RecordWrite),
    Load {
        request: LoadRequest,
        observation: LoadObservation,
    },
    Service {
        request: StorageRequest,
        observation: StorageObservation,
    },
    Child(LoadEffect),
    File(FileEffect),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoragePlan {
    pub memory: RenderingMemory,
    pub effects: Vec<StorageEffect>,
    pub flow: LoadFlow,
    pub native_return: Option<u32>,
}
pub struct Execution {
    pub plan: StoragePlan,
    pub limits: StorageLimits,
    remaining: usize,
}
impl Execution {
    pub fn new(
        memory: &RenderingMemory,
        limits: StorageLimits,
        native_return: Option<u32>,
    ) -> Self {
        Self {
            plan: StoragePlan {
                memory: memory.clone(),
                effects: vec![],
                flow: LoadFlow::Returned,
                native_return,
            },
            limits,
            remaining: limits.maximum_iterations,
        }
    }
    pub fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(RuleError::Unsupported {
                action: "storage_menu.processing",
                dependency: "the caller's loop budget is exhausted",
            })?;
        Ok(())
    }
    pub fn bytes(&self, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
        (0..length)
            .map(|i| self.plan.memory.byte(address.wrapping_add(i as u32)))
            .collect()
    }
    pub fn word(&self, address: u32) -> Result<i16, RuleError> {
        Ok(self.plan.memory.word(address)? as i16)
    }
    pub fn long(&self, address: u32) -> Result<u32, RuleError> {
        self.plan.memory.long(address)
    }
    pub fn byte(&self, address: u32) -> Result<u8, RuleError> {
        self.plan.memory.byte(address)
    }
    pub fn write(&mut self, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(address, bytes.len())?;
        self.plan.memory.insert_region(address, bytes);
        self.plan.effects.push(StorageEffect::Write(RecordWrite {
            address,
            bytes: bytes.to_vec(),
        }));
        Ok(())
    }
    pub fn ww(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }
    pub fn lw(&mut self, address: u32, value: u32) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }
    pub fn add(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.ww(address, self.word(address)?.wrapping_add(value))
    }
    pub fn load(
        &mut self,
        request: LoadRequest,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<u32, RuleError> {
        let observation = services.observe_load(&request, &self.plan.memory)?;
        if observation.flow != LoadFlow::Returned {
            return Err(RuleError::Invalid(
                "termination outside the recovered runtime boundary".into(),
            ));
        }
        let value = observation.value;
        self.plan.memory = observation.memory.clone();
        self.plan.effects.push(StorageEffect::Load {
            request,
            observation,
        });
        Ok(value)
    }
    pub fn service(
        &mut self,
        request: StorageRequest,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<u32, RuleError> {
        if let StorageRequest::Write {
            size,
            count,
            source,
            ..
        } = request
        {
            let length = size
                .checked_mul(count)
                .ok_or_else(|| RuleError::Invalid("stdio write span overflow".into()))?;
            self.bytes(source, length as usize)?;
        }
        let observation = services.observe_storage(&request, &self.plan.memory)?;
        let value = observation.value;
        self.plan.memory = observation.memory.clone();
        self.plan.effects.push(StorageEffect::Service {
            request,
            observation,
        });
        Ok(value)
    }
    pub fn toolbox(
        &mut self,
        opcode: u16,
        selector: Option<u32>,
        arguments: Vec<LoadArg>,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<u32, RuleError> {
        self.load(
            LoadRequest::Toolbox {
                opcode,
                selector,
                arguments,
            },
            services,
        )
    }
    pub fn active(&self) -> bool {
        self.plan.flow == LoadFlow::Returned
    }
    pub fn fatal(
        &mut self,
        child_frame: u32,
        code: i16,
        second: bool,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.ww(child_frame.wrapping_add(8), code)?;
        let child = if second {
            recovered_load_control::scratch2(&self.plan.memory, child_frame, services)?
        } else {
            recovered_load_control::scratch(&self.plan.memory, child_frame, services)?
        };
        self.plan.memory = child.memory;
        self.plan.flow = child.flow;
        self.plan
            .effects
            .extend(child.effects.into_iter().map(StorageEffect::Child));
        if !self.active() {
            self.plan.native_return = None;
        }
        Ok(())
    }
    fn cstring(&self, address: u32) -> Result<Vec<u8>, RuleError> {
        let mut bytes = vec![];
        for i in 0..=self.plan.memory.bytes.len() {
            let byte = self.byte(address.wrapping_add(i as u32))?;
            bytes.push(byte);
            if byte == 0 {
                return Ok(bytes);
            }
        }
        Err(RuleError::Invalid("unterminated filename C string".into()))
    }
    pub fn filename(
        &mut self,
        name: u32,
        temporary: bool,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<(), RuleError> {
        let mut context = FileContext {
            scenario_directory: if temporary {
                vec![]
            } else {
                self.cstring(0x44260c)?
            },
            variant: self.byte(0x44274c)?,
            filename: self.bytes(0x4389a4, self.limits.filename_capacity)?,
            ..FileContext::default()
        };
        let number = services.filename_number(context.variant, &self.plan.memory)?;
        let observation = FileObservation {
            request: FileRequest::NumberToString(i32::from(context.variant)),
            reply: FileReply::Pascal(number),
        };
        let name = self.cstring(name)?;
        let child = if temporary {
            recovered_files::get_temp_filename(&mut context, &name, &[observation])?
        } else {
            recovered_files::get_filename(&mut context, &name, &[observation])?
        };
        for effect in child.effects {
            if let FileEffect::Write {
                destination: FileBuffer::Filename,
                offset,
                bytes,
            } = &effect
            {
                self.write(0x4389a4_u32.wrapping_add(*offset as u32), bytes)?;
            } else {
                self.plan.effects.push(StorageEffect::File(effect));
            }
        }
        Ok(())
    }
    pub fn open(
        &mut self,
        name: u32,
        mode: u32,
        temporary: bool,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<u32, RuleError> {
        self.filename(name, temporary, services)?;
        self.load(
            LoadRequest::Open {
                name: 0x4389a4,
                mode,
            },
            services,
        )
    }
    pub fn read(
        &mut self,
        handle: u32,
        size: u32,
        destination: u32,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<u32, RuleError> {
        self.load(
            LoadRequest::Read {
                handle,
                size,
                count: 1,
                destination,
            },
            services,
        )
    }
    pub fn fwrite(
        &mut self,
        handle: u32,
        size: u32,
        source: u32,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<u32, RuleError> {
        self.service(
            StorageRequest::Write {
                handle,
                size,
                count: 1,
                source,
            },
            services,
        )
    }
    pub fn close(
        &mut self,
        handle: u32,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.load(LoadRequest::Close(handle), services)?;
        Ok(())
    }
    pub fn seek(
        &mut self,
        handle: u32,
        offset: u32,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.load(
            LoadRequest::Seek {
                handle,
                offset,
                origin: 0,
            },
            services,
        )?;
        Ok(())
    }
    fn checked_write(
        &mut self,
        handle: u32,
        size: u32,
        source: u32,
        code: i16,
        fatal_frame: u32,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<(), RuleError> {
        if self.fwrite(handle, size, source, services)? == 0 {
            self.fatal(fatal_frame, code, false, services)?;
        }
        Ok(())
    }
}
const LAND: [(u32, u32); 6] = [
    (0x478e42, 0x1068),
    (0x443086, 0x284),
    (0x4358e6, 0x1fa4),
    (0x4085b8, 0x3f48),
    (0x40c500, 0x3f48),
    (0x414390, 0x7e90),
];

/// saveland, 0x151e24. The index is an actual LONG argument, reread after services.
pub fn save_land(
    memory: &RenderingMemory,
    frame: u32,
    limits: StorageLimits,
    services: &(impl StorageServices + ?Sized),
) -> Result<StoragePlan, RuleError> {
    let mut e = Execution::new(memory, limits, Some(0));
    if e.long(frame.wrapping_add(8))? as i32 > 20 {
        e.fatal(frame.wrapping_sub(14), 8, true, services)?;
        if !e.active() {
            return Ok(e.plan);
        }
    }
    let handle = e.open(0x40181c, 0x401818, true, services)?;
    if handle == 0 {
        e.fatal(frame.wrapping_sub(14), 153, false, services)?;
        if !e.active() {
            return Ok(e.plan);
        }
    }
    e.seek(
        handle,
        e.long(frame.wrapping_add(8))?.wrapping_mul(0x12fb0),
        services,
    )?;
    for (source, size) in LAND {
        e.fwrite(handle, size, source, services)?;
    }
    e.close(handle, services)?;
    Ok(e.plan)
}
/// saveencounter, 0x152dc4. Both selectors are native WORD arguments.
pub fn save_encounter(
    memory: &RenderingMemory,
    frame: u32,
    limits: StorageLimits,
    services: &(impl StorageServices + ?Sized),
) -> Result<StoragePlan, RuleError> {
    let mut e = Execution::new(memory, limits, Some(0));
    let first = e.word(frame.wrapping_add(10))? == 1;
    let handle = e.open(
        if first { 0x401796 } else { 0x40178e },
        0x401792,
        true,
        services,
    )?;
    if handle == 0 {
        e.fatal(
            frame.wrapping_sub(14),
            if first { 154 } else { 155 },
            false,
            services,
        )?;
        if !e.active() {
            return Ok(e.plan);
        }
    }
    e.seek(
        handle,
        u32::from(e.word(frame.wrapping_add(8))? as u16) * if first { 0x1aa } else { 0x204 },
        services,
    )?;
    e.fwrite(
        handle,
        if first { 0x6a } else { 0x9c },
        if first { 0x443432 } else { 0x443396 },
        services,
    )?;
    e.close(handle, services)?;
    Ok(e.plan)
}
/// saveshop, 0x13006e. Its selected shop is a sign-extended BYTE then unsigned WORD.
pub fn save_shop(
    memory: &RenderingMemory,
    limits: StorageLimits,
    services: &(impl StorageServices + ?Sized),
) -> Result<StoragePlan, RuleError> {
    let mut e = Execution::new(memory, limits, Some(0));
    let handle = e.open(0x4031b2, 0x4031ae, true, services)?;
    if handle != 0 {
        e.seek(
            handle,
            u32::from(i16::from(e.byte(0x44273b)? as i8) as u16) * 0xbba,
            services,
        )?;
        e.fwrite(handle, 0xbba, 0x478288, services)?;
        e.close(handle, services)?;
    }
    Ok(e.plan)
}

/// resetgamefiles, 0x14f96e. Observed writes describe temporary-game copies only.
pub fn reset_game_files(
    memory: &RenderingMemory,
    frame: u32,
    limits: StorageLimits,
    services: &(impl StorageServices + ?Sized),
) -> Result<StoragePlan, RuleError> {
    let mut e = Execution::new(memory, limits, Some(0));
    e.ww(0x44255e, i16::from(e.byte(0x44274c)?))?;
    for index in 0..100 {
        e.write(0x44258c + index, &[0])?;
    }
    e.write(0x44274c, &[0])?;
    let fatal_frame = frame.wrapping_sub(30);
    while e.byte(0x44274c)? < 3 {
        e.step()?;
        for row in 0..90_u32 {
            e.ww(0x4434ac, 0)?;
            while e.word(0x4434ac)? < 90 {
                e.step()?;
                let column = i32::from(e.word(0x4434ac)?) as u32;
                e.write(
                    0x4358e6_u32.wrapping_add(row * 90).wrapping_add(column),
                    &[0],
                )?;
                e.add(0x4434ac, 1)?;
            }
        }
        let spells = e.open(0x401916, 0x401912, false, services)?;
        if spells != 0 {
            e.read(spells, 0x17ca, 0x4463f0, services)?;
            e.close(spells, services)?;
        }
        let doors = e.open(0x40190a, 0x401912, false, services)?;
        if doors != 0 {
            let output = e.open(0x401906, 0x401902, true, services)?;
            if output == 0 {
                e.fatal(fatal_frame, 164, false, services)?;
                if !e.active() {
                    return Ok(e.plan);
                }
            }
            let land = e.open(0x4018fa, 0x401912, false, services)?;
            if land == 0 {
                e.fatal(fatal_frame, 165, false, services)?;
                if !e.active() {
                    return Ok(e.plan);
                }
            }
            let rooms = e.open(0x4018f2, 0x401912, false, services)?;
            if rooms == 0 {
                e.fatal(fatal_frame, 166, false, services)?;
                if !e.active() {
                    return Ok(e.plan);
                }
            }
            while e.read(doors, 0x1068, 0x478e42, services)? == 1 {
                e.step()?;
                e.read(rooms, 0x284, 0x443086, services)?;
                for (destination, size) in LAND.into_iter().skip(3) {
                    e.read(land, size, destination, services)?;
                }
                for ((source, size), code) in LAND.into_iter().zip([167, 169, 170, 500, 501, 502]) {
                    e.checked_write(output, size, source, code, fatal_frame, services)?;
                    if !e.active() {
                        return Ok(e.plan);
                    }
                }
            }
            for handle in [doors, land, rooms, output] {
                e.close(handle, services)?;
            }
        }
        for (name, temp, source, size, open_code, write_code) in [
            (0x4018ea, 0x4018e6, 0x478288, 0xbba, 177, 178),
            (0x4018dc, 0x4018d6, 0x439d14, 0x28, 181, 182),
        ] {
            let input = e.open(name, 0x401912, false, services)?;
            if input != 0 {
                let output = e.open(temp, 0x401902, true, services)?;
                if output == 0 {
                    e.fatal(fatal_frame, open_code, false, services)?;
                    if !e.active() {
                        return Ok(e.plan);
                    }
                }
                while e.read(input, size, source, services)? == 1 {
                    e.step()?;
                    e.checked_write(output, size, source, write_code, fatal_frame, services)?;
                    if !e.active() {
                        return Ok(e.plan);
                    }
                }
                e.close(input, services)?;
                e.close(output, services)?;
            }
        }
        let input = e.open(0x4018ce, 0x401912, false, services)?;
        if input != 0 {
            let output = e.open(0x4018ca, 0x401902, true, services)?;
            if output == 0 {
                e.fatal(fatal_frame, 183, false, services)?;
                if !e.active() {
                    return Ok(e.plan);
                }
            }
            while e.read(input, 0x6a, 0x443432, services)? == 1 {
                e.step()?;
                e.ww(0x4434ac, 0)?;
                while e.word(0x4434ac)? < 4 {
                    e.step()?;
                    let destination = string_slot(e.word(0x4434ac)?);
                    e.load(
                        LoadRequest::GetIndexedString {
                            resource: 1,
                            index: 3,
                            destination,
                        },
                        services,
                    )?;
                    let destination = string_slot(e.word(0x4434ac)?);
                    e.read(input, 0x50, destination, services)?;
                    e.add(0x4434ac, 1)?;
                }
                e.checked_write(output, 0x6a, 0x443432, 184, fatal_frame, services)?;
                if !e.active() {
                    return Ok(e.plan);
                }
                e.ww(0x4434ac, 0)?;
                while e.word(0x4434ac)? < 4 {
                    e.step()?;
                    let source = string_slot(e.word(0x4434ac)?);
                    e.checked_write(output, 0x50, source, 185, fatal_frame, services)?;
                    if !e.active() {
                        return Ok(e.plan);
                    }
                    e.add(0x4434ac, 1)?;
                }
            }
            e.close(input, services)?;
            e.close(output, services)?;
        }
        let input = e.open(0x4018c0, 0x401912, false, services)?;
        if input != 0 {
            let output = e.open(0x4018bc, 0x401902, true, services)?;
            if output == 0 {
                e.fatal(fatal_frame, 186, false, services)?;
                if !e.active() {
                    return Ok(e.plan);
                }
            }
            while e.read(input, 0x9c, 0x443396, services)? != 0 {
                e.step()?;
                for index in 0..9 {
                    e.read(input, 0x28, string_slot(index), services)?;
                }
                e.checked_write(output, 0x9c, 0x443396, 187, fatal_frame, services)?;
                if !e.active() {
                    return Ok(e.plan);
                }
                for index in 0..9 {
                    e.checked_write(output, 0x28, string_slot(index), 188, fatal_frame, services)?;
                    if !e.active() {
                        return Ok(e.plan);
                    }
                }
            }
            e.close(input, services)?;
            e.close(output, services)?;
        }
        e.write(0x44274c, &[e.byte(0x44274c)?.wrapping_add(1)])?;
    }
    e.write(0x44274c, &[e.byte(0x44255f)?])?;
    Ok(e.plan)
}
pub fn string_slot(index: i16) -> u32 {
    0x438af4_u32.wrapping_add((i32::from(index) as u32).wrapping_shl(8))
}
