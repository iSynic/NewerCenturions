//! Startup, menu, file and field control lifted from complete CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{
    ApplicationControlEffect, ApplicationRegisters, Execution, RegisterBits, Result, Signal,
};
use crate::recovered_inventory_interaction::{ItemsCpu, ItemsRegisters};
use crate::recovered_load_control::LoadFlow;
use crate::recovered_native_services::{NativeCall, NativeRequest};
use crate::recovered_prompt_control::PromptReturn;

fn registers(r: &ItemsCpu) -> ApplicationRegisters {
    ApplicationRegisters {
        data: std::array::from_fn(|n| RegisterBits {
            value: r.data[n],
            known_mask: r.known_data_bits[n],
        }),
        address: std::array::from_fn(|n| {
            r.address[n].map_or(RegisterBits::default(), RegisterBits::full)
        }),
        ccr: r.ccr,
        ccr_known_mask: r.known_ccr,
    }
}

pub fn cpu(e: &Execution<'_>, frame: u32) -> ItemsCpu {
    ItemsCpu::new(
        &ItemsRegisters {
            data: e.registers.data.map(|value| value.value),
            known_data_bits: e.registers.data.map(|value| value.known_mask),
            address: e
                .registers
                .address
                .map(|value| (value.known_mask == u32::MAX).then_some(value.value)),
            condition_codes: e.registers.ccr,
            known_condition_codes: e.registers.ccr_known_mask,
        },
        frame,
    )
}

fn sync(e: &mut Execution<'_>, r: &ItemsCpu) {
    e.registers = registers(r);
    e.stack = r.address[7].unwrap_or(e.stack);
    let d0 = e.registers.data[0];
    e.plan.native_return = if d0.known_mask == u32::MAX {
        PromptReturn::Full(d0.value)
    } else if d0.known_mask & 0xffff == 0xffff {
        PromptReturn::LowWord(d0.value as u16)
    } else {
        PromptReturn::Unchanged
    };
}

pub fn child(e: &mut Execution<'_>, r: &mut ItemsCpu, entry: u32) -> Result<()> {
    sync(e, r);
    e.call(entry, r.address(7)?.wrapping_sub(8))?;
    for n in 0..8 {
        r.data[n] = e.registers.data[n].value;
        r.known_data_bits[n] = e.registers.data[n].known_mask;
        // Native RTD children pop arguments and expose their actual final A7.
        if n != 7 || e.resolver.owns_native_stack(entry) {
            let value = e.registers.address[n];
            r.address[n] = (value.known_mask == u32::MAX).then_some(value.value);
        }
    }
    r.ccr = e.registers.ccr;
    r.known_ccr = e.registers.ccr_known_mask;
    Ok(())
}

pub fn imported(e: &mut Execution<'_>, r: &mut ItemsCpu, call: NativeCall) -> Result<()> {
    if let NativeCall::Library(entry) = call
        && entry < 0x400000
    {
        return Err(RuleError::Invalid(
            "application gameplay cannot use the native import boundary".into(),
        )
        .into());
    }
    // The resolved CODE 3 abs_short leaf modifies only D0.W/D1.W and flags.
    if call == NativeCall::Library(0x400038) {
        let value = u32::from(e.w(r.address(7)?)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        let absolute = if (value as u16 as i16) < 0 {
            r.subtract_flags(0, value, 16, true)
        } else {
            value
        };
        r.put_data(0, 16, absolute);
        return Ok(());
    }
    let stack = match call {
        NativeCall::Library(_) => r.address(7)?.wrapping_sub(4),
        _ => r.address(7)?,
    };
    let mut input = registers(r);
    input.address[7] = RegisterBits::full(stack);
    let request = NativeRequest {
        call,
        stack,
        registers: input,
    };
    let observation = e.services.observe_native(&request, &e.plan.state.memory)?;
    r.after_call(None, PromptReturn::Unchanged);
    for n in 0..8 {
        if let Some(value) = observation.data[n] {
            r.data[n] = value.value;
            r.known_data_bits[n] = value.known_mask;
        }
        if let Some(value) = observation.address[n] {
            r.address[n] = (value.known_mask == u32::MAX).then_some(value.value);
        }
    }
    r.put_address(7, observation.stack_after);
    if let Some((ccr, mask)) = observation.ccr {
        r.ccr = ccr & 0x1f;
        r.known_ccr = mask & 0x1f;
    }
    e.plan.state.memory = observation.memory.clone();
    e.plan.flow = observation.flow;
    let flow = observation.flow;
    e.plan
        .effects
        .push(ApplicationControlEffect::NativeService {
            request,
            observation: Box::new(observation),
        });
    sync(e, r);
    if flow != LoadFlow::Returned {
        e.plan.native_return = PromptReturn::NoReturn;
        return Err(Signal::Stopped);
    }
    Ok(())
}

pub fn finish(e: &mut Execution<'_>, r: &mut ItemsCpu) -> Result<()> {
    r.offset_address(7, 4)?;
    sync(e, r);
    Ok(())
}

pub fn owns_frame(entry: u32) -> bool {
    matches!(
        entry,
        0x11177e
            | 0x12f0cc
            | 0x13164e
            | 0x132780
            | 0x13283e
            | 0x132886
            | 0x134e82
            | 0x135354
            | 0x135fc8
            | 0x1366f8
            | 0x13bd52
            | 0x14913e
            | 0x154016
            | 0x1540c4
            | 0x15d2f0
    )
}

pub fn divide_long(dividend: u32, divisor: u32) -> std::result::Result<(u32, u32), RuleError> {
    let dividend = i64::from(dividend as i32);
    let divisor = i64::from(divisor as i32);
    if divisor == 0 {
        return Err(RuleError::Unsupported {
            action: "system_control.divide_long",
            dependency: "the original long-division exception handler",
        });
    }
    let quotient = dividend / divisor;
    if !(-2147483648..=2147483647).contains(&quotient) {
        return Err(RuleError::Unsupported {
            action: "system_control.divide_long",
            dependency: "the original long-division overflow behavior",
        });
    }
    Ok((quotient as u32, (dividend % divisor) as u32))
}
