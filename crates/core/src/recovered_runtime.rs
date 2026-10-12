//! CODE 1 relocation, runtime callbacks, arithmetic and graphics maintenance.

use std::collections::BTreeMap;

use crate::{RandomState, RuleError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMemory {
    pub bytes: BTreeMap<u32, u8>,
}

impl RuntimeMemory {
    /// Seed a caller-owned memory region; this does not read original assets.
    pub fn insert_region(&mut self, address: u32, bytes: &[u8]) {
        for (index, byte) in bytes.iter().enumerate() {
            self.bytes.insert(address.wrapping_add(index as u32), *byte);
        }
    }

    pub fn read(&self, address: u32, length: usize) -> Result<Vec<u8>, RuleError> {
        (0..length)
            .map(|index| {
                let location = address.wrapping_add(index as u32);
                self.bytes.get(&location).copied().ok_or_else(|| {
                    RuleError::Invalid(format!(
                        "runtime memory byte 0x{location:08x} has not been supplied"
                    ))
                })
            })
            .collect()
    }

    pub fn long(&self, address: u32) -> Result<u32, RuleError> {
        Ok(u32::from_be_bytes(
            self.read(address, 4)?.try_into().expect("four bytes"),
        ))
    }

    pub fn word(&self, address: u32) -> Result<i16, RuleError> {
        Ok(i16::from_be_bytes(
            self.read(address, 2)?.try_into().expect("two bytes"),
        ))
    }

    fn write(&mut self, address: u32, bytes: &[u8]) -> Result<(), RuleError> {
        self.read(address, bytes.len())?;
        self.insert_region(address, bytes);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeContext {
    pub memory: RuntimeMemory,
    pub random: RandomState,
    /// An explicit adapter budget for relocation entries and callback iterations.
    pub step_limit: usize,
}

impl RuntimeContext {
    pub fn new(random: RandomState, step_limit: usize) -> Self {
        Self {
            memory: RuntimeMemory::default(),
            random,
            step_limit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeWrite {
    pub address: u32,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeRequest {
    UnlockPixels(u32),
    MaxMem,
    GetGWorldPixMap(u32),
    LockPixels(u32),
    SetPort(u32),
    ForeColor(i32),
    BackColor(i32),
    CopyBits {
        source_bitmap: u32,
        destination_bitmap: u32,
        source_rectangle: [u8; 8],
        destination_rectangle: [u8; 8],
        mode: i16,
        mask_region: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeReply {
    Pointer(u32),
    LockPixels { boolean: u8, native_d0: u32 },
    Word(i16),
    Long(i32),
    MaxMem { d0: u32, a0: u32 },
    Ignored,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeObservation {
    pub request: RuntimeRequest,
    pub reply: RuntimeReply,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DestructorCallback {
    pub function: u32,
    pub argument: u32,
    pub reason: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DestructorContinuation {
    pub node: u32,
    pub unlink: RuntimeWrite,
    pub callback: DestructorCallback,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEffect {
    Service(RuntimeObservation),
    MemoryWrite(RuntimeWrite),
    Relocation {
        entry: u32,
        encoded: Vec<u8>,
        offset: i32,
        target: u32,
        addend: u32,
        before: u32,
        after: u32,
    },
    /// Offsets are within the source's eight-byte local scratch.
    LocalWrite {
        offset: usize,
        bytes: Vec<u8>,
    },
    CallbackEntered(DestructorCallback),
    CallbackReturned,
    Random {
        bound: i16,
        quickdraw_word: i16,
        result: i16,
        before: RandomState,
        after: RandomState,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeResult {
    Pointer(u32),
    Long(u32),
    Word {
        value: i16,
        native_d0: u32,
    },
    /// 0x1002ec writes the transfer address into the adjusted return stack.
    Transfer {
        target: u32,
        stack_after: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePlan {
    pub effects: Vec<RuntimeEffect>,
    pub consumed_observations: usize,
    pub result: RuntimeResult,
}

/// Resolve actual recovered Rust callbacks; unimplemented pointers must diagnose.
/// A handler mutates only the candidate context and returns its ordered effects.
pub trait RuntimeCallbacks {
    fn invoke(
        &self,
        context: &mut RuntimeContext,
        callback: &DestructorCallback,
    ) -> Result<Vec<RuntimeEffect>, RuleError>;
}

pub struct UnresolvedCallbacks;

impl RuntimeCallbacks for UnresolvedCallbacks {
    fn invoke(
        &self,
        _context: &mut RuntimeContext,
        _callback: &DestructorCallback,
    ) -> Result<Vec<RuntimeEffect>, RuleError> {
        Err(RuleError::Unsupported {
            action: "runtime.destructor_callback",
            dependency: "the indirect callback has no recovered Rust handler; inspect next_destructor for its continuation",
        })
    }
}

struct Execution<'a> {
    observations: &'a [RuntimeObservation],
    consumed: usize,
    steps: usize,
    effects: Vec<RuntimeEffect>,
}

impl Execution<'_> {
    fn request(&mut self, request: RuntimeRequest) -> Result<RuntimeReply, RuleError> {
        let observation = self.observations.get(self.consumed).ok_or(RuleError::Unsupported {
            action: "runtime.service",
            dependency: "the next Memory Manager or QuickDraw service result has not been supplied",
        })?;
        if observation.request != request {
            return Err(RuleError::Invalid(format!(
                "runtime observation {} does not match request {request:?}",
                self.consumed
            )));
        }
        self.consumed += 1;
        self.effects
            .push(RuntimeEffect::Service(observation.clone()));
        Ok(observation.reply.clone())
    }

    fn ignored(&mut self, request: RuntimeRequest) -> Result<(), RuleError> {
        match self.request(request)? {
            RuntimeReply::Ignored | RuntimeReply::Word(_) | RuntimeReply::Long(_) => Ok(()),
            _ => Err(RuleError::Invalid(
                "unexpected discarded runtime service result".into(),
            )),
        }
    }

    fn step(&mut self, limit: usize) -> Result<(), RuleError> {
        if self.steps >= limit {
            return Err(RuleError::Invalid(
                "runtime loop exceeds the supplied step limit".into(),
            ));
        }
        self.steps += 1;
        Ok(())
    }

    fn write(
        &mut self,
        context: &mut RuntimeContext,
        address: u32,
        bytes: &[u8],
    ) -> Result<(), RuleError> {
        context.memory.write(address, bytes)?;
        self.effects.push(RuntimeEffect::MemoryWrite(RuntimeWrite {
            address,
            bytes: bytes.to_vec(),
        }));
        Ok(())
    }
}

fn execute(
    context: &mut RuntimeContext,
    observations: &[RuntimeObservation],
    body: impl FnOnce(&mut RuntimeContext, &mut Execution<'_>) -> Result<RuntimeResult, RuleError>,
) -> Result<RuntimePlan, RuleError> {
    let mut candidate = context.clone();
    let mut execution = Execution {
        observations,
        consumed: 0,
        steps: 0,
        effects: vec![],
    };
    let result = body(&mut candidate, &mut execution)?;
    *context = candidate;
    Ok(RuntimePlan {
        effects: execution.effects,
        consumed_observations: execution.consumed,
        result,
    })
}

fn relocation_body(
    context: &mut RuntimeContext,
    execution: &mut Execution<'_>,
    table: u32,
    base: u32,
    addend: u32,
) -> Result<u32, RuleError> {
    let mut count = context.memory.long(table)? as i32;
    let mut cursor = table.wrapping_add(4);
    let mut offset = 0_i32;
    while count > 0 {
        execution.step(context.step_limit)?;
        let entry = cursor;
        let first = context.memory.read(cursor, 1)?[0];
        cursor = cursor.wrapping_add(1);
        let mut encoded = vec![first];
        if first & 0x80 != 0 {
            offset = offset.wrapping_add(i32::from(first.wrapping_mul(2) as i8));
        } else {
            let second = context.memory.read(cursor, 1)?[0];
            cursor = cursor.wrapping_add(1);
            encoded.push(second);
            if first & 0x40 != 0 {
                let word = u16::from_be_bytes([first, second]);
                let delta = i32::from((word.wrapping_shl(2) as i16) >> 1);
                offset = offset.wrapping_add(delta);
            } else {
                let third = context.memory.read(cursor, 1)?[0];
                cursor = cursor.wrapping_add(1);
                let fourth = context.memory.read(cursor, 1)?[0];
                cursor = cursor.wrapping_add(1);
                encoded.extend_from_slice(&[third, fourth]);
                let absolute = u32::from_be_bytes([first, second, third, fourth]);
                offset = (absolute.wrapping_shl(2) as i32) >> 1;
            }
        }
        let target = base.wrapping_add(offset as u32);
        let before = context.memory.long(target)?;
        let after = before.wrapping_add(addend);
        execution.write(context, target, &after.to_be_bytes())?;
        execution.effects.push(RuntimeEffect::Relocation {
            entry,
            encoded,
            offset,
            target,
            addend,
            before,
            after,
        });
        count = count.wrapping_sub(1);
    }
    Ok(cursor)
}

/// CODE 1 0x100212: signed LONG entry count and 1/2/4-byte relocation encoding.
pub fn relocate(
    context: &mut RuntimeContext,
    table: u32,
    base: u32,
    addend: u32,
) -> Result<RuntimePlan, RuleError> {
    execute(context, &[], |context, execution| {
        Ok(RuntimeResult::Pointer(relocation_body(
            context, execution, table, base, addend,
        )?))
    })
}

fn three_body(
    context: &mut RuntimeContext,
    execution: &mut Execution<'_>,
    table: u32,
    base: u32,
    a5: u32,
    inverse: bool,
) -> Result<RuntimeResult, RuleError> {
    let sign = |value: u32| if inverse { value.wrapping_neg() } else { value };
    let cursor = relocation_body(context, execution, table, base, sign(a5))?;
    // This global is read after the first relocation table, including aliases.
    let second_addend = context.memory.long(a5.wrapping_sub(0x1de2))?;
    let cursor = relocation_body(context, execution, cursor, base, sign(second_addend))?;
    let cursor = relocation_body(context, execution, cursor, base, sign(base))?;
    Ok(RuntimeResult::Pointer(cursor))
}

/// CODE 1 0x100286: three consecutive tables, adding A5, its LONG global, then A1.
pub fn relocate_three(
    context: &mut RuntimeContext,
    table: u32,
    base: u32,
    a5: u32,
) -> Result<RuntimePlan, RuleError> {
    execute(context, &[], |context, execution| {
        three_body(context, execution, table, base, a5, false)
    })
}

/// CODE 1 0x1002b2: the same sequence with wrapped LONG negation of each addend.
pub fn unrelocate_three(
    context: &mut RuntimeContext,
    table: u32,
    base: u32,
    a5: u32,
) -> Result<RuntimePlan, RuleError> {
    execute(context, &[], |context, execution| {
        three_body(context, execution, table, base, a5, true)
    })
}

/// CODE 1 0x1002ec: selector at entry SP+4, saved A5 at CODE1 LONG 0x10030e.
pub fn runtime_transfer(
    context: &mut RuntimeContext,
    stack_pointer: u32,
) -> Result<RuntimePlan, RuleError> {
    execute(context, &[], |context, execution| {
        let saved_a5 = context.memory.long(0x10030e)?;
        let mut target = context.memory.long(saved_a5.wrapping_sub(0x1de6))?;
        if context.memory.word(stack_pointer.wrapping_add(4))? == 0 {
            target = context.memory.long(saved_a5.wrapping_sub(0x1dea))?;
        }
        execution.write(
            context,
            stack_pointer.wrapping_add(2),
            &target.to_be_bytes(),
        )?;
        Ok(RuntimeResult::Transfer {
            target,
            stack_after: stack_pointer.wrapping_add(6),
        })
    })
}

/// Inspect the next unlink/call without mutation, including any head/node aliases.
pub fn next_destructor(
    context: &RuntimeContext,
    a5: u32,
) -> Result<Option<DestructorContinuation>, RuleError> {
    let head_address = a5.wrapping_sub(0x1dcc);
    let node = context.memory.long(head_address)?;
    if node == 0 {
        return Ok(None);
    }
    let next = context.memory.long(node)?;
    let unlink = RuntimeWrite {
        address: head_address,
        bytes: next.to_be_bytes().to_vec(),
    };
    let mut memory = context.memory.clone();
    memory.write(head_address, &unlink.bytes)?;
    // Source pushes the argument before loading the callback function pointer.
    let argument = memory.long(node.wrapping_add(8))?;
    let function = memory.long(node.wrapping_add(4))?;
    Ok(Some(DestructorContinuation {
        node,
        unlink,
        callback: DestructorCallback {
            function,
            argument,
            reason: -1,
        },
    }))
}

/// CODE 1 0x1004ae: unlink each head, call its recovered handler, then reread head.
pub fn run_destructors(
    context: &mut RuntimeContext,
    a5: u32,
    callbacks: &impl RuntimeCallbacks,
) -> Result<RuntimePlan, RuleError> {
    execute(context, &[], |context, execution| {
        let head_address = a5.wrapping_sub(0x1dcc);
        loop {
            let node = context.memory.long(head_address)?;
            if node == 0 {
                return Ok(RuntimeResult::Long(0));
            }
            execution.step(context.step_limit)?;
            let next = context.memory.long(node)?;
            execution.write(context, head_address, &next.to_be_bytes())?;
            let argument = context.memory.long(node.wrapping_add(8))?;
            let function = context.memory.long(node.wrapping_add(4))?;
            let callback = DestructorCallback {
                function,
                argument,
                reason: -1,
            };
            execution
                .effects
                .push(RuntimeEffect::CallbackEntered(callback.clone()));
            execution
                .effects
                .extend(callbacks.invoke(context, &callback)?);
            execution.effects.push(RuntimeEffect::CallbackReturned);
        }
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConditionCodes {
    pub extend: bool,
    pub negative: bool,
    pub zero: bool,
    pub overflow: bool,
    pub carry: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArithmeticResult {
    pub d0: u32,
    pub d1: u32,
    pub conditions: ConditionCodes,
}

fn conditions(value: u32, extend: bool) -> ConditionCodes {
    ConditionCodes {
        extend,
        negative: (value as i32) < 0,
        zero: value == 0,
        overflow: false,
        carry: false,
    }
}

/// CODE 1 0x1004d2 selects MULU.L at 0x1004de; low LONG product, D1 unchanged.
pub fn multiply_long(d0: u32, d1: u32, extend: bool) -> ArithmeticResult {
    let result = d0.wrapping_mul(d1);
    let mut flags = conditions(result, extend);
    flags.overflow = u64::from(d0) * u64::from(d1) > u64::from(u32::MAX);
    ArithmeticResult {
        d0: result,
        d1,
        conditions: flags,
    }
}

/// CODE 1 0x100504 selects DIVU.L at 0x100510; zero divisor is a CPU exception.
pub fn divide_unsigned_long(d0: u32, d1: u32, extend: bool) -> Result<ArithmeticResult, RuleError> {
    if d1 == 0 {
        return Err(RuleError::Invalid(
            "native unsigned LONG divide by zero would take a CPU exception".into(),
        ));
    }
    let quotient = d0 / d1;
    Ok(ArithmeticResult {
        d0: quotient,
        d1,
        conditions: conditions(quotient, extend),
    })
}

/// CODE 1 0x100584 selects DIVU.L at 0x100590, then moves remainder D1 into D0.
pub fn remainder_unsigned_long(
    d0: u32,
    d1: u32,
    extend: bool,
) -> Result<ArithmeticResult, RuleError> {
    if d1 == 0 {
        return Err(RuleError::Invalid(
            "native unsigned LONG remainder by zero would take a CPU exception".into(),
        ));
    }
    let remainder = d0 % d1;
    Ok(ArithmeticResult {
        d0: remainder,
        d1: remainder,
        conditions: conditions(remainder, extend),
    })
}

/// The arithmetic after Random at 0x153cb4, including raw WORD -32768.
pub fn rand_from_word(bound: i16, quickdraw_word: i16) -> (i16, u32) {
    let magnitude = i32::from(quickdraw_word).abs();
    let product = i32::from(bound).wrapping_mul(magnitude);
    let bias = ((product >> 14) as u32) >> 17;
    let shifted = product.wrapping_add(bias as i32) >> 15;
    let value = shifted.wrapping_add(1) as i16;
    let native_d0 = (product as u32 & 0xffff0000) | u32::from(value as u16);
    (value, native_d0)
}

/// CODE 1 0x153ca4: exactly one actual RandomState draw for every signed WORD bound.
pub fn rand(context: &mut RuntimeContext, bound: i16) -> Result<RuntimePlan, RuleError> {
    execute(context, &[], |context, execution| {
        if context.random.draws == u64::MAX {
            return Err(RuleError::Invalid(
                "runtime RNG draw counter is exhausted".into(),
            ));
        }
        let before = context.random.clone();
        let quickdraw_word = context.random.quickdraw();
        let (value, native_d0) = rand_from_word(bound, quickdraw_word);
        execution.effects.push(RuntimeEffect::Random {
            bound,
            quickdraw_word,
            result: value,
            before,
            after: context.random.clone(),
        });
        Ok(RuntimeResult::Word { value, native_d0 })
    })
}

/// CODE 1 0x13e13c: five pixel unlocks, MaxMem, then five pixmap/lock pairs.
pub fn compact_heap(
    context: &mut RuntimeContext,
    observations: &[RuntimeObservation],
) -> Result<RuntimePlan, RuleError> {
    execute(context, observations, |context, execution| {
        for address in [0x4386d8, 0x4386e0, 0x4386e4, 0x4386e8, 0x4386dc] {
            execution.ignored(RuntimeRequest::UnlockPixels(context.memory.long(address)?))?;
        }
        match execution.request(RuntimeRequest::MaxMem)? {
            RuntimeReply::MaxMem { d0, a0 } => {
                execution.effects.push(RuntimeEffect::LocalWrite {
                    offset: 4,
                    bytes: a0.to_be_bytes().to_vec(),
                });
                execution.effects.push(RuntimeEffect::LocalWrite {
                    offset: 0,
                    bytes: d0.to_be_bytes().to_vec(),
                });
            }
            _ => {
                return Err(RuleError::Invalid(
                    "missing MaxMem D0/A0 output registers".into(),
                ));
            }
        }
        let mut final_d0 = 0;
        for (world_address, pixmap_address) in [
            (0x4386b4, 0x4386d8),
            (0x4386bc, 0x4386e0),
            (0x4386c0, 0x4386e4),
            (0x4386c4, 0x4386e8),
            (0x4386b8, 0x4386dc),
        ] {
            let pixmap = match execution.request(RuntimeRequest::GetGWorldPixMap(
                context.memory.long(world_address)?,
            ))? {
                RuntimeReply::Pointer(pointer) => pointer,
                _ => return Err(RuleError::Invalid("missing GetGWorldPixMap handle".into())),
            };
            execution.write(context, pixmap_address, &pixmap.to_be_bytes())?;
            final_d0 = match execution.request(RuntimeRequest::LockPixels(pixmap))? {
                RuntimeReply::LockPixels { boolean, native_d0 } => {
                    (native_d0 & !0xff) | u32::from(boolean)
                }
                _ => {
                    return Err(RuleError::Invalid(
                        "missing LockPixels Boolean and post-trap D0".into(),
                    ));
                }
            };
        }
        Ok(RuntimeResult::Long(final_d0))
    })
}

/// CODE 1 0x13a498: restore the shifted source rectangle into the original target.
pub fn copy_in(
    context: &mut RuntimeContext,
    observations: &[RuntimeObservation],
) -> Result<RuntimePlan, RuleError> {
    execute(context, observations, |context, execution| {
        let source = context.memory.long(0x43879c)?;
        if source == 0 {
            return Ok(RuntimeResult::Long(0));
        }
        let mut local: [u8; 8] = context
            .memory
            .read(0x4471b4, 8)?
            .try_into()
            .expect("eight bytes");
        execution.effects.push(RuntimeEffect::LocalWrite {
            offset: 0,
            bytes: local[..4].to_vec(),
        });
        execution.effects.push(RuntimeEffect::LocalWrite {
            offset: 4,
            bytes: local[4..].to_vec(),
        });
        for offset in [6, 2] {
            let value = i16::from_be_bytes([local[offset], local[offset + 1]]).wrapping_add(320);
            local[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
            execution.effects.push(RuntimeEffect::LocalWrite {
                offset,
                bytes: value.to_be_bytes().to_vec(),
            });
        }
        execution.write(context, 0x4386a8, &[1])?;
        execution.ignored(RuntimeRequest::SetPort(context.memory.long(0x43878c)?))?;
        execution.ignored(RuntimeRequest::ForeColor(33))?;
        execution.ignored(RuntimeRequest::BackColor(30))?;
        let source_bitmap = context.memory.long(0x43879c)?.wrapping_add(2);
        let destination = context.memory.long(0x4386bc)?;
        let destination_rectangle = context
            .memory
            .read(0x4471b4, 8)?
            .try_into()
            .expect("eight bytes");
        execution.ignored(RuntimeRequest::CopyBits {
            source_bitmap,
            destination_bitmap: destination.wrapping_add(2),
            source_rectangle: local,
            destination_rectangle,
            mode: 0,
            mask_region: 0,
        })?;
        Ok(RuntimeResult::Long(0))
    })
}
