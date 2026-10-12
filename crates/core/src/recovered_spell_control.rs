//! Spell application and natural condition decay from CODE 1.

use crate::recovered_application_control::{ApplicationControlServices, Execution, Result};
use crate::recovered_character_upgrade::UpgradeSaneRequest;
use crate::recovered_feedback::FeedbackArg;
use crate::recovered_prompt_control::PromptReturn;
use FeedbackArg::{Long as L, Pointer as P, Word as W};

macro_rules! invoke {
    ($e:expr,$sp:expr,$entry:expr,$words:expr,$cleanup:expr) => {{
        let words = $words.to_vec();
        call($e, $sp, $entry, &words, $cleanup)
    }};
}
macro_rules! absolute {
    ($e:expr,$sp:expr,$value:expr,$cleanup:expr) => {{
        let value = $value;
        abs($e, $sp, value, $cleanup)
    }};
}
macro_rules! play {
    ($e:expr,$sp:expr,$value:expr) => {{
        let value = $value;
        sound($e, $sp, value)
    }};
}
macro_rules! push_word {
    ($e:expr,$sp:expr,$value:expr) => {{
        let value = $value;
        push($e, $sp, value)
    }};
}
macro_rules! animate {
    ($e:expr,$sp:expr,$body:expr,$mode:expr) => {{
        let body = $body;
        let mode = $mode;
        spell_effect($e, $sp, body, mode)
    }};
}
macro_rules! report {
    ($e:expr,$sp:expr,$body:expr,$selector:expr,$attacker:expr) => {{
        let body = $body;
        let selector = $selector;
        let attacker = $attacker;
        show($e, $sp, body, selector, attacker)
    }};
}
macro_rules! message {
    ($e:expr,$sp:expr,$pointer:expr,$h:expr,$v:expr,$wait:expr,$sound:expr,$cleanup:expr) => {{
        let pointer = $pointer;
        let h = $h;
        let v = $v;
        let wait = $wait;
        let sound = $sound;
        flash($e, $sp, (pointer, h, v, wait, sound), $cleanup)
    }};
}

fn warrior(n: u16) -> u32 {
    0x448aa0u32.wrapping_add(u32::from(n) * 3138)
}
fn monster(n: u16) -> u32 {
    0x439d3cu32.wrapping_add(u32::from(n.wrapping_sub(10)) * 288)
}
fn at(base: u32, n: i16, stride: i32) -> u32 {
    base.wrapping_add(i32::from(n).wrapping_mul(stride) as u32)
}
fn cell(x: u16, y: i16) -> u32 {
    0x4085b8u32
        .wrapping_add(u32::from(x) * 180)
        .wrapping_add(i32::from(y).wrapping_mul(2) as u32)
}
fn acting<S: ApplicationControlServices>(e: &Execution<'_, S>) -> Result<i16> {
    e.sw(at(0x4470b8, e.sw(0x447194)?, 2))
}
fn push<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    n: u16,
) -> Result<()> {
    *sp = sp.wrapping_sub(2);
    e.word(*sp, n)
}
fn args<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    words: &[u16],
) -> Result<()> {
    for &word in words.iter().rev() {
        push_word!(e, sp, word)?;
    }
    Ok(())
}
fn call<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    entry: u32,
    words: &[u16],
    cleanup: u32,
) -> Result<()> {
    args(e, sp, words)?;
    e.stack = *sp;
    e.call(entry, sp.wrapping_sub(8))?;
    *sp = sp.wrapping_add(cleanup);
    e.stack = *sp;
    Ok(())
}
fn abs<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    value: u16,
    cleanup: bool,
) -> Result<u16> {
    push_word!(e, sp, value)?;
    e.lib(0x400038, vec![W(value)])?;
    if cleanup {
        *sp = sp.wrapping_add(2);
    }
    e.d0_word()
}
fn rectangle_trap<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    opcode: u16,
    pushes: &[FeedbackArg],
    arguments: Vec<FeedbackArg>,
) -> Result<()> {
    let initial_sp = *sp;
    for argument in pushes {
        match *argument {
            W(value) => {
                *sp = sp.wrapping_sub(2);
                e.word(*sp, value)?;
            }
            L(value) | P(value) => {
                *sp = sp.wrapping_sub(4);
                e.long(*sp, value)?;
            }
            FeedbackArg::Byte(value) => {
                *sp = sp.wrapping_sub(2);
                e.byte(*sp, value)?;
            }
        }
        e.stack = *sp;
    }
    e.trap(opcode, arguments)?;
    *sp = initial_sp;
    e.stack = *sp;
    Ok(())
}
fn sound<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    value: u16,
) -> Result<()> {
    invoke!(e, sp, 0x133a18, &[value], 2)
}
fn update<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    body: u16,
    mode: u16,
    short: bool,
) -> Result<()> {
    invoke!(
        e,
        sp,
        if short { 0x1600fc } else { 0x15f66c },
        &[body, mode],
        4
    )
}
fn show<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    body: u16,
    selector: u16,
    attacker: Option<u16>,
) -> Result<()> {
    match attacker {
        Some(a) => invoke!(e, sp, 0x158454, &[body, selector, a], 6),
        None => invoke!(e, sp, 0x158454, &[body, selector], 4),
    }
}
fn spell_effect<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    body: u16,
    mode: u16,
) -> Result<()> {
    invoke!(e, sp, 0x15b2f6, &[body, mode], 4)
}
fn flash<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    message: (u32, u16, u16, u16, u16),
    cleanup: u32,
) -> Result<()> {
    let (pointer, h, v, wait, sound_id) = message;
    invoke!(
        e,
        sp,
        0x123d1c,
        &[(pointer >> 16) as u16, pointer as u16, h, v, wait, sound_id],
        cleanup
    )
}

pub(crate) fn heal_body<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    frame: u32,
) -> Result<()> {
    let body = i16::from(e.sb(frame.wrapping_add(9))?) as u16;
    let mut sp = frame.wrapping_sub(20);
    e.stack = sp;
    if e.b(frame.wrapping_add(13))? != 0 {
        e.byte(0x443760, 4)?;
        animate!(e, &mut sp, body, 0)?;
    }
    let base = warrior(body);
    let was_alive = e.sw(base + 0x50)? > 0;
    let was_revivable = e.sw(base + 0x50)? >= -9;
    let was_unconscious = e.sw(base + 0x50)? < 1 && was_revivable;
    if was_revivable {
        e.addw(base + 0x50, e.sw(frame.wrapping_add(10))?)?;
    }
    if e.sw(base + 0x50)? < -10 {
        e.word(base + 0x50, (-10i16) as u16)?;
    }
    if e.sw(base + 0x50)? > 0 && was_unconscious {
        e.byte(0x442739, e.b(0x442739)?.wrapping_sub(1))?;
    }
    if e.sw(base + 0x50)? > e.sw(base + 0x52)? {
        e.word(base + 0x50, e.w(base + 0x52)?)?;
    }
    if e.sw(base + 0x50)? < -9 && was_revivable && !was_alive {
        invoke!(e, &mut sp, 0x1600fc, &[body, 1], 0)?;
        let n = e.rand(2, sp)?;
        play!(e, &mut sp, n.wrapping_add(506) as u16)?;
        sp = sp.wrapping_add(4);
        e.stack = sp;
    }
    if e.sw(base + 0x50)? < 1 && was_alive {
        invoke!(e, &mut sp, 0x12e4e4, &[body, 0], 4)?;
        if e.w(0x438776)? == 0 && e.w(0x447198)? == 0 {
            invoke!(e, &mut sp, 0x10f186, &[], 0)?;
        }
    } else {
        let mode = u16::from(e.sw(base + 0x50)? > 0 && !was_alive);
        update(e, &mut sp, body, mode, true)?;
    }
    invoke!(e, &mut sp, 0x13a498, &[], 0)?;
    let casualties = e.b(0x442739)?;
    e.registers.data[0].assign_byte(casualties);
    if e.sb(0x442739)? > e.sb(0x44273a)? {
        invoke!(e, &mut sp, 0x14913e, &[0], 2)?;
    }
    if e.b(0x442750)? != 0 {
        e.full(i32::from(body as i16 as i8) as u32);
        e.word(at(0x442798, body as i16, 2), u16::MAX)?;
    } else {
        let result = e.registers.data[0];
        if result.known_mask == u32::MAX {
            e.full(result.value);
        } else if result.known_mask & 0xffff == 0xffff {
            e.low(result.word("D0")?);
        } else {
            e.plan.native_return = PromptReturn::Unchanged;
        }
    }
    Ok(())
}

pub(crate) fn spell_targets_body<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    frame: u32,
) -> Result<()> {
    let target = e.w(frame + 8)?;
    let filter = e.sw(frame + 10)?;
    let scoring = e.w(frame + 12)? != 0;
    let disable_reflection = e.w(frame + 14)? != 0;
    let mut sp = frame.wrapping_sub(34);
    e.stack = sp;
    e.word(frame.wrapping_sub(2), 1)?;
    invoke!(e, &mut sp, 0x13154c, &[18], 2)?;
    for off in [10, 14, 8] {
        e.word(frame.wrapping_sub(off), 0)?;
    }
    for n in 0..110 {
        e.step()?;
        e.word(0x4434e0 + n * 2, 0)?;
    }
    if e.b(0x44374f)? == 4 {
        e.word(frame.wrapping_sub(2), e.w(0x4435cc)?)?;
    }
    if e.b(0x44374f)? == 3 {
        e.word(frame.wrapping_sub(2), u16::from(e.b(0x44376d)?))?;
    }
    if e.w(0x443776)? == 58 {
        e.word(
            frame.wrapping_sub(2),
            u16::from(e.b(0x44376d)?).wrapping_add(8199),
        )?;
    }
    if e.b(0x443755)? != 0 {
        e.addw(frame.wrapping_sub(2), e.sw(0x44331a)?)?;
    }
    if e.b(0x443752)? != 0 && !scoring {
        for n in 0..60 {
            e.step()?;
            let base = 0x442bd6 + n * 20;
            if e.b(base + 15)? != 0 {
                continue;
            }
            for (offset, source) in [(18, 0x447195), (16, 0x4435cd), (19, 0x4435c9)] {
                e.byte(base + offset, e.b(source)?)?;
            }
            e.byte(base + 17, 0)?;
            if e.b(0x443755)? != 0 {
                e.byte(base + 17, e.b(0x44331b)?)?;
            }
            e.byte(base + 14, e.b(frame.wrapping_sub(1))?)?;
            e.byte(base + 12, e.w(0x443722)?.wrapping_sub(3) as u8)?;
            e.byte(base + 13, e.w(0x443724)?.wrapping_sub(3) as u8)?;
            e.word(base, i16::from(e.sb(base + 13)?) as u16)?;
            e.word(base + 2, i16::from(e.sb(base + 12)?) as u16)?;
            e.word(base + 6, e.w(base + 2)?.wrapping_add(7))?;
            e.word(base + 4, e.w(base)?.wrapping_add(7))?;
            e.word(
                base + 8,
                e.w(0x4435c8)?
                    .wrapping_mul(100)
                    .wrapping_add(e.w(0x4435c6)?.wrapping_mul(1000))
                    .wrapping_add(e.w(0x4435ca)?)
                    .wrapping_add(1101),
            )?;
            let value = e.range(i16::from(e.sb(0x44375b)?), i16::from(e.sb(0x44375c)?), sp)?;
            e.byte(base + 15, value as u8)?;
            let mut i = 0i16;
            while i < e.sw(0x4435cc)? {
                e.step()?;
                let value = e.range(i16::from(e.sb(0x44375d)?), i16::from(e.sb(0x44375e)?), sp)?;
                e.byte(base + 15, e.b(base + 15)?.wrapping_add(value as u8))?;
                i = i.wrapping_add(1);
            }
            e.word(base + 10, i16::from(e.sb(0x443752)?) as u16)?;
            invoke!(e, &mut sp, 0x10e6b6, &[e.w(0x408570)?, e.w(0x408572)?], 4)?;
            break;
        }
    }
    let row = 0x443722 + u32::from(target) * 6;
    let (x, y) = if (e.sb(0x44374f)? > 1) && e.w(row + 4)? == 3 {
        (e.w(row)?.wrapping_sub(3), e.w(row + 2)?.wrapping_sub(3))
    } else {
        let body = e.sw(row)?;
        let pos = if e.w(row + 4)? == 2 {
            at(0x446f0a, body.wrapping_sub(10), 4)
        } else {
            at(0x44709a, body, 4)
        };
        (
            (e.sw(pos)?.wrapping_sub(3) as i32).wrapping_add(e.sl(0x442780)?) as u16,
            (e.sw(pos + 2)?.wrapping_sub(3) as i32).wrapping_add(e.sl(0x442784)?) as u16,
        )
    };
    e.word(frame.wrapping_sub(4), x)?;
    e.word(frame.wrapping_sub(6), y)?;
    for dy in 0u16..7 {
        for dx in 0u16..7 {
            e.step()?;
            let shape = u32::from(e.w(frame.wrapping_sub(2))?.wrapping_sub(1)) * 49;
            if e.b(0x438210 + shape + u32::from(dy) * 7 + u32::from(dx))? == 0 {
                continue;
            }
            let body = absolute!(
                e,
                &mut sp,
                e.w(cell(x.wrapping_add(dx), y.wrapping_add(dy) as i16))?,
                true
            )? as i16;
            if !((body < 110 && filter == -1) || body == filter) {
                continue;
            }
            if e.b(0x443761)? != 9 {
                if body < 9 && e.w(warrior(body as u16) + 0x9a)? != 0 {
                    let rolled = e.rand(100, sp)?;
                    if rolled < 34 && !disable_reflection {
                        e.word(at(0x4434e0, acting(e)?, 2), 1)?;
                        continue;
                    }
                }
                if body > 9 && e.b(monster(body as u16) + 0xd0)? != 0 {
                    let rolled = e.rand(100, sp)?;
                    if rolled < 34 && !disable_reflection {
                        e.word(at(0x4434e0, acting(e)?, 2), 1)?;
                        continue;
                    }
                }
            }
            let eligible = body <= 9 || e.sw(monster(body as u16) + 10)? <= 100;
            e.word(at(0x4434e0, body, 2), u16::from(eligible))?;
        }
    }
    if scoring {
        e.word(
            frame.wrapping_sub(12),
            e.w(0x439d3c + u32::from(e.w(0x447196)?) * 288 + 14)?,
        )?;
        let mut n = 0i16;
        while n <= i16::from(e.sb(0x44273a)?) {
            e.step()?;
            if e.w(at(0x4434e0, n, 2))? != 0 {
                if e.w(frame.wrapping_sub(12))? == e.w(warrior(n as u16) + 16)? {
                    e.addw(frame.wrapping_sub(10), 1)?;
                } else {
                    e.addw(frame.wrapping_sub(8), 1)?;
                    if e.w(warrior(n as u16) + 88)? != 0 {
                        e.addw(frame.wrapping_sub(8), 1)?;
                    }
                }
            }
            n = n.wrapping_add(1);
        }
        for n in 10i16..60 {
            e.step()?;
            if e.w(at(0x4434e0, n, 2))? != 0 {
                let base = monster(n as u16);
                if e.w(frame.wrapping_sub(12))? == e.w(base + 14)? {
                    e.addw(frame.wrapping_sub(10), 1)?;
                } else {
                    e.addw(
                        frame.wrapping_sub(8),
                        i16::from(e.sb(base + 168)?).wrapping_add(1),
                    )?;
                }
            }
        }
        if e.b(0x443757)? == 4 {
            let value = e.w(frame.wrapping_sub(8))?;
            e.word(frame.wrapping_sub(8), e.w(frame.wrapping_sub(10))?)?;
            e.word(frame.wrapping_sub(10), value)?;
        }
        for n in 0..60 {
            e.step()?;
            e.word(0x4434e0 + n * 2, 0)?;
        }
        e.returned(PromptReturn::LowWord(
            e.w(frame.wrapping_sub(8))?
                .wrapping_sub(e.w(frame.wrapping_sub(10))?),
        ));
    } else {
        e.word(0x438760, 1)?;
        invoke!(e, &mut sp, 0x151022, &[], 0)?;
        for selected in [e.w(0x442bd4)?, target] {
            let base = 0x443722 + u32::from(selected) * 6;
            for offset in [4, 2, 0] {
                e.word(base + offset, u16::MAX)?;
            }
        }
        e.word(0x438760, 0)?;
        if acting(e)? < 10 {
            invoke!(e, &mut sp, 0x10e6b6, &[e.w(0x408570)?, e.w(0x408572)?], 4)?;
        }
        invoke!(e, &mut sp, 0x13154c, &[19], 2)?;
        e.moveq_return(0);
    }
    Ok(())
}

fn decay_result<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    sp: &mut u32,
    body: u16,
    selector: u16,
    message: u32,
    sound_id: u16,
) -> Result<()> {
    if e.sw(warrior(body) + 80)? < 1 {
        invoke!(e, sp, 0x12e4e4, &[body, 0], 0)?;
        message!(e, sp, message, 30, 100, 100, sound_id, 16)?;
    } else if e.w(0x438776)? != 0 && e.b(warrior(body) + 0x9df)? != 0 {
        report!(e, sp, body, selector, None)?;
    } else {
        update(e, sp, body, 0, true)?;
    }
    Ok(())
}
pub(crate) fn reduce_body<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    frame: u32,
) -> Result<()> {
    let mut sp = frame.wrapping_sub(36);
    e.stack = sp;
    let rect = frame.wrapping_sub(16);
    rectangle_trap(
        e,
        &mut sp,
        0xa8a7,
        &[P(rect), L(0x014a016f), L(0x016a018f)],
        vec![P(rect), W(0x014a), W(0x016f), W(0x016a), W(0x018f)],
    )?;
    for n in 1..10 {
        e.step()?;
        if e.w(0x442562 + n * 2)? == 1 && n != 5 {
            e.trap(0xa873, vec![L(e.l(0x43878c)?)])?;
            let shift = e.w(0x439582)?.wrapping_shl(5);
            for (off, value) in [
                (8, e.w(0x40856e)?.wrapping_add(321).wrapping_sub(shift)),
                (4, e.w(0x40856e)?.wrapping_add(460).wrapping_sub(shift)),
                (6, e.w(0x40856c)?.wrapping_add(308)),
                (2, e.w(0x40856c)?.wrapping_add(640)),
            ] {
                e.word(frame.wrapping_sub(off), value)?;
            }
            let pattern = e.l(0x439564)?;
            rectangle_trap(
                e,
                &mut sp,
                0xa8f6,
                &[L(pattern), P(frame.wrapping_sub(8))],
                vec![L(pattern), P(frame.wrapping_sub(8))],
            )?;
            invoke!(e, &mut sp, 0x16043e, &[], 0)?;
        }
        if n == 5 {
            rectangle_trap(
                e,
                &mut sp,
                0xa8a8,
                &[P(rect), L(0x28ff64)],
                vec![P(rect), W(40), W(0xff64)],
            )?;
        }
        rectangle_trap(
            e,
            &mut sp,
            0xa8a8,
            &[P(rect), L(39)],
            vec![P(rect), W(0), W(39)],
        )?;
    }
    for n in 0..10 {
        e.step()?;
        let address = 0x442562 + n * 2;
        if e.sw(address)? > 0 {
            e.addw(address, -1)?;
        }
    }
    if e.w(0x442562)? != 0 {
        e.addw(0x442562, -1)?;
    }
    let mut body = 0i16;
    while body <= i16::from(e.sb(0x44273a)?) {
        e.step()?;
        let base = warrior(body as u16);
        if e.w(base + 102)? != 0
            && e.sw(base + 80)? > -10
            && e.sw(base + 80)? < e.sw(base + 82)?
            && e.w(base + 144)? == 0
            && (e.w(0x438776)? == 0 || e.sw(base + 80)? > 0)
        {
            let value = absolute!(e, &mut sp, e.w(base + 102)?, false)?;
            e.word(0x4434b8, value)?;
            invoke!(e, &mut sp, 0x1287c2, &[body as u16, value, 0], 0)?;
            e.word(0x438766, 1)?;
            sp = sp.wrapping_add(8);
            e.stack = sp;
            if e.w(0x4434b8)? != 0 && e.w(0x438776)? != 0 && e.b(base + 0x9df)? != 0 {
                report!(e, &mut sp, body as u16, 0, None)?;
            } else {
                update(e, &mut sp, body as u16, 0, true)?;
            }
            e.word(0x438766, 0)?;
        }
        for (offset, selector, message, sound_id, requires_live) in [
            (150, 29, 0x401960, 10122, true),
            (138, 23, 0x401948, 10122, false),
            (140, 22, 0x40193a, 10122, false),
            (100, 0, 0x401922, 684, true),
        ] {
            if e.w(base + offset)? == 0
                || e.sw(base + 80)? <= 0
                || (requires_live && e.sw(base + 144)? <= -1)
            {
                continue;
            }
            if offset == 100 {
                e.byte(0x443760, 7)?;
            }
            let value = if offset == 140 {
                2
            } else {
                absolute!(e, &mut sp, e.w(base + offset)?, true)?
            };
            e.word(0x4434b8, value)?;
            e.addw(base + 80, (value as i16).wrapping_neg())?;
            if offset != 100 {
                e.byte(0x443760, 7)?;
            }
            if offset == 100 {
                e.word(0x438784, 1)?;
            }
            decay_result(e, &mut sp, body as u16, selector, message, sound_id)?;
            if offset == 100 {
                e.word(0x438784, 0)?;
            }
        }
        if e.w(base + 162)? != 0 && e.sw(base + 80)? > 0 {
            let value = absolute!(e, &mut sp, e.w(base + 162)?, true)?;
            e.addw(base + 88, (value as i16).wrapping_neg())?;
            if e.sw(base + 88)? < 0 {
                e.word(base + 88, 0)?;
            }
            update(e, &mut sp, body as u16, 0, true)?;
        }
        if e.w(base + 160)? != 0
            && e.w(base + 90)? != 0
            && e.sw(base + 80)? > 0
            && e.b(0x44274a)? == 0
        {
            let value = absolute!(e, &mut sp, e.w(base + 160)?, true)?;
            e.addw(base + 88, value as i16)?;
            if e.sw(base + 88)? > e.sw(base + 90)? {
                e.word(base + 88, e.w(base + 90)?)?;
            }
            update(e, &mut sp, body as u16, 0, true)?;
        }
        if e.w(base + 152)? != 0 && e.w(base + 16)? != 0 {
            e.word(base + 16, 0)?;
            e.byte(0x442739, e.b(0x442739)?.wrapping_sub(1))?;
            e.addw(0x4435c4, -1)?;
        }
        let mut expired = false;
        for n in 0..40 {
            e.step()?;
            let address = base + 94 + n * 2;
            if e.w(address)? == 1 {
                expired = true;
            }
            if e.sw(address)? > 0 {
                e.addw(address, -1)?;
            }
        }
        if expired {
            update(e, &mut sp, body as u16, 3, false)?;
        }
        for n in 0..6 {
            e.step()?;
            let address = 0x440f8e + u32::from(body as u16) * 288 + n;
            if e.sb(address)? > 0 {
                e.byte(address, e.b(address)?.wrapping_sub(1))?;
            }
        }
        body = body.wrapping_add(1);
    }
    e.moveq_return(0);
    Ok(())
}

fn sane<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    pointers: &[u32],
    selector: u16,
) -> Result<()> {
    let mut arguments = pointers.iter().copied().map(P).collect::<Vec<_>>();
    arguments.push(W(selector));
    e.sane(UpgradeSaneRequest {
        opcode: 0xa9eb,
        arguments,
    })?;
    Ok(())
}
fn summon_scale<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    frame: u32,
    base: u32,
) -> Result<()> {
    let a = frame.wrapping_sub(52);
    let constant = frame.wrapping_sub(42);
    let value = frame.wrapping_sub(32);
    let multiplier = frame.wrapping_sub(8);
    let temporary = frame.wrapping_sub(4);
    e.word(a, e.w(0x442576)?)?;
    sane(e, &[a, a], 0x200e)?;
    e.long(constant, 0x3ffda8f5)?;
    e.long(constant + 4, 0xc28f5c28)?;
    e.word(constant + 8, 0xf5c3)?;
    sane(e, &[a, constant], 4)?;
    e.long(value, 0x3fff8000)?;
    e.long(value + 4, 0)?;
    e.word(value + 8, 0)?;
    sane(e, &[constant, value], 0)?;
    sane(e, &[value, frame.wrapping_sub(22)], 0x1010)?;
    e.long(multiplier, e.l(frame.wrapping_sub(22))?)?;
    for (offset, first) in [(0x10e, true), (0x110, false)] {
        e.word(value, e.w(base + offset)?)?;
        sane(e, &[value, value], 0x200e)?;
        sane(e, &[value, value], 0x1010)?;
        e.long(temporary, e.l(value)?)?;
        sane(e, &[temporary, value], 0x100e)?;
        sane(e, &[multiplier, value], 0x1004)?;
        sane(e, &[value, temporary], 0x1010)?;
        sane(e, &[temporary, value], 0x100e)?;
        sane(e, &[value], 0x16)?;
        sane(e, &[value, value], 0x2810)?;
        e.word(base + offset, e.w(value + 2)?)?;
        if first {
            e.word(base + 0x52, e.w(base + 0x10e)?)?;
        }
    }
    Ok(())
}
fn summon<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    frame: u32,
    sp: &mut u32,
) -> Result<()> {
    let mut possible = false;
    let mut n = e.sw(0x43874a)?;
    while n < 100 {
        e.step()?;
        if e.sw(0x439d3c + u32::from(n as u16) * 288 + 0x110)? < 1 {
            possible = true;
        }
        n = n.wrapping_add(1);
    }
    let target = 0x443722 + u32::from(e.w(0x442bd4)?) * 6;
    let x = e.w(target)?;
    let y = e.sw(target + 2)?;
    if e.sw(cell(x, y))? <= 110 {
        possible = false;
    }
    if e.sw(cell(x, y.wrapping_sub(1)))? <= 110
        && (e.w(0x440dcc)? == 1 || absolute!(e, sp, e.w(0x440dcc)?, true)? == 3)
    {
        possible = false;
    }
    if e.sw(cell(x.wrapping_sub(1), y))? <= 110
        && (absolute!(e, sp, e.w(0x440dcc)?, true)? as i16) > 1
    {
        possible = false;
    }
    if e.sw(cell(x.wrapping_sub(1), y.wrapping_sub(1)))? <= 110
        && absolute!(e, sp, e.w(0x440dcc)?, true)? == 3
    {
        possible = false;
    }
    if possible {
        play!(e, sp, e.w(0x443774)?)?;
        let mut n = e.sw(0x43874a)?;
        while n < 100 {
            e.step()?;
            let base = 0x439d3c + u32::from(n as u16) * 288;
            if e.sw(base + 0x110)? > 0 {
                n = n.wrapping_add(1);
                continue;
            }
            e.addw(0x43874a, 1)?;
            e.copy(0x440dbc, base, 288)?;
            e.word(base + 0x10c, u16::MAX)?;
            let positions = at(0x446f0a, n, 4);
            e.word(
                positions,
                (i32::from(e.sw(target)?).wrapping_sub(e.sl(0x442780)?)) as u16,
            )?;
            e.word(
                positions + 2,
                (i32::from(e.sw(target + 2)?).wrapping_sub(e.sl(0x442784)?)) as u16,
            )?;
            let mut order = 109i16;
            while order > 0 {
                e.step()?;
                let address = at(0x4470b8, order, 2);
                if e.sw(address)? == -1 {
                    e.word(address, n.wrapping_add(10) as u16)?;
                    report!(e, sp, n.wrapping_add(10) as u16, (-12i16) as u16, None)?;
                    break;
                }
                order = order.wrapping_sub(1);
            }
            e.word(base + 0x110, e.w(base + 2)?)?;
            let mut dice = 0i16;
            while dice < e.sw(base)? {
                e.step()?;
                let value = e.rand(8, *sp)?;
                e.addw(base + 0x110, value)?;
                dice = dice.wrapping_add(1);
            }
            e.word(base + 0x112, e.w(base + 0x110)?)?;
            e.word(base + 14, e.w(warrior(e.w(0x4435d2)?) + 16)?)?;
            e.byte(base + 0x108, 0)?;
            e.byte(base + 0x107, 0)?;
            if e.w(base + 8)? != 0 {
                let value = e.rand(3, *sp)?.wrapping_sub(2);
                e.addw(base + 8, value)?;
            }
            if e.sw(base + 8)? < 0 {
                e.word(base + 8, 0)?;
            }
            let value = e.rand(3, *sp)?.wrapping_sub(2);
            e.addw(base + 4, value)?;
            if e.sw(base + 4)? < 1 {
                e.word(base + 4, 1)?;
            }
            e.word(0x4434b6, (e.sw(base + 0x10e)? / 10) as u16)?;
            let value = e.range(e.sw(0x4434b6)?.wrapping_neg(), e.sw(0x4434b6)?, *sp)?;
            e.addw(base + 0x10e, value)?;
            if e.sw(base + 10)? < 99 {
                e.addw(base + 10, e.sw(0x442576)?.wrapping_mul(3))?;
            }
            for i in 0..6 {
                e.step()?;
                e.byte(
                    base + 139 + i,
                    e.b(base + 139 + i)?
                        .wrapping_add(e.w(0x442576)?.wrapping_mul(7) as u8),
                )?;
            }
            e.addw(base + 8, e.sw(0x442576)?.wrapping_mul(-2))?;
            e.addw(base + 4, e.sw(0x442576)?)?;
            summon_scale(e, frame, base)?;
            if e.sw(base + 0x110)? < 1 {
                e.word(base + 0x110, 1)?;
            }
            e.word(base + 0x112, e.w(base + 0x110)?)?;
            if e.sw(base + 78)? < 0 {
                invoke!(e, sp, 0x11d406, &[e.w(base + 78)?, 0], 4)?;
                let value = e.d0_word()?;
                e.word(base + 38, value)?;
                e.word(base + 78, value)?;
            }
            invoke!(
                e,
                sp,
                0x149e0e,
                &[e.w(positions)?, e.w(positions + 2)?, n as u16],
                0
            )?;
            invoke!(e, sp, 0x15b2f6, &[n.wrapping_add(10) as u16, 0], 0)?;
            *sp = sp.wrapping_add(10);
            e.moveq_return(0);
            return Ok(());
        }
    }
    invoke!(e, sp, 0x133a18, &[133], 0)?;
    report!(e, sp, acting(e)? as u16, (-9i16) as u16, None)?;
    *sp = sp.wrapping_add(2);
    e.moveq_return(0);
    Ok(())
}
pub(crate) fn spell_list_body<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    frame: u32,
) -> Result<()> {
    let mut body = e.sw(frame + 8)?;
    let selector = e.sw(frame + 10)?;
    let mut sp = frame.wrapping_sub(72);
    e.stack = sp;
    e.long(frame.wrapping_sub(18), 0)?;
    e.long(frame.wrapping_sub(8), 0)?;
    e.byte(frame.wrapping_sub(13), 0)?;
    e.byte(frame.wrapping_sub(9), 0)?;
    if selector == 0 {
        e.moveq_return(0);
        return Ok(());
    }
    if body < 9 && e.w(warrior(body as u16) + 154)? != 0 && e.rand(100, sp)? < 34 {
        body = acting(e)?;
    }
    let base = warrior(body as u16);
    let enemy = monster(body as u16);
    if matches!(selector, 59..=61) {
        if selector > 59 {
            e.word(0x4434b8, e.sw(0x4434b8)?.wrapping_neg() as u16)?;
        }
        if body < 9 {
            e.addw(base + 88, e.sw(0x4434b8)?)?;
            e.word(0x4434b8, 0)?;
            if e.sw(base + 88)? < 0 {
                e.word(base + 88, 0)?;
            }
            if e.sw(base + 88)? > e.sw(base + 90)? {
                e.word(base + 88, e.w(base + 90)?)?;
            }
            update(e, &mut sp, body as u16, 0, true)?;
        } else {
            e.addw(enemy + 270, e.sw(0x4434b8)?)?;
            if e.sw(enemy + 270)? < 0 {
                e.word(enemy + 270, 0)?;
            }
        }
        if selector == 61 {
            let value = e.sw(0x4434b8)? / 2;
            let destination = if acting(e)? < 9 {
                warrior(e.w(0x4435d2)?) + 88
            } else {
                0x439d3c + u32::from(e.w(0x447196)?) * 288 + 270
            };
            e.addw(destination, value)?;
        }
        if body >= 9 {
            e.word(0x4434b8, 0)?;
        }
        e.moveq_return(0);
        return Ok(());
    }
    if selector == 68 {
        e.word(0x442578, 1)?;
        invoke!(e, &mut sp, 0x160a8e, &[1, 0, 1, 0], 8)?;
        e.moveq_return(0);
        return Ok(());
    }
    if matches!(selector, 49 | 27) {
        let value = if body < 9 {
            e.w(base + 80)?
        } else {
            e.w(enemy + 272)?
        };
        e.word(0x438750, value.wrapping_add(10))?;
        e.word(0x4434b8, e.w(0x438750)?)?;
    }
    if selector == 64 && (e.sw(base + 80)? < -9 || e.w(base + 144)? != 0) {
        e.word(base + 144, 0)?;
        e.word(base + 80, (-9i16) as u16)?;
        e.byte(0x4427b3, 1)?;
        update(e, &mut sp, body as u16, 3, false)?;
    }
    if matches!(selector, 53 | 54) {
        if body < 9 {
            e.addw(base + 96, e.sw(0x4427a4)?)?;
            e.word(base + 64, 0)?;
            e.word(base + 60, 0)?;
            update(e, &mut sp, body as u16, 3, false)?;
        } else {
            e.byte(enemy + 179, e.b(enemy + 179)?.wrapping_add(e.b(0x4427a5)?))?;
            e.byte(enemy + 262, 0)?;
        }
    }
    // The source's second selector61 body is unreachable after the earlier return.
    if selector == 58 && e.b(frame.wrapping_sub(9))? == 0 {
        return summon(e, frame, &mut sp);
    }
    if selector == 90 {
        e.word(
            frame.wrapping_sub(12),
            e.w(warrior(e.w(0x4435d2)?) + 58)?
                .wrapping_mul(3)
                .wrapping_add(e.w(0x4435cc)?.wrapping_mul(5)),
        )?;
        if (e.b(enemy + 169)? != 0 || e.b(enemy + 170)? != 0)
            && e.w(enemy + 14)? != 0
            && e.w(enemy + 90)? != 255
            && e.sw(enemy + 272)? > 0
        {
            let value = e
                .w(enemy)?
                .wrapping_mul(5)
                .wrapping_add(100u16.wrapping_sub(e.w(frame.wrapping_sub(12))?));
            e.word(0x4434b4, value)?;
            if e.sw(0x4434b4)? < 25 {
                e.word(0x4434b4, 25)?;
            }
            let value = e.rand(100, sp)?;
            e.word(0x4434b6, (value as u16).wrapping_sub(e.w(0x4434b4)?))?;
            if e.sw(0x4434b6)? <= 0 {
                e.moveq_return(0);
                return Ok(());
            }
            if e.sw(0x4434b6)? < 30 {
                e.addw(0x43877c, -1)?;
                e.word(enemy + 272, 0)?;
                invoke!(e, &mut sp, 0x12e4e4, &[body as u16, 0], 4)?;
            } else {
                e.addw(0x4435c4, -1)?;
                e.addw(0x43877c, -1)?;
                invoke!(e, &mut sp, 0x133a18, &[630], 0)?;
                invoke!(e, &mut sp, 0x158454, &[body as u16, (-13i16) as u16, 0], 0)?;
                e.word(enemy + 14, e.w(warrior(e.w(0x4435d2)?) + 16)?)?;
                invoke!(e, &mut sp, 0x15b2f6, &[body as u16, 0], 0)?;
                sp = sp.wrapping_add(12);
                e.stack = sp;
            }
        }
    }
    if matches!(selector, 51 | 52) {
        let allegiance = if acting(e)? < 9 {
            e.w(warrior(e.w(0x4435d2)?) + 16)? as u8 as i8
        } else {
            e.w(0x439d3c + u32::from(e.w(0x447196)?) * 288 + 14)? as u8 as i8
        };
        e.byte(frame.wrapping_sub(10), allegiance as u8)?;
        if allegiance == 0 {
            if body <= 9 {
                if e.w(base + 16)? != 0 {
                    e.word(base + 16, 0)?;
                    e.byte(0x442739, e.b(0x442739)?.wrapping_sub(1))?;
                    e.addw(0x4435c4, -1)?;
                }
            } else if e.w(enemy + 14)? != 0 {
                e.word(enemy + 268, u16::MAX)?;
                e.word(enemy + 14, 0)?;
                e.addw(0x4435c4, -1)?;
            }
        } else if body < 9 {
            if i16::from(allegiance) != e.sw(base + 16)? {
                if e.w(base + 16)? == 0 {
                    e.addw(0x4435c4, 1)?;
                    e.byte(0x442739, e.b(0x442739)?.wrapping_add(1))?;
                }
                e.word(base + 24, u16::MAX)?;
                e.word(base + 16, i16::from(allegiance) as u16)?;
                update(e, &mut sp, body as u16, 3, false)?;
            }
        } else {
            if e.w(enemy + 14)? == 0 {
                e.addw(0x4435c4, 1)?;
            }
            e.word(enemy + 14, i16::from(allegiance) as u16)?;
            e.word(enemy + 268, u16::MAX)?;
        }
    }
    if selector == 2 {
        if body < 9 {
            e.word(base + 60, 0)?;
            update(e, &mut sp, body as u16, 3, false)?;
        } else {
            e.byte(enemy + 262, 0)?;
        }
    }
    if matches!(selector, 7 | 3) {
        if body < 9 {
            e.word(base + 60, (e.sw(base + 60)? / 2) as u16)?;
            update(e, &mut sp, body as u16, 3, false)?;
        } else {
            e.byte(enemy + 262, (e.sb(enemy + 262)? / 2) as u8)?;
        }
    }
    if selector == 26 && e.sw(base + 80)? < -9 {
        e.word(base + 144, u16::MAX)?;
        let hp = e.sw(base + 82)?;
        e.word(
            base + 80,
            hp.wrapping_add(((hp >> 1) as u16 >> 14) as i16)
                .wrapping_shr(2) as u16,
        )?;
        update(e, &mut sp, body as u16, 3, false)?;
    }
    if selector == 10 {
        if body < 9 {
            if e.sw(base + 144)? < 0 {
                e.word(base + 100, 0)?;
            }
            update(e, &mut sp, body as u16, 3, false)?;
        } else if e.sb(enemy + 144)? > 100 || e.b(enemy + 203)? != 0 {
            e.byte(enemy + 181, 0)?;
        }
    }
    if selector == 110 {
        if body < 9 {
            e.word(base + 150, 0)?;
            e.word(base + 100, 0)?;
            update(e, &mut sp, body as u16, 3, false)?;
        } else {
            e.byte(enemy + 206, 0)?;
            e.byte(enemy + 181, 0)?;
        }
    }
    e.moveq_return(1);
    Ok(())
}

pub(crate) fn resolve_spell_body<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    frame: u32,
) -> Result<()> {
    let mut sp = frame.wrapping_sub(24);
    e.stack = sp;
    e.word(frame.wrapping_sub(2), 1)?;
    e.word(0x4427a4, 0)?;
    e.word(0x4434b8, 0)?;
    invoke!(e, &mut sp, 0x13154c, &[16], 0)?;
    let mut selector = absolute!(e, &mut sp, e.w(0x443776)?, false)? as i16;
    let second = absolute!(e, &mut sp, i16::from(e.sb(0x443762)?) as u16, false)?;
    e.word(frame.wrapping_sub(4), second)?;
    let category = i16::from(e.sb(0x443761)?);
    let amount = e.range(i16::from(e.sb(0x44375b)?), i16::from(e.sb(0x44375c)?), sp)?;
    e.word(0x4427a4, amount as u16)?;
    sp = sp.wrapping_add(6);
    e.stack = sp;
    if e.w(0x438776)? != 0
        && acting(e)? < 6
        && e.b(0x435662)? == 1
        && e.sw(0x4435d2)? < 6
        && acting(e)? > -1
        && e.w(0x438778)? == 0
    {
        let base = 0x4378d2 + u32::from(e.w(0x4435d2)?) * 80;
        for (offset, source) in [(0, 0x4435c6), (2, 0x4435c8), (4, 0x4435ca), (6, 0x4435cc)] {
            e.word(base + offset, e.w(source)?)?;
        }
    } else if e.w(0x438776)? == 0 && e.b(0x435662)? == 1 {
        let base = 0x4378d2 + u32::from(i16::from(e.sb(0x44273f)?) as u16) * 80;
        for (offset, source) in [(0, 0x4435c6), (2, 0x4435c8), (4, 0x4435ca), (6, 0x4435cc)] {
            e.word(base + offset, e.w(source)?)?;
        }
        e.byte(0x435662, 0)?;
    }
    if e.b(0x44375d)? != 0 {
        let mut n = 0i16;
        while n < e.sw(0x4435cc)? {
            e.step()?;
            let value = e.range(i16::from(e.sb(0x44375d)?), i16::from(e.sb(0x44375e)?), sp)?;
            e.addw(0x4427a4, value)?;
            n = n.wrapping_add(1);
        }
    }
    if e.b(0x44374f)? == 7 {
        if selector == 50 {
            play!(e, &mut sp, 601)?;
            let value = e.sw(0x4435cc)?.wrapping_mul(30);
            if value > e.sw(0x442562)? {
                e.word(0x442562, value.wrapping_sub(1) as u16)?;
                if e.l(0x3fe302)? == e.l(0x43878c)? || e.l(0x3fe302)? == e.l(0x43879c)? {
                    let handle = e.l(e.l(0x439480)?)?;
                    e.long(0x44292c, e.l(handle + 8)?)?;
                    e.long(0x442930, e.l(handle + 12)?)?;
                    invoke!(e, &mut sp, 0x153720, &[1], 0)?;
                    rectangle_trap(
                        e,
                        &mut sp,
                        0xa8a9,
                        &[P(0x44292c), L(0x10001)],
                        vec![P(0x44292c), W(1), W(1)],
                    )?;
                    rectangle_trap(e, &mut sp, 0xa8a3, &[P(0x44292c)], vec![P(0x44292c)])?;
                    rectangle_trap(
                        e,
                        &mut sp,
                        0xa8a9,
                        &[P(0x44292c), L(u32::MAX)],
                        vec![P(0x44292c), W(u16::MAX), W(u16::MAX)],
                    )?;
                    sp = sp.wrapping_add(2);
                    e.stack = sp;
                }
                if e.trap(0xa924, vec![])? == e.l(0x43879c)? {
                    invoke!(e, &mut sp, 0x10f186, &[0], 2)?;
                }
            }
            e.word(0x438760, 0)?;
            if e.l(0x3fe302)? == e.l(0x43878c)? {
                invoke!(e, &mut sp, 0x112650, &[0x321, 0, 0xffff, 0], 8)?;
                if e.d0_word()? != 0 {
                    invoke!(e, &mut sp, 0x1537ee, &[1], 2)?;
                }
            }
        } else {
            let address = at(0x442562, selector, 2);
            if e.sw(address)? < e.sw(0x4427a4)? {
                e.word(address, e.w(0x4427a4)?)?;
            }
            e.word(0x438760, 0)?;
        }
        e.moveq_return(0);
        return Ok(());
    }
    if selector == 66 {
        if e.b(0x44376d)? == 0 {
            let value = e.rand(5, sp)?;
            e.byte(0x44376d, value as u8)?;
        }
        let mut n = 0i16;
        while n <= i16::from(e.sb(0x44273a)?) {
            e.step()?;
            if e.w(at(0x4434e0, n, 2))? != 0 {
                let base = warrior(n as u16);
                invoke!(e, &mut sp, 0x13a7c4, &[0, e.w(base + 52)?], 4)?;
                e.addw(
                    at(base + 0x9e2, i16::from(e.b(0x44376d)?).wrapping_sub(1), 2),
                    1,
                )?;
                if e.b(0x44376d)? == 10 {
                    let value = e.rand(8, sp)?;
                    e.addw(base + 82, value)?;
                } else if e.b(0x44376d)? == 11 && e.sw(base + 90)? > 0 {
                    let value = e.rand(20, sp)?;
                    e.addw(base + 90, value)?;
                }
            }
            n = n.wrapping_add(1);
        }
    }
    if matches!(selector, 55 | 56) {
        if e.w(0x438786)? == 0 {
            invoke!(e, &mut sp, 0x16474a, &[87], 2)?;
            e.moveq_return(0);
            return Ok(());
        }
        let body = e.w(0x4435d2)?;
        let base = warrior(body);
        if selector == 55 {
            e.word(base + 60, 0)?;
            e.word(base + 0xac8, e.w(base + 0xac6)?)?;
        }
        invoke!(
            e,
            &mut sp,
            0x158454,
            &[body, selector as u16, acting(e)? as u16],
            0
        )?;
        invoke!(e, &mut sp, 0x133a18, &[699], 0)?;
        invoke!(e, &mut sp, 0x15b2f6, &[body, 0], 0)?;
        invoke!(e, &mut sp, 0x105572, &[body, 0], 0)?;
        invoke!(e, &mut sp, 0x105366, &[body], 0)?;
        let position = at(0x44709a, body as i16, 4);
        e.word(
            position,
            (i32::from(e.sw(0x443722)?).wrapping_sub(e.sl(0x442780)?)) as u16,
        )?;
        e.word(
            position + 2,
            (i32::from(e.sw(0x443724)?).wrapping_sub(e.sl(0x442784)?)) as u16,
        )?;
        invoke!(
            e,
            &mut sp,
            0x10e6b6,
            &[e.w(position)?, e.w(position + 2)?],
            0
        )?;
        let x = e.sl(0x442780)?.wrapping_add(i32::from(e.sw(position)?));
        let y = e.sl(0x442784)?.wrapping_add(i32::from(e.sw(position + 2)?));
        let address = 0x4085b8u32
            .wrapping_add(x.wrapping_mul(180) as u32)
            .wrapping_add(y.wrapping_mul(2) as u32);
        e.word(0x4434b6, e.w(address)?)?;
        e.word(at(0x437a6e, body as i16, 2), e.w(0x4434b6)?)?;
        sp = sp.wrapping_add(22);
        e.stack = sp;
        let tile = e.sw(0x4434b6)?;
        let lethal = tile < 999
            || (tile > 999
                && e.w(0x431680 + u32::from((tile as u16).wrapping_sub(1000)) * 40)? != 0);
        e.word(address, body)?;
        invoke!(e, &mut sp, 0x133a18, &[658], 0)?;
        invoke!(e, &mut sp, 0x15b2f6, &[body, 0], 0)?;
        if lethal {
            invoke!(e, &mut sp, 0x133a18, &[631], 0)?;
            invoke!(e, &mut sp, 0x16474a, &[123], 0)?;
            e.word(base + 80, (-10i16) as u16)?;
            invoke!(e, &mut sp, 0x12e4e4, &[body, 0], 0)?;
            sp = sp.wrapping_add(14);
            e.stack = sp;
        } else {
            sp = sp.wrapping_add(6);
            e.stack = sp;
            if e.b(0x44376d)? != 0 {
                e.addw(0x447194, -1)?;
            }
            e.word(base + 20, 1)?;
        }
        e.moveq_return(0);
        return Ok(());
    }
    if selector == 58 {
        if e.sw(0x43874a)? > 99 {
            invoke!(e, &mut sp, 0x13822a, &[], 0)?;
            e.word(0x4435c0, 0)?;
            e.word(0x442bd4, 6)?;
            message!(e, &mut sp, 0x401860, 50, 100, 0, 6000, 0)?;
            e.word(0x438760, 0)?;
            invoke!(e, &mut sp, 0x11f1a8, &[e.w(0x4435d2)?], 0)?;
            sp = sp.wrapping_add(14);
            e.stack = sp;
        } else {
            invoke!(e, &mut sp, 0x15b9d8, &[0, selector as u16], 4)?;
        }
        e.moveq_return(0);
        return Ok(());
    }
    if matches!(e.b(0x44374f)?, 9 | 10) && e.w(0x438776)? != 0 {
        let pointer = if e.b(0x44374f)? == 9 {
            0x401840
        } else {
            0x401820
        };
        message!(
            e,
            &mut sp,
            pointer,
            e.w(0x40856c)?.wrapping_add(325),
            322,
            60,
            6001,
            12
        )?;
    }
    e.byte(0x4358c1, 0)?;
    if e.w(0x439588)? != 0
        && (matches!(e.b(0x44374f)?, 3 | 4) || e.sb(0x44374f)? > 8)
        && e.b(0x4358c2)? == 0
        && e.w(0x438776)? != 0
    {
        e.byte(0x4358c1, 1)?;
        play!(e, &mut sp, e.w(0x443774)?)?;
        if e.w(0x438776)? != 0 {
            animate!(e, &mut sp, e.w(0x4434b6)?, 1)?;
        }
    }
    for body in 0u16..110 {
        e.step()?;
        if e.sw(at(0x4434e0, body as i16, 2))? <= 0 {
            continue;
        }
        if body > 9 {
            e.copy(monster(body), 0x440dbc, 288)?;
        }
        invoke!(e, &mut sp, 0x150114, &[body], 2)?;
        let resisted = e.d0_word()? as u8;
        let condition = e.b(0x4358c2)?;
        if (condition == 0 || e.b(0x435884 + u32::from(condition))? == 0)
            && (resisted == 0 || e.w(0x438776)? == 0)
        {
            e.byte(0x435884 + u32::from(condition), 1)?;
            invoke!(
                e,
                &mut sp,
                0x15075a,
                &[body, 1, 1, 1, e.w(0x45375a)?.wrapping_add(e.w(0x4537f0)?)],
                10
            )?;
            e.word(0x4434b8, e.d0_word()?)?;
            if selector == 57 {
                selector = 0;
            }
            if selector < 41 && selector != 0 {
                if selector == 28 {
                    e.word(0x4434b8, e.w(0x4427a4)?)?;
                }
                if body <= 9 {
                    let address = at(warrior(body) + 94, selector.wrapping_sub(1), 2);
                    if e.sw(address)?.wrapping_add(e.sw(0x4427a4)?) < 100 && e.sw(address)? > -1 {
                        e.addw(address, e.sw(0x4427a4)?)?;
                    }
                } else {
                    let address = at(monster(body) + 178, selector.wrapping_sub(1), 1);
                    let value = i16::from(e.sb(address)?).wrapping_add(e.sw(0x4427a4)?);
                    if (absolute!(e, &mut sp, value as u16, true)? as i16) < 125
                        && e.sb(address)? > -1
                    {
                        e.byte(address, e.b(address)?.wrapping_add(e.b(0x4427a5)?))?;
                    }
                }
            } else if selector > 99 {
                if body <= 9 {
                    e.word(at(warrior(body) + 94, selector.wrapping_sub(101), 2), 0)?;
                    update(e, &mut sp, body, 3, false)?;
                } else {
                    e.byte(at(monster(body) + 178, selector.wrapping_sub(101), 1), 0)?;
                }
            }
            invoke!(e, &mut sp, 0x15b9d8, &[body, selector as u16], 4)?;
            if e.d0_word()? != 0 && e.w(0x4434b8)? == 0 && selector != 0 {
                play!(e, &mut sp, e.w(0x443774)?)?;
                if e.b(0x4358c1)? == 0 {
                    animate!(e, &mut sp, body, 0)?;
                }
                push_word!(e, &mut sp, acting(e)? as u16)?;
                let code = absolute!(e, &mut sp, selector as u16, true)?;
                invoke!(e, &mut sp, 0x158454, &[body, code], 6)?;
            }
            if body < 9 && e.w(0x4434b8)? != 0 && e.sw(warrior(body) + 80)? > -10 {
                if e.b(0x443761)? != 9 {
                    play!(e, &mut sp, e.w(0x443774)?)?;
                }
                if e.sw(0x4434b8)? < 0 {
                    push_word!(e, &mut sp, 1)?;
                    let amount = absolute!(e, &mut sp, e.w(0x4434b8)?, true)?;
                    invoke!(e, &mut sp, 0x1287c2, &[body, amount], 6)?;
                } else {
                    if e.sw(warrior(body) + 80)? < 0 && e.sw(0x4434b8)? > 0 {
                        continue;
                    }
                    e.addw(warrior(body) + 80, e.sw(0x4434b8)?.wrapping_neg())?;
                    e.word(at(0x442798, body as i16, 2), u16::MAX)?;
                    if e.sw(0x4434b8)? > 0 {
                        e.word(warrior(body) + 20, 1)?;
                    }
                    if e.sw(warrior(body) + 80)? <= 0
                        && e.sw(warrior(body) + 80)?.wrapping_add(e.sw(0x4434b8)?) > 0
                    {
                        invoke!(e, &mut sp, 0x12e4e4, &[body, 0], 4)?;
                    } else {
                        if e.w(0x438776)? != 0 {
                            if e.b(0x4358c1)? == 0 {
                                animate!(e, &mut sp, body, 0)?;
                            }
                            push_word!(e, &mut sp, acting(e)? as u16)?;
                            let code = absolute!(e, &mut sp, selector as u16, true)?;
                            invoke!(e, &mut sp, 0x158454, &[body, code], 6)?;
                        }
                        if e.w(0x438776)? == 0 {
                            invoke!(e, &mut sp, 0x15b2f6, &[body, 0], 0)?;
                            invoke!(e, &mut sp, 0x1205f8, &[0], 0)?;
                            push_word!(e, &mut sp, acting(e)? as u16)?;
                            let code = absolute!(e, &mut sp, selector as u16, true)?;
                            invoke!(e, &mut sp, 0x158454, &[body, code], 0)?;
                            e.trap(0xa873, vec![L(e.l(0x43878c)?)])?;
                            invoke!(e, &mut sp, 0x1500d8, &[0], 14)?;
                        }
                    }
                }
            } else if e.w(0x4434b8)? != 0 && body > 9 {
                invoke!(e, &mut sp, 0x1107cc, &[body, 0], 4)?;
                if e.b(0x443761)? != 9 {
                    play!(e, &mut sp, e.w(0x443774)?)?;
                }
                let enemy = monster(body);
                e.byte(enemy + 261, 1)?;
                e.addw(enemy + 272, e.sw(0x4434b8)?.wrapping_neg())?;
                if (0..6).contains(&acting(e)?) {
                    let caster = warrior(e.w(0x4435d2)?);
                    e.addl(caster + 0xa46, e.sw(0x4434b8)? as i32 as u32)?;
                }
                if (0..6).contains(&acting(e)?) && category == 9 {
                    e.addl(warrior(e.w(0x4435d2)?) + 0xa4a, 1)?;
                }
                if e.sw(enemy + 272)? < 1 {
                    if (0..6).contains(&acting(e)?) {
                        e.addl(warrior(e.w(0x4435d2)?) + 0xa5a, 1)?;
                    }
                    invoke!(e, &mut sp, 0x12e4e4, &[body, 0], 4)?;
                } else {
                    play!(e, &mut sp, e.w(0x453776)?)?;
                    if e.b(0x4358c1)? == 0 {
                        animate!(e, &mut sp, body, 0)?;
                    }
                    if e.w(0x4434b8)? != 0 {
                        report!(e, &mut sp, body, 0, Some(acting(e)? as u16))?;
                    }
                }
            }
        }
        if e.w(0x438776)? != 0 {
            let delay = if e.w(0x438760)? != 0 && e.sb(0x44374f)? > 8 {
                5
            } else {
                0
            };
            invoke!(e, &mut sp, 0x1205f8, &[delay], 2)?;
        }
    }
    invoke!(e, &mut sp, 0x13154c, &[17], 2)?;
    e.moveq_return(0);
    Ok(())
}

crate::recovered_application_control::entry!(resolve_spell, 0x151022);
crate::recovered_application_control::entry!(spell_list, 0x15b9d8);
crate::recovered_application_control::entry!(spell_targets, 0x15cb0e);
crate::recovered_application_control::entry!(heal, 0x1287c2);
crate::recovered_application_control::entry!(reduce, 0x14de04);
