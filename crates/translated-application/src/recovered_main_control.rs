//! Main application controllers recovered from CODE 1.

use crate::recovered_application_control::{self as application, Execution};
use crate::recovered_feedback::FeedbackArg;
use FeedbackArg::{Byte as B, Long as L, Pointer as P, Word as W};

fn push(arg: &FeedbackArg, e: &mut Execution<'_>) -> application::Result<()> {
    match *arg {
        W(v) => {
            e.stack = e.stack.wrapping_sub(2);
            e.word(e.stack, v)?;
        }
        B(v) => {
            e.stack = e.stack.wrapping_sub(2);
            e.byte(e.stack, v)?;
        }
        L(v) | P(v) => {
            e.stack = e.stack.wrapping_sub(4);
            e.long(e.stack, v)?;
        }
    }
    Ok(())
}

fn pascal(
    opcode: u16,
    args: Vec<FeedbackArg>,
    returned: u32,
    e: &mut Execution<'_>,
) -> application::Result<u32> {
    let top = e.stack;
    e.stack = top.wrapping_sub(if returned == 1 { 2 } else { returned });
    let slot = e.stack;
    for arg in &args {
        push(arg, e)?;
    }
    let value = e.trap(opcode, args)?;
    if returned == 1 {
        e.byte(slot, value as u8)?;
        e.full((e.d0_long()? & 0xffffff00) | (value & 0xff));
    }
    if returned == 2 {
        e.word(slot, value as u16)?;
        e.low(value as u16);
    }
    if returned == 4 {
        e.long(slot, value)?;
        if matches!(opcode, 0xa858 | 0xa93d | 0xa93e) {
            e.full(value);
        } else {
            e.registers.address[0] = application::RegisterBits::full(value);
        }
    }
    e.stack = top;
    Ok(value)
}

fn pascal_existing(
    opcode: u16,
    args: Vec<FeedbackArg>,
    returned: u32,
    e: &mut Execution<'_>,
) -> application::Result<u32> {
    let slot = e.stack;
    for arg in &args {
        push(arg, e)?;
    }
    let value = e.trap(opcode, args)?;
    match returned {
        1 => {
            e.byte(slot, value as u8)?;
            e.full((e.d0_long()? & 0xffffff00) | (value & 0xff));
        }
        2 => {
            e.word(slot, value as u16)?;
            e.low(value as u16);
        }
        4 => {
            e.long(slot, value)?;
            e.registers.address[0] = application::RegisterBits::full(value);
        }
        _ => unreachable!(),
    }
    e.stack = slot.wrapping_add(if returned == 1 { 2 } else { returned });
    Ok(value)
}

fn child(frame: u32, entry: u32, return_pc: u32, e: &mut Execution<'_>) -> application::Result<()> {
    let sp = e.stack;
    e.long(sp.wrapping_sub(4), return_pc)?;
    e.long(sp.wrapping_sub(8), frame)?;
    e.call(entry, sp.wrapping_sub(8))?;
    e.stack = sp;
    Ok(())
}

fn with_args(
    frame: u32,
    entry: u32,
    return_pc: u32,
    args: &[FeedbackArg],
    cleanup: u32,
    e: &mut Execution<'_>,
) -> application::Result<()> {
    for arg in args {
        push(arg, e)?;
    }
    child(frame, entry, return_pc, e)?;
    e.stack = e.stack.wrapping_add(cleanup);
    Ok(())
}

fn flush(e: &mut Execution<'_>) -> application::Result<()> {
    push(&L(0xffff), e)?;
    e.full(e.l(e.stack)?);
    e.stack = e.stack.wrapping_add(4);
    e.register_trap(0xa032, Some(0xffff), None, vec![])?;
    Ok(())
}

pub fn update_main_body(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    e.stack = frame.wrapping_sub(8);
    pascal(0xa93c, vec![L(e.l(0x4427ba)?)], 0, e)?;
    pascal(0xa935, vec![L(e.l(0x43871a)?), W(0xffff)], 0, e)?;
    pascal(0xa935, vec![L(e.l(0x438716)?), W(0xffff)], 0, e)?;
    pascal(0xa937, vec![], 0, e)?;
    pascal(0xa873, vec![L(e.l(0x43878c)?)], 0, e)?;
    pascal(0xa887, vec![W(e.w(0x402f3c)?)], 0, e)?;
    if e.w(frame.wrapping_add(10))? == 0 || e.b(0x4427b3)? != 0 {
        with_args(frame, 0x13a1dc, 0x13bb6c, &[W(3)], 2, e)?;
    } else {
        child(frame, 0x13a510, 0x13bb74, e)?;
        child(frame, 0x13a234, 0x13bb78, e)?;
    }
    if e.w(0x438776)? == 0 {
        if e.w(frame.wrapping_add(8))? == 0 {
            with_args(frame, 0x10f186, 0x13bb90, &[W(0)], 2, e)?;
        }
        with_args(frame, 0x160a8e, 0x13bba0, &[L(0), L(0x10000)], 0, e)?;
        let top = e
            .w(0x40856e)?
            .wrapping_add(0x141)
            .wrapping_sub(e.w(0x439582)?.wrapping_shl(5));
        e.word(frame.wrapping_sub(8), top)?;
        e.word(frame.wrapping_sub(6), 0)?;
        let bottom = e
            .w(0x40856e)?
            .wrapping_add(0x1cc)
            .wrapping_sub(e.w(0x439582)?.wrapping_shl(5));
        e.word(frame.wrapping_sub(4), bottom)?;
        e.word(frame.wrapping_sub(2), e.w(0x40856c)?.wrapping_add(0x134))?;
        pascal(
            0xa8f6,
            vec![L(e.l(0x439560)?), P(frame.wrapping_sub(8))],
            0,
            e,
        )?;
        e.word(frame.wrapping_sub(6), e.w(0x40856c)?.wrapping_add(0x134))?;
        e.word(frame.wrapping_sub(2), e.w(0x40856c)?.wrapping_add(0x280))?;
        pascal(
            0xa8f6,
            vec![L(e.l(0x439564)?), P(frame.wrapping_sub(8))],
            0,
            e,
        )?;
        e.stack = e.stack.wrapping_add(8);
        if e.w(0x439582)? != 0 {
            e.word(0x4434bc, 0)?;
            e.word(0x4434c0, 0x1e0)?;
            e.word(0x4434ba, 0x1a0)?;
            e.word(0x4434be, 0x1ab)?;
            with_args(
                frame,
                0x1382a8,
                0x13bc5a,
                &[L(e.l(0x4434be)?), L(e.l(0x4434ba)?), W(0xce)],
                0,
                e,
            )?;
            e.word(0x4434bc, 0)?;
            e.word(0x4434c0, 0x1e0)?;
            e.word(0x4434ba, 0x239)?;
            e.word(0x4434be, 0x244)?;
            with_args(
                frame,
                0x1382a8,
                0x13bc8c,
                &[L(e.l(0x4434be)?), L(e.l(0x4434ba)?), W(0xce)],
                0,
                e,
            )?;
            e.addw(0x4434bc, 0x1e0)?;
            e.addw(0x4434c0, 0x1e0)?;
            with_args(
                frame,
                0x1382a8,
                0x13bcb0,
                &[L(e.l(0x4434be)?), L(e.l(0x4434ba)?), W(0xce)],
                0,
                e,
            )?;
            e.word(0x4434bc, 0x1e0)?;
            e.word(0x4434c0, 0x320)?;
            e.word(0x4434ba, 0x12c)?;
            e.word(0x4434be, 0x198)?;
            with_args(
                frame,
                0x1382a8,
                0x13bcea,
                &[L(e.l(0x4434be)?), L(e.l(0x4434ba)?), W(0xe0)],
                40,
                e,
            )?;
        }
        child(frame, 0x15ec56, 0x13bcf4, e)?;
        with_args(
            frame,
            0x16043e,
            0x13bd04,
            &[W(e.sb(0x44273f)? as i16 as u16)],
            0,
            e,
        )?;
        with_args(frame, 0x15e148, 0x13bd0c, &[L(0)], 0, e)?;
        let reload = e.b(0x4386b2)? != 0;
        e.stack = e.stack.wrapping_add(6);
        if reload {
            e.moveq_return(0);
            return Ok(());
        }
    } else {
        with_args(frame, 0x160a8e, 0x13bd28, &[L(0x10000), L(0)], 8, e)?;
    }
    pascal(0xa873, vec![L(e.l(0x43878c)?)], 0, e)?;
    with_args(frame, 0x12382a, 0x13bd3e, &[L(0x5e0066)], 0, e)?;
    e.moveq_return(0);
    Ok(())
}

pub fn main_screen_body(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    e.stack = frame.wrapping_sub(16);
    e.registers.data[3] = application::RegisterBits::full(0);
    pascal(0xa93c, vec![L(e.l(0x4427ba)?)], 0, e)?;
    pascal(0xa935, vec![L(e.l(0x43871a)?), W(0xffff)], 0, e)?;
    pascal(0xa935, vec![L(e.l(0x438716)?), W(0xffff)], 0, e)?;
    pascal(0xa937, vec![], 0, e)?;
    let mut reload = true;
    loop {
        e.step()?;
        if reload {
            with_args(
                frame,
                0x13bb20,
                0x13b65c,
                &[W(e.w(frame.wrapping_add(8))?), W(0)],
                0,
                e,
            )?;
            flush(e)?;
            e.stack = e.stack.wrapping_add(4);
            reload = false;
        }
        if e.b(0x4427b2)? == 0 {
            pascal(0xa972, vec![P(0x4471ee)], 0, e)?;
            let inside = pascal(0xa8ad, vec![L(e.l(0x4471ee)?), P(0x4471b4)], 1, e)? as u8;
            if inside == 0 {
                pascal(0xaa1c, vec![L(e.l(0x43952c)?)], 0, e)?;
                e.word(0x4471a4, 0x14)?;
            } else {
                with_args(frame, 0x15f210, 0x13b6a8, &[W(1)], 2, e)?;
            }
        }
        e.word(0x443316, 0)?;
        if pascal(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(0x400)], 4, e)? != 0 {
            with_args(frame, 0x16474a, 0x13b6d0, &[W(0x15)], 2, e)?;
        }
        child(frame, 0x15ec56, 0x13b6d8, e)?;
        pascal(0xa9b4, vec![], 0, e)?;
        with_args(frame, 0x133d46, 0x13b6e2, &[W(0)], 0, e)?;
        let event = pascal_existing(0xa970, vec![W(0xffff), P(0x44720a)], 1, e)? as u8;
        e.registers.data[4].assign_word(u16::from(event));
        let dialog_event = pascal(0xa97f, vec![P(0x44720a)], 1, e)? as u8;
        let mut keyboard = false;
        let mut control = false;
        if dialog_event != 0 {
            let handled = pascal(0xa980, vec![P(0x44720a), P(0x4394c0), P(0x4434b2)], 1, e)? as u8;
            e.registers.data[4].assign_word(u16::from(handled));
            if e.w(0x44720a)? != 3 {
                continue;
            }
            keyboard = true;
        } else {
            match e.w(0x44720a)? {
                1 => {
                    let part = pascal(
                        0xa92c,
                        vec![L(e.l(0x447214)?), P(frame.wrapping_sub(4))],
                        2,
                        e,
                    )? as u16;
                    e.word(0x4471a6, part)?;
                    if part == 1 {
                        child(frame, 0x13e13c, 0x13b92c, e)?;
                        let choice = pascal(0xa93d, vec![L(e.l(0x447214)?)], 4, e)?;
                        e.long(0x438742, choice)?;
                        child(frame, 0x1279bc, 0x13b944, e)?;
                        e.long(0x439404, 0)?;
                        if e.b(0x4386b2)? != 0 {
                            e.byte(0x4386b2, 0)?;
                            reload = true;
                        }
                        continue;
                    }
                    if part != 3 {
                        continue;
                    }
                    e.long(0x4471ee, e.l(0x447214)?)?;
                    pascal(0xa871, vec![P(0x4471ee)], 0, e)?;
                    let selected = pascal(
                        0xa96c,
                        vec![L(e.l(0x4471ee)?), L(e.l(0x43878c)?), P(0x439404)],
                        2,
                        e,
                    )?;
                    e.word(0x4471a6, selected as u16)?;
                    flush(e)?;
                    control = true;
                }
                3 | 5 => keyboard = true,
                6 => {
                    for address in [0x43879c, 0x4386ae, 0x43878c] {
                        pascal(0xa922, vec![L(e.l(address)?)], 0, e)?;
                        pascal(0xa923, vec![L(e.l(address)?)], 0, e)?;
                    }
                    if e.l(0x438794)? != 0 {
                        pascal(0xa922, vec![L(e.l(0x438794)?)], 0, e)?;
                        pascal(0xa923, vec![L(e.l(0x438794)?)], 0, e)?;
                    }
                }
                7 => {
                    if e.w(0x44720c)? != 0 {
                        pascal(0xa880, vec![P(0x4471ee), L(0x28001e)], 0, e)?;
                        pascal(
                            0xa9e9,
                            vec![L(e.l(0x4471ee)?), L(e.l(0x44720c)?), W(0)],
                            2,
                            e,
                        )?;
                    }
                }
                15 if e.b(0x44720c)? == 1 => {
                    if e.b(0x4427b2)? == 0 {
                        e.byte(0x4427b2, 1)?;
                    } else {
                        child(frame, 0x13e13c, 0x13ba2c, e)?;
                        e.byte(0x4427b2, 0)?;
                        with_args(frame, 0x13bb20, 0x13ba3a, &[L(0)], 0, e)?;
                        flush(e)?;
                        e.stack = e.stack.wrapping_add(4);
                    }
                }
                _ => {}
            }
        }
        if keyboard {
            let command = pascal(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(0x100)], 4, e)? != 0;
            e.word(0x443316, e.w(0x44720e)?)?;
            if command {
                with_args(frame, 0x111984, 0x13b818, &[W(1)], 2, e)?;
                if e.b(0x437a6a)? == 0 {
                    let key = pascal(0xa858, vec![L(e.l(0x44720c)?), L(0xff)], 4, e)? as u8;
                    e.registers.data[5].assign_byte(key);
                    let choice = pascal(0xa93e, vec![W(key as i8 as i16 as u16)], 4, e)?;
                    e.long(0x438742, choice)?;
                    child(frame, 0x1279bc, 0x13b84c, e)?;
                    e.long(0x439404, 0)?;
                    if e.b(0x4386b2)? != 0 {
                        e.byte(0x4386b2, 0)?;
                        reload = true;
                        continue;
                    }
                }
            } else {
                e.long(0x439404, 0)?;
                with_args(frame, 0x111984, 0x13b87c, &[W(0)], 0, e)?;
                with_args(frame, 0x111984, 0x13b886, &[W(1)], 4, e)?;
            }
            control = true;
        }
        if !control {
            continue;
        }
        let mut choice = 0i8;
        e.registers.data[3] = application::RegisterBits::full(0);
        loop {
            e.step()?;
            with_args(
                frame,
                0x10950e,
                if keyboard { 0x13b896 } else { 0x13b9a6 },
                &[W(choice as i16 as u16)],
                2,
                e,
            )?;
            choice = e.d0_word()? as u8 as i8;
            e.registers.data[3].assign_byte(choice as u8);
            if e.b(0x4386b2)? != 0 {
                e.byte(0x4386b2, 0)?;
                reload = true;
                break;
            }
            match choice {
                -3 => {
                    with_args(
                        frame,
                        0x13bb20,
                        if keyboard { 0x13b8f8 } else { 0x13ba08 },
                        &[L(0xffff)],
                        4,
                        e,
                    )?;
                    break;
                }
                -2 => {
                    e.long(
                        0x439404,
                        e.l(if e.b(0x442753)? == 0 {
                            0x439444
                        } else {
                            0x439430
                        })?,
                    )?;
                }
                -1 => {
                    e.long(0x439404, e.l(0x43945c)?)?;
                }
                _ => break,
            }
        }
    }
}

fn init_control(
    resource: u16,
    global: u32,
    horizontal_offset: bool,
    vertical_offset: bool,
    e: &mut Execution<'_>,
) -> application::Result<()> {
    let handle = pascal(0xa9be, vec![W(resource), L(e.l(0x43878c)?)], 4, e)?;
    e.registers.address[0] = application::RegisterBits::full(handle);
    e.long(global, handle)?;
    // Each coordinate dereferences the control again, in native read order.
    let control = e.l(e.l(global)?)?;
    let mut h = e.w(control.wrapping_add(10))?;
    if horizontal_offset {
        h = h.wrapping_add(e.w(0x40856c)?);
    }
    let control = e.l(e.l(global)?)?;
    let mut v = e.w(control.wrapping_add(8))?;
    if vertical_offset {
        v = v
            .wrapping_add(e.w(0x40856e)?)
            .wrapping_sub(e.w(0x439582)?.wrapping_shl(5));
    }
    pascal(0xa959, vec![L(e.l(global)?), W(h), W(v)], 0, e)?;
    Ok(())
}

pub fn main_screen_init_body(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    e.stack = frame;
    pascal(0xa939, vec![L(e.l(0x438722)?), W(0)], 0, e)?;
    if e.w(frame.wrapping_add(8))? == 0 {
        let window = pascal(0xaa46, vec![W(0x80), L(0), L(0)], 4, e)?;
        e.registers.address[0] = application::RegisterBits::full(window);
        e.long(0x43878c, window)?;
        let h = e.w(0x40856c)?.wrapping_add(0x280);
        let v = e
            .w(0x40856e)?
            .wrapping_add(0x1cc)
            .wrapping_sub(e.w(0x439582)?.wrapping_mul(0x14));
        pascal(0xa91d, vec![L(e.l(0x43878c)?), W(h), W(v), B(0)], 0, e)?;
        pascal(
            0xa91b,
            vec![
                L(e.l(0x43878c)?),
                W(e.w(0x4395a4)?),
                W(e.w(0x4395a2)?),
                B(0),
            ],
            0,
            e,
        )?;
        pascal(0xa873, vec![L(e.l(0x43878c)?)], 0, e)?;
        pascal(0xaa0b, vec![L(e.l(0x4358ce)?)], 0, e)?;
        pascal(0xa889, vec![W(1)], 0, e)?;
        pascal(0xa88a, vec![W(0x12)], 0, e)?;
        pascal(0xa887, vec![W(e.w(0x402f3c)?)], 0, e)?;
        for (resource, global) in [
            (300, 0x439470),
            (301, 0x439450),
            (302, 0x439474),
            (303, 0x43947c),
            (304, 0x439430),
            (305, 0x439444),
            (306, 0x439438),
            (307, 0x43944c),
            (308, 0x439434),
            (309, 0x439478),
            (142, 0x439448),
            (155, 0x43945c),
        ] {
            e.step()?;
            init_control(resource, global, true, true, e)?;
        }
        let background = pascal(
            0xa9be,
            vec![W(e.w(0x439582)?.wrapping_add(1000)), L(e.l(0x43878c)?)],
            4,
            e,
        )?;
        e.long(0x439454, background)?;
        init_control(154, 0x439458, true, true, e)?;
        init_control(500, 0x439440, true, false, e)?;
        init_control(167, 0x439490, false, true, e)?;
        init_control(149, 0x43942c, false, true, e)?;
        init_control(175, 0x439480, true, true, e)?;
        init_control(147, 0x43943c, true, true, e)?;
        init_control(168, 0x439494, true, true, e)?;
        init_control(169, 0x439498, true, true, e)?;
        init_control(177, 0x4393f8, false, true, e)?;
        init_control(166, 0x439488, false, true, e)?;
        init_control(164, 0x439484, false, true, e)?;
        init_control(165, 0x43948c, false, true, e)?;
        init_control(197, 0x4393f4, false, true, e)?;
        e.word(0x4434aa, 0)?;
        while e.sw(0x4434aa)? < 6 {
            e.step()?;
            let resource = e.w(0x4434aa)?.wrapping_add(190);
            let handle = pascal(0xa9be, vec![W(resource), L(e.l(0x43878c)?)], 4, e)?;
            let global = 0x43949cu32.wrapping_add((e.sw(0x4434aa)? as i32 as u32).wrapping_mul(4));
            e.long(global, handle)?;
            let h = e
                .w(0x40856c)?
                .wrapping_add(e.w(e.l(e.l(global)?)?.wrapping_add(10))?);
            let v = e.w(e.l(e.l(global)?)?.wrapping_add(8))?;
            pascal(0xa959, vec![L(e.l(global)?), W(h), W(v)], 0, e)?;
            e.addw(0x4434aa, 1)?;
        }
        for address in [0x4386ae, 0x43879c] {
            let window = pascal(0xaa46, vec![W(0x83), L(0), L(0)], 4, e)?;
            e.long(address, window)?;
            pascal(
                0xa91b,
                vec![L(e.l(address)?), W(e.w(0x4395a4)?), W(e.w(0x4395a2)?), B(0)],
                0,
                e,
            )?;
        }
        if e.w(0x439582)? != 0 {
            let top = e.stack;
            let window = e.l(0x43879c)?;
            push(&L(window), e)?;
            push(&L(0x1a001e0), e)?;
            push(&B(0), e)?;
            e.trap(0xa91d, vec![L(window), W(480), W(416), B(0)])?;
            e.stack = top;
        }
        for address in [0x43878c, 0x4386ae, 0x43879c] {
            pascal(0xa915, vec![L(e.l(address)?)], 0, e)?;
        }
        for address in [0x43878c, 0x43879c, 0x4386ae] {
            pascal(0xa920, vec![L(e.l(address)?)], 0, e)?;
        }
    }
    pascal(0xa873, vec![L(e.l(0x43879c)?)], 0, e)?;
    pascal(0xaa0b, vec![L(e.l(0x4358ce)?)], 0, e)?;
    pascal(0xa862, vec![L(0x21)], 0, e)?;
    pascal(0xa863, vec![L(0x1e)], 0, e)?;
    pascal(0xa89c, vec![W(2)], 0, e)?;
    let top = e.stack;
    push(&L(0x20002), e)?;
    e.trap(0xa89b, vec![W(2), W(2)])?;
    e.stack = top;
    pascal(0xa88a, vec![W(12)], 0, e)?;
    if e.w(frame.wrapping_add(8))? == 0 {
        pascal(0xa983, vec![L(e.l(0x479fd8)?)], 0, e)?;
        e.long(0x479fd8, 0)?;
    }
    pascal(0xa937, vec![], 0, e)?;
    with_args(frame, 0x13b624, 0x149df6, &[W(0)], 0, e)?;
    e.moveq_return(0);
    Ok(())
}

const PREFERENCE_CONTROLS: [(u16, u32); 14] = [
    (9, 0x439574),
    (10, 0x439588),
    (11, 0x439586),
    (12, 0x43957c),
    (13, 0x439590),
    (14, 0x43958e),
    (15, 0x43957e),
    (16, 0x43958c),
    (17, 0x439584),
    (18, 0x439592),
    (19, 0x4435ce),
    (20, 0x43958a),
    (21, 0x439580),
    (22, 0x439582),
];

fn preference_item(
    frame: u32,
    window: u32,
    item: u16,
    rect: u32,
    e: &mut Execution<'_>,
) -> application::Result<()> {
    pascal(
        0xa98d,
        vec![
            L(window),
            W(item),
            P(frame.wrapping_sub(6)),
            P(frame.wrapping_sub(4)),
            P(rect),
        ],
        0,
        e,
    )?;
    Ok(())
}

pub fn preference_body(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    e.stack = frame.wrapping_sub(34);
    e.word(frame.wrapping_sub(26), e.sb(0x438589)? as i16 as u16)?;
    if e.l(0x479fd8)? != 0 {
        let front = pascal(0xa924, vec![], 4, e)?;
        let selected = front == e.l(0x43879c)? || pascal(0xa924, vec![], 4, e)? == e.l(0x4386ae)?;
        if selected && e.l(0x43879c)? != 0 {
            child(frame, 0x13a498, 0x14b680, e)?;
        }
    }
    let window = pascal(0xa97c, vec![W(0xa7), L(0), L(u32::MAX)], 4, e)?;
    e.registers.address[2] = application::RegisterBits::full(window);
    let resolution = e.b(0x439583)?;
    e.registers.data[3].assign_byte(resolution);
    loop {
        e.step()?;
        pascal(0xa873, vec![L(window)], 0, e)?;
        pascal(0xa862, vec![L(0x45)], 0, e)?;
        pascal(0xaa0b, vec![L(e.l(0x4358ce)?)], 0, e)?;
        pascal(0xa887, vec![W(e.w(0x402f44)?)], 0, e)?;
        for opcode in [0xa915, 0xa981, 0xa922, 0xa923] {
            pascal(opcode, vec![L(window)], 0, e)?;
        }
        e.long(0x4394b8, window)?;
        for (item, address) in PREFERENCE_CONTROLS {
            e.step()?;
            preference_item(frame, window, item, frame.wrapping_sub(16), e)?;
            pascal(
                0xa963,
                vec![L(e.l(frame.wrapping_sub(4))?), W(e.w(address)?)],
                0,
                e,
            )?;
        }
        preference_item(frame, window, 4, frame.wrapping_sub(16), e)?;
        with_args(
            frame,
            0x14a02c,
            0x14b8b6,
            &[
                L(e.l(frame.wrapping_sub(12))?),
                L(e.l(frame.wrapping_sub(16))?),
                W(e.w(0x43957a)?.wrapping_add(0x2429)),
            ],
            10,
            e,
        )?;
        loop {
            e.step()?;
            pascal(0xa873, vec![L(window)], 0, e)?;
            pascal(0xa862, vec![L(0x45)], 0, e)?;
            pascal(0xa887, vec![W(e.w(0x402f44)?)], 0, e)?;
            e.w(0x439580)?;
            e.word(0x402f44, 0x7788)?;
            for (item, address) in PREFERENCE_CONTROLS {
                e.step()?;
                preference_item(frame, window, item, frame.wrapping_sub(16), e)?;
                let value = pascal(0xa960, vec![L(e.l(frame.wrapping_sub(4))?)], 2, e)?;
                e.word(address, value as u16)?;
            }
            flush(e)?;
            pascal(0xa991, vec![L(0), P(frame.wrapping_sub(8))], 0, e)?;
            preference_item(
                frame,
                window,
                e.w(frame.wrapping_sub(8))?,
                frame.wrapping_sub(16),
                e,
            )?;
            if e.sw(frame.wrapping_sub(8))? < 3 {
                preference_item(frame, window, 2, frame.wrapping_sub(24), e)?;
                with_args(
                    frame,
                    0x14ce2a,
                    0x14bb36,
                    &[
                        L(e.l(frame.wrapping_sub(20))?),
                        L(e.l(frame.wrapping_sub(24))?),
                        W(0),
                        L(0x10001),
                    ],
                    0,
                    e,
                )?;
                with_args(frame, 0x133a18, 0x14bb40, &[W(0x8d)], 0, e)?;
                pascal(0xa983, vec![L(window)], 0, e)?;
                child(frame, 0x14c01c, 0x14bb4a, e)?;
                let front = pascal_existing(0xa924, vec![], 4, e)?;
                e.stack = e.stack.wrapping_add(12);
                if front == e.l(0x479fd8)? {
                    pascal(0xa981, vec![L(e.l(0x479fd8)?)], 0, e)?;
                } else {
                    with_args(frame, 0x13bb20, 0x14bb72, &[L(0xffff)], 4, e)?;
                }
                if e.w(0x43958a)? == 0 {
                    if e.l(0x438794)? != 0 {
                        pascal(0xa914, vec![L(e.l(0x438794)?)], 0, e)?;
                    }
                    e.long(0x438794, 0)?;
                } else if e.l(0x438794)? == 0 {
                    let window = pascal(0xaa46, vec![W(0x84), L(0), L(e.l(0x43878c)?)], 4, e)?;
                    e.long(0x438794, window)?;
                    pascal(
                        0xa91d,
                        vec![
                            L(e.l(0x438794)?),
                            W(e.w(0x43959e)?),
                            W(e.w(0x4395a0)?),
                            B(0),
                        ],
                        0,
                        e,
                    )?;
                }
                if u16::from(resolution) != e.w(0x439582)? {
                    with_args(
                        frame,
                        0x123d1c,
                        0x14bc82,
                        &[L(0xf01770), L(0x1e012c), P(0x4019b8)],
                        0,
                        e,
                    )?;
                    child(frame, 0x13a87c, 0x14bc88, e)?;
                    e.stack = e.stack.wrapping_add(12);
                }
                e.moveq_return(0);
                return Ok(());
            }
            if e.w(frame.wrapping_sub(6))? == 5 {
                let value = pascal(0xa960, vec![L(e.l(frame.wrapping_sub(4))?)], 2, e)?;
                pascal(
                    0xa963,
                    vec![L(e.l(frame.wrapping_sub(4))?), W(u16::from(value == 0))],
                    0,
                    e,
                )?;
                let args = vec![
                    P(0x479ed8),
                    W(0x1771),
                    W(e.w(frame.wrapping_sub(8))?.wrapping_sub(8)),
                ];
                for arg in &args {
                    push(arg, e)?;
                }
                e.lib(0x400178, args)?;
                e.stack = e.stack.wrapping_add(8);
                with_args(frame, 0x153daa, 0x14bbc8, &[P(0x479ed8), W(8)], 6, e)?;
            }
            if e.w(frame.wrapping_sub(8))? == 3 {
                break;
            }
        }
        with_args(frame, 0x133a18, 0x14bbde, &[W(0x8d)], 0, e)?;
        pascal(
            0xa98b,
            vec![P(0x401a02), P(0x401a00), P(0x401a00), P(0x401a00)],
            0,
            e,
        )?;
        with_args(frame, 0x1289fa, 0x14bc02, &[W(2)], 4, e)?;
    }
}

fn menu_enable(entries: &[(u32, u16, bool)], e: &mut Execution<'_>) -> application::Result<()> {
    for &(address, item, enable) in entries {
        e.step()?;
        pascal(
            if enable { 0xa939 } else { 0xa93a },
            vec![L(e.l(address)?), W(item)],
            0,
            e,
        )?;
    }
    Ok(())
}

fn message(
    frame: u32,
    text: u32,
    return_pc: u32,
    e: &mut Execution<'_>,
) -> application::Result<()> {
    with_args(
        frame,
        0x123d1c,
        return_pc,
        &[L(5000), L(0x320032), P(text)],
        0,
        e,
    )
}

fn question(
    frame: u32,
    args: &[FeedbackArg],
    return_pc: u32,
    cleanup: u32,
    e: &mut Execution<'_>,
) -> application::Result<bool> {
    with_args(frame, 0x14cff6, return_pc, args, cleanup, e)?;
    Ok(e.d0_word()? != 0)
}

fn tutorial_clear(e: &mut Execution<'_>) -> application::Result<()> {
    e.word(0x4434aa, 0)?;
    while e.sw(0x4434aa)? < 1000 {
        e.step()?;
        e.byte(0x4042f2u32.wrapping_add(e.sw(0x4434aa)? as i32 as u32), 0)?;
        e.addw(0x4434aa, 1)?;
    }
    Ok(())
}

fn library_call(
    entry: u32,
    return_pc: u32,
    args: Vec<FeedbackArg>,
    e: &mut Execution<'_>,
) -> application::Result<u32> {
    for arg in &args {
        push(arg, e)?;
    }
    e.long(e.stack.wrapping_sub(4), return_pc)?;
    let result = e.lib(entry, args)?;
    e.registers.address[0] = application::RegisterBits::full(result.value);
    Ok(result.value)
}

fn tutorial_write(
    open_pc: u32,
    first_pc: u32,
    second_pc: u32,
    close_pc: u32,
    e: &mut Execution<'_>,
) -> application::Result<()> {
    let file = library_call(0x400070, open_pc, vec![P(0x403500), P(0x403504)], e)?;
    e.registers.address[2] = application::RegisterBits::full(file);
    e.full(file);
    e.stack = e.stack.wrapping_add(8);
    if file != 0 {
        library_call(
            0x400050,
            first_pc,
            vec![L(file), L(1), L(1000), P(0x4042f2)],
            e,
        )?;
        library_call(
            0x400050,
            second_pc,
            vec![L(file), L(1), L(16000), P(0x4046ec)],
            e,
        )?;
        library_call(0x400068, close_pc, vec![L(file)], e)?;
        e.stack = e.stack.wrapping_add(36);
    }
    Ok(())
}

fn menu_combat_refresh(
    frame: u32,
    center_pc: u32,
    update_pc: u32,
    controls_pc: u32,
    e: &mut Execution<'_>,
) -> application::Result<()> {
    with_args(frame, 0x110644, center_pc, &[W(0)], 0, e)?;
    with_args(frame, 0x11f1a8, update_pc, &[W(e.w(0x443318)?)], 0, e)?;
    child(frame, 0x16043e, controls_pc, e)?;
    e.stack = e.stack.wrapping_add(4);
    Ok(())
}

pub fn handle_menu_choice_body(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    e.stack = frame.wrapping_sub(10);
    e.registers.address[2] = application::RegisterBits::full(0);
    e.word(0x438746, (e.sl(0x438742)? >> 16) as u16)?;
    e.word(0x438748, e.w(0x438744)?)?;
    pascal(0xa938, vec![W(0)], 0, e)?;
    match e.w(0x438746)? {
        0x80 => {
            child(frame, 0x13a498, 0x127e6c, e)?;
            match e.w(0x438748)? {
                1 => child(frame, 0x1540c4, 0x127e82, e)?,
                2 => child(frame, 0x146b6a, 0x127e8a, e)?,
                _ => {
                    pascal(
                        0xa946,
                        vec![L(e.l(0x438706)?), W(e.w(0x438748)?), P(0x479ed8)],
                        0,
                        e,
                    )?;
                    let accessory = pascal(0xa9b6, vec![P(0x479ed8)], 2, e)?;
                    e.word(frame.wrapping_sub(2), accessory as u16)?;
                }
            }
            if e.w(0x438776)? != 0 {
                with_args(frame, 0x13bb20, 0x127ec4, &[L(0x10000)], 0, e)?;
                with_args(frame, 0x11f1a8, 0x127ed0, &[W(e.w(0x4435d2)?)], 0, e)?;
                let index = (e.sw(0x4435d2)? as i32 as u32).wrapping_mul(4);
                let y = e.w(0x44709cu32.wrapping_add(index))?;
                let x = e.w(0x44709au32.wrapping_add(index))?;
                with_args(frame, 0x10e6b6, 0x127ef6, &[W(y), W(x)], 0, e)?;
                child(frame, 0x16043e, 0x127efc, e)?;
                let draw_spell = e.w(0x438760)? != 0;
                e.stack = e.stack.wrapping_add(10);
                if draw_spell {
                    child(frame, 0x1377f4, 0x127f0e, e)?;
                }
                flush(e)?;
            } else {
                let front = pascal(0xa924, vec![], 4, e)?;
                if front == e.l(0x43879c)? {
                    with_args(frame, 0x13bb20, 0x127f32, &[L(0)], 4, e)?;
                } else if pascal(0xa924, vec![], 4, e)? == e.l(0x479fd8)? {
                    pascal(0xa981, vec![L(e.l(0x479fd8)?)], 0, e)?;
                }
            }
        }
        0x81 => {
            let item = e.w(0x438748)?;
            match item {
                1 | 2 => {
                    child(
                        frame,
                        0x12f0cc,
                        if item == 1 { 0x12825a } else { 0x128222 },
                        e,
                    )?;
                    if e.d0_word()? == 0 {
                        if item == 2 {
                            if e.w(0x438776)? == 0 {
                                child(frame, 0x10f186, 0x128240, e)?;
                            } else {
                                with_args(frame, 0x110644, 0x128236, &[W(0)], 2, e)?;
                            }
                            child(frame, 0x13a498, 0x128246, e)?;
                        }
                        e.moveq_return(0);
                        return Ok(());
                    }
                    if item == 2 {
                        e.byte(0x4386b2, 1)?;
                    }
                    menu_enable(
                        &[
                            (0x43872a, 0, true),
                            (0x43871e, 0, true),
                            (0x43871e, 1, true),
                            (0x43871e, 2, true),
                            (0x43871e, 3, true),
                            (0x43871e, 7, true),
                            (0x43871e, 5, false),
                            (0x43870e, 3, true),
                            (0x43870e, 1, false),
                            (0x438722, 0, true),
                            (0x43870a, 3, true),
                            (0x43870a, 4, true),
                            (0x43870a, 2, true),
                            (0x43870a, 1, false),
                        ],
                        e,
                    )?;
                    pascal(0xa887, vec![W(e.w(0x402f44)?)], 0, e)?;
                    pascal(0xa937, vec![], 0, e)?;
                    if e.b(0x4386b2)? == 0 {
                        with_args(frame, 0x149452, 0x128320, &[W(0)], 2, e)?;
                    }
                }
                3 => with_args(frame, 0x152378, 0x128200, &[W(0)], 2, e)?,
                4 => {
                    with_args(frame, 0x152378, 0x128210, &[W(0xffff)], 0, e)?;
                    child(frame, 0x10f186, 0x128216, e)?;
                    e.stack = e.stack.wrapping_add(2);
                }
                6 => {
                    if e.l(0x479fd8)? == 0 {
                        if e.w(0x438776)? == 0 {
                            if !question(frame, &[L(0x10000)], 0x1281de, 4, e)? {
                                with_args(frame, 0x152378, 0x1281ec, &[W(0)], 2, e)?;
                            }
                        } else if !question(frame, &[L(0xa0000)], 0x1281c6, 4, e)? {
                            e.moveq_return(0);
                            return Ok(());
                        }
                    }
                    child(frame, 0x13a87c, 0x1281f4, e)?;
                }
                _ => {}
            }
        }
        0x82 => match e.w(0x438748)? {
            1 => {
                e.byte(0x44273a, 0xff)?;
                e.word(0x442578, 4)?;
                e.word(0x4471ea, 1)?;
                if e.sw(0x442560)? < 5 {
                    e.word(0x442560, 5)?;
                }
                if e.w(0x442560)? == 5 {
                    e.word(0x4471ea, 0)?;
                }
                with_args(
                    frame,
                    0x1366f8,
                    0x1283fa,
                    &[W(e.w(0x4471ea)?), W(e.w(0x442560)?)],
                    4,
                    e,
                )?;
                if e.d0_word()? == 0 {
                    e.moveq_return(0);
                    return Ok(());
                }
                with_args(frame, 0x1476cc, 0x12840a, &[W(0)], 0, e)?;
                pascal(0xa873, vec![L(e.l(0x479fd8)?)], 0, e)?;
                pascal(0xa863, vec![L(0x21)], 0, e)?;
                pascal(0xa981, vec![L(e.l(0x479fd8)?)], 0, e)?;
                let has_party = e.sb(0x44273a)? >= 0;
                e.stack = e.stack.wrapping_add(2);
                if !has_party {
                    e.moveq_return(0);
                    return Ok(());
                }
                menu_enable(
                    &[
                        (0x438722, 0, true),
                        (0x43872a, 0, true),
                        (0x43871e, 0, true),
                        (0x43871e, 1, true),
                        (0x43871e, 2, true),
                        (0x43871e, 3, true),
                        (0x43871e, 7, true),
                        (0x43871e, 5, false),
                        (0x43870a, 3, true),
                        (0x43870a, 4, true),
                        (0x43870a, 2, true),
                        (0x43870a, 1, false),
                        (0x43870e, 1, false),
                        (0x43870e, 3, true),
                    ],
                    e,
                )?;
                for (address, value) in [
                    (0x4427c6, 0),
                    (0x4427c8, 0),
                    (0x4427ca, 0),
                    (0x4427cc, 1),
                    (0x4427ce, 0),
                    (0x4427d0, 100),
                    (0x4427d4, 1),
                    (0x4427d6, 0xffff),
                ] {
                    e.word(address, value)?;
                }
                child(frame, 0x1538ba, 0x128510, e)?;
                e.word(0x439596, 1)?;
                tutorial_clear(e)?;
                tutorial_write(0x128552, 0x128570, 0x128586, 0x12858e, e)?;
                e.word(0x4434aa, 0)?;
                while e.sw(0x4434aa)? <= i16::from(e.sb(0x44273a)?) {
                    e.step()?;
                    let warrior =
                        0x448aa0u32.wrapping_add(u32::from(e.w(0x4434aa)?).wrapping_mul(0xc42));
                    if e.sw(warrior.wrapping_add(0x50))? < 1 {
                        e.byte(0x442739, e.b(0x442739)?.wrapping_add(1))?;
                    }
                    e.word(0x4434ac, 0)?;
                    while e.sw(0x4434ac)? < 7 {
                        e.step()?;
                        let selected = 0x442580u32
                            .wrapping_add((e.sw(0x4434aa)? as i32 as u32).wrapping_mul(2));
                        e.byte(selected, 0)?;
                        let warrior = 0x448aa0u32
                            .wrapping_add(u32::from(e.w(0x4434aa)?).wrapping_mul(0xc42))
                            .wrapping_add(e.sw(0x4434ac)? as i32 as u32);
                        if e.b(warrior.wrapping_add(0x9f0))? != 0 {
                            e.byte(selected.wrapping_add(1), e.b(0x4434ad)?)?;
                        }
                        e.addw(0x4434ac, 1)?;
                    }
                    e.addw(0x4434aa, 1)?;
                }
                with_args(frame, 0x149452, 0x128634, &[W(0)], 2, e)?;
                e.moveq_return(1);
                return Ok(());
            }
            2 => with_args(
                frame,
                0x13ebde,
                0x128654,
                &[L(0x870000), W(e.w(0x435500)?.wrapping_add(1000))],
                6,
                e,
            )?,
            3 => {
                if e.w(0x438776)? == 0 {
                    if !question(frame, &[L(0x10000)], 0x12866e, 4, e)? {
                        with_args(frame, 0x152378, 0x12867c, &[W(0)], 2, e)?;
                    }
                } else if !question(frame, &[L(0xa0000)], 0x12868c, 4, e)? {
                    e.moveq_return(0);
                    return Ok(());
                }
                child(frame, 0x13684a, 0x12869e, e)?;
                e.word(0x4434aa, 0)?;
                while e.sw(0x4434aa)? <= i16::from(e.sb(0x44273a)?) {
                    e.step()?;
                    with_args(frame, 0x1374bc, 0x1286b2, &[W(e.w(0x4434aa)?)], 0, e)?;
                    e.addw(0x4434aa, 1)?;
                    e.stack = e.stack.wrapping_add(2);
                }
                with_args(frame, 0x14913e, 0x1286d2, &[W(0)], 2, e)?;
            }
            17 => with_args(frame, 0x135fc8, 0x1286e2, &[W(1)], 2, e)?,
            18 => with_args(frame, 0x135fc8, 0x1286f0, &[W(0)], 2, e)?,
            _ => {
                let front = pascal(0xa924, vec![], 4, e)?;
                if front != e.l(0x479fd8)? {
                    pascal(0xa873, vec![L(e.l(0x43879c)?)], 0, e)?;
                    with_args(frame, 0x16474a, 0x128716, &[W(0x3f)], 2, e)?;
                    e.moveq_return(0);
                    return Ok(());
                }
                if e.sw(0x438748)? > 4 {
                    pascal(
                        0xa946,
                        vec![L(e.l(0x43870e)?), W(e.w(0x438748)?), P(0x479ed8)],
                        0,
                        e,
                    )?;
                    library_call(0x400180, 0x128748, vec![P(0x479ed8)], e)?;
                    library_call(0x400090, 0x12875a, vec![P(0x479ed8), P(0x4389a4)], e)?;
                    e.stack = e.stack.wrapping_add(12);
                }
                child(frame, 0x13a6d4, 0x128764, e)?;
                if e.d0_word()? == 0 {
                    e.moveq_return(0);
                    return Ok(());
                }
                pascal(
                    0xa945,
                    vec![L(e.l(0x43870e)?), W(e.w(0x442560)?), B(0)],
                    0,
                    e,
                )?;
                pascal(
                    0xa945,
                    vec![L(e.l(0x43870e)?), W(e.w(0x438748)?), B(1)],
                    0,
                    e,
                )?;
                e.word(0x442560, e.w(0x438748)?)?;
                with_args(frame, 0x1366f8, 0x1287a2, &[W(0), W(e.w(0x442560)?)], 4, e)?;
            }
        },
        0x84 | 0x92 => {
            let monster_menu = e.w(0x438746)? == 0x92;
            child(
                frame,
                0x13a498,
                if monster_menu { 0x127c7c } else { 0x127e06 },
                e,
            )?;
            if monster_menu {
                let monster = 0x440edcu32
                    .wrapping_add(u32::from(e.w(0x438748)?.wrapping_sub(1)).wrapping_mul(0x120));
                with_args(
                    frame,
                    0x103ebe,
                    0x127ca2,
                    &[W(e.sb(monster.wrapping_add(0x103))? as i16 as u16), L(1)],
                    6,
                    e,
                )?;
            } else {
                with_args(
                    frame,
                    0x103ebe,
                    0x127e18,
                    &[L(0x10000), W(e.w(0x438748)?)],
                    6,
                    e,
                )?;
            }
            if e.sb(0x44273a)? > -1 {
                with_args(frame, 0x13bb20, 0x127e30, &[L(0xffff)], 4, e)?;
            } else {
                pascal(0xa981, vec![L(e.l(0x479fd8)?)], 0, e)?;
            }
            if e.w(0x438776)? != 0 {
                menu_combat_refresh(frame, 0x127e4e, 0x127e5a, 0x127e60, e)?;
            }
        }
        0x86 => {
            pascal(
                0xa944,
                vec![L(e.l(0x438716)?), W(e.w(0x43956a)?), W(0)],
                0,
                e,
            )?;
            pascal(
                0xa944,
                vec![L(e.l(0x438716)?), W(e.w(0x438748)?), W(0x13)],
                0,
                e,
            )?;
            e.word(0x43956a, e.w(0x438748)?)?;
            e.word(0x439568, e.w(0x438748)?.wrapping_mul(5).wrapping_sub(5))?;
            child(frame, 0x14c01c, 0x127f96, e)?;
        }
        0x87 => {
            with_args(
                frame,
                0x13846a,
                0x127fac,
                &[L(0x2000c), W(e.w(0x438748)?)],
                6,
                e,
            )?;
            if e.d0_word()? != 0 {
                e.long(0x439570, e.sw(0x438748)?.wrapping_sub(2) as i32 as u32)?;
                with_args(
                    frame,
                    0x153584,
                    0x127fd2,
                    &[L(u32::MAX), L(e.l(0x439570)?)],
                    0,
                    e,
                )?;
                with_args(frame, 0x133a18, 0x127fdc, &[W(0x282)], 10, e)?;
            }
            with_args(
                frame,
                0x13846a,
                0x127ff2,
                &[L(0x10001a), W(e.w(0x438748)?)],
                6,
                e,
            )?;
            if e.d0_word()? != 0 {
                e.long(0x43956c, e.sw(0x438748)?.wrapping_sub(16) as i32 as u32)?;
                with_args(
                    frame,
                    0x153584,
                    0x128018,
                    &[L(e.l(0x43956c)?), L(u32::MAX)],
                    8,
                    e,
                )?;
            }
            e.word(0x4434aa, 1)?;
            while e.sw(0x4434aa)? < 27 {
                e.step()?;
                pascal(
                    0xa944,
                    vec![L(e.l(0x43871a)?), W(e.w(0x4434aa)?), W(0)],
                    0,
                    e,
                )?;
                e.addw(0x4434aa, 1)?;
            }
            pascal(
                0xa944,
                vec![
                    L(e.l(0x43871a)?),
                    W(e.l(0x439570)?.wrapping_add(2) as u16),
                    W(0x13),
                ],
                0,
                e,
            )?;
            pascal(
                0xa944,
                vec![
                    L(e.l(0x43871a)?),
                    W(e.l(0x43956c)?.wrapping_add(16) as u16),
                    W(0x13),
                ],
                0,
                e,
            )?;
            child(frame, 0x14c01c, 0x128076, e)?;
        }
        0x88 => match e.w(0x438748)? {
            1 => {
                if e.w(0x4394b6)? != 0 {
                    message(frame, 0x4036b2, 0x127a78, e)?;
                    e.stack = e.stack.wrapping_add(12);
                } else {
                    child(frame, 0x152188, 0x127a86, e)?;
                }
            }
            2 => with_args(frame, 0x13bd52, 0x127a94, &[W(1)], 2, e)?,
            3 => with_args(frame, 0x13bd52, 0x127aa4, &[W(2)], 2, e)?,
            5 => {
                e.word(0x447254, 0)?;
                e.word(0x447250, 0)?;
                e.word(0x44724e, 0)?;
                child(frame, 0x13f36a, 0x127ac2, e)?;
            }
            7 => {
                with_args(frame, 0x1476cc, 0x127ad0, &[W(1)], 0, e)?;
                with_args(frame, 0x13bb20, 0x127ad8, &[L(0)], 0, e)?;
                with_args(frame, 0x16474a, 0x127ae2, &[W(0x14)], 8, e)?;
            }
            8 => {
                let development = e.w(0x4046ea)? == 1;
                if e.w(0x4471ec)? == 1 {
                    e.word(0x4471ec, 0)?;
                    message(
                        frame,
                        0x403686,
                        if development { 0x127b1a } else { 0x127bd0 },
                        e,
                    )?;
                    pascal(
                        0xa947,
                        vec![
                            L(e.l(0x43871e)?),
                            W(8),
                            P(if development { 0x403670 } else { 0x4035a2 }),
                        ],
                        0,
                        e,
                    )?;
                    e.stack = e.stack.wrapping_add(12);
                } else if question(
                    frame,
                    &[W(0x13)],
                    if development { 0x127b3e } else { 0x127bf4 },
                    2,
                    e,
                )? {
                    e.word(0x4471ec, 1)?;
                    pascal(0xa947, vec![L(e.l(0x43871e)?), W(8), P(0x403658)], 0, e)?;
                    message(
                        frame,
                        0x40361a,
                        if development { 0x127b76 } else { 0x127c2c },
                        e,
                    )?;
                    message(
                        frame,
                        0x4035fa,
                        if development { 0x127b8c } else { 0x127c42 },
                        e,
                    )?;
                    message(
                        frame,
                        0x4035be,
                        if development { 0x127ba2 } else { 0x127c58 },
                        e,
                    )?;
                    if development {
                        e.stack = e.stack.wrapping_add(36);
                    } else {
                        message(frame, 0x40356a, 0x127c6e, e)?;
                        e.stack = e.stack.wrapping_add(48);
                    }
                }
            }
            _ => {}
        },
        0x89 => match e.w(0x438748)? {
            3 => {
                child(frame, 0x13a498, 0x128096, e)?;
                child(frame, 0x14b63a, 0x12809c, e)?;
                if e.w(0x438776)? != 0 {
                    child(frame, 0x13a510, 0x1280ac, e)?;
                    menu_combat_refresh(frame, 0x1280b4, 0x1280c0, 0x1280c6, e)?;
                }
            }
            4 => {
                if pascal(0xa924, vec![], 4, e)? == e.l(0x43879c)? {
                    with_args(frame, 0x13bb20, 0x1280e4, &[L(0)], 4, e)?;
                }
            }
            5 => {
                e.word(0x439596, 1)?;
                if question(frame, &[L(0x90000)], 0x1280fe, 4, e)? {
                    tutorial_clear(e)?;
                    tutorial_write(0x128140, 0x128160, 0x128176, 0x12817e, e)?;
                }
            }
            _ => {}
        },
        0x91 => {
            if e.w(0x438748)? == 1 {
                e.word(0x439594, u16::from(e.w(0x439594)? == 0))?;
                if e.w(0x439594)? != 0 {
                    pascal(0xa947, vec![L(e.l(0x4386a0)?), W(1), P(0x40355e)], 0, e)?;
                    pascal(0xa947, vec![L(e.l(0x4386a0)?), W(4), P(0x403526)], 0, e)?;
                } else {
                    pascal(0xa947, vec![L(e.l(0x4386a0)?), W(1), P(0x40351a)], 0, e)?;
                }
                with_args(frame, 0x133d46, 0x127d14, &[W(0)], 2, e)?;
            } else if e.sw(0x438748)? > 9 {
                let index = e.w(0x438748)?.wrapping_sub(10);
                e.registers.data[3] =
                    application::RegisterBits::full(0xffff0000 | u32::from(index));
                pascal(0xa860, vec![W(0xffff), P(0x44720a), L(0), L(0)], 1, e)?;
                if pascal(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(0x200)], 4, e)? == 0 {
                    // This native music-preview arm was removed by the decompiler.
                    e.word(0x439594, 1)?;
                    with_args(frame, 0x133d46, 0x127d68, &[W(0)], 0, e)?;
                    e.word(0x439594, 0)?;
                    with_args(
                        frame,
                        0x133d46,
                        0x127d7c,
                        &[W(index.wrapping_add(0x4ea0))],
                        4,
                        e,
                    )?;
                } else if pascal(0xa858, vec![L(u32::from(e.w(0x447218)?)), L(0x200)], 4, e)? != 0 {
                    let address = 0x44270eu32.wrapping_add(index as i16 as i32 as u32);
                    e.byte(address, u8::from(e.b(address)? == 0))?;
                    let checked = if e.b(address)? == 0 { 0 } else { 0x13 };
                    pascal(
                        0xa944,
                        vec![L(e.l(0x4386a0)?), W(index.wrapping_add(10)), W(checked)],
                        0,
                        e,
                    )?;
                }
            }
        }
        200 => {
            if e.sw(0x438748)? > 2 {
                with_args(
                    frame,
                    0x134e82,
                    0x128346,
                    &[W(e.w(0x438748)?.wrapping_sub(4))],
                    0,
                    e,
                )?;
                pascal(
                    0xa945,
                    vec![L(e.l(0x43872a)?), W(e.w(0x438748)?), B(0)],
                    0,
                    e,
                )?;
                e.stack = e.stack.wrapping_add(2);
            } else if e.w(0x438748)? == 1 {
                with_args(
                    frame,
                    0x13ebde,
                    0x128380,
                    &[L(0x870000), W(e.w(0x435500)?.wrapping_add(1000))],
                    6,
                    e,
                )?;
            }
        }
        _ => {}
    }
    e.moveq_return(0);
    Ok(())
}

crate::recovered_application_control::entry!(main_screen, 0x13b624);
crate::recovered_application_control::entry!(update_main, 0x13bb20);
crate::recovered_application_control::entry!(handle_menu_choice, 0x1279bc);
crate::recovered_application_control::entry!(main_screen_init, 0x149452);
crate::recovered_application_control::entry!(preference, 0x14b63a);
