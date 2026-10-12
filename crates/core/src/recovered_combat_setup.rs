//! Provisional combat setup and turn advancement from mapped CODE 1.

use crate::recovered_application_control::{self as application, Execution, RegisterBits};
use crate::recovered_character_upgrade::UpgradeSaneRequest;
use crate::recovered_feedback::FeedbackArg;
use FeedbackArg::{Long as L, Pointer as P, Word as W};

fn warrior(n: u16) -> u32 {
    0x448aa0u32.wrapping_add(u32::from(n).wrapping_mul(0xc42))
}
fn monster(n: u16) -> u32 {
    0x439d3cu32.wrapping_add(u32::from(n).wrapping_mul(0x120))
}
fn indexed(base: u32, n: i16, stride: u32) -> u32 {
    base.wrapping_add((n as i32 as u32).wrapping_mul(stride))
}
fn turn(e: &Execution<'_>) -> application::Result<i16> {
    e.sw(indexed(0x4470b8, e.sw(0x447194)?, 2))
}
fn current_warrior(e: &Execution<'_>) -> application::Result<u32> {
    Ok(warrior(e.w(0x4435d2)?))
}
fn current_monster(e: &Execution<'_>) -> application::Result<u32> {
    Ok(monster(e.w(0x447196)?))
}

fn push(arg: &FeedbackArg, e: &mut Execution<'_>) -> application::Result<()> {
    match *arg {
        W(v) => {
            e.stack = e.stack.wrapping_sub(2);
            e.word(e.stack, v)?;
        }
        L(v) | P(v) => {
            e.stack = e.stack.wrapping_sub(4);
            e.long(e.stack, v)?;
        }
        FeedbackArg::Byte(v) => {
            e.stack = e.stack.wrapping_sub(2);
            e.byte(e.stack, v)?;
        }
    }
    Ok(())
}
fn call(
    frame: u32,
    entry: u32,
    next: u32,
    args: &[FeedbackArg],
    cleanup: u32,
    e: &mut Execution<'_>,
) -> application::Result<()> {
    for arg in args {
        push(arg, e)?;
    }
    let stack = e.stack;
    e.long(stack.wrapping_sub(4), next)?;
    e.long(stack.wrapping_sub(8), frame)?;
    e.call(entry, stack.wrapping_sub(8))?;
    e.stack = stack.wrapping_add(cleanup);
    Ok(())
}
fn library(
    entry: u32,
    next: u32,
    args: &[FeedbackArg],
    cleanup: u32,
    e: &mut Execution<'_>,
) -> application::Result<u32> {
    for arg in args {
        push(arg, e)?;
    }
    let stack = e.stack;
    e.long(stack.wrapping_sub(4), next)?;
    let observed = e.lib(entry, args.to_vec())?;
    e.stack = stack.wrapping_add(cleanup);
    Ok(observed.value)
}
fn absolute(
    value: i16,
    next: u32,
    cleanup: u32,
    e: &mut Execution<'_>,
) -> application::Result<u16> {
    library(0x400038, next, &[W(value as u16)], cleanup, e)?;
    e.d0_word()
}
fn seek(file: u32, offset: u32, next: u32, e: &mut Execution<'_>) -> application::Result<()> {
    let origin = e.w(e.stack)?;
    push(&L(offset), e)?;
    push(&L(file), e)?;
    e.long(e.stack.wrapping_sub(4), next)?;
    e.lib(0x400078, vec![W(origin), L(offset), L(file)])?;
    Ok(())
}
fn random(bound: i16, next: u32, e: &mut Execution<'_>) -> application::Result<i16> {
    // Rand, CODE 1 entry 0x153ca4, projects through the existing runtime port.
    e.long(e.stack.wrapping_sub(6), next)?;
    e.rand(bound, e.stack)
}
fn sane(pointers: &[u32], selector: u16, e: &mut Execution<'_>) -> application::Result<u8> {
    let mut args: Vec<_> = pointers.iter().copied().map(P).collect();
    args.push(W(selector));
    let stack = e.stack;
    for arg in &args {
        push(arg, e)?;
    }
    let flags = e.sane(UpgradeSaneRequest {
        opcode: 0xa9eb,
        arguments: args,
    })?;
    e.stack = stack;
    Ok(flags)
}
fn ge(ccr: u8) -> bool {
    (ccr & 8 != 0) == (ccr & 2 != 0)
}
fn gt(ccr: u8) -> bool {
    ge(ccr) && ccr & 4 == 0
}
fn pascal(
    opcode: u16,
    args: &[FeedbackArg],
    result_bytes: u32,
    e: &mut Execution<'_>,
) -> application::Result<u32> {
    let stack = e.stack;
    e.stack = e.stack.wrapping_sub(result_bytes);
    let slot = e.stack;
    for arg in args {
        push(arg, e)?;
    }
    let value = e.trap(opcode, args.to_vec())?;
    if result_bytes == 2 {
        e.byte(slot, value as u8)?;
        e.full((e.d0_long()? & 0xffffff00) | (value & 0xff));
    }
    e.stack = stack;
    Ok(value)
}
fn clear_collisions(register: usize, e: &mut Execution<'_>) -> application::Result<()> {
    e.registers.data[register] = RegisterBits::full(0);
    for n in 0..60u16 {
        e.step()?;
        e.byte(0x435884 + u32::from(n), 0)?;
        e.registers.data[register].assign_word(n.wrapping_add(1));
    }
    Ok(())
}
fn finish_turn(frame: u32, e: &mut Execution<'_>) -> application::Result<()> {
    e.word(0x438786, 1)?;
    call(frame, 0x13154c, 0x1270a2, &[W(0x28)], 0, e)?;
    push(&L(0xffff), e)?;
    e.full(e.l(e.stack)?);
    e.stack = e.stack.wrapping_add(4);
    e.register_trap(0xa032, Some(0xffff), None, vec![])?;
    e.moveq_return(0);
    e.stack = e.stack.wrapping_add(2);
    Ok(())
}

fn advance_round(frame: u32, e: &mut Execution<'_>) -> application::Result<()> {
    call(frame, 0x13154c, 0x125df6, &[W(0x24)], 0, e)?;
    call(frame, 0x14de04, 0x125dfc, &[], 0, e)?;
    call(frame, 0x13154c, 0x125e06, &[W(0x25)], 4, e)?;
    e.registers.data[3] = RegisterBits::full(0);
    let mut n = 0u16;
    while (n as i16) <= i16::from(e.sb(0x44273a)?) {
        e.step()?;
        call(frame, 0x160fb2, 0x125e18, &[W(0), W(n)], 4, e)?;
        clear_collisions(4, e)?;
        if e.sw(warrior(n) + 0x50)? > 0 {
            call(frame, 0x1140ae, 0x125e4c, &[W(0), W(n)], 4, e)?;
        }
        e.word(warrior(n) + 0x14, 0)?;
        if e.b(warrior(n) + 0x9de)? != 0 {
            if e.sw(warrior(n) + 0x50)? > -10 {
                e.addw(warrior(n) + 0x50, -1)?;
                if e.sw(warrior(n) + 0x50)? < -9 {
                    e.byte(warrior(n) + 0x9de, 0)?;
                    e.addl(warrior(n) + 0xa62, u32::MAX)?;
                    e.addl(warrior(n) + 0xa5e, 1)?;
                    call(frame, 0x161b80, 0x125ef2, &[L(0x10000), W(n)], 0, e)?;
                    let id = (random(2, 0x125efc, e)? as u16).wrapping_add(0x1fa);
                    call(frame, 0x133a18, 0x125f0a, &[W(id)], 0, e)?;
                    call(
                        frame,
                        0x123d1c,
                        0x125f22,
                        &[L(0xb4278a), L(0x1e0064), P(0x403750)],
                        20,
                        e,
                    )?;
                }
            }
            call(frame, 0x15f66c, 0x125f32, &[W(3), W(n)], 4, e)?;
        }
        n = n.wrapping_add(1);
        e.registers.data[3].assign_word(n);
    }
    e.registers.data[3] = RegisterBits::full(0);
    for n in 0..100u16 {
        e.step()?;
        e.registers.data[4] = RegisterBits::full(0);
        for condition in 0..40u16 {
            e.step()?;
            let address = monster(n) + 0xb2 + u32::from(condition);
            if e.sb(address)? > 0 {
                e.byte(address, e.b(address)?.wrapping_sub(1))?;
            }
            e.registers.data[4].assign_word(condition.wrapping_add(1));
        }
        e.byte(monster(n) + 0x105, 0)?;
        if e.b(monster(n) + 0xb5)? != 0
            && e.sw(monster(n) + 0x110)? > 0
            && e.sw(monster(n) + 0x110)? < e.sw(monster(n) + 0x112)?
        {
            let amount = absolute(e.sb(monster(n) + 0xb5)? as i16, 0x126002, 0, e)?;
            e.word(0x4434b8, amount)?;
            e.addw(monster(n) + 0x110, amount as i16)?;
            let overflow = e.sw(monster(n) + 0x110)? > e.sw(monster(n) + 0x112)?;
            e.stack = e.stack.wrapping_add(2);
            if overflow {
                // Native subtracts two reads of the same HP word here.
                e.word(
                    0x4434b8,
                    e.w(0x4434b8)?.wrapping_sub(
                        e.w(monster(n) + 0x110)?
                            .wrapping_sub(e.w(monster(n) + 0x110)?),
                    ),
                )?;
                e.word(monster(n) + 0x110, e.w(monster(n) + 0x112)?)?;
            }
            e.word(0x438766, 1)?;
            if e.w(0x4434b8)? != 0 {
                call(
                    frame,
                    0x158454,
                    0x1260b2,
                    &[W(0), W(n.wrapping_add(10))],
                    4,
                    e,
                )?;
            }
            e.word(0x438766, 0)?;
        }
        if e.b(monster(n) + 0xce)? != 0
            && e.b(monster(n) + 0xcb)? == 0
            && e.sw(monster(n) + 0x110)? > 0
        {
            let amount = absolute(e.sb(monster(n) + 0xce)? as i16, 0x126116, 0, e)?;
            e.word(0x4434b8, amount)?;
            e.addw(monster(n) + 0x110, (amount as i16).wrapping_neg())?;
            call(
                frame,
                0x158454,
                0x126144,
                &[W(0x1d), W(n.wrapping_add(10))],
                0,
                e,
            )?;
            let dead = e.sw(monster(n) + 0x110)? < 1;
            e.stack = e.stack.wrapping_add(6);
            if dead {
                call(
                    frame,
                    0x12e4e4,
                    0x12616a,
                    &[W(0), W(n.wrapping_add(10))],
                    4,
                    e,
                )?;
            }
        }
        if e.b(monster(n) + 0xb5)? != 0
            && e.b(monster(n) + 0xcb)? == 0
            && e.sw(monster(n) + 0x110)? > 0
        {
            let amount = absolute(e.sb(monster(n) + 0xb5)? as i16, 0x1261ca, 0, e)?;
            e.word(0x4434b8, amount)?;
            e.addw(monster(n) + 0x110, (amount as i16).wrapping_neg())?;
            e.word(0x438784, 1)?;
            call(
                frame,
                0x158454,
                0x1261fe,
                &[W(0), W(n.wrapping_add(10))],
                0,
                e,
            )?;
            e.word(0x438784, 0)?;
            let dead = e.sw(monster(n) + 0x110)? < 1;
            e.stack = e.stack.wrapping_add(6);
            if dead {
                call(
                    frame,
                    0x12e4e4,
                    0x12622a,
                    &[W(0), W(n.wrapping_add(10))],
                    4,
                    e,
                )?;
            }
        }
        if e.b(monster(n) + 0xd4)? != 0 {
            e.addw(
                monster(n) + 0x10e,
                (e.sb(monster(n) + 0xd4)? as i16).wrapping_neg(),
            )?;
            if e.sw(monster(n) + 0x10e)? < 0 {
                e.word(monster(n) + 0x10e, 0)?;
            }
        }
        if e.b(monster(n) + 0xd3)? != 0 && e.b(monster(n) + 0xa8)? != 0 {
            e.addw(monster(n) + 0x10e, e.sb(monster(n) + 0xd4)? as i16)?;
        }
        clear_collisions(4, e)?;
        if e.sw(monster(n) + 0x110)? > 0 {
            call(
                frame,
                0x1140ae,
                0x126312,
                &[L(0), W(n.wrapping_add(10))],
                6,
                e,
            )?;
        }
        e.byte(monster(n) + 0x105, 0)?;
        e.registers.data[3].assign_word(n.wrapping_add(1));
    }
    if e.w(0x43958e)? == 0 {
        e.registers.data[6] = RegisterBits::full(0);
        e.registers.data[3] = RegisterBits::full(0);
        let mut n = 0u16;
        while (n as i16) <= i16::from(e.sb(0x44273a)?) {
            e.step()?;
            if e.b(warrior(n) + 0x9de)? != 0 {
                e.registers.data[6] = RegisterBits::full(1);
            }
            n = n.wrapping_add(1);
            e.registers.data[3].assign_word(n);
        }
        if e.registers.data[6].word("D6")? != 0 {
            call(
                frame,
                0x123d1c,
                0x12637e,
                &[L(0x78278a), L(0x1e0064), P(0x403708)],
                0,
                e,
            )?;
            call(frame, 0x12382a, 0x12638a, &[L(0x760076)], 16, e)?;
        }
    }
    call(frame, 0x12382a, 0x12639a, &[L(0x740075)], 0, e)?;
    let show = e.w(0x439590)? == 0;
    e.stack = e.stack.wrapping_add(4);
    if show {
        call(
            frame,
            0x123d1c,
            0x1263bc,
            &[L(0xffff0000), L(0x1e0064), P(0x4036f2)],
            12,
            e,
        )?;
    }
    call(frame, 0x133a18, 0x1263ca, &[W((-139i16) as u16)], 0, e)?;
    call(frame, 0x133a18, 0x1263d4, &[W((-139i16) as u16)], 0, e)?;
    let show = e.w(0x439590)? == 0;
    e.stack = e.stack.wrapping_add(4);
    if show {
        call(
            frame,
            0x123d1c,
            0x1263f4,
            &[L((-139i32) as u32), L(0x1e0064), P(0x4036f1)],
            12,
            e,
        )?;
    }
    Ok(())
}

pub fn get_up_body(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    e.stack = frame.wrapping_sub(0x36);
    e.long(frame.wrapping_sub(4), 0)?;
    call(frame, 0x13154c, 0x125a50, &[W(0x22)], 0, e)?;
    call(frame, 0x13822a, 0x125a56, &[], 0, e)?;
    e.word(0x438786, 1)?;
    call(frame, 0x133d46, 0x125a66, &[W(0)], 0, e)?;
    call(frame, 0x1368c8, 0x125a6c, &[], 0, e)?;
    call(frame, 0x13154c, 0x125a76, &[W(0x23)], 0, e)?;
    for address in [0x443761, 0x438582, 0x438586, 0x4386aa] {
        e.byte(address, 0)?;
    }
    e.full(0);
    e.word(0x44719a, 0)?;
    e.word(0x438782, 0)?;
    e.byte(0x435662, 0)?;
    e.byte(0x43565e, 0)?;
    e.word(0x41f712, 0)?;
    let party_lost = e.sb(0x442739)? > e.sb(0x44273a)?;
    e.stack = e.stack.wrapping_add(6);
    if party_lost {
        call(frame, 0x14913e, 0x125acc, &[W(0)], 2, e)?;
    }
    e.word(0x4471ea, 0)?;
    e.registers.data[3] = RegisterBits::full(0);
    let mut n = 0u16;
    while (n as i16) <= i16::from(e.sb(0x44273a)?) {
        e.step()?;
        if e.b(warrior(n) + 0x9df)? != 0 && e.w(warrior(n) + 0x10)? == 0 {
            e.word(0x4471ea, 1)?;
        }
        n = n.wrapping_add(1);
        e.registers.data[3].assign_word(n);
    }
    if e.w(0x4471ea)? == 0 {
        e.word(0x44374c, e.w(0x4435c4)?)?;
        e.byte(0x4386fc, 1)?;
        call(frame, 0x105748, 0x125b3a, &[L(0x20000)], 4, e)?;
        e.moveq_return(0);
        return Ok(());
    }
    'prepare: loop {
        e.step()?;
        if e.w(frame.wrapping_add(8))? == 0 && turn(e)? > -1 {
            let show = (turn(e)? > 9 && e.b(monster(e.w(0x447196)?) + 0x104)? != 0)
                || (turn(e)? < 9 && e.w(warrior(turn(e)? as u16) + 0x16)? != 0);
            if show {
                call(
                    frame,
                    0x158454,
                    0x125bd8,
                    &[W((-5i16) as u16), W(turn(e)? as u16)],
                    4,
                    e,
                )?;
            }
        }
        for address in [0x438760, 0x443316, 0x438764, 0x43877e, 0x438780] {
            e.word(address, 0)?;
        }
        if turn(e)? < 9 && turn(e)? > -1 {
            call(frame, 0x160fb2, 0x125c3a, &[W(1), W(turn(e)? as u16)], 0, e)?;
            let clear = e.sw(warrior(turn(e)? as u16) + 0x90)? > 0;
            e.stack = e.stack.wrapping_add(4);
            if clear {
                e.word(warrior(turn(e)? as u16) + 0x90, 0)?;
            }
        }
        if e.sw(0x439576)? < 2 {
            e.registers.data[3] = RegisterBits::full(0);
            let mut n = 0i16;
            while n <= e.sw(0x439576)? {
                e.step()?;
                call(frame, 0x13b56c, 0x125c94, &[W(n as u16)], 0, e)?;
                n = n.wrapping_add(1);
                e.registers.data[3].assign_word(n as u16);
                e.stack = e.stack.wrapping_add(2);
            }
        }
        loop {
            e.step()?;
            e.addw(0x447194, 1)?;
            e.byte(0x435659, 0)?;
            e.registers.data[3] = RegisterBits::full(0);
            for n in 0..60u16 {
                e.step()?;
                let effect = 0x442bd6 + u32::from(n) * 20;
                if e.b(effect + 15)? != 0 && i16::from(e.sb(effect + 18)?) == e.sw(0x447194)? {
                    e.byte(effect + 15, e.b(effect + 15)?.wrapping_sub(1))?;
                }
                e.registers.data[3].assign_word(n.wrapping_add(1));
            }
            if e.sw(0x44374c)? >= e.sw(0x4435c4)? {
                e.moveq_return(0);
                return Ok(());
            }
            clear_collisions(3, e)?;
            call(frame, 0x121524, 0x125d1e, &[], 0, e)?;
            e.word(0x4435c2, e.w(0x447194)?)?;
            if e.sw(0x447194)? > 109 {
                e.byte(0x44255c, e.b(0x44255c)?.wrapping_add(1))?;
                e.word(0x408576, 0)?;
                while e.sw(0x408576)? < 5 {
                    e.step()?;
                    let door = e.w(indexed(0x439cec, e.sw(0x408576)?, 2))?;
                    if door != 0 {
                        e.byte(0x438583, 1)?;
                        call(frame, 0x123c60, 0x125d7e, &[W(door)], 0, e)?;
                        call(
                            frame,
                            0x1400e4,
                            0x125d90,
                            &[W(0), L(0xffff0000), L(0), L(0)],
                            0,
                            e,
                        )?;
                        e.byte(0x438583, 0)?;
                        let ended = e.w(0x438776)? == 0;
                        e.stack = e.stack.wrapping_add(16);
                        if ended {
                            e.moveq_return(0);
                            return Ok(());
                        }
                    }
                    e.addw(0x408576, 1)?;
                }
                let elapsed = e.sw(0x4427a6)?;
                e.addw(0x4427a6, 1)?;
                if elapsed > 8 {
                    e.word(0x44374c, e.w(0x4435c4)?)?;
                    e.byte(0x4386fc, 1)?;
                    continue;
                }
                e.word(0x4435c2, 0)?;
                e.word(0x447194, 0)?;
                advance_round(frame, e)?;
            }
            if turn(e)? < 0 {
                continue;
            }
            call(frame, 0x13154c, 0x126416, &[W(0x26)], 0, e)?;
            let actor = turn(e)?;
            e.stack = e.stack.wrapping_add(2);
            if actor < 9 {
                if e.b(warrior(actor as u16) + 0x9df)? == 0 {
                    continue;
                }
            } else {
                let body = monster(actor.wrapping_sub(10) as u16);
                if e.sw(body + 0x110)? < 1 || e.sb(body + 0xb0)? == -1 {
                    continue;
                }
            }
            if turn(e)? < 9 && e.w(warrior(turn(e)? as u16) + 0x60)? != 0 {
                continue;
            }
            if turn(e)? > 9 && e.b(monster(turn(e)?.wrapping_sub(10) as u16) + 0xb3)? != 0 {
                continue;
            }
            if turn(e)? < 9 && e.w(warrior(turn(e)? as u16) + 0x10)? != 0 {
                e.word(0x438764, 1)?;
                e.word(warrior(turn(e)? as u16) + 0x90, 1)?;
            }
            if turn(e)? < 9 {
                e.word(0x438778, 0)?;
                let selected = turn(e)? as u8;
                e.byte(0x44273f, selected)?;
                e.word(0x4435d2, selected as i8 as i16 as u16)?;
                let n = e.sw(0x4435d2)?;
                e.byte(
                    0x438740,
                    e.l(0x442780)?
                        .wrapping_add(e.sw(indexed(0x44709a, n, 4))? as i32 as u32)
                        as u8,
                )?;
                e.byte(
                    0x438741,
                    e.l(0x442784)?
                        .wrapping_add(e.sw(indexed(0x44709c, n, 4))? as i32 as u32)
                        as u8,
                )?;
                if e.w(current_warrior(e)? + 0x5e)? != 0 && e.w(current_warrior(e)? + 0x90)? == 0 {
                    e.word(current_warrior(e)? + 0x90, 1)?;
                }
                if e.sw(current_warrior(e)? + 0x90)? > -1 && e.w(current_warrior(e)? + 0x98)? != 0 {
                    let roll = random(100, 0x12666c, e)?;
                    e.word(0x4434b6, roll as u16)?;
                    if e.sw(0x4434b6)? < 30 {
                        if random(2, 0x12668a, e)? == 1 && e.w(current_warrior(e)? + 0x10)? == 0 {
                            e.addw(current_warrior(e)? + 0x10, 1)?;
                            if e.w(current_warrior(e)? + 0x90)? == 0 {
                                e.word(current_warrior(e)? + 0x90, 1)?;
                            }
                            e.addw(0x4435c4, 1)?;
                            e.byte(0x442739, e.b(0x442739)?.wrapping_add(1))?;
                        } else if random(2, 0x126704, e)? == 2
                            && e.w(current_warrior(e)? + 0x10)? != 0
                        {
                            e.word(current_warrior(e)? + 0x10, 0)?;
                            e.addw(0x4435c4, -1)?;
                            e.byte(0x442739, e.b(0x442739)?.wrapping_sub(1))?;
                        }
                    } else if e.sw(0x4434b6)? > 70 {
                        e.addw(current_warrior(e)? + 0x5e, 30)?;
                    } else {
                        e.addw(current_warrior(e)? + 0x60, 1)?;
                        continue;
                    }
                }
                call(frame, 0x12006e, 0x12678a, &[W(e.w(0x4435d2)?)], 0, e)?;
                let positive = e.sw(current_warrior(e)? + 0x40)? > 0;
                e.stack = e.stack.wrapping_add(2);
                if positive {
                    e.word(current_warrior(e)? + 0x40, 1)?;
                }
                if e.sw(current_warrior(e)? + 0x40)? < 0 {
                    e.word(current_warrior(e)? + 0x40, 0)?;
                }
                let amount = e
                    .w(current_warrior(e)? + 0x12)?
                    .wrapping_add(e.w(current_warrior(e)? + 0x1e)?);
                e.addw(current_warrior(e)? + 0x40, amount as i16)?;
                if e.w(current_warrior(e)? + 0x8c)? != 0 {
                    e.addw(current_warrior(e)? + 0x40, 4)?;
                }
                call(frame, 0x136994, 0x12685a, &[W(e.w(0x4435d2)?)], 0, e)?;
                e.word(current_warrior(e)? + 0x3c, e.w(current_warrior(e)? + 0x3e)?)?;
                e.word(current_warrior(e)? + 0xacc, 0)?;
                e.word(current_warrior(e)? + 0xac8, 0)?;
                call(frame, 0x11f1a8, 0x1268b8, &[W(e.w(0x4435d2)?)], 0, e)?;
                call(frame, 0x16043e, 0x1268be, &[], 0, e)?;
                call(frame, 0x12946c, 0x1268ca, &[W(e.w(0x4435d2)?)], 0, e)?;
                let in_window = e.d0_word()? != 0;
                e.stack = e.stack.wrapping_add(6);
                let args = if in_window {
                    [W(e.w(0x408572)?), W(e.w(0x408570)?)]
                } else {
                    [
                        W(e.w(indexed(0x44709c, e.sw(0x4435d2)?, 4))?),
                        W(e.w(indexed(0x44709a, e.sw(0x4435d2)?, 4))?),
                    ]
                };
                call(
                    frame,
                    0x10e6b6,
                    if in_window { 0x12690c } else { 0x1268f6 },
                    &args,
                    4,
                    e,
                )?;
                if e.w(current_warrior(e)? + 0x90)? == 0 {
                    call(frame, 0x133a18, 0x126932, &[W(0x8a)], 2, e)?;
                }
                return finish_turn(frame, e);
            }
            e.word(0x438778, 1)?;
            e.word(0x447196, turn(e)?.wrapping_sub(10) as u16)?;
            e.word(frame.wrapping_sub(0x26), e.w(current_monster(e)? + 0x5c)?)?;
            if random(100, 0x126978, e)? <= e.sw(frame.wrapping_sub(0x26))? {
                call(
                    frame,
                    0x123c60,
                    0x12699c,
                    &[W(e.w(current_monster(e)? + 0x5e)?)],
                    0,
                    e,
                )?;
                e.word(0x43800e, e.w(0x447196)?)?;
                call(
                    frame,
                    0x10e6b6,
                    0x1269cc,
                    &[
                        W(e.w(indexed(0x446f0c, e.sw(0x447196)?, 4))?),
                        W(e.w(indexed(0x446f0a, e.sw(0x447196)?, 4))?),
                    ],
                    0,
                    e,
                )?;
                call(
                    frame,
                    0x1400e4,
                    0x1269f4,
                    &[
                        W(e.w(0x447196)?),
                        W(e.w(current_monster(e)? + 0x5e)?),
                        W(0xffff),
                        L(0),
                        L(0),
                    ],
                    0,
                    e,
                )?;
                let gone = e.w(current_monster(e)? + 0x110)? == 0;
                e.stack = e.stack.wrapping_add(20);
                if gone {
                    continue;
                }
                e.word(0x43800e, 0)?;
            }
            if e.sw(current_monster(e)? + 0x5c)? < 0 {
                e.word(
                    current_monster(e)? + 0x5c,
                    e.w(current_monster(e)? + 0x5c)?.wrapping_neg(),
                )?;
            }
            if e.sb(current_monster(e)? + 0x109)? > 1 {
                e.byte(current_monster(e)? + 0x109, 1)?;
            }
            e.byte(
                current_monster(e)? + 0x109,
                e.b(current_monster(e)? + 0x109)?
                    .wrapping_add(e.b(current_monster(e)? + 0x8a)?),
            )?;
            if e.b(current_monster(e)? + 0xcf)? != 0 {
                let roll = random(100, 0x126ada, e)?;
                e.word(0x4434b6, roll as u16)?;
                if e.sw(0x4434b6)? < 40 {
                    if random(2, 0x126af8, e)? == 1 && e.w(current_monster(e)? + 0xe)? == 0 {
                        e.addw(current_monster(e)? + 0xe, 1)?;
                        e.addw(0x4435c4, 1)?;
                    } else if random(2, 0x126b3c, e)? == 1 && e.w(current_monster(e)? + 0xe)? != 0 {
                        e.word(current_monster(e)? + 0xe, 0)?;
                        e.addw(0x4435c4, -1)?;
                    }
                } else if e.sw(0x4434b6)? > 60 {
                    e.byte(
                        current_monster(e)? + 0xb2,
                        e.b(current_monster(e)? + 0xb2)?.wrapping_add(1),
                    )?;
                } else {
                    e.byte(
                        current_monster(e)? + 0xb3,
                        e.b(current_monster(e)? + 0xb3)?.wrapping_add(1),
                    )?;
                    continue;
                }
            }
            e.byte(
                current_monster(e)? + 0x106,
                e.w(current_monster(e)? + 6)? as u8,
            )?;
            e.byte(
                current_monster(e)? + 0x106,
                e.b(current_monster(e)? + 0x106)?
                    .wrapping_sub(e.b(current_monster(e)? + 0xb4)?),
            )?;
            if e.b(current_monster(e)? + 0xb8)? != 0 {
                e.byte(
                    current_monster(e)? + 0x106,
                    (e.sb(current_monster(e)? + 0x106)? / 2) as u8,
                )?;
            }
            if e.b(current_monster(e)? + 0xc9)? != 0 {
                e.byte(
                    current_monster(e)? + 0x106,
                    e.b(current_monster(e)? + 0x106)?.wrapping_mul(2),
                )?;
                e.byte(
                    current_monster(e)? + 0x109,
                    e.b(current_monster(e)? + 0x109)?.wrapping_add(4),
                )?;
                if e.sb(current_monster(e)? + 0x109)? > 11 {
                    e.byte(current_monster(e)? + 0x109, 11)?;
                }
                e.registers.data[5] = RegisterBits::full(1);
                let mut n = 1i16;
                while n < i16::from(e.sb(current_monster(e)? + 0x109)?) / 2 {
                    e.step()?;
                    let address =
                        current_monster(e)?.wrapping_add((n as i32 as u32).wrapping_mul(4));
                    if e.b(address + 0x93)? == 0 {
                        for offset in 0x93..=0x96 {
                            let value = e.b(current_monster(e)? + offset)?;
                            e.byte(address + offset, value)?;
                        }
                    }
                    n = n.wrapping_add(1);
                    e.registers.data[5].assign_word(n as u16);
                }
            }
            call(frame, 0x13154c, 0x126de0, &[W(0x27)], 0, e)?;
            call(frame, 0x11f1a8, 0x126df6, &[W(turn(e)? as u16)], 0, e)?;
            call(frame, 0x12946c, 0x126e0c, &[W(turn(e)? as u16)], 0, e)?;
            let in_window = e.d0_word()? != 0;
            e.stack = e.stack.wrapping_add(6);
            let args = if in_window {
                [W(e.w(0x408572)?), W(e.w(0x408570)?)]
            } else {
                [
                    W(e.w(indexed(0x446f0c, e.sw(0x447196)?, 4))?),
                    W(e.w(indexed(0x446f0a, e.sw(0x447196)?, 4))?),
                ]
            };
            call(
                frame,
                0x10e6b6,
                if in_window { 0x126e4e } else { 0x126e38 },
                &args,
                4,
                e,
            )?;
            let number = frame.wrapping_sub(0x24);
            let ext = frame.wrapping_sub(0x1a);
            e.word(number, e.w(current_monster(e)? + 0x110)?)?;
            sane(&[number, number], 0x200e, e)?;
            sane(&[number, number], 0x1010, e)?;
            e.long(frame.wrapping_sub(8), e.l(number)?)?;
            e.word(number, e.w(current_monster(e)? + 0x110)?)?;
            sane(&[number, number], 0x200e, e)?;
            sane(&[number, number], 0x1010, e)?;
            e.long(frame.wrapping_sub(12), e.l(number)?)?;
            sane(&[frame.wrapping_sub(8), number], 0x100e, e)?;
            sane(&[frame.wrapping_sub(12), number], 0x1006, e)?;
            e.long(ext, 0x4005c800)?;
            e.long(ext + 4, 0)?;
            e.word(ext + 8, 0)?;
            sane(&[number, ext], 4, e)?;
            sane(&[ext, frame.wrapping_sub(16)], 0x1010, e)?;
            e.long(frame.wrapping_sub(4), e.l(frame.wrapping_sub(16))?)?;
            e.word(ext, e.w(current_monster(e)? + 0x46)?)?;
            sane(&[ext, ext], 0x200e, e)?;
            sane(&[ext, ext], 0x1010, e)?;
            sane(&[frame.wrapping_sub(4), number], 0x100e, e)?;
            if !ge(sane(&[ext, number], 0x1008, e)?) {
                let selector = if e.w(current_monster(e)? + 0x46)? == 101 {
                    -8i16
                } else {
                    -7i16
                };
                call(
                    frame,
                    0x158454,
                    if selector == -8 { 0x126f90 } else { 0x126fa8 },
                    &[W(selector as u16), W(e.w(0x447196)?.wrapping_add(10))],
                    4,
                    e,
                )?;
                e.word(current_monster(e)? + 0x110, 0)?;
                call(frame, 0x105572, 0x126fd8, &[W(0), W(turn(e)? as u16)], 0, e)?;
                call(frame, 0x105366, 0x126fee, &[W(turn(e)? as u16)], 0, e)?;
                let active = e.w(current_monster(e)? + 0xe)? != 0;
                e.stack = e.stack.wrapping_add(6);
                if active {
                    e.addw(0x44374c, 1)?;
                }
                continue 'prepare;
            }
            e.word(ext, e.w(current_monster(e)? + 0x44)?)?;
            sane(&[ext, ext], 0x200e, e)?;
            sane(&[ext, ext], 0x1010, e)?;
            sane(&[frame.wrapping_sub(4), number], 0x100e, e)?;
            if !ge(sane(&[ext, number], 0x1008, e)?) {
                e.byte(current_monster(e)? + 0xb2, 0xff)?;
                call(
                    frame,
                    0x158454,
                    0x12708e,
                    &[W((-6i16) as u16), W(e.w(0x447196)?.wrapping_add(10))],
                    4,
                    e,
                )?;
            }
            return finish_turn(frame, e);
        }
    }
}

fn setup_monster(e: &Execution<'_>) -> application::Result<u32> {
    Ok(monster(e.w(0x43874a)?))
}
fn encounter_cell(e: &Execution<'_>) -> application::Result<u32> {
    Ok(0x43979cu32
        .wrapping_add(u32::from(e.w(0x4434aa)?).wrapping_mul(52))
        .wrapping_add((e.sw(0x4434ac)? as i32 as u32).wrapping_mul(2)))
}
fn frame_at(frame: u32, offset: u32) -> u32 {
    frame.wrapping_sub(offset)
}
fn extended(
    address: u32,
    first: u32,
    second: u32,
    last: u16,
    e: &mut Execution<'_>,
) -> application::Result<()> {
    e.long(address, first)?;
    e.long(address.wrapping_add(4), second)?;
    e.word(address.wrapping_add(8), last)
}
fn retained_random(bound: i16, next: u32, e: &mut Execution<'_>) -> application::Result<i16> {
    let value = random(bound, next, e)?;
    e.stack = e.stack.wrapping_sub(2);
    Ok(value)
}

fn setup_orientation(frame: u32, e: &mut Execution<'_>) -> application::Result<()> {
    let ext = frame_at(frame, 0x186);
    let work = frame_at(frame, 0x17c);
    let other = frame_at(frame, 0x16e);
    let angle = retained_random(360, 0x11d5c4, e)?;
    e.word(ext, angle as u16)?;
    sane(&[ext, ext], 0x200e, e)?;
    sane(&[ext, ext], 0x1010, e)?;
    e.long(frame_at(frame, 0x12c), e.l(ext)?)?;
    extended(ext, 0x4004e533, 0x33333333, 0x3333, e)?;
    sane(&[frame_at(frame, 0x12c), work], 0x100e, e)?;
    sane(&[ext, work], 6, e)?;
    sane(&[work, frame_at(frame, 0x172)], 0x1010, e)?;
    e.long(frame_at(frame, 0x130), e.l(frame_at(frame, 0x172))?)?;
    let radius = retained_random(i16::from(e.sb(0x439cf6)?), 0x11d640, e)?;
    e.byte(0x439cf6, radius as u8)?;
    sane(&[frame_at(frame, 0x130), work], 0x100e, e)?;
    library(
        0x400190,
        0x11d66a,
        &[
            L(e.l(work + 6)?),
            L(e.l(work + 2)?),
            W(e.w(work)?),
            P(frame_at(frame, 0x14e)),
        ],
        0,
        e,
    )?;
    e.word(ext, i16::from(e.sb(0x439cf6)?) as u16)?;
    sane(&[ext, ext], 0x200e, e)?;
    sane(&[frame_at(frame, 0x14e), ext], 4, e)?;
    extended(other, 0xc001a000, 0, 0, e)?;
    sane(&[ext, other], 0, e)?;
    e.copy(other, frame_at(frame, 0x13a), 10)?;
    sane(&[frame_at(frame, 0x130), other], 0x100e, e)?;
    library(
        0x400198,
        0x11d6e6,
        &[
            L(e.l(other + 6)?),
            L(e.l(other + 2)?),
            W(e.w(other)?),
            P(frame_at(frame, 0x158)),
        ],
        0,
        e,
    )?;
    e.word(work, i16::from(e.sb(0x439cf6)?) as u16)?;
    sane(&[work, work], 0x200e, e)?;
    sane(&[frame_at(frame, 0x158), work], 4, e)?;
    extended(ext, 0xc001a000, 0, 0, e)?;
    sane(&[work, ext], 0, e)?;
    e.copy(ext, frame_at(frame, 0x144), 10)?;
    e.copy(frame_at(frame, 0x13a), other, 10)?;
    sane(&[other], 0x16, e)?;
    sane(&[other, other], 0x2810, e)?;
    e.registers.data[5].assign_word(e.w(other + 2)?);
    e.copy(frame_at(frame, 0x144), other, 10)?;
    sane(&[other], 0x16, e)?;
    sane(&[other, other], 0x2810, e)?;
    e.registers.data[6].assign_word(e.w(other + 2)?);
    e.byte(0x4386fc, 0)?;
    e.word(0x43877c, 0)?;
    e.word(0x4435c4, 0)?;
    e.word(0x43874a, 0)?;
    e.byte(frame_at(frame, 0x127), 0)?;
    e.registers.data[4] = RegisterBits::full(1);
    e.byte(frame_at(frame, 0x121), 0xff)?;
    let compare = frame_at(frame, 0x172);
    e.long(compare, 0x42340000)?;
    sane(&[frame_at(frame, 0x12c), other], 0x100e, e)?;
    let first = sane(&[compare, other], 0x1008, e)?;
    e.stack = e.stack.wrapping_add(62);
    let mut perpendicular = !ge(first);
    if !perpendicular {
        e.long(compare, 0x439d8000)?;
        sane(&[frame_at(frame, 0x12c), other], 0x100e, e)?;
        perpendicular = gt(sane(&[compare, other], 0x1008, e)?);
        if !perpendicular {
            e.long(compare, 0x43070000)?;
            sane(&[frame_at(frame, 0x12c), other], 0x100e, e)?;
            if gt(sane(&[compare, other], 0x1008, e)?) {
                e.long(compare, 0x43610000)?;
                sane(&[frame_at(frame, 0x12c), other], 0x100e, e)?;
                perpendicular = !ge(sane(&[compare, other], 0x1008, e)?);
            }
        }
    }
    if perpendicular {
        e.byte(frame_at(frame, 0x127), 1)?;
    }
    e.long(compare, 0x43340000)?;
    sane(&[frame_at(frame, 0x12c), other], 0x100e, e)?;
    if !ge(sane(&[compare, other], 0x1008, e)?) {
        e.registers.data[4] = RegisterBits::full(u32::MAX);
    }
    e.long(compare, 0x42b40000)?;
    sane(&[frame_at(frame, 0x12c), other], 0x100e, e)?;
    if gt(sane(&[compare, other], 0x1008, e)?) {
        e.long(compare, 0x43870000)?;
        sane(&[frame_at(frame, 0x12c), other], 0x100e, e)?;
        if !ge(sane(&[compare, other], 0x1008, e)?) {
            e.byte(frame_at(frame, 0x121), 1)?;
        }
    }
    Ok(())
}

fn monster_terrain(seeded: bool, e: &mut Execution<'_>) -> application::Result<Option<u32>> {
    let reject = if seeded { 0x11e0c0 } else { 0x11e53c };
    if e.sw(setup_monster(e)? + 0x11e)? <= 999 {
        return Ok(Some(reject));
    }
    let tile = e.w(setup_monster(e)? + 0x11e)?;
    if e.w(0x43167cu32
        .wrapping_add(u32::from(tile.wrapping_sub(1000)).wrapping_mul(40))
        .wrapping_add(4))?
        != 0
        && e.w(setup_monster(e)? + 0x10)? == 0
    {
        return Ok(Some(reject));
    }
    if e.sw(setup_monster(e)? + 0x11e)? < 1000 {
        return Ok(Some(reject));
    }
    let tile = e.w(setup_monster(e)? + 0x11e)?;
    if e.w(0x43167cu32
        .wrapping_add(u32::from(tile.wrapping_sub(1000)).wrapping_mul(40))
        .wrapping_add(4))?
        == 2
    {
        return Ok(Some(reject));
    }
    if e.w(0x4435ce)? == 1 || e.w(0x4435ce)? == 3 {
        if e.sw(setup_monster(e)? + 0x11c)? < 1000 {
            return Ok(Some(0x11e53c));
        }
        let tile = e.w(setup_monster(e)? + 0x11c)?;
        if e.sw(0x43167cu32
            .wrapping_add(u32::from(tile.wrapping_sub(1000)).wrapping_mul(40))
            .wrapping_add(4))?
            > 1
        {
            return Ok(Some(reject));
        }
    }
    if e.w(0x4435ce)? == 3 {
        if e.sw(setup_monster(e)? + 0x118)? < 1000 {
            return Ok(Some(0x11e53c));
        }
        let tile = e.w(setup_monster(e)? + 0x118)?;
        if e.sw(0x43167cu32
            .wrapping_add(u32::from(tile.wrapping_sub(1000)).wrapping_mul(40))
            .wrapping_add(4))?
            > 1
        {
            return Ok(Some(reject));
        }
    }
    if e.sw(0x4435ce)? > 1 {
        if e.sw(setup_monster(e)? + 0x11a)? < 1000 {
            return Ok(Some(0x11e53c));
        }
        let tile = e.w(setup_monster(e)? + 0x11a)?;
        if e.sw(0x43167cu32
            .wrapping_add(u32::from(tile.wrapping_sub(1000)).wrapping_mul(40))
            .wrapping_add(4))?
            > 1
        {
            return Ok(Some(reject));
        }
    }
    Ok(None)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlacementCompletion {
    Seeded,
    Encounter,
}

fn place_setup_monster(
    frame: u32,
    mut seeded: bool,
    e: &mut Execution<'_>,
) -> application::Result<PlacementCompletion> {
    e.word(frame_at(frame, 0x124), 0xffff)?;
    e.word(setup_monster(e)? + 0x10c, 0xffff)?;
    e.byte(setup_monster(e)? + 0x82, 0)?;
    e.byte(setup_monster(e)? + 0x104, 0)?;
    e.registers.data[7] = RegisterBits::full(1);
    let size = absolute(
        e.sw(setup_monster(e)? + 0x10)?,
        if seeded { 0x11de64 } else { 0x11e238 },
        0,
        e,
    )?;
    e.word(0x4435ce, size)?;
    e.stack = e.stack.wrapping_add(2);
    loop {
        e.step()?;
        let mut y = e.sw(frame_at(frame, 0x124))?;
        e.registers.data[4].assign_word(y as u16);
        while y < e.registers.data[7].word("D7")? as i16 {
            e.step()?;
            let mut x = e.sw(frame_at(frame, 0x124))?;
            e.registers.data[3].assign_word(x as u16);
            while x < e.registers.data[7].word("D7")? as i16 {
                e.step()?;
                let (h, v) = if seeded {
                    (
                        e.w(0x44709a)?.wrapping_add((x as u16).wrapping_mul(2)),
                        e.w(0x44709c)?.wrapping_add((y as u16).wrapping_mul(2)),
                    )
                } else {
                    let h = e
                        .w(0x4434aa)?
                        .wrapping_mul(3)
                        .wrapping_add(e.registers.data[5].word("D5")?)
                        .wrapping_add(x as u16);
                    let v = e
                        .w(0x4434ac)?
                        .wrapping_mul(3)
                        .wrapping_add(e.registers.data[6].word("D6")?)
                        .wrapping_add(y as u16);
                    (
                        h.wrapping_add(3)
                            .wrapping_add(e.w(0x439582)?.wrapping_mul(2)),
                        v.wrapping_add(3)
                            .wrapping_add(e.w(0x439582)?.wrapping_mul(2)),
                    )
                };
                if seeded {
                    e.word(0x4471f0, h)?;
                    e.word(0x4471ee, v)?;
                } else {
                    // Native extends the summed word before adding world origins.
                    let hp = h
                        .wrapping_sub(3)
                        .wrapping_sub(e.w(0x439582)?.wrapping_mul(2));
                    let vp = v
                        .wrapping_sub(3)
                        .wrapping_sub(e.w(0x439582)?.wrapping_mul(2));
                    e.word(
                        0x4471f0,
                        e.l(0x442780)?
                            .wrapping_add(hp as i16 as i32 as u32)
                            .wrapping_add(3)
                            .wrapping_add((e.w(0x439582)?.wrapping_mul(2) as i16) as i32 as u32)
                            as u16,
                    )?;
                    e.word(
                        0x4471ee,
                        e.l(0x442784)?
                            .wrapping_add(vp as i16 as i32 as u32)
                            .wrapping_add(3)
                            .wrapping_add((e.w(0x439582)?.wrapping_mul(2) as i16) as i32 as u32)
                            as u16,
                    )?;
                }
                if pascal(0xa8ad, &[L(e.l(0x4471ee)?), P(frame_at(frame, 8))], 2, e)? as u8 != 0 {
                    let index = e.sw(0x43874a)?;
                    e.word(indexed(0x446f0a, index, 4), h)?;
                    e.word(indexed(0x446f0c, index, 4), v)?;
                    if !seeded && e.w(setup_monster(e)? + 0x10)? == 0 {
                        let amount = retained_random(3, 0x11e324, e)?.wrapping_sub(2);
                        e.addw(indexed(0x446f0a, e.sw(0x43874a)?, 4), amount)?;
                        let amount = retained_random(3, 0x11e342, e)?.wrapping_sub(2);
                        e.addw(indexed(0x446f0c, e.sw(0x43874a)?, 4), amount)?;
                        e.stack = e.stack.wrapping_add(4);
                    }
                    call(
                        frame,
                        0x149e0e,
                        if seeded { 0x11df04 } else { 0x11e384 },
                        &[
                            W(e.w(0x43874a)?),
                            W(e.w(indexed(0x446f0c, e.sw(0x43874a)?, 4))?),
                            W(e.w(indexed(0x446f0a, e.sw(0x43874a)?, 4))?),
                        ],
                        6,
                        e,
                    )?;
                    if let Some(next) = monster_terrain(seeded, e)? {
                        call(
                            frame,
                            0x105366,
                            next,
                            &[W(e.w(0x43874a)?.wrapping_add(10))],
                            2,
                            e,
                        )?;
                        // Three seeded branches enter the encounter retry loop.
                        if seeded && next == 0x11e53c {
                            seeded = false;
                        }
                    } else {
                        return Ok(if seeded {
                            PlacementCompletion::Seeded
                        } else {
                            PlacementCompletion::Encounter
                        });
                    }
                }
                x = x.wrapping_add(1);
                e.registers.data[3].assign_word(x as u16);
            }
            y = y.wrapping_add(1);
            e.registers.data[4].assign_word(y as u16);
        }
        e.addw(frame_at(frame, 0x124), -1)?;
        e.registers.data[7].assign_word(e.registers.data[7].word("D7")?.wrapping_add(1));
    }
}

fn setup_monster_stats(frame: u32, e: &mut Execution<'_>) -> application::Result<()> {
    e.byte(setup_monster(e)? + 0x107, 1)?;
    e.byte(setup_monster(e)? + 0x108, 1)?;
    if e.w(setup_monster(e)? + 8)? != 0 {
        let amount = random(3, 0x11e5a8, e)?.wrapping_sub(2);
        e.addw(setup_monster(e)? + 8, amount)?;
    }
    if e.sw(setup_monster(e)? + 8)? < 0 {
        e.word(setup_monster(e)? + 8, 0)?;
    }
    let amount = random(3, 0x11e5f6, e)?.wrapping_sub(2);
    e.addw(setup_monster(e)? + 4, amount)?;
    if e.sw(setup_monster(e)? + 4)? < 1 {
        e.word(setup_monster(e)? + 4, 1)?;
    }
    let margin = e.sw(setup_monster(e)? + 0x52)? / 10;
    e.word(frame_at(frame, 10), margin as u16)?;
    e.long(e.stack.wrapping_sub(8), 0x11e66e)?;
    // RandRange, CODE 1 entry 0x13849e.
    let amount = e.range(margin.wrapping_neg(), margin, e.stack)?;
    e.addw(setup_monster(e)? + 0x52, amount)?;
    e.word(setup_monster(e)? + 0x10e, e.w(setup_monster(e)? + 0x52)?)?;
    e.word(setup_monster(e)? + 0x110, e.w(setup_monster(e)? + 2)?)?;
    e.byte(frame_at(frame, 11), 0)?;
    while i16::from(e.sb(frame_at(frame, 11))?) < e.sw(setup_monster(e)?)? {
        e.step()?;
        let amount = random(8, 0x11e6e6, e)?;
        e.addw(setup_monster(e)? + 0x110, amount)?;
        e.byte(
            frame_at(frame, 11),
            e.b(frame_at(frame, 11))?.wrapping_add(1),
        )?;
    }
    e.word(frame_at(frame, 16), e.w(setup_monster(e)? + 0x4e)?)?;
    if e.sw(setup_monster(e)? + 10)? < 99 {
        e.addw(setup_monster(e)? + 10, e.sw(0x442576)?.wrapping_mul(3))?;
    }
    e.byte(frame_at(frame, 11), 0)?;
    while e.sb(frame_at(frame, 11))? < 6 {
        e.step()?;
        let address = setup_monster(e)?
            .wrapping_add(e.sb(frame_at(frame, 11))? as i32 as u32)
            .wrapping_add(0x8b);
        e.byte(
            address,
            e.b(address)?
                .wrapping_add((e.w(0x442576)?.wrapping_mul(10)) as u8),
        )?;
        e.byte(
            frame_at(frame, 11),
            e.b(frame_at(frame, 11))?.wrapping_add(1),
        )?;
    }
    e.addw(setup_monster(e)? + 8, e.sw(0x442576)?.wrapping_mul(-3))?;
    e.addw(setup_monster(e)? + 4, e.sw(0x442576)?)?;
    let ext = frame_at(frame, 0x186);
    let work = frame_at(frame, 0x17c);
    let other = frame_at(frame, 0x16e);
    let integer = frame_at(frame, 0x162);
    e.word(other, e.w(0x442576)?)?;
    sane(&[other, other], 0x200e, e)?;
    extended(work, 0x3ffdcccc, 0xcccccccc, 0xcccd, e)?;
    sane(&[other, work], 4, e)?;
    extended(ext, 0x3fff8000, 0, 0, e)?;
    sane(&[work, ext], 0, e)?;
    let address = setup_monster(e)? + 0x10e;
    e.registers.address[0] = RegisterBits::full(address.wrapping_sub(0x10e));
    sane(&[address, integer], 0x200e, e)?;
    sane(&[ext, integer], 4, e)?;
    sane(&[integer], 0x16, e)?;
    sane(&[integer, integer], 0x2810, e)?;
    e.word(address, e.w(integer + 2)?)?;
    e.word(setup_monster(e)? + 0x52, e.w(setup_monster(e)? + 0x10e)?)?;
    e.word(integer, e.w(0x442576)?)?;
    sane(&[integer, integer], 0x200e, e)?;
    extended(other, 0x3ffdcccc, 0xcccccccc, 0xcccd, e)?;
    sane(&[integer, other], 4, e)?;
    extended(work, 0x3fff8000, 0, 0, e)?;
    sane(&[other, work], 0, e)?;
    let address = setup_monster(e)? + 0x110;
    e.registers.address[0] = RegisterBits::full(address.wrapping_sub(0x110));
    sane(&[address, ext], 0x200e, e)?;
    sane(&[work, ext], 4, e)?;
    sane(&[ext], 0x16, e)?;
    sane(&[ext, ext], 0x2810, e)?;
    e.word(address, e.w(ext + 2)?)?;
    if e.sw(setup_monster(e)? + 0x110)? < 1 {
        e.word(setup_monster(e)? + 0x110, 1)?;
    }
    e.word(setup_monster(e)? + 0x112, e.w(setup_monster(e)? + 0x110)?)?;
    if e.sw(encounter_cell(e)?)? < 0 {
        e.word(
            setup_monster(e)? + 0xe,
            u16::from(e.w(setup_monster(e)? + 0xe)? == 0),
        )?;
    }
    if e.sw(frame_at(frame, 16))? < 0 {
        let roll = retained_random(100, 0x11ea18, e)?;
        e.word(frame_at(frame, 0x126), roll as u16)?;
        let row = absolute(e.sw(frame_at(frame, 16))?, 0x11ea26, 0, e)?.wrapping_sub(1);
        e.word(frame_at(frame, 16), row)?;
        e.word(frame_at(frame, 10), 0)?;
        e.stack = e.stack.wrapping_add(4);
        while e.sw(frame_at(frame, 10))? < 5 {
            e.step()?;
            let offset = u32::from(e.w(frame_at(frame, 16))?)
                .wrapping_mul(30)
                .wrapping_add(u32::from(e.w(frame_at(frame, 10))?).wrapping_mul(6));
            let address = frame_at(frame, 0x11e).wrapping_add(offset);
            call(
                frame,
                0x13846a,
                0x11ea6c,
                &[
                    W(e.w(address + 2)?),
                    W(e.w(address)?),
                    W(e.w(frame_at(frame, 0x126))?),
                ],
                6,
                e,
            )?;
            if e.d0_word()? != 0 {
                e.word(setup_monster(e)? + 0x26, e.w(address + 4)?)?;
                e.word(setup_monster(e)? + 0x4e, e.w(address + 4)?)?;
            }
            e.addw(frame_at(frame, 10), 1)?;
        }
    }
    if e.w(frame_at(frame, 16))? == 0 && e.sw(setup_monster(e)? + 0x28)? > 0 {
        let value = e.w(setup_monster(e)? + 0x28)?;
        e.word(setup_monster(e)? + 0x4e, value)?;
        e.word(frame_at(frame, 16), value)?;
    }
    if e.w(setup_monster(e)? + 0xe)? != 0 {
        e.addw(0x4435c4, 1)?;
        if e.b(setup_monster(e)? + 0xa9)? != 0 || e.b(setup_monster(e)? + 0xaa)? != 0 {
            e.addw(0x43877c, 1)?;
        }
    }
    e.addw(0x43874a, 1)?;
    Ok(())
}

fn place_party(frame: u32, e: &mut Execution<'_>) -> application::Result<()> {
    e.word(0x4434aa, 0)?;
    while e.sw(0x4434aa)? <= i16::from(e.sb(0x44273a)?) {
        e.step()?;
        e.word(frame_at(frame, 0x124), 0xffff)?;
        e.registers.data[7] = RegisterBits::full(1);
        let n = e.w(0x4434aa)?;
        for offset in [0x16, 0xacc, 0xac8, 0x10] {
            e.word(warrior(n) + offset, 0)?;
        }
        e.byte(warrior(n) + 0x9de, 0)?;
        e.byte(warrior(n) + 0x9dd, 0)?;
        e.byte(warrior(n) + 0x9e0, 1)?;
        if e.w(warrior(e.w(0x4434aa)?) + 0xca)? != 0 {
            call(
                frame,
                0x130afe,
                0x11d9be,
                &[W(e.w(warrior(e.w(0x4434aa)?) + 0xca)?)],
                0,
                e,
            )?;
            e.word(warrior(e.w(0x4434aa)?) + 0x128, e.w(0x45364a)?)?;
            e.stack = e.stack.wrapping_add(2);
        } else {
            if e.w(warrior(e.w(0x4434aa)?) + 0xe4)? != 0 {
                e.byte(warrior(e.w(0x4434aa)?) + 0x9dd, 1)?;
            }
            e.word(
                warrior(e.w(0x4434aa)?) + 0x128,
                e.w(warrior(e.w(0x4434aa)?) + 0x38)?
                    .wrapping_shl(3)
                    .wrapping_add(30),
            )?;
        }
        'spiral: loop {
            e.step()?;
            e.word(frame_at(frame, 0x120), e.w(frame_at(frame, 0x124))?)?;
            while e.sw(frame_at(frame, 0x120))? < e.registers.data[7].word("D7")? as i16 {
                e.step()?;
                e.word(frame_at(frame, 14), e.w(frame_at(frame, 0x124))?)?;
                while e.sw(frame_at(frame, 14))? < e.registers.data[7].word("D7")? as i16 {
                    e.step()?;
                    let group = e.sw(0x4434aa)? / 3;
                    let coordinate = indexed(0x44709a, e.sw(0x4434aa)?, 4);
                    if e.b(frame_at(frame, 0x127))? == 0 {
                        let magnitude = absolute(group, 0x11daa0, 0, e)?;
                        let projected =
                            (e.sb(frame_at(frame, 0x121))? as i16).wrapping_mul(magnitude as i16);
                        e.word(frame_at(frame, 0x164), projected as u16)?;
                        let magnitude = absolute(group, 0x11dac0, 0, e)?;
                        let h = e
                            .w(0x439ce8)?
                            .wrapping_mul(3)
                            .wrapping_add(e.w(0x4434aa)?)
                            .wrapping_sub(magnitude.wrapping_mul(3))
                            .wrapping_add(e.w(frame_at(frame, 0x164))?)
                            .wrapping_add(e.w(frame_at(frame, 14))?);
                        e.word(coordinate, h)?;
                        let v = e
                            .w(0x439cea)?
                            .wrapping_mul(3)
                            .wrapping_add(
                                (group as u16).wrapping_mul(e.registers.data[4].word("D4")?),
                            )
                            .wrapping_add(e.w(frame_at(frame, 0x120))?);
                        e.word(coordinate + 2, v)?;
                        e.stack = e.stack.wrapping_add(4);
                    } else {
                        let h = e
                            .w(0x439ce8)?
                            .wrapping_mul(3)
                            .wrapping_add(
                                (group as u16)
                                    .wrapping_mul(e.sb(frame_at(frame, 0x121))? as i16 as u16),
                            )
                            .wrapping_add(e.w(frame_at(frame, 14))?);
                        e.word(coordinate, h)?;
                        let magnitude = absolute(group, 0x11db64, 0, e)?;
                        let projected = magnitude.wrapping_mul(e.registers.data[4].word("D4")?);
                        e.word(frame_at(frame, 0x164), projected)?;
                        let magnitude = absolute(group, 0x11db7e, 0, e)?;
                        let v = e
                            .w(0x439cea)?
                            .wrapping_mul(3)
                            .wrapping_add(e.w(0x4434aa)?)
                            .wrapping_sub(magnitude.wrapping_mul(3))
                            .wrapping_add(e.w(frame_at(frame, 0x164))?)
                            .wrapping_add(e.w(frame_at(frame, 0x120))?);
                        e.word(coordinate + 2, v)?;
                        e.stack = e.stack.wrapping_add(4);
                    }
                    e.word(
                        0x4471f0,
                        e.l(0x442780)?.wrapping_add(e.sw(coordinate)? as i32 as u32) as u16,
                    )?;
                    e.word(
                        0x4471ee,
                        e.l(0x442784)?
                            .wrapping_add(e.sw(coordinate + 2)? as i32 as u32)
                            as u16,
                    )?;
                    if pascal(0xa8ad, &[L(e.l(0x4471ee)?), P(frame_at(frame, 8))], 2, e)? as u8 != 0
                    {
                        let address = 0x4085b8u32
                            .wrapping_add(u32::from(e.w(0x4471f0)?).wrapping_mul(180))
                            .wrapping_add((e.sw(0x4471ee)? as i32 as u32).wrapping_mul(2));
                        let tile = e.w(address)?;
                        e.word(frame_at(frame, 10), tile)?;
                        if tile as i16 > 999
                            && e.w(0x43167cu32
                                .wrapping_add(u32::from(tile.wrapping_sub(1000)).wrapping_mul(40))
                                .wrapping_add(4))?
                                == 0
                        {
                            e.word(indexed(0x437a6e, e.sw(0x4434aa)?, 2), tile)?;
                            e.word(address, e.w(0x4434aa)?)?;
                            break 'spiral;
                        }
                    }
                    e.addw(frame_at(frame, 14), 1)?;
                }
                e.addw(frame_at(frame, 0x120), 1)?;
            }
            e.addw(frame_at(frame, 0x124), -1)?;
            e.registers.data[7].assign_word(e.registers.data[7].word("D7")?.wrapping_add(1));
        }
        let n = e.w(0x4434aa)?;
        e.word(warrior(n) + 0x18, 0xffff)?;
        e.word(warrior(n) + 0x14, 0)?;
        e.word(warrior(n) + 0x40, 0)?;
        if e.sw(warrior(n) + 0x50)? > 0 {
            call(frame, 0x136994, 0x11dcee, &[W(e.w(0x4434aa)?)], 0, e)?;
            e.word(
                warrior(e.w(0x4434aa)?) + 0x3c,
                e.w(warrior(e.w(0x4434aa)?) + 0x3e)?,
            )?;
            e.stack = e.stack.wrapping_add(2);
        }
        e.addw(0x4434aa, 1)?;
    }
    Ok(())
}

pub fn combat_setup_body(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    e.stack = frame.wrapping_sub(0x19e);
    e.registers.address[2] = RegisterBits::full(0);
    e.byte(frame_at(frame, 0x127), 1)?;
    e.copy(0x403ac6, frame_at(frame, 0x11e), 270)?;
    for address in [0x439450, 0x439430, 0x439474] {
        pascal(0xa958, &[L(e.l(address)?)], 0, e)?;
    }
    e.byte(0x435658, 0)?;
    for address in [0x438778, 0x4435d2, 0x43800c, 0x437a90] {
        e.word(address, 0)?;
    }
    e.byte(0x438584, 0)?;
    if e.w(frame.wrapping_add(8))? != 0 {
        let roll = retained_random(100, 0x11d47c, e)?;
        e.word(frame_at(frame, 0x126), roll as u16)?;
        let row = absolute(e.sw(frame.wrapping_add(8))?, 0x11d48a, 0, e)?.wrapping_sub(1);
        e.word(frame_at(frame, 16), row)?;
        e.word(frame_at(frame, 10), 0)?;
        e.stack = e.stack.wrapping_add(4);
        while e.sw(frame_at(frame, 10))? < 5 {
            e.step()?;
            let offset = u32::from(e.w(frame_at(frame, 10))?).wrapping_mul(6);
            let address = frame_at(frame, 0x11e)
                .wrapping_add(u32::from(e.w(frame_at(frame, 16))?).wrapping_mul(30))
                .wrapping_add(offset);
            call(
                frame,
                0x13846a,
                0x11d4ce,
                &[
                    W(e.w(address + 2)?),
                    W(e.w(address)?),
                    W(e.w(frame_at(frame, 0x126))?),
                ],
                6,
                e,
            )?;
            if e.d0_word()? != 0 {
                e.full((offset & 0xffff0000) | u32::from(e.w(address + 4)?));
                return Ok(());
            }
            e.addw(frame_at(frame, 10), 1)?;
        }
        e.moveq_return(0);
        return Ok(());
    }
    e.word(0x438776, 1)?;
    e.word(frame_at(frame, 6), 5)?;
    e.word(frame_at(frame, 8), 5)?;
    e.word(frame_at(frame, 4), 0x55)?;
    e.word(frame_at(frame, 2), 0x55)?;
    e.long(0x442780, e.sw(0x4427a8)?.wrapping_add(5) as i32 as u32)?;
    e.long(0x442784, e.sw(0x4427aa)?.wrapping_add(5) as i32 as u32)?;
    call(frame, 0x136cfa, 0x11d54c, &[P(0x403abe)], 0, e)?;
    let file = library(0x400070, 0x11d55e, &[P(0x403aba), P(0x4389a4)], 12, e)?;
    e.registers.address[0] = RegisterBits::full(file);
    e.registers.address[2] = RegisterBits::full(file);
    e.full(file);
    if file == 0 {
        call(frame, 0x138be8, 0x11d572, &[W(8)], 2, e)?;
    }
    push(&W(0), e)?;
    let offset = u32::from(absolute(e.sw(0x43874e)?, 0x11d582, 2, e)?).wrapping_mul(0x55c);
    seek(file, offset, 0x11d592, e)?;
    library(
        0x400048,
        0x11d5a8,
        &[L(file), L(1), L(0x55c), P(0x43979c)],
        0,
        e,
    )?;
    library(0x400068, 0x11d5b0, &[L(file)], 0, e)?;
    e.word(0x443314, e.w(0x439ce6)?)?;
    setup_orientation(frame, e)?;
    place_party(frame, e)?;
    e.word(0x4434aa, 0)?;
    while e.sw(0x4434aa)? < 100 {
        e.step()?;
        for offset in [0x11e, 0x11a, 0x11c, 0x118, 0x110] {
            e.word(monster(e.w(0x4434aa)?) + offset, 0)?;
        }
        e.addw(0x4434aa, 1)?;
    }
    let mut encounter_continuation = false;
    if e.w(0x408578)? == 0 && e.w(0x44257c)? != 0 {
        e.word(0x4434aa, 0)?;
        while e.sw(0x4434aa)? < e.sw(0x44257c)? {
            e.step()?;
            e.copy(
                0x440edcu32.wrapping_add(u32::from(e.w(0x4434aa)?).wrapping_mul(0x120)),
                setup_monster(e)?,
                288,
            )?;
            if place_setup_monster(frame, true, e)? == PlacementCompletion::Encounter {
                encounter_continuation = true;
                break;
            }
            e.addw(0x43874a, 1)?;
            // Retain the original no-op word read/write.
            e.word(frame_at(frame, 10), e.w(frame_at(frame, 10))?)?;
            e.addw(0x4434aa, 1)?;
        }
    }
    let file = if encounter_continuation {
        // This route bypasses the second open and retains the first handle.
        setup_monster_stats(frame, e)?;
        e.byte(frame_at(frame, 0x127), 0)?;
        e.addw(0x4434aa, 1)?;
        file
    } else {
        call(frame, 0x136cfa, 0x11e10a, &[P(0x403ab2)], 0, e)?;
        let next_file = library(0x400070, 0x11e11c, &[P(0x403aba), P(0x4389a4)], 0, e)?;
        e.registers.address[0] = RegisterBits::full(next_file);
        e.registers.address[2] = RegisterBits::full(next_file);
        e.word(0x4434ac, 0)?;
        e.stack = e.stack.wrapping_add(12);
        next_file
    };
    while encounter_continuation || e.sw(0x4434ac)? < 26 {
        e.step()?;
        if !encounter_continuation {
            e.word(0x4434aa, 0)?;
        }
        encounter_continuation = false;
        while e.sw(0x4434aa)? < 26 {
            e.step()?;
            if e.w(encounter_cell(e)?)? != 0 && e.sw(0x43874a)? < 100 {
                push(&W(0), e)?;
                let offset = u32::from(absolute(e.sw(encounter_cell(e)?)?, 0x11e182, 2, e)?)
                    .wrapping_mul(288);
                seek(file, offset, 0x11e192, e)?;
                library(
                    0x400048,
                    0x11e1b6,
                    &[L(file), L(1), L(288), P(setup_monster(e)?)],
                    0,
                    e,
                )?;
                let populated = e.w(setup_monster(e)?)? != 0;
                e.stack = e.stack.wrapping_add(26);
                if populated {
                    place_setup_monster(frame, false, e)?;
                    setup_monster_stats(frame, e)?;
                }
            }
            e.byte(frame_at(frame, 0x127), 0)?;
            e.addw(0x4434aa, 1)?;
        }
        e.addw(0x4434ac, 1)?;
    }
    e.word(0x43874c, e.w(0x4435c4)?)?;
    e.word(0x4434aa, 0)?;
    while e.sw(0x4434aa)? < e.sw(0x43874a)? {
        e.step()?;
        let h = e.sw(indexed(0x446f0a, e.sw(0x4434aa)?, 4))?;
        if h < e.sw(0x44709a)? {
            e.byte(monster(e.w(0x4434aa)?) + 0x107, 1)?;
        } else if e.sw(indexed(0x446f0a, e.sw(0x4434aa)?, 4))? > e.sw(0x44709a)? {
            e.byte(monster(e.w(0x4434aa)?) + 0x107, 0xff)?;
        }
        e.addw(0x4434aa, 1)?;
    }
    call(
        frame,
        0x124d9e,
        0x11ec18,
        &[W(e.w(frame.wrapping_add(10))?)],
        0,
        e,
    )?;
    e.word(0x447194, 0)?;
    e.word(0x4434aa, 0)?;
    e.stack = e.stack.wrapping_add(2);
    while e.sw(0x4434aa)? <= i16::from(e.sb(0x44273a)?) {
        e.step()?;
        e.byte(warrior(e.w(0x4434aa)?) + 0x9df, 1)?;
        if e.sw(warrior(e.w(0x4434aa)?) + 0x50)? < 1
            || (e.w(frame.wrapping_add(12))? != 0
                && e.w(indexed(0x4434e0, e.sw(0x4434aa)?, 2))? == 0)
        {
            call(frame, 0x105366, 0x11ec82, &[W(e.w(0x4434aa)?)], 0, e)?;
            e.byte(warrior(e.w(0x4434aa)?) + 0x9df, 0)?;
            e.word(
                indexed(0x4470b8, e.sw(warrior(e.w(0x4434aa)?) + 0x20)?, 2),
                0xffff,
            )?;
            e.word(warrior(e.w(0x4434aa)?) + 0x20, 0xffff)?;
            e.word(indexed(0x44709c, e.sw(0x4434aa)?, 4), 0xffff)?;
            e.word(indexed(0x44709a, e.sw(0x4434aa)?, 4), 0xffff)?;
            e.stack = e.stack.wrapping_add(2);
        }
        call(frame, 0x160fb2, 0x11ed02, &[W(0), W(e.w(0x4434aa)?)], 0, e)?;
        e.addw(0x4434aa, 1)?;
        e.stack = e.stack.wrapping_add(4);
    }
    e.word(0x44257c, 0)?;
    let actor = e.sw(0x4470ba)?;
    let (base, index, next) = if actor < 10 {
        (0x44709a, actor, 0x11ed52)
    } else {
        (0x446f0a, actor.wrapping_sub(10), 0x11ed80)
    };
    call(
        frame,
        0x10e6b6,
        next,
        &[
            W(e.w(indexed(base + 2, index, 4))?),
            W(e.w(indexed(base, index, 4))?),
        ],
        4,
        e,
    )?;
    pascal(0xa93a, &[L(e.l(0x43870a)?), W(3)], 0, e)?;
    pascal(0xa93a, &[L(e.l(0x43870a)?), W(4)], 0, e)?;
    library(0x400068, 0x11eda2, &[L(file)], 0, e)?;
    call(frame, 0x125a3a, 0x11edaa, &[W(0)], 0, e)?;
    e.moveq_return(0);
    e.stack = e.stack.wrapping_add(6);
    Ok(())
}

crate::recovered_application_control::entry!(get_up, 0x125a3a);
crate::recovered_application_control::entry!(combat_setup, 0x11d406);
