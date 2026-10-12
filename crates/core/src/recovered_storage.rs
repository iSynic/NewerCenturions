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
pub(crate) struct Execution {
    pub plan: StoragePlan,
    pub limits: StorageLimits,
    remaining: usize,
}
impl Execution {
    pub(crate) fn new(
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
    pub(crate) fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(RuleError::Unsupported {
                action: "storage_menu.processing",
                dependency: "the caller's loop budget is exhausted",
            })?;
        Ok(())
    }
    pub(crate) fn bytes(&self, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
        (0..length)
            .map(|i| self.plan.memory.byte(address.wrapping_add(i as u32)))
            .collect()
    }
    pub(crate) fn word(&self, address: u32) -> Result<i16, RuleError> {
        Ok(self.plan.memory.word(address)? as i16)
    }
    pub(crate) fn long(&self, address: u32) -> Result<u32, RuleError> {
        self.plan.memory.long(address)
    }
    pub(crate) fn byte(&self, address: u32) -> Result<u8, RuleError> {
        self.plan.memory.byte(address)
    }
    pub(crate) fn write(&mut self, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.bytes(address, bytes.len())?;
        self.plan.memory.insert_region(address, bytes);
        self.plan.effects.push(StorageEffect::Write(RecordWrite {
            address,
            bytes: bytes.to_vec(),
        }));
        Ok(())
    }
    pub(crate) fn ww(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }
    pub(crate) fn lw(&mut self, address: u32, value: u32) -> Result<(), RuleError> {
        self.write(address, &value.to_be_bytes())
    }
    pub(crate) fn add(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.ww(address, self.word(address)?.wrapping_add(value))
    }
    pub(crate) fn load(
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
    pub(crate) fn service(
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
    pub(crate) fn toolbox(
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
    pub(crate) fn active(&self) -> bool {
        self.plan.flow == LoadFlow::Returned
    }
    pub(crate) fn fatal(
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
    pub(crate) fn filename(
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
    pub(crate) fn open(
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
    pub(crate) fn read(
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
    pub(crate) fn fwrite(
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
    pub(crate) fn close(
        &mut self,
        handle: u32,
        services: &(impl StorageServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.load(LoadRequest::Close(handle), services)?;
        Ok(())
    }
    pub(crate) fn seek(
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
pub(crate) fn string_slot(index: i16) -> u32 {
    0x438af4_u32.wrapping_add((i32::from(index) as u32).wrapping_shl(8))
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use crate::recovered_audio::{AudioContext, AudioObservation};
    use crate::recovered_drawing::{BuildingWorld, DrawingServices, PictureObservation};
    use crate::recovered_equipment::test_support as equipment;
    use crate::recovered_feedback::{
        FeedbackMemory, FeedbackObservation, FeedbackRequest, FeedbackServices,
    };
    use crate::recovered_gamma::{
        GammaError, GammaMemory, GammaObservation, GammaRequest, GammaServices,
    };
    use crate::recovered_gamma_control::{GammaControlRequest, GammaControlServices};
    use crate::recovered_geometry::BodyRect;
    use crate::recovered_load_control::VersionInputs;
    use crate::recovered_text::PascalString;
    use std::cell::RefCell;
    use std::collections::{BTreeMap, VecDeque};
    pub const FRAME: u32 = equipment::FRAME;
    pub const LIMITS: StorageLimits = StorageLimits {
        filename_capacity: 256,
        maximum_iterations: 100000,
        maximum_polls: 4,
    };
    pub struct ReadReply {
        pub value: u32,
        pub bytes: Vec<u8>,
    }
    #[derive(Default)]
    pub struct Services {
        pub(crate) base: equipment::Services,
        pub loads: RefCell<Vec<LoadRequest>>,
        pub requests: RefCell<Vec<StorageRequest>>,
        pub opens: RefCell<VecDeque<u32>>,
        pub reads: RefCell<BTreeMap<u32, VecDeque<ReadReply>>>,
        pub menu: RefCell<Vec<Vec<u8>>>,
        pub open_argument: Option<(u32, u32)>,
        pub toggle_menu_flag: bool,
        pub fail_write: bool,
        pub fail_read: bool,
        pub cache_ready_handle: Option<u32>,
    }
    pub fn word(memory: &mut RenderingMemory, address: u32, value: i16) {
        equipment::put_word(memory, address, value);
    }
    pub fn long(memory: &mut RenderingMemory, address: u32, value: u32) {
        equipment::put_long(memory, address, value);
    }
    pub fn fixture() -> RenderingMemory {
        let mut memory = equipment::fixture();
        long(&mut memory, 0x439570, 0);
        for (address, size) in [
            (0x402600, 512),
            (0x401800, 512),
            (0x403100, 256),
            (0x4042c0, 32),
            (0x440dbc, 288),
            (0x41d1c4, 0x2440),
            (0x478288, 0x1d22),
        ] {
            memory.insert_region(address, &vec![0; size]);
        }
        memory.insert_region(0x44260c, b"Scenario\0");
        for (address, text) in [
            (0x40181c, b"Land\0".as_slice()),
            (0x401818, b"r+b\0"),
            (0x401796, b"ED\0"),
            (0x40178e, b"ED2\0"),
            (0x401792, b"r+b\0"),
            (0x4031b2, b"SD\0"),
            (0x4031ae, b"r+b\0"),
            (0x401e1a, b"NI\0"),
            (0x402700, b"MENU\0"),
            (0x4026f4, b"MD\0"),
            (0x4026fc, b"rb\0"),
            (0x4026d0, b"wb\0"),
            (0x4026d4, b"\0"),
            (0x4026d6, b"Loading\0"),
            (0x4026c9, b"\0"),
            (0x4026ca, b"DATA\0"),
            (0x402436, b"NEWC\0"),
            (0x4028c0, b"Mission\0"),
            (0x4028bc, b"rb\0"),
        ] {
            memory.insert_region(address, text);
        }
        memory
    }
    fn cstring(memory: &RenderingMemory, address: u32) -> Result<Vec<u8>, RuleError> {
        let mut bytes = vec![];
        for index in 0..256 {
            let value = memory.byte(address.wrapping_add(index))?;
            if value == 0 {
                return Ok(bytes);
            }
            bytes.push(value);
        }
        Err(RuleError::Invalid("test C string unavailable".into()))
    }
    impl FeedbackServices for Services {
        fn observe(
            &self,
            request: &FeedbackRequest,
            memory: &FeedbackMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            self.base.observe(request, memory)
        }
        fn sound_observations(
            &self,
            id: i16,
            context: &AudioContext,
            memory: &FeedbackMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            self.base.sound_observations(id, context, memory)
        }
    }
    impl GammaServices for Services {
        fn observe(
            &self,
            _: &GammaRequest,
            _: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            Err(GammaError::MissingService("unexpected gamma".into()))
        }
    }
    impl GammaControlServices for Services {
        fn observe_control(
            &self,
            _: &GammaControlRequest,
            _: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            Err(GammaError::MissingService(
                "unexpected gamma control".into(),
            ))
        }
    }
    impl LoadServices for Services {
        fn observe_load(
            &self,
            request: &LoadRequest,
            memory: &RenderingMemory,
        ) -> Result<LoadObservation, RuleError> {
            self.loads.borrow_mut().push(request.clone());
            let mut next = memory.clone();
            let mut value = 0;
            match request {
                LoadRequest::Open { .. } => {
                    value = self.opens.borrow_mut().pop_front().unwrap_or(9);
                    if let Some((address, argument)) = self.open_argument {
                        long(&mut next, address, argument);
                    }
                }
                LoadRequest::Read {
                    handle,
                    destination,
                    ..
                } => {
                    if self.cache_ready_handle == Some(*handle) {
                        word(&mut next, 0x4042c8, memory.word(FRAME + 8)? as i16);
                    }
                    if self.fail_read {
                        return Err(RuleError::Invalid("missing read observation".into()));
                    }
                    if let Some(reply) = self
                        .reads
                        .borrow_mut()
                        .get_mut(handle)
                        .and_then(VecDeque::pop_front)
                    {
                        value = reply.value;
                        next.insert_region(*destination, &reply.bytes);
                    }
                }
                LoadRequest::GetIndexedString { destination, .. } => {
                    next.insert_region(*destination, &[1, b'L']);
                }
                LoadRequest::CopyCString {
                    source,
                    destination,
                } => {
                    let mut bytes = cstring(memory, *source)?;
                    bytes.push(0);
                    next.insert_region(*destination, &bytes);
                }
                LoadRequest::CToPascal(address) => {
                    next.insert_region(
                        *address,
                        PascalString::from_payload(&cstring(memory, *address)?)?.raw(),
                    );
                }
                LoadRequest::PascalToC(address) => {
                    let length = usize::from(memory.byte(*address)?);
                    let mut bytes = (0..length)
                        .map(|i| memory.byte(address.wrapping_add(i as u32 + 1)))
                        .collect::<Result<Vec<_>, _>>()?;
                    bytes.push(0);
                    next.insert_region(*address, &bytes);
                }
                LoadRequest::Toolbox {
                    opcode, arguments, ..
                } => match *opcode {
                    0xa950 => value = self.menu.borrow().len() as u32,
                    0xa947 => {
                        if let [L(_), W(index), P(address)] = arguments.as_slice() {
                            let length = usize::from(memory.byte(*address)?);
                            let text = (0..length)
                                .map(|i| memory.byte(address.wrapping_add(i as u32 + 1)))
                                .collect::<Result<Vec<_>, _>>()?;
                            let mut menu = self.menu.borrow_mut();
                            let length = menu.len().max(usize::from(*index));
                            menu.resize(length, vec![]);
                            menu[usize::from(*index) - 1] = text;
                        }
                        if self.toggle_menu_flag {
                            next.insert_region(0x442722, &[1]);
                        }
                    }
                    0xa933 => {
                        if let [L(_), P(address)] = arguments.as_slice() {
                            let length = usize::from(memory.byte(*address)?);
                            self.menu.borrow_mut().push(
                                (0..length)
                                    .map(|i| memory.byte(address.wrapping_add(i as u32 + 1)))
                                    .collect::<Result<Vec<_>, _>>()?,
                            );
                        }
                    }
                    0xa946 => {
                        if let [L(_), W(index), P(address)] = arguments.as_slice() {
                            let menu = self.menu.borrow();
                            let bytes = menu
                                .get(usize::from(*index) - 1)
                                .ok_or_else(|| RuleError::Invalid("test menu index".into()))?;
                            next.insert_region(*address, PascalString::from_payload(bytes)?.raw());
                        }
                    }
                    0xa874 => {
                        if let [P(address)] = arguments.as_slice() {
                            long(&mut next, *address, 123);
                        }
                    }
                    0xaa1b => value = 55,
                    0xa97c => value = 77,
                    0xa991 => word(&mut next, 0x4434b2, 4),
                    0xa98d => {
                        if let [L(_), W(_), P(kind), P(handle), P(rect)] = arguments.as_slice() {
                            word(&mut next, *kind, 4);
                            long(&mut next, *handle, 88);
                            next.insert_region(*rect, &[0; 8]);
                        }
                    }
                    0xa990 => {
                        if let [L(_), P(address)] = arguments.as_slice() {
                            next.insert_region(*address, &[4, b'W', b'o', b'r', b'd']);
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
            Ok(LoadObservation {
                memory: next,
                value,
                d0: 0x12345678,
                flow: LoadFlow::Returned,
            })
        }
        fn version_inputs(
            &self,
            _: i16,
            _: i16,
            _: &RenderingMemory,
        ) -> Result<VersionInputs, RuleError> {
            Err(RuleError::Invalid("unexpected version".into()))
        }
        fn filename_number(
            &self,
            variant: u8,
            _: &RenderingMemory,
        ) -> Result<PascalString, RuleError> {
            PascalString::from_payload(variant.to_string().as_bytes())
        }
    }
    use LoadArg::{Long as L, Pointer as P, Word as W};
    impl StorageServices for Services {
        fn observe_storage(
            &self,
            request: &StorageRequest,
            memory: &RenderingMemory,
        ) -> Result<StorageObservation, RuleError> {
            self.requests.borrow_mut().push(request.clone());
            let value = match request {
                StorageRequest::Write { .. } if self.fail_write => {
                    return Err(RuleError::Invalid("missing write observation".into()));
                }
                StorageRequest::Write { .. } => 1,
                StorageRequest::StringLength(address) => cstring(memory, *address)?.len() as u32,
                StorageRequest::CompareCString { left, right } => {
                    match cstring(memory, *left)?.cmp(&cstring(memory, *right)?) {
                        std::cmp::Ordering::Less => u32::MAX,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Greater => 1,
                    }
                }
                _ => 0,
            };
            Ok(StorageObservation {
                memory: memory.clone(),
                value,
                d0: value,
            })
        }
    }
    impl DrawingServices for Services {
        fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
            self.base.color_icon(id)
        }
        fn picture(&mut self, id: i16) -> Result<PictureObservation, RuleError> {
            self.base.picture(id)
        }
        fn indexed_string(&mut self, resource: i16, index: i16) -> Result<PascalString, RuleError> {
            self.base.indexed_string(resource, index)
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            self.base.current_world()
        }
        fn new_world(
            &mut self,
            depth: i16,
            rect: BodyRect,
        ) -> Result<(i16, BuildingWorld), RuleError> {
            self.base.new_world(depth, rect)
        }
    }
    impl crate::recovered_scenario_menus::ScenarioMenuServices for Services {}
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    #[test]
    fn save_family_keeps_long_reread_word_indices_and_ordered_spans() {
        let mut memory = fixture();
        for (source, size) in LAND {
            memory.insert_region(source, &vec![0; size as usize]);
        }
        long(&mut memory, FRAME + 8, 0);
        let services = Services {
            open_argument: Some((FRAME + 8, 0xfffffff9)),
            ..Services::default()
        };
        let land = save_land(&memory, FRAME, LIMITS, &services).unwrap();
        assert_eq!(land.native_return, Some(0));
        assert!(services.loads.borrow().contains(&LoadRequest::Seek {
            handle: 9,
            offset: 0xfffffff9_u32.wrapping_mul(0x12fb0),
            origin: 0
        }));
        let writes = services
            .requests
            .borrow()
            .iter()
            .filter_map(|request| {
                if let StorageRequest::Write { source, size, .. } = request {
                    Some((*source, *size))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(writes, LAND);
        word(&mut memory, FRAME + 8, -1);
        word(&mut memory, FRAME + 10, 1);
        let services = Services::default();
        save_encounter(&memory, FRAME, LIMITS, &services).unwrap();
        assert!(services.loads.borrow().contains(&LoadRequest::Seek {
            handle: 9,
            offset: 65535 * 0x1aa,
            origin: 0
        }));
        word(&mut memory, FRAME + 10, 2);
        let services = Services::default();
        save_encounter(&memory, FRAME, LIMITS, &services).unwrap();
        assert!(services.loads.borrow().contains(&LoadRequest::Seek {
            handle: 9,
            offset: 65535 * 0x204,
            origin: 0
        }));
        memory.insert_region(0x44273b, &[255]);
        let services = Services::default();
        save_shop(&memory, LIMITS, &services).unwrap();
        assert!(services.loads.borrow().contains(&LoadRequest::Seek {
            handle: 9,
            offset: 65535 * 0xbba,
            origin: 0
        }));
        let before = memory.clone();
        let services = Services {
            fail_write: true,
            ..Services::default()
        };
        assert!(save_shop(&memory, LIMITS, &services).is_err());
        assert_eq!(memory, before);
    }
    #[test]
    fn reset_preserves_three_variants_clear_order_and_original_variant() {
        let mut memory = fixture();
        memory.insert_region(0x44274c, &[7]);
        memory.insert_region(0x4358e6, &vec![9; 8100]);
        memory.insert_region(0x44258c, &[9; 100]);
        let services = Services::default();
        services.opens.borrow_mut().extend([0; 18]);
        let result = reset_game_files(&memory, FRAME, LIMITS, &services).unwrap();
        assert_eq!(result.memory.byte(0x44274c).unwrap(), 7);
        assert_eq!(result.memory.word(0x44255e).unwrap(), 7);
        assert_eq!(result.memory.word(0x4434ac).unwrap(), 90);
        assert_eq!(result.memory.byte(0x4358e6 + 8099).unwrap(), 0);
        assert_eq!(
            services
                .loads
                .borrow()
                .iter()
                .filter(|request| matches!(request, LoadRequest::Open { .. }))
                .count(),
            18
        );
        assert!(services.requests.borrow().is_empty());
        assert!(
            reset_game_files(
                &memory,
                FRAME,
                StorageLimits {
                    maximum_iterations: 1,
                    ..LIMITS
                },
                &Services::default()
            )
            .is_err()
        );
        assert_eq!(memory.byte(0x44274c).unwrap(), 7);
    }
}
