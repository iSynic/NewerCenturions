//! Missile preparation from CODE 1, with observed SANE and Toolbox calls.

use crate::recovered_feedback::FeedbackArg;
use crate::recovered_records::RecordWrite;
use crate::recovered_rendering::RenderingMemory;
use crate::{RuleError, recovered_spatial};
use FeedbackArg::{Long as L, Pointer as P, Word as W};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationRequest {
    pub opcode: u16,
    /// Pascal push order; extended arguments remain in caller memory.
    pub arguments: Vec<FeedbackArg>,
    pub d0_input: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationObservation {
    pub memory: RenderingMemory,
    pub value: u32,
    pub d0: u32,
    /// Required only when the zero-vector direction branch inherits D1.
    pub d1: Option<u32>,
}

pub trait AnimationServices {
    fn observe_animation(
        &mut self,
        request: &AnimationRequest,
        memory: &RenderingMemory,
    ) -> Result<AnimationObservation, RuleError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationEffect {
    Write(RecordWrite),
    Service {
        request: AnimationRequest,
        observation: AnimationObservation,
    },
    Direction {
        first: f64,
        second: f64,
        incoming_d1: i8,
        result: i16,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationPlan {
    pub memory: RenderingMemory,
    pub effects: Vec<AnimationEffect>,
    pub native_return: u32,
}

struct Execution {
    plan: AnimationPlan,
    remaining: u32,
    last_d1: Option<u32>,
}
impl Execution {
    fn new(memory: &RenderingMemory, limit: u32) -> Self {
        Self {
            plan: AnimationPlan {
                memory: memory.clone(),
                effects: vec![],
                native_return: 0,
            },
            remaining: limit,
            last_d1: None,
        }
    }
    fn step(&mut self) -> Result<(), RuleError> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            RuleError::Invalid("spell animation processing budget exhausted".into())
        })?;
        Ok(())
    }
    fn w(&self, a: u32) -> Result<i16, RuleError> {
        Ok(self.plan.memory.word(a)? as i16)
    }
    fn l(&self, a: u32) -> Result<u32, RuleError> {
        self.plan.memory.long(a)
    }
    fn write(&mut self, a: u32, bytes: &[u8]) -> Result<(), RuleError> {
        for i in 0..bytes.len() {
            self.plan.memory.byte(a.wrapping_add(i as u32))?;
        }
        self.plan.memory.insert_region(a, bytes);
        self.plan.effects.push(AnimationEffect::Write(RecordWrite {
            address: a,
            bytes: bytes.to_vec(),
        }));
        Ok(())
    }
    fn ww(&mut self, a: u32, v: i16) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn lw(&mut self, a: u32, v: u32) -> Result<(), RuleError> {
        self.write(a, &v.to_be_bytes())
    }
    fn trap(
        &mut self,
        opcode: u16,
        arguments: Vec<FeedbackArg>,
        s: &mut (impl AnimationServices + ?Sized),
    ) -> Result<u32, RuleError> {
        self.service(opcode, arguments, None, s)
    }
    fn dispatch(
        &mut self,
        selector: u32,
        arguments: Vec<FeedbackArg>,
        s: &mut (impl AnimationServices + ?Sized),
    ) -> Result<u32, RuleError> {
        self.service(0xab1d, arguments, Some(selector), s)
    }
    fn service(
        &mut self,
        opcode: u16,
        arguments: Vec<FeedbackArg>,
        d0_input: Option<u32>,
        s: &mut (impl AnimationServices + ?Sized),
    ) -> Result<u32, RuleError> {
        self.step()?;
        let request = AnimationRequest {
            opcode,
            arguments,
            d0_input,
        };
        let observation = s.observe_animation(&request, &self.plan.memory)?;
        let value = observation.value;
        self.last_d1 = observation.d1;
        self.plan.memory = observation.memory.clone();
        self.plan.effects.push(AnimationEffect::Service {
            request,
            observation,
        });
        Ok(value)
    }
    fn sane(
        &mut self,
        source: u32,
        destination: u32,
        selector: u16,
        s: &mut (impl AnimationServices + ?Sized),
    ) -> Result<(), RuleError> {
        self.trap(0xa9eb, vec![P(source), P(destination), W(selector)], s)?;
        Ok(())
    }
}
fn idx(a: u32, n: i16) -> u32 {
    a.wrapping_add((i32::from(n) * 4) as u32)
}
fn extended(memory: &RenderingMemory, address: u32) -> Result<f64, RuleError> {
    let sign_exponent = memory.word(address)?;
    let mut significand = 0_u64;
    for i in 2..10 {
        significand = (significand << 8) | u64::from(memory.byte(address.wrapping_add(i))?);
    }
    let exponent = sign_exponent & 0x7fff;
    let magnitude = if exponent == 0x7fff {
        if significand == 0x8000_0000_0000_0000 {
            f64::INFINITY
        } else {
            f64::NAN
        }
    } else if significand == 0 {
        0.0
    } else {
        (significand as f64 / 9223372036854775808.0)
            * 2.0_f64.powi(i32::from(exponent.max(1)) - 16383)
    };
    Ok(if sign_exponent & 0x8000 != 0 {
        -magnitude
    } else {
        magnitude
    })
}

/// missileset, 0x15f048. Uses the existing direction child's declared SINGLE model.
pub fn missileset(
    memory: &RenderingMemory,
    frame: u32,
    limit: u32,
    s: &mut (impl AnimationServices + ?Sized),
) -> Result<AnimationPlan, RuleError> {
    let mut e = Execution::new(memory, limit);
    e.sane(frame.wrapping_add(8), frame.wrapping_sub(40), 0x1010, s)?;
    e.lw(frame.wrapping_sub(12), e.l(frame.wrapping_sub(40))?)?;
    e.sane(frame.wrapping_add(18), frame.wrapping_sub(40), 0x1010, s)?;
    e.lw(frame.wrapping_sub(16), e.l(frame.wrapping_sub(40))?)?;
    e.trap(0xa873, vec![L(e.l(0x43878c)?)], s)?;
    e.ww(frame.wrapping_sub(8), 320)?;
    e.ww(frame.wrapping_sub(2), 32)?;
    e.ww(frame.wrapping_sub(6), 0)?;
    e.ww(frame.wrapping_sub(4), 352)?;
    if e.w(0x453788)? != 16 && e.w(0x453788)? != 18 {
        return Ok(e.plan);
    }
    e.lw(0x439598, 0)?;
    let difference = e
        .w(0x44719e)?
        .wrapping_sub(e.w(idx(0x44709a, e.w(0x4435d2)?))?);
    e.ww(frame.wrapping_sub(36), difference)?;
    e.sane(frame.wrapping_sub(36), frame.wrapping_sub(36), 0x200e, s)?;
    e.sane(frame.wrapping_sub(36), frame.wrapping_sub(36), 0x1010, s)?;
    e.lw(frame.wrapping_sub(12), e.l(frame.wrapping_sub(36))?)?;
    let difference = e
        .w(0x4471a0)?
        .wrapping_sub(e.w(idx(0x44709c, e.w(0x4435d2)?))?);
    e.ww(frame.wrapping_sub(36), difference)?;
    e.sane(frame.wrapping_sub(36), frame.wrapping_sub(36), 0x200e, s)?;
    e.sane(frame.wrapping_sub(36), frame.wrapping_sub(36), 0x1010, s)?;
    e.lw(frame.wrapping_sub(16), e.l(frame.wrapping_sub(36))?)?;
    e.sane(frame.wrapping_sub(16), frame.wrapping_sub(26), 0x100e, s)?;
    e.lw(frame.wrapping_sub(48), e.l(frame.wrapping_sub(20))?)?;
    e.lw(frame.wrapping_sub(52), e.l(frame.wrapping_sub(24))?)?;
    e.ww(frame.wrapping_sub(54), e.w(frame.wrapping_sub(26))?)?;
    e.sane(frame.wrapping_sub(12), frame.wrapping_sub(36), 0x100e, s)?;
    e.lw(frame.wrapping_sub(58), e.l(frame.wrapping_sub(30))?)?;
    e.lw(frame.wrapping_sub(62), e.l(frame.wrapping_sub(34))?)?;
    e.ww(frame.wrapping_sub(64), e.w(frame.wrapping_sub(36))?)?;
    let first = extended(&e.plan.memory, frame.wrapping_sub(64))?;
    let second = extended(&e.plan.memory, frame.wrapping_sub(54))?;
    let incoming = if first as f32 == 0.0 && second as f32 == 0.0 {
        e.last_d1.ok_or(RuleError::Unsupported {
            action: "missileset.direction",
            dependency: "actual incoming D1 for the zero-vector branch",
        })? as i8
    } else {
        0
    };
    let direction = recovered_spatial::direction(first, second, incoming)
        .map_err(|error| RuleError::Invalid(error.to_string()))?;
    e.plan.effects.push(AnimationEffect::Direction {
        first,
        second,
        incoming_d1: incoming,
        result: direction,
    });
    let id = i16::from(e.plan.memory.byte(0x44375f)? as i8)
        .wrapping_shl(3)
        .wrapping_add(0x2ed8)
        .wrapping_add(direction);
    let picture = e.trap(0xaa1e, vec![W(id as u16)], s)?;
    e.lw(0x439598, picture)?;
    if e.l(0x439598)? == 0 {
        return Ok(e.plan);
    }
    e.trap(0xa862, vec![L(33)], s)?;
    e.trap(0xa863, vec![L(30)], s)?;
    e.trap(0xaa1f, vec![P(0x4436f2), L(e.l(0x439598)?)], s)?;
    e.dispatch(0x80005, vec![P(0x4046da), P(0x4046de)], s)?;
    e.dispatch(0x80006, vec![L(e.l(0x4386c8)?), L(0)], s)?;
    e.trap(0xa8a3, vec![P(frame.wrapping_sub(8))], s)?;
    e.trap(0xaa1f, vec![P(frame.wrapping_sub(8)), L(e.l(0x439598)?)], s)?;
    e.trap(0xaa25, vec![L(e.l(0x439598)?)], s)?;
    e.dispatch(0x80006, vec![L(e.l(0x4046da)?), L(e.l(0x4046de)?)], s)?;
    Ok(e.plan)
}

/// finddxdy, 0x15edfa. Source and destination have different body-9 thresholds.
pub fn finddxdy(
    memory: &RenderingMemory,
    frame: u32,
    limit: u32,
    s: &mut (impl AnimationServices + ?Sized),
) -> Result<AnimationPlan, RuleError> {
    let mut e = Execution::new(memory, limit);
    let from = e.plan.memory.byte(frame.wrapping_add(9))? as i8;
    let to = e.plan.memory.byte(frame.wrapping_add(11))? as i8;
    e.lw(frame.wrapping_sub(6), 0)?;
    e.lw(frame.wrapping_sub(10), 0)?;
    let a = if from > 9 {
        idx(0x446f0a, i16::from(from.wrapping_sub(10)))
    } else {
        idx(0x44709a, i16::from(from))
    };
    let x = e.w(a)? as i8;
    e.write(frame.wrapping_sub(1), &[e.w(a.wrapping_add(2))? as u8])?;
    let a = if to >= 9 {
        idx(0x446f0a, i16::from(to.wrapping_sub(10)))
    } else {
        idx(0x44709a, i16::from(to))
    };
    let tx = e.w(a)? as i8;
    let ty = e.w(a.wrapping_add(2))? as i8;
    if i16::from(tx) < i16::from(x) - 1 {
        e.lw(frame.wrapping_sub(6), (-1.0_f32).to_bits())?;
    }
    if i16::from(tx) > i16::from(x) + 1 {
        e.lw(frame.wrapping_sub(6), 1.0_f32.to_bits())?;
    }
    let y = e.plan.memory.byte(frame.wrapping_sub(1))? as i8;
    if i16::from(ty) < i16::from(y) - 1 {
        e.lw(frame.wrapping_sub(10), (-1.0_f32).to_bits())?;
    }
    let y = e.plan.memory.byte(frame.wrapping_sub(1))? as i8;
    if i16::from(ty) > i16::from(y) + 1 {
        e.lw(frame.wrapping_sub(10), 1.0_f32.to_bits())?;
    }
    e.sane(frame.wrapping_sub(10), frame.wrapping_sub(20), 0x100e, s)?;
    e.lw(frame.wrapping_sub(54), e.l(frame.wrapping_sub(14))?)?;
    e.lw(frame.wrapping_sub(58), e.l(frame.wrapping_sub(18))?)?;
    e.ww(frame.wrapping_sub(60), e.w(frame.wrapping_sub(20))?)?;
    e.sane(frame.wrapping_sub(6), frame.wrapping_sub(30), 0x100e, s)?;
    e.lw(frame.wrapping_sub(64), e.l(frame.wrapping_sub(24))?)?;
    e.lw(frame.wrapping_sub(68), e.l(frame.wrapping_sub(28))?)?;
    e.ww(frame.wrapping_sub(70), e.w(frame.wrapping_sub(30))?)?;
    let child = missileset(&e.plan.memory, frame.wrapping_sub(78), e.remaining, s)?;
    e.plan.memory = child.memory;
    e.plan.effects.extend(child.effects);
    e.plan.native_return = child.native_return;
    Ok(e.plan)
}
