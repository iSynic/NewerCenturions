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
pub use StartupArg::{Byte as B, Long as L, Pointer as P, Word as W};
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
pub struct Execution {
    pub e: StorageExecution,
    pub random: RandomState,
    pub effects: Vec<StartupEffect>,
    pub d0: u32,
}
impl Execution {
    pub fn new(
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
    pub fn finish(mut self) -> StartupPlan {
        self.flush();
        StartupPlan {
            memory: self.e.plan.memory,
            random: self.random,
            effects: self.effects,
            flow: self.e.plan.flow,
            native_return: self.e.plan.native_return,
        }
    }
    pub fn flush(&mut self) {
        self.effects
            .extend(self.e.plan.effects.drain(..).map(StartupEffect::Storage));
    }
    pub fn request(
        &mut self,
        request: StartupRequest,
        s: &(impl StartupServices + ?Sized),
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
    pub fn tb(
        &mut self,
        opcode: u16,
        selector: Option<u32>,
        arguments: Vec<StartupArg>,
        s: &(impl StartupServices + ?Sized),
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
    pub fn ww(&mut self, address: u32, value: i16) -> Result<(), RuleError> {
        self.e.ww(address, value)
    }
    pub fn lw(&mut self, address: u32, value: u32) -> Result<(), RuleError> {
        self.e.lw(address, value)
    }
    pub fn get_item(
        &mut self,
        dialog: u32,
        item: i16,
        handle: u32,
        s: &(impl StartupServices + ?Sized),
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
    pub fn di_str(
        &mut self,
        item: i16,
        text: u32,
        s: &(impl StartupServices + ?Sized),
    ) -> Result<(), RuleError> {
        let dialog = self.e.long(0x4394b8)?;
        self.get_item(dialog, item, 0x4434dc, s)?;
        self.tb(0xa98f, None, vec![L(self.e.long(0x4434dc)?), P(text)], s)?;
        self.d0 = 0;
        Ok(())
    }
    pub fn number(
        &mut self,
        item: i16,
        value: i32,
        s: &(impl StartupServices + ?Sized),
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
    pub fn warn(
        &mut self,
        frame: u32,
        id: i16,
        s: &(impl StartupServices + ?Sized),
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
    fn save_pref(&mut self, s: &(impl StartupServices + ?Sized)) -> Result<(), RuleError> {
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
    fn app_name(
        &mut self,
        frame: u32,
        s: &(impl StartupServices + ?Sized),
    ) -> Result<(), RuleError> {
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
        s: &(impl StartupServices + ?Sized),
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
    s: &(impl StartupServices + ?Sized),
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
    s: &(impl StartupServices + ?Sized),
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
    s: &(impl StartupServices + ?Sized),
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
    s: &(impl StartupServices + ?Sized),
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
    s: &mut (impl StartupServices + ?Sized),
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
