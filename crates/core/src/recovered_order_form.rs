//! Order-dialog interaction and printing, with observed Toolbox and SANE calls.

use crate::{
    RandomState, RuleError,
    recovered_load_control::LoadRequest,
    recovered_rendering::RenderingMemory,
    recovered_startup::{
        Execution, L, P, StartupEffect, StartupPlan, StartupRequest, StartupServices, W,
    },
    recovered_storage::StorageLimits,
};

fn control(
    x: &mut Execution,
    dialog: u32,
    item: i16,
    s: &(impl StartupServices + ?Sized),
) -> Result<i16, RuleError> {
    x.get_item(dialog, item, 0x4434dc, s)?;
    Ok(x.tb(0xa960, None, vec![L(x.e.long(0x4434dc)?)], s)? as i16)
}
fn set_control(
    x: &mut Execution,
    dialog: u32,
    item: i16,
    value: i16,
    s: &(impl StartupServices + ?Sized),
) -> Result<(), RuleError> {
    x.get_item(dialog, item, 0x4434dc, s)?;
    x.tb(
        0xa963,
        None,
        vec![L(x.e.long(0x4434dc)?), W(value as u16)],
        s,
    )?;
    Ok(())
}
fn sane(
    x: &mut Execution,
    selector: u16,
    pointers: &[u32],
    s: &(impl StartupServices + ?Sized),
) -> Result<(), RuleError> {
    x.tb(
        0xa9eb,
        Some(u32::from(selector)),
        pointers.iter().copied().map(P).collect(),
        s,
    )?;
    Ok(())
}
fn get_number(
    x: &mut Execution,
    item: i16,
    s: &(impl StartupServices + ?Sized),
) -> Result<i16, RuleError> {
    let dialog = x.e.long(0x4394b8)?;
    x.get_item(dialog, item, 0x4434dc, s)?;
    x.tb(0xa990, None, vec![L(x.e.long(0x4434dc)?), P(0x479ed8)], s)?;
    x.request(
        StartupRequest::StringToNumber {
            source: 0x479ed8,
            destination: 0x4471dc,
        },
        s,
    )?;
    x.e.word(0x4471de)
}
fn extended(x: &mut Execution, address: u32, a: u32, b: u32, c: u16) -> Result<(), RuleError> {
    x.lw(address, a)?;
    x.lw(address.wrapping_add(4), b)?;
    x.ww(address.wrapping_add(8), c as i16)
}

fn totals(
    x: &mut Execution,
    dialog: u32,
    frame: u32,
    s: &(impl StartupServices + ?Sized),
) -> Result<(), RuleError> {
    x.ww(0x4434b6, 0)?;
    let mut total = 0_u32;
    let mut disks = false;
    if control(x, dialog, 53, s)? != 0 {
        x.lw(frame.wrapping_sub(4), 0x3f800000)?;
    }
    // D6 remains zero throughout the source body; the old disk-format labels are not reached.
    if control(x, dialog, 16, s)? == 0 {
        x.di_str(54, 0x401b5f, s)?;
    }
    if control(x, dialog, 12, s)? != 0 {
        total = total.wrapping_add(20);
    } else if crate::recovered_verification::doreg() == 0 {
        x.tb(0xa963, None, vec![L(x.e.long(0x4434dc)?), W(1)], s)?;
        total = total.wrapping_add(20);
    }
    for (item, price) in [
        (13, 13),
        (14, 13),
        (56, 45),
        (19, 13),
        (52, 13),
        (20, 13),
        (15, 6),
        (16, 6),
        (17, 13),
    ] {
        if control(x, dialog, item, s)? != 0 {
            total = total.wrapping_add(price);
            if item == 16 || item == 17 {
                disks = true;
            }
        }
    }
    x.get_item(dialog, 45, 0x4434dc, s)?;
    if get_number(x, 45, s)? > 1 && disks {
        total = total.wrapping_add(3);
        x.di_str(26, 0x401b40, s)?;
        x.number(25, 2, s)?;
    } else {
        x.di_str(26, 0x401b5f, s)?;
        x.di_str(25, 0x401b5f, s)?;
    }
    let a = frame.wrapping_sub(34);
    let b = frame.wrapping_sub(24);
    if control(x, dialog, 18, s)? != 0 {
        x.lw(a, total)?;
        sane(x, 0x280e, &[a, a], s)?;
        extended(x, b, 0x3ffae147, 0xae147ae1, 0x47ae)?;
        sane(x, 4, &[a, b], s)?;
        sane(x, 0x16, &[b], s)?;
        sane(x, 0x2810, &[b, b], s)?;
        x.ww(0x4434b6, x.e.word(frame.wrapping_sub(22))?)?;
        x.number(32, i32::from(x.e.word(0x4434b6)?), s)?;
        extended(x, b, 0x3fff870a, 0x3d70a3d7, 0x0a3d)?;
        x.lw(a, total)?;
        sane(x, 0x280e, &[a, a], s)?;
        sane(x, 4, &[b, a], s)?;
        sane(x, 0x16, &[a], s)?;
        sane(x, 0x2810, &[a, a], s)?;
        total = x.e.long(a)?;
        x.di_str(27, 0x401b36, s)?;
        x.di_str(29, 0x401b34, s)?;
    } else {
        for i in [27, 29, 32] {
            x.di_str(i, 0x401b5f, s)?;
        }
    }
    extended(x, b, 0x3fff8000, 0, 0)?;
    let o = x.request(
        StartupRequest::Toolbox {
            opcode: 0xa9eb,
            selector: Some(0x1008),
            arguments: vec![P(frame.wrapping_sub(4)), P(b)],
            d0: None,
            a0: None,
        },
        s,
    )?;
    if !o.zero {
        for (i, p) in [
            (43, 0x401b26),
            (44, 0x401b24),
            (46, 0x401b1a),
            (47, 0x401b34),
            (30, 0x401b5f),
        ] {
            x.di_str(i, p, s)?;
        }
        x.number(48, total as i32, s)?;
    } else {
        for (i, p) in [
            (43, 0x401b5f),
            (44, 0x401b5f),
            (45, 0x401b5f),
            (30, 0x401b34),
            (46, 0x401b5f),
            (47, 0x401b5f),
            (48, 0x401b5f),
        ] {
            x.di_str(i, p, s)?;
        }
    }
    x.lw(b, total)?;
    sane(x, 0x280e, &[b, b], s)?;
    sane(x, 0x1004, &[frame.wrapping_sub(4), b], s)?;
    sane(x, 0x16, &[b], s)?;
    sane(x, 0x2810, &[b, b], s)?;
    total = x.e.long(b)?;
    x.number(33, total as i32, s)?;
    Ok(())
}

fn print_form(
    x: &mut Execution,
    dialog: u32,
    frame: u32,
    s: &mut (impl StartupServices + ?Sized),
) -> Result<(), RuleError> {
    let handle = x
        .request(
            StartupRequest::Toolbox {
                opcode: 0xa322,
                selector: None,
                arguments: vec![],
                d0: Some(120),
                a0: None,
            },
            s,
        )?
        .a0;
    x.tb(0xa8fd, Some(0xc8000000), vec![], s)?;
    x.tb(0xa8fd, Some(0x20040480), vec![L(handle)], s)?;
    x.tb(0xa8fd, Some(0x2a040484), vec![L(handle)], s)?;
    if x.tb(0xa8fd, Some(0x32040488), vec![L(handle)], s)? as u8 != 0 {
        let port = x.tb(0xa8fd, Some(0x04000c00), vec![L(handle), L(0), L(0)], s)?;
        x.tb(0xa8fd, Some(0x10000808), vec![L(port), L(0)], s)?;
        x.ww(0x4434aa, 3)?;
        while x.e.word(0x4434aa)? < 100 {
            x.e.step()?;
            x.lw(0x4434dc, 0)?;
            x.get_item(dialog, x.e.word(0x4434aa)?, 0x4434dc, s)?;
            if x.e.long(0x4434dc)? != 0 {
                if matches!(x.e.word(0x4434b0)?, 5 | 6)
                    && x.tb(0xa960, None, vec![L(x.e.long(0x4434dc)?)], s)? as u16 != 0
                {
                    x.tb(0xa95e, None, vec![L(x.e.long(0x4434dc)?), P(0x479ed8)], s)?;
                    let h = x.e.word(0x4434bc)?;
                    let v = x.e.word(0x4434ba)?.wrapping_add(40);
                    x.tb(0xa893, None, vec![W(h as u16), W(v as u16)], s)?;
                    x.tb(0xa884, None, vec![P(0x479ed8)], s)?;
                }
                if matches!(x.e.word(0x4434b0)?, 8 | 16) {
                    let child = frame.wrapping_sub(78);
                    let v = x.e.word(0x4434ba)?.wrapping_add(40);
                    x.ww(child.wrapping_add(10), v)?;
                    x.ww(child.wrapping_add(8), x.e.word(0x4434aa)?)?;
                    x.flush();
                    let p = crate::recovered_item_ui::print_stat(&x.e.plan.memory, child, s)?;
                    x.e.plan.memory = p.memory;
                    x.effects
                        .extend(p.effects.into_iter().map(StartupEffect::ItemUi));
                }
            }
            x.e.add(0x4434aa, 1)?;
        }
        x.ww(0x4434ba, 20)?;
        x.ww(0x4434bc, 0)?;
        x.ww(0x4434c0, x.e.word(0x4434bc)?.wrapping_add(277))?;
        x.ww(0x4434be, x.e.word(0x4434ba)?.wrapping_add(110))?;
        let child = frame.wrapping_sub(84);
        let last = x.e.long(0x4434be)?;
        x.lw(child.wrapping_add(14), last)?;
        let first = x.e.long(0x4434ba)?;
        x.lw(child.wrapping_add(10), first)?;
        x.ww(child.wrapping_add(8), 164)?;
        let picture = x.tb(
            0xa9bc,
            None,
            vec![W(x.e.word(child.wrapping_add(8))? as u16)],
            s,
        )?;
        x.lw(child.wrapping_sub(4), picture)?;
        x.tb(
            0xa8f6,
            None,
            vec![
                L(x.e.long(child.wrapping_sub(4))?),
                P(child.wrapping_add(10)),
            ],
            s,
        )?;
        x.tb(0xa8fd, Some(0x1800040c), vec![L(port)], s)?;
        x.tb(0xa8fd, Some(0x08000484), vec![L(port)], s)?;
    }
    x.tb(0xa8fd, Some(0xd0000000), vec![], s)?;
    Ok(())
}

/// orderform, 0x146b6a. The return preserves observed post-DisposeDialog D0.
pub fn order_form(
    memory: &RenderingMemory,
    random: &RandomState,
    frame: u32,
    limits: StorageLimits,
    s: &mut (impl StartupServices + ?Sized),
) -> Result<StartupPlan, RuleError> {
    let mut x = Execution::new(memory, random, limits, None);
    x.lw(frame.wrapping_sub(4), 0x3f800000)?;
    x.e.write(frame.wrapping_sub(13), &[0])?;
    let dialog = x.tb(0xa97c, None, vec![W(146), L(0), L(u32::MAX)], s)?;
    x.lw(0x4394b8, dialog)?;
    x.tb(0xa915, None, vec![L(dialog)], s)?;
    x.tb(0xa873, None, vec![L(dialog)], s)?;
    x.ww(0x4471ea, 0)?;
    x.ww(0x438754, 0)?;
    x.di_str(34, 0x4387a4, s)?;
    x.e.load(
        LoadRequest::NumberToString {
            value: x.e.long(0x43859c)? as i32,
            destination: 0x479ed8,
        },
        s,
    )?;
    x.di_str(36, 0x479ed8, s)?;
    x.tb(0xa97e, None, vec![L(dialog), W(3), W(0), W(250)], s)?;
    x.tb(0xa873, None, vec![L(dialog)], s)?;
    x.tb(0xa97e, None, vec![L(dialog), W(3), W(0), W(250)], s)?;
    x.tb(0xa981, None, vec![L(dialog)], s)?;
    if crate::recovered_verification::doreg() == 0 {
        set_control(&mut x, dialog, 12, 1, s)?;
        x.number(33, 20, s)?;
    }
    x.tb(0xa922, None, vec![L(dialog)], s)?;
    x.tb(0xa923, None, vec![L(dialog)], s)?;
    loop {
        x.e.step()?;
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
        x.tb(0xa991, None, vec![L(0), P(0x4434b2)], s)?;
        if x.e.word(0x4434b2)? == 49 {
            if control(&mut x, dialog, 53, s)? != 0 {
                x.warn(frame.wrapping_sub(76), 130, s)?;
                x.warn(frame.wrapping_sub(78), 131, s)?;
            } else {
                let popup = x.tb(0xa97c, None, vec![W(171), L(0), L(u32::MAX)], s)?;
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
                x.tb(0xa991, None, vec![L(0), P(0x4434b2)], s)?;
                x.tb(0xa983, None, vec![L(popup)], s)?;
                x.e.load(
                    LoadRequest::GetIndexedString {
                        resource: 4,
                        index: x.e.word(0x4434b2)?,
                        destination: 0x479ed8,
                    },
                    s,
                )?;
                x.di_str(45, 0x479ed8, s)?;
                let index = x.e.word(0x4434b2)? as u16;
                let currencies = [
                    0x3f800000, 0x3f800000, 0x3fcccccd, 0x42180000, 0x3f400000, 0x3fc00000,
                    0x40e66666, 0x40c9999a, 0x43910000, 0x3ff33333, 0x44e74000, 0, 0x40066666,
                    0x3fe66666, 0x40f66666, 0x3ff33333, 0x4101999a, 0x3fc66666, 0x420c0000,
                    0x425c0000, 0x3f800000,
                ];
                if (1..=20).contains(&index) {
                    x.lw(frame.wrapping_sub(4), currencies[index as usize])?;
                }
                x.ww(0x4434b2, 0)?;
            }
            totals(&mut x, dialog, frame, s)?;
        } else {
            if x.e.word(0x4434b2)? == 39 {
                break;
            }
            if x.e.word(0x4434b2)? == 2 {
                x.ww(0x438754, 1)?;
                x.ww(0x4471ea, 1)?;
            }
            let item = x.e.word(0x4434b2)?;
            x.get_item(dialog, item, 0x4434dc, s)?;
            if x.e.word(0x4434b0)? == 6 {
                set_control(&mut x, dialog, 53, 0, s)?;
                set_control(&mut x, dialog, 21, 0, s)?;
                let item = x.e.word(0x4434b2)?;
                set_control(&mut x, dialog, item, 1, s)?;
                if x.e.word(0x4434b2)? == 53 {
                    x.warn(frame.wrapping_sub(76), 94, s)?;
                    x.lw(frame.wrapping_sub(4), 0x3f800000)?;
                }
                totals(&mut x, dialog, frame, s)?;
            } else if x.e.word(0x4434b0)? == 5 {
                let item = x.e.word(0x4434b2)?;
                let v = control(&mut x, dialog, item, s)?;
                x.tb(
                    0xa963,
                    None,
                    vec![L(x.e.long(0x4434dc)?), W(u16::from(v as u8 == 0))],
                    s,
                )?;
                totals(&mut x, dialog, frame, s)?;
            }
        }
        if x.e.word(0x438754)? != 0 {
            x.get_item(dialog, 53, frame.wrapping_sub(8), s)?;
            x.get_item(dialog, 21, frame.wrapping_sub(12), s)?;
            let first = x.tb(0xa960, None, vec![L(x.e.long(frame.wrapping_sub(8))?)], s)? as u16;
            if first == 0
                && x.tb(0xa960, None, vec![L(x.e.long(frame.wrapping_sub(12))?)], s)? as u16 == 0
            {
                x.warn(frame.wrapping_sub(76), 93, s)?;
                x.ww(0x438754, 0)?;
                continue;
            }
            if x.e.word(0x4471ea)? != 0 {
                print_form(&mut x, dialog, frame, s)?;
            }
            break;
        }
    }
    x.tb(0xa983, None, vec![L(dialog)], s)?;
    x.e.plan.native_return = Some(x.d0);
    Ok(x.finish())
}
