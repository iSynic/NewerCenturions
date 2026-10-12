//! Application startup and registration bookkeeping with observed OS services.

use crate::{
    RandomState, RuleError, recovered_fade_ui, recovered_feedback as feedback,
    recovered_feedback::FeedbackEffect,
    recovered_gamma::GammaMemory,
    recovered_gamma_control::GammaControlEffect,
    recovered_item_ui::ItemUiServices,
    recovered_load_control::{LoadFlow, LoadRequest},
    recovered_preferences::{
        self as preferences, HandleMemory, PreferenceContext, PreferenceEffect,
        PreferenceObservation,
    },
    recovered_rendering::RenderingMemory,
    recovered_runtime::{self as runtime, RuntimeContext, RuntimeEffect, RuntimeResult},
    recovered_storage::{
        Execution as StorageExecution, StorageEffect, StorageLimits, StorageServices,
    },
    recovered_timing::TimingServices,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartupArg {
    Byte(u8),
    Word(u16),
    Long(u32),
    Pointer(u32),
}
pub(crate) use StartupArg::{Byte as B, Long as L, Pointer as P, Word as W};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartupRequest {
    Toolbox {
        opcode: u16,
        selector: Option<u32>,
        arguments: Vec<StartupArg>,
        d0: Option<u32>,
        a0: Option<u32>,
    },
    Uppercase(i16),
    PascalNameToC(u32),
    CNameToPascal(u32),
    Rename {
        source: u32,
        destination: u32,
    },
    Unlink(u32),
    StringToNumber {
        source: u32,
        destination: u32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartupObservation {
    pub memory: RenderingMemory,
    /// Pascal result slot, separate from registers after an OS trap.
    pub value: u32,
    pub d0: u32,
    pub a0: u32,
    pub zero: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferenceInputs {
    pub handles: BTreeMap<u32, HandleMemory>,
    pub observations: Vec<PreferenceObservation>,
}
pub trait StartupServices: StorageServices + ItemUiServices + TimingServices {
    /// Return observations only; applying effects is the adapter's later boundary.
    fn observe_startup(
        &self,
        request: &StartupRequest,
        memory: &RenderingMemory,
    ) -> Result<StartupObservation, RuleError>;
    fn preference_inputs(&self, memory: &RenderingMemory) -> Result<PreferenceInputs, RuleError>;
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartupEffect {
    Storage(StorageEffect),
    Service {
        request: StartupRequest,
        observation: StartupObservation,
    },
    Feedback(FeedbackEffect),
    Fade(GammaControlEffect),
    Preference(PreferenceEffect),
    Random(RuntimeEffect),
    ItemUi(crate::recovered_item_ui::ItemUiEffect),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartupPlan {
    pub memory: RenderingMemory,
    pub random: RandomState,
    pub effects: Vec<StartupEffect>,
    pub flow: LoadFlow,
    pub native_return: Option<u32>,
}
pub(crate) struct Execution {
    pub e: StorageExecution,
    pub random: RandomState,
    pub effects: Vec<StartupEffect>,
    pub d0: u32,
}
impl Execution {
    pub(crate) fn new(
        memory: &RenderingMemory,
        random: &RandomState,
        limits: StorageLimits,
        ret: Option<u32>,
    ) -> Self {
        Self {
            e: StorageExecution::new(memory, limits, ret),
            random: random.clone(),
            effects: vec![],
            d0: 0,
        }
    }
    pub(crate) fn finish(mut self) -> StartupPlan {
        self.flush();
        StartupPlan {
            memory: self.e.plan.memory,
            random: self.random,
            effects: self.effects,
            flow: self.e.plan.flow,
            native_return: self.e.plan.native_return,
        }
    }
    pub(crate) fn flush(&mut self) {
        self.effects
            .extend(self.e.plan.effects.drain(..).map(StartupEffect::Storage));
    }
    pub(crate) fn request(
        &mut self,
        request: StartupRequest,
        s: &impl StartupServices,
    ) -> Result<StartupObservation, RuleError> {
        self.flush();
        let o = s.observe_startup(&request, &self.e.plan.memory)?;
        self.d0 = o.d0;
        self.e.plan.memory = o.memory.clone();
        self.effects.push(StartupEffect::Service {
            request,
            observation: o.clone(),
        });
        Ok(o)
    }
    pub(crate) fn tb(
        &mut self,
        opcode: u16,
        selector: Option<u32>,
        arguments: Vec<StartupArg>,
        s: &impl StartupServices,
    ) -> Result<u32, RuleError> {
        Ok(self
            .request(
                StartupRequest::Toolbox {
                    opcode,
                    selector,
                    arguments,
                    d0: None,
                    a0: None,
                },
                s,
            )?
            .value)
    }
    pub(crate) fn ww(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.e.ww(address, value)
    }
    pub(crate) fn lw(&mut self, address: u32, value: u32) -> Result<(), RuleError> {
        self.e.lw(address, value)
    }
    pub(crate) fn get_item(
        &mut self,
        dialog: u32,
        item: i16,
        handle: u32,
        s: &impl StartupServices,
    ) -> Result<(), RuleError> {
        self.tb(
            0xa98d,
            None,
            vec![
                L(dialog),
                W(item as u16),
                P(0x4434b0),
                P(handle),
                P(0x4434ba),
            ],
            s,
        )?;
        Ok(())
    }
    pub(crate) fn di_str(
        &mut self,
        item: i16,
        text: u32,
        s: &impl StartupServices,
    ) -> Result<(), RuleError> {
        let dialog = self.e.long(0x4394b8)?;
        self.get_item(dialog, item, 0x4434dc, s)?;
        self.tb(0xa98f, None, vec![L(self.e.long(0x4434dc)?), P(text)], s)?;
        self.d0 = 0;
        Ok(())
    }
    pub(crate) fn number(
        &mut self,
        item: i16,
        value: i32,
        s: &impl StartupServices,
    ) -> Result<(), RuleError> {
        let dialog = self.e.long(0x4394b8)?;
        self.get_item(dialog, item, 0x4434dc, s)?;
        self.e.load(
            LoadRequest::NumberToString {
                value,
                destination: 0x479ed8,
            },
            s,
        )?;
        self.tb(
            0xa98f,
            None,
            vec![L(self.e.long(0x4434dc)?), P(0x479ed8)],
            s,
        )?;
        self.d0 = 0;
        Ok(())
    }
    pub(crate) fn warn(
        &mut self,
        frame: u32,
        id: i16,
        s: &impl StartupServices,
    ) -> Result<(), RuleError> {
        self.ww(frame.wrapping_add(8), id)?;
        self.flush();
        let p = feedback::warn(
            &self.e.plan.memory,
            frame,
            id,
            self.e.limits.maximum_polls,
            s,
        )?;
        self.e.plan.memory = p.memory;
        self.effects
            .extend(p.effects.into_iter().map(StartupEffect::Feedback));
        self.d0 = p.native_return.unwrap_or(0);
        Ok(())
    }
    fn save_pref(&mut self, s: &impl StartupServices) -> Result<(), RuleError> {
        let input = s.preference_inputs(&self.e.plan.memory)?;
        let mut c = PreferenceContext::new(self.random.clone());
        c.handles = input.handles;
        for (a, n) in [
            (0x439568, 2),
            (0x43956a, 2),
            (0x43956c, 4),
            (0x439570, 4),
            (0x439574, 2),
            (0x439578, 2),
            (0x43957a, 2),
            (0x43957c, 2),
            (0x43957e, 2),
            (0x439580, 2),
            (0x439582, 2),
            (0x439584, 2),
            (0x439586, 2),
            (0x439588, 2),
            (0x43958a, 2),
            (0x43958c, 2),
            (0x43958e, 2),
            (0x439590, 2),
            (0x439592, 2),
            (0x438589, 1),
            (0x402f44, 2),
            (0x442560, 2),
            (0x438598, 4),
            (0x439596, 2),
            (0x4046ea, 2),
        ] {
            c.globals.insert(a, self.e.bytes(a, n)?);
        }
        for i in 0..20 {
            c.globals
                .insert(0x44270e + i, vec![self.e.byte(0x44270e + i)?]);
        }
        let n = usize::from(self.e.byte(0x4385a0)?) + 1;
        c.globals.insert(0x4385a0, self.e.bytes(0x4385a0, n)?);
        let p = preferences::save_preferences(&mut c, &input.observations)?;
        self.flush();
        for f in p.effects {
            if let PreferenceEffect::GlobalWrite { address, bytes } = &f {
                self.e.write(*address, bytes)?;
                self.flush();
            } else {
                self.effects.push(StartupEffect::Preference(f));
            }
        }
        self.random = c.random;
        self.d0 = 0;
        Ok(())
    }
    fn app_name(&mut self, frame: u32, s: &impl StartupServices) -> Result<(), RuleError> {
        self.tb(
            0xa9f5,
            None,
            vec![
                P(0x479ed8),
                P(frame.wrapping_sub(2)),
                P(frame.wrapping_sub(6)),
            ],
            s,
        )?;
        self.e.load(LoadRequest::PascalToC(0x479ed8), s)?;
        self.d0 = 0;
        Ok(())
    }
    fn fade(
        &mut self,
        frame: u32,
        direction: i16,
        s: &impl StartupServices,
    ) -> Result<(), RuleError> {
        self.lw(
            frame.wrapping_add(8),
            (25_u32 << 16) | u32::from(direction as u16),
        )?;
        self.flush();
        let p = recovered_fade_ui::fadeinout(
            &GammaMemory {
                bytes: self.e.plan.memory.bytes.clone(),
            },
            frame,
            25,
            direction,
            s,
        )
        .map_err(|x| RuleError::Invalid(x.to_string()))?;
        self.e.plan.memory.bytes = p.memory.bytes;
        self.effects
            .extend(p.effects.into_iter().map(StartupEffect::Fade));
        self.d0 = 0;
        Ok(())
    }
}

/// GWorldInit, 0x127462. The final overwritten pixmap slot is preserved.
pub fn initialize_worlds(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: StorageLimits,
    s: &impl StartupServices,
) -> Result<StartupPlan, RuleError> {
    let mut x = Execution::new(memory, random, limits, Some(0));
    x.tb(0xab1d, Some(0x80005), vec![P(0x4046da), P(0x4046de)], s)?;
    for (index, (world, pixmap, width, height, picture)) in [
        (0x4386b8, 0x4386dc, 480, 480, None),
        (0x4386b4, 0x4386d8, 480, 480, None),
        (0x4386bc, 0x4386e0, 480, 480, None),
        (0x4386c0, 0x4386e4, 480, 480, None),
        (0x4386cc, 0x4386f0, 206, 129, None),
        (0x4386c8, 0x4386ec, 960, 192, Some(192)),
        (0x4386d0, 0x4386f4, 50, 50, Some(401)),
        (0x4386d4, 0x4386f4, 500, 500, Some(196)),
        (0x4386c4, 0x4386e8, 640, 640, Some(302)),
    ]
    .into_iter()
    .enumerate()
    {
        let mut held_picture = None;
        if let Some(id) = picture {
            let h = x.tb(0xa9bc, None, vec![W(id)], s)?;
            if index == 8 {
                held_picture = Some(h);
            } else {
                let a = x.e.long(h)?;
                x.tb(0xa8f6, None, vec![L(h), P(a.wrapping_add(2))], s)?;
            }
        }
        if matches!(index, 0 | 4 | 5 | 6 | 7 | 8) {
            x.tb(
                0xa8a7,
                None,
                vec![P(0x4434ba), W(0), W(0), W(width), W(height)],
                s,
            )?;
        }
        let depth = if index == 0 {
            1
        } else {
            i16::from(x.e.byte(0x43873f)? as i8)
        };
        let err = x.tb(
            0xab1d,
            Some(0x160000),
            vec![P(world), W(depth as u16), P(0x4434ba), L(0), L(0), L(0)],
            s,
        )? as u16;
        if err != 0 {
            x.e.fatal(frame.wrapping_sub(14), 2, true, s)?;
            if !x.e.active() {
                return Ok(x.finish());
            }
        }
        x.tb(0xab1d, Some(0x80006), vec![L(x.e.long(world)?), L(0)], s)?;
        if index == 1 {
            x.tb(0xa89b, None, vec![W(3), W(3)], s)?;
        }
        x.tb(0xa862, None, vec![L(33)], s)?;
        x.tb(0xa863, None, vec![L(30)], s)?;
        let p = x.tb(0xab1d, Some(0x40017), vec![L(x.e.long(world)?)], s)?;
        x.lw(pixmap, p)?;
        let lock = if index == 7 { 0x4386f8 } else { pixmap };
        x.tb(0xab1d, Some(0x40001), vec![L(x.e.long(lock)?)], s)?;
        if let Some(h) = held_picture {
            let a = x.e.long(h)?;
            x.tb(0xa8f6, None, vec![L(h), P(a.wrapping_add(2))], s)?;
        }
    }
    x.tb(
        0xab1d,
        Some(0x80006),
        vec![L(x.e.long(0x4046da)?), L(x.e.long(0x4046de)?)],
        s,
    )?;
    Ok(x.finish())
}
/// setfree, 0x15311a. A matching nonzero table entry consumes one Random draw.
pub fn set_free(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: StorageLimits,
    s: &impl StartupServices,
) -> Result<StartupPlan, RuleError> {
    let mut x = Execution::new(memory, random, limits, Some(0));
    let key = x.e.long(frame.wrapping_add(8))?;
    for i in 0..100_u32 {
        let v = x.e.long(0x4015fe + i * 4)?;
        x.lw(frame.wrapping_sub(400).wrapping_add(i * 4), v)?;
    }
    let mut found = false;
    for i in 0..100_u32 {
        x.e.step()?;
        if x.e.long(frame.wrapping_sub(400).wrapping_add(i * 4))? == key
            && key != 0
            && x.e.long(frame.wrapping_sub(400).wrapping_add(i * 4))? != 0
        {
            found = true;
        }
    }
    if found {
        x.tb(0xa85e, None, vec![P(0x438598), L(6)], s)?;
        let mut c = RuntimeContext::new(x.random.clone(), 1);
        let p = runtime::rand(&mut c, 32000)?;
        let RuntimeResult::Word { value, .. } = p.result else {
            unreachable!()
        };
        x.random = c.random;
        x.flush();
        x.effects
            .extend(p.effects.into_iter().map(StartupEffect::Random));
        let v =
            x.e.long(0x438598)?
                .wrapping_add(i32::from(value.wrapping_add(32000)) as u32);
        x.lw(0x438598, v)?;
        x.lw(0x43859c, x.e.long(0x438598)?)?;
        x.e.write(0x4385a0, &[0])?;
        x.save_pref(s)?;
        x.request(StartupRequest::Unlink(0x4015e4), s)?;
    }
    Ok(x.finish())
}
/// setfree2, 0x1531c0. Every comparison is reached even after a match.
pub fn set_free_name(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: StorageLimits,
    s: &impl StartupServices,
) -> Result<StartupPlan, RuleError> {
    let mut x = Execution::new(memory, random, limits, Some(0));
    let force = x.e.word(frame.wrapping_add(8))?;
    x.request(StartupRequest::PascalNameToC(0x438aa4), s)?;
    for i in 0..40_u32 {
        x.e.step()?;
        let b = x.e.byte(0x438aa4 + i)? as i8;
        let v = x.request(StartupRequest::Uppercase(i16::from(b)), s)?.value;
        x.e.write(0x438aa4 + i, &[v as u8])?;
    }
    let mut matched = false;
    for right in [
        0x4015ce, 0x4015b6, 0x4015ac, 0x401598, 0x401592, 0x40158a, 0x401580, 0x401572, 0x401566,
        0x40155c, 0x40154e, 0x401544, 0x40153c, 0x401532, 0x40152a, 0x401522, 0x40151a, 0x401512,
        0x401544, 0x401508, 0x4014fc, 0x4014f2, 0x4014e8, 0x4014e2, 0x4014d4, 0x4014cc, 0x4014bc,
        0x4014ae,
    ] {
        if x.e.service(
            crate::recovered_storage::StorageRequest::CompareCString {
                left: 0x438aa4,
                right,
            },
            s,
        )? as u16
            == 0
        {
            matched = true;
        }
    }
    if matched || force == 1 {
        x.tb(0xa85e, None, vec![P(0x438598), L(6)], s)?;
        x.ww(0x4046ea, 0)?;
        x.lw(0x43859c, x.e.long(0x438598)?)?;
        x.e.write(0x4385a0, &[0])?;
        x.e.write(0x442604, &[0])?;
        x.save_pref(s)?;
        x.request(
            StartupRequest::Rename {
                source: 0x401498,
                destination: 0x401480,
            },
            s,
        )?;
        x.request(
            StartupRequest::Rename {
                source: 0x40146a,
                destination: 0x401498,
            },
            s,
        )?;
        x.e.fatal(
            frame.wrapping_sub(22),
            if force != 0 { 451 } else { 453 },
            false,
            s,
        )?;
        if !x.e.active() {
            return Ok(x.finish());
        }
    }
    x.request(StartupRequest::CNameToPascal(0x438aa4), s)?;
    Ok(x.finish())
}
/// checkformismatch, 0x14c318. Short reads retain the previous seed tail.
pub fn check_for_mismatch(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: StorageLimits,
    s: &impl StartupServices,
) -> Result<StartupPlan, RuleError> {
    let mut x = Execution::new(memory, random, limits, Some(0));
    x.app_name(frame.wrapping_sub(12), s)?;
    let h = x.e.load(
        LoadRequest::Open {
            name: 0x479ed8,
            mode: 0x4019b0,
        },
        s,
    )?;
    x.lw(0x4434a6, h)?;
    if h != 0 {
        x.e.read(x.e.long(0x4434a6)?, 4, 0x479fdc, s)?;
        x.e.close(x.e.long(0x4434a6)?, s)?;
    } else {
        x.e.fatal(frame.wrapping_sub(14), 7, true, s)?;
        if !x.e.active() {
            return Ok(x.finish());
        }
    }
    let equal = x.e.long(0x43859c)? == x.e.long(0x479fdc)?;
    if !equal && x.e.long(0x479fdc)? as i32 > 0 {
        x.lw(0x438598, x.e.long(0x479fdc)?)?;
        x.tb(0xa85e, None, vec![P(0x438598), L(6)], s)?;
        x.e.write(0x4385a0, &[0])?;
        x.lw(0x479fdc, x.e.long(0x438598)?)?;
        x.lw(0x43859c, x.e.long(0x438598)?)?;
    }
    x.lw(0x479fdc, x.e.long(0x43859c)?)?;
    x.app_name(frame.wrapping_sub(12), s)?;
    let h = x.e.load(
        LoadRequest::Open {
            name: 0x479ed8,
            mode: 0x4019b4,
        },
        s,
    )?;
    x.lw(0x4434a6, h)?;
    if h != 0 {
        x.e.fwrite(x.e.long(0x4434a6)?, 4, 0x479fdc, s)?;
        x.e.close(x.e.long(0x4434a6)?, s)?;
    }
    if !equal {
        x.save_pref(s)?;
    }
    Ok(x.finish())
}

/// showlogo, 0x157d38. A negative duration waits for event kinds 1 or 3.
pub fn show_logo(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: StorageLimits,
    s: &mut impl StartupServices,
) -> Result<StartupPlan, RuleError> {
    let mut x = Execution::new(memory, random, limits, Some(0));
    let duration = x.e.word(frame.wrapping_add(8))?;
    x.tb(0xa852, None, vec![], s)?;
    if x.e.byte(0x438587)? == 0 {
        x.fade(frame.wrapping_sub(20), 1, s)?;
    }
    let hide = frame.wrapping_sub(16);
    x.ww(0x4042c2, x.e.word(0xbaa)?)?;
    x.ww(0xbaa, 0)?;
    let r = x.tb(0xa8d8, None, vec![], s)?;
    x.lw(0x4042c4, r)?;
    x.tb(
        0xa8dc,
        None,
        vec![L(x.e.long(0x9ee)?), L(x.e.long(0x4042c4)?)],
        s,
    )?;
    let region = x.tb(0xa8d8, None, vec![], s)?;
    let dev = x.tb(0xaa2a, None, vec![], s)?;
    x.lw(hide.wrapping_sub(12), dev)?;
    let data = x.e.long(x.e.long(hide.wrapping_sub(12))?)?;
    x.lw(hide.wrapping_sub(8), x.e.long(data.wrapping_add(34))?)?;
    x.lw(hide.wrapping_sub(4), x.e.long(data.wrapping_add(38))?)?;
    x.tb(0xa8df, None, vec![L(region), P(hide.wrapping_sub(8))], s)?;
    x.tb(
        0xa8e5,
        None,
        vec![L(x.e.long(0x4042c4)?), L(region), L(x.e.long(0x9ee)?)],
        s,
    )?;
    x.tb(0xa90c, None, vec![L(0), L(x.e.long(0x4042c4)?)], s)?;
    let window = x.tb(0xaa46, None, vec![W(132), L(0), L(u32::MAX)], s)?;
    x.tb(
        0xa91d,
        None,
        vec![
            L(window),
            W(x.e.word(0x43959e)? as u16),
            W(x.e.word(0x4395a0)? as u16),
            B(0),
        ],
        s,
    )?;
    let dialog = x.tb(0xa97c, None, vec![W(175), L(0), L(u32::MAX)], s)?;
    x.lw(0x41f604, dialog)?;
    let half = |w: i16| w.wrapping_add(((w as u16) >> 15) as i16) >> 1;
    let h =
        x.e.word(0x4395a4)?
            .wrapping_add(1)
            .wrapping_add(half(x.e.word(0x40856c)?));
    let v =
        x.e.word(0x4395a2)?
            .wrapping_add(1)
            .wrapping_add(half(x.e.word(0x40856e)?));
    x.tb(
        0xa91b,
        None,
        vec![L(x.e.long(0x41f604)?), W(h as u16), W(v as u16), B(0)],
        s,
    )?;
    x.tb(
        0xa91d,
        None,
        vec![L(x.e.long(0x41f604)?), W(640), W(460), B(0)],
        s,
    )?;
    x.tb(0xa981, None, vec![L(x.e.long(0x41f604)?)], s)?;
    if x.e.byte(0x438587)? == 0 {
        x.fade(frame.wrapping_sub(20), 0, s)?;
    }
    if duration < 0 {
        x.tb(0xa853, None, vec![], s)?;
        x.request(
            StartupRequest::Toolbox {
                opcode: 0xa032,
                selector: None,
                arguments: vec![],
                d0: Some(0xffff),
                a0: None,
            },
            s,
        )?;
        loop {
            x.e.step()?;
            x.tb(0xa860, None, vec![W(u16::MAX), P(0x44720a), L(0), L(0)], s)?;
            if matches!(x.e.word(0x44720a)?, 1 | 3) {
                break;
            }
        }
        x.request(
            StartupRequest::Toolbox {
                opcode: 0xa032,
                selector: None,
                arguments: vec![],
                d0: Some(0xffff),
                a0: None,
            },
            s,
        )?;
    } else if duration != 0 {
        let child = frame.wrapping_sub(18);
        x.ww(child.wrapping_add(8), duration)?;
        let start = x.tb(0xa975, None, vec![], s)?;
        loop {
            x.e.step()?;
            if x.tb(0xa974, None, vec![], s)? as u8 != 0 {
                break;
            }
            x.lw(child.wrapping_sub(4), i32::from(duration) as u32)?;
            let elapsed = x.tb(0xa975, None, vec![], s)?.wrapping_sub(start);
            if elapsed > x.e.long(child.wrapping_sub(4))? {
                break;
            }
        }
    }
    if x.e.byte(0x438587)? == 0 {
        x.fade(frame.wrapping_sub(20), 1, s)?;
    }
    x.ww(0xbaa, x.e.word(0x4042c2)?)?;
    x.tb(
        0xa8e4,
        None,
        vec![
            L(x.e.long(0x4042c4)?),
            L(x.e.long(0x9ee)?),
            L(x.e.long(0x9ee)?),
        ],
        s,
    )?;
    x.tb(0xa937, None, vec![], s)?;
    x.tb(0xa937, None, vec![], s)?;
    x.tb(0xa914, None, vec![L(window)], s)?;
    x.tb(0xa983, None, vec![L(x.e.long(0x41f604)?)], s)?;
    x.tb(0xa853, None, vec![], s)?;
    Ok(x.finish())
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use crate::recovered_audio::{AudioContext, AudioObservation};
    use crate::recovered_drawing::{BuildingWorld, DrawingServices, PictureObservation};
    use crate::recovered_feedback::{
        FeedbackMemory, FeedbackObservation, FeedbackRequest, FeedbackServices,
    };
    use crate::recovered_gamma::{GammaError, GammaObservation, GammaRequest, GammaServices};
    use crate::recovered_gamma_control::{GammaControlRequest, GammaControlServices};
    use crate::recovered_geometry::BodyRect;
    use crate::recovered_item_ui::{ItemUiReply, ItemUiRequest};
    use crate::recovered_load_control::{LoadObservation, LoadServices, VersionInputs};
    use crate::recovered_preferences::{PreferenceReply as V, PreferenceRequest as R};
    use crate::recovered_storage::test_support as storage;
    use crate::recovered_storage::{StorageObservation, StorageRequest};
    use crate::recovered_text::{NumericServices, PascalString};
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    pub use storage::{FRAME, LIMITS, long, word};
    #[derive(Default)]
    pub struct Services {
        pub base: storage::Services,
        pub requests: RefCell<Vec<StartupRequest>>,
        pub events: RefCell<VecDeque<i16>>,
        pub controls: RefCell<BTreeMap<i16, i16>>,
        pub print_accept: bool,
        pub fail: bool,
        pub(crate) ticks: Cell<u32>,
    }
    impl FeedbackServices for Services {
        fn observe(
            &self,
            r: &FeedbackRequest,
            m: &FeedbackMemory,
        ) -> Result<FeedbackObservation, RuleError> {
            FeedbackServices::observe(&self.base, r, m)
        }
        fn sound_observations(
            &self,
            id: i16,
            c: &AudioContext,
            m: &FeedbackMemory,
        ) -> Result<Vec<AudioObservation>, RuleError> {
            self.base.sound_observations(id, c, m)
        }
    }
    impl GammaServices for Services {
        fn observe(
            &self,
            r: &GammaRequest,
            m: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            GammaServices::observe(&self.base, r, m)
        }
    }
    impl GammaControlServices for Services {
        fn observe_control(
            &self,
            r: &GammaControlRequest,
            m: &GammaMemory,
        ) -> Result<GammaObservation, GammaError> {
            self.base.observe_control(r, m)
        }
    }
    impl LoadServices for Services {
        fn observe_load(
            &self,
            r: &LoadRequest,
            m: &RenderingMemory,
        ) -> Result<LoadObservation, RuleError> {
            let mut o = self.base.observe_load(r, m)?;
            if let LoadRequest::NumberToString { value, destination } = r {
                o.memory.insert_region(
                    *destination,
                    PascalString::from_payload(value.to_string().as_bytes())?.raw(),
                );
            }
            Ok(o)
        }
        fn version_inputs(
            &self,
            id: i16,
            f: i16,
            m: &RenderingMemory,
        ) -> Result<VersionInputs, RuleError> {
            self.base.version_inputs(id, f, m)
        }
        fn filename_number(&self, v: u8, m: &RenderingMemory) -> Result<PascalString, RuleError> {
            self.base.filename_number(v, m)
        }
    }
    impl StorageServices for Services {
        fn observe_storage(
            &self,
            r: &StorageRequest,
            m: &RenderingMemory,
        ) -> Result<StorageObservation, RuleError> {
            self.base.observe_storage(r, m)
        }
    }
    impl NumericServices for Services {
        fn number_to_string(&mut self, v: i32) -> Result<PascalString, RuleError> {
            self.base.base.number_to_string(v)
        }
        fn string_to_number(&mut self, p: &PascalString) -> Result<i32, RuleError> {
            self.base.base.string_to_number(p)
        }
    }
    impl DrawingServices for Services {
        fn color_icon(&mut self, id: i16) -> Result<u32, RuleError> {
            self.base.color_icon(id)
        }
        fn indexed_string(&mut self, r: i16, i: i16) -> Result<PascalString, RuleError> {
            self.base.indexed_string(r, i)
        }
        fn current_world(&mut self) -> Result<(u32, u32), RuleError> {
            self.base.current_world()
        }
        fn picture(&mut self, id: i16) -> Result<PictureObservation, RuleError> {
            self.base.picture(id)
        }
        fn new_world(&mut self, d: i16, b: BodyRect) -> Result<(i16, BuildingWorld), RuleError> {
            self.base.new_world(d, b)
        }
    }
    impl ItemUiServices for Services {
        fn observe(
            &mut self,
            r: &ItemUiRequest,
            _: &RenderingMemory,
        ) -> Result<ItemUiReply, RuleError> {
            match r {
                ItemUiRequest::GetDialogItem {
                    item_type,
                    handle,
                    rectangle,
                    ..
                } => Ok(ItemUiReply::Writes(vec![
                    crate::recovered_records::RecordWrite {
                        address: *item_type,
                        bytes: 8_i16.to_be_bytes().to_vec(),
                    },
                    crate::recovered_records::RecordWrite {
                        address: *handle,
                        bytes: 77_u32.to_be_bytes().to_vec(),
                    },
                    crate::recovered_records::RecordWrite {
                        address: *rectangle,
                        bytes: vec![0; 8],
                    },
                ])),
                ItemUiRequest::GetDialogItemText { .. } => {
                    Ok(ItemUiReply::Pascal(PascalString::from_payload(b"A\rB")?))
                }
                _ => Err(RuleError::Invalid("unexpected fixture item request".into())),
            }
        }
    }
    impl TimingServices for Services {
        fn tick_count(&mut self) -> Result<u32, RuleError> {
            let v = self.ticks.get();
            self.ticks.set(v + 100);
            Ok(v)
        }
        fn button(&mut self) -> Result<bool, RuleError> {
            Ok(false)
        }
    }
    impl StartupServices for Services {
        fn observe_startup(
            &self,
            r: &StartupRequest,
            m: &RenderingMemory,
        ) -> Result<StartupObservation, RuleError> {
            self.requests.borrow_mut().push(r.clone());
            if self.fail {
                return Err(RuleError::Invalid("absent startup observation".into()));
            }
            let mut n = m.clone();
            let mut value = 0;
            let mut a0 = 0;
            let mut zero = false;
            match r {
                StartupRequest::Uppercase(v) => value = (*v as u8).to_ascii_uppercase().into(),
                StartupRequest::PascalNameToC(a) => {
                    let len = usize::from(m.byte(*a)?);
                    let mut b = (0..len)
                        .map(|i| m.byte(*a + 1 + i as u32))
                        .collect::<Result<Vec<_>, _>>()?;
                    b.push(0);
                    n.insert_region(*a, &b);
                }
                StartupRequest::CNameToPascal(a) => {
                    let b = (0..40)
                        .map(|i| m.byte(*a + i))
                        .collect::<Result<Vec<_>, _>>()?;
                    let len = b.iter().position(|b| *b == 0).unwrap_or(40);
                    n.insert_region(*a, PascalString::from_payload(&b[..len])?.raw());
                }
                StartupRequest::StringToNumber { destination, .. } => long(&mut n, *destination, 2),
                StartupRequest::Toolbox {
                    opcode,
                    selector,
                    arguments,
                    ..
                } => match opcode {
                    0xa8a7 => {
                        if let [P(p), W(l), W(t), W(r), W(b)] = arguments.as_slice() {
                            for (i, v) in [*t, *l, *b, *r].into_iter().enumerate() {
                                word(&mut n, *p + i as u32 * 2, v as i16);
                            }
                        }
                    }
                    0xab1d => match selector {
                        Some(0x160000) => {
                            if let Some(P(p)) = arguments.first() {
                                long(&mut n, *p, *p + 0x100000);
                            }
                        }
                        Some(0x40017) => {
                            if let Some(L(p)) = arguments.first() {
                                value = p + 4;
                            }
                        }
                        _ => (),
                    },
                    0xa9bc => {
                        value = 0x500000;
                        if let Some(W(id)) = arguments.first() {
                            value += u32::from(*id) * 4;
                        }
                        long(&mut n, value, value + 64);
                    }
                    0xa9f5 => n.insert_region(0x479ed8, &[4, b'G', b'a', b'm', b'e']),
                    0xa85e => {
                        if let Some(P(p)) = arguments.first() {
                            long(&mut n, *p, m.long(*p)? | 0x02000000);
                        }
                    }
                    0xa8d8 => value = 0x600000,
                    0xaa2a => {
                        value = 0x600100;
                        long(&mut n, value, 0x600200);
                        n.insert_region(0x600200 + 34, &[0, 1, 0, 2, 0, 3, 0, 4]);
                    }
                    0xaa46 => value = 888,
                    0xa97c => value = 999,
                    0xa860 => {
                        word(
                            &mut n,
                            0x44720a,
                            self.events.borrow_mut().pop_front().unwrap_or(3),
                        );
                        value = 1;
                    }
                    0xa991 => word(
                        &mut n,
                        0x4434b2,
                        self.events.borrow_mut().pop_front().unwrap_or(39),
                    ),
                    0xa98d => {
                        if let [L(_), W(i), P(t), P(h), P(b)] = arguments.as_slice() {
                            let printing = m.word(0x4434aa)? as i16 >= 3;
                            let kind = if printing && *i == 3 { 8 } else { 4 };
                            word(&mut n, *t, kind);
                            long(
                                &mut n,
                                *h,
                                if printing && *i != 3 {
                                    0
                                } else {
                                    u32::from(*i)
                                },
                            );
                            n.insert_region(*b, &[0; 8]);
                        }
                    }
                    0xa960 => {
                        if let Some(L(h)) = arguments.first() {
                            value = self
                                .controls
                                .borrow()
                                .get(&(*h as i16))
                                .copied()
                                .unwrap_or(0) as u16 as u32;
                        }
                    }
                    0xa963 => {
                        if let [L(h), W(v)] = arguments.as_slice() {
                            self.controls.borrow_mut().insert(*h as i16, *v as i16);
                        }
                    }
                    0xa9eb => {
                        zero = selector == &Some(0x1008);
                        if selector == &Some(0x2810)
                            && let Some(P(dest)) = arguments.last()
                        {
                            long(&mut n, *dest, 31);
                        }
                    }
                    0xa322 => a0 = 0x600300,
                    0xa8fd => match selector {
                        Some(0x32040488) => value = u32::from(self.print_accept),
                        Some(0x04000c00) => value = 0x600400,
                        _ => (),
                    },
                    0xa975 => {
                        value = self.ticks.get();
                        self.ticks.set(value + 100);
                    }
                    _ => (),
                },
                _ => (),
            }
            Ok(StartupObservation {
                memory: n,
                value,
                d0: 0xaabb0011,
                a0,
                zero,
            })
        }
        fn preference_inputs(&self, _: &RenderingMemory) -> Result<PreferenceInputs, RuleError> {
            let obs = |request, reply| PreferenceObservation { request, reply };
            let ignored = |request| obs(request, V::Ignored);
            let name = PascalString::from_payload(b"Prefs")?;
            Ok(PreferenceInputs {
                handles: BTreeMap::new(),
                observations: vec![
                    obs(R::CurrentResourceFile, V::Word(12)),
                    obs(
                        R::FindFolder {
                            volume: i16::MIN,
                            kind: 0x70726566,
                            create: false,
                        },
                        V::Folder {
                            volume: -3,
                            directory: 42,
                            error: 0,
                        },
                    ),
                    obs(
                        R::MakeFileSpecification {
                            volume: -3,
                            directory: 42,
                            name: PascalString::from_payload(b"New Centurions Prefs")?,
                        },
                        V::FileSpecification {
                            bytes: vec![0; 70],
                            error: 0,
                        },
                    ),
                    obs(
                        R::OpenResourceFile {
                            specification: vec![0; 70],
                            permission: 0,
                        },
                        V::Word(24),
                    ),
                    ignored(R::UseResourceFile(24)),
                    obs(R::NewHandleClear(362), V::Handle(700)),
                    ignored(R::LockHandle(700)),
                    obs(
                        R::GetResource {
                            kind: 0x5052464e,
                            id: 128,
                            current_only: false,
                        },
                        V::Resource {
                            handle: 800,
                            memory: Some(HandleMemory {
                                bytes: vec![0; 362],
                                logical_size: 362,
                            }),
                        },
                    ),
                    obs(
                        R::ResourceInfo(800),
                        V::ResourceInfo {
                            kind: 0x5052464e,
                            id: 128,
                            name: name.clone(),
                        },
                    ),
                    obs(R::HomeResourceFile(800), V::Word(24)),
                    ignored(R::RemoveResource(800)),
                    ignored(R::AddResource {
                        handle: 700,
                        kind: 0x5052464e,
                        id: 128,
                        name,
                    }),
                    ignored(R::WriteResource(700)),
                    ignored(R::UnlockHandle(700)),
                    ignored(R::ReleaseResource(700)),
                    ignored(R::CloseResourceFile(24)),
                    ignored(R::UseResourceFile(12)),
                ],
            })
        }
    }
    pub fn fixture() -> RenderingMemory {
        let mut m = storage::fixture();
        for (a, n) in [
            (0x401400, 1400),
            (0xbaa, 2),
            (0x9ee, 4),
            (0x4046d0, 64),
            (0x41f604, 4),
        ] {
            m.insert_region(a, &vec![0; n]);
        }
        m.insert_region(0x438587, &[1]);
        m.insert_region(0x438aa4, &[3, b'b', b'o', b'b', 0]);
        long(&mut m, 0x4386f8, 0x12345678);
        m
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    #[test]
    fn world_setup_preserves_asymmetric_rects_early_picture_draws_and_overwritten_slot() {
        let m = fixture();
        let r = RandomState::new(7);
        let s = Services::default();
        let p = initialize_worlds(&m, &r, FRAME, LIMITS, &s).unwrap();
        assert_eq!(p.random, r);
        assert_eq!(p.memory.long(0x4386f4).unwrap(), 0x5386d8);
        let q = s.requests.borrow();
        let locks = q
            .iter()
            .filter_map(|r| {
                if let StartupRequest::Toolbox {
                    opcode: 0xab1d,
                    selector: Some(0x40001),
                    arguments,
                    ..
                } = r
                {
                    Some(arguments.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(locks.len(), 9);
        assert_eq!(locks[7], vec![L(0x12345678)]);
        assert!(q.iter().any(|r|matches!(r,StartupRequest::Toolbox{opcode:0xa8a7,arguments,..}if arguments==&vec![P(0x4434ba),W(0),W(0),W(206),W(129)])));
        assert!(q.iter().any(|r|matches!(r,StartupRequest::Toolbox{opcode:0xa8a7,arguments,..}if arguments==&vec![P(0x4434ba),W(0),W(0),W(960),W(192)])));
        let pic=q.iter().position(|r|matches!(r,StartupRequest::Toolbox{opcode:0xa9bc,arguments,..}if arguments==&vec![W(192)])).unwrap();
        assert!(matches!(
            q[pic + 1],
            StartupRequest::Toolbox { opcode: 0xa8f6, .. }
        ));
        let failed = Services {
            fail: true,
            ..Services::default()
        };
        assert!(initialize_worlds(&m, &r, FRAME, LIMITS, &failed).is_err());
        assert_eq!(m, fixture());
    }
    #[test]
    fn seed_and_name_bookkeeping_keeps_one_draw_all_comparisons_and_short_read_tail() {
        let mut m = fixture();
        long(&mut m, FRAME + 8, 123);
        long(&mut m, 0x4015fe, 123);
        let r = RandomState::new(17);
        let s = Services::default();
        let p = set_free(&m, &r, FRAME, LIMITS, &s).unwrap();
        assert_eq!(p.random.draws, 1);
        assert_ne!(p.memory.long(0x438598).unwrap(), 0x00801000);
        assert_eq!(
            p.memory.long(0x43859c).unwrap(),
            p.memory.long(0x438598).unwrap() | 0x02000000
        );
        let n = set_free_name(&fixture(), &r, FRAME, LIMITS, &s).unwrap();
        assert_eq!(n.random, r);
        assert_eq!(n.memory.byte(0x438aa4).unwrap(), 3);
        assert_eq!(n.memory.byte(0x438aa5).unwrap(), b'B');
        assert_eq!(
            s.base
                .requests
                .borrow()
                .iter()
                .filter(|q| matches!(
                    q,
                    crate::recovered_storage::StorageRequest::CompareCString { .. }
                ))
                .count(),
            28
        );
        let mut m = fixture();
        long(&mut m, 0x43859c, 0x80000001);
        long(&mut m, 0x479fdc, 0x80000001);
        s.base.reads.borrow_mut().insert(
            9,
            [crate::recovered_storage::test_support::ReadReply {
                value: 0,
                bytes: vec![0x80],
            }]
            .into(),
        );
        let p = check_for_mismatch(&m, &r, FRAME, LIMITS, &s).unwrap();
        assert_eq!(p.memory.long(0x479fdc).unwrap(), 0x80000001);
        assert_eq!(p.random, r);
    }
    #[test]
    fn logo_waits_for_native_event_kind_then_restores_menu_and_closes_dialog() {
        let mut m = fixture();
        word(&mut m, FRAME + 8, -1);
        word(&mut m, 0xbaa, 20);
        long(&mut m, 0x9ee, 123);
        m.insert_region(0x438587, &[0, 1]);
        let mut s = Services::default();
        s.events.borrow_mut().extend([2, 3]);
        let p = show_logo(&m, &RandomState::new(1), FRAME, LIMITS, &mut s).unwrap();
        assert_eq!(p.memory.word(0xbaa).unwrap(), 20);
        assert_eq!(p.memory.word(0x4042c2).unwrap(), 20);
        assert_eq!(p.memory.word(0x44720a).unwrap(), 3);
        assert_eq!(
            p.effects
                .iter()
                .filter_map(|effect| match effect {
                    StartupEffect::Storage(StorageEffect::Write(write))
                        if write.address == FRAME - 12 =>
                        Some(write.bytes.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![
                0x00190001_u32.to_be_bytes().to_vec(),
                0x00190000_u32.to_be_bytes().to_vec(),
                0x00190001_u32.to_be_bytes().to_vec()
            ]
        );
        let q = s.requests.borrow();
        assert!(q.iter().any(|request| matches!(request, StartupRequest::Toolbox { opcode:0xa91d, arguments,.. } if arguments == &vec![L(999), W(640), W(460), B(0)])));
        assert_eq!(
            q.iter()
                .filter(|r| matches!(r, StartupRequest::Toolbox { opcode: 0xa860, .. }))
                .count(),
            2
        );
        assert_eq!(
            q.iter()
                .filter(|r| matches!(r, StartupRequest::Toolbox { opcode: 0xa937, .. }))
                .count(),
            2
        );
        assert!(matches!(
            q.last(),
            Some(StartupRequest::Toolbox { opcode: 0xa853, .. })
        ));
    }
}
