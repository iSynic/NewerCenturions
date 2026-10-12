//! Battle construction and queue control from CODE 1.

use crate::recovered_application_control::{self as application, Execution, RegisterBits};
use crate::recovered_character_upgrade::UpgradeSaneRequest;
use crate::recovered_feedback::{
    FeedbackArg,
    FeedbackArg::{Long as L, Pointer as P, Word as W},
};

fn child(
    e: &mut Execution<'_>,
    entry: u32,
    arguments: &[FeedbackArg],
    cleanup: u32,
) -> application::Result<()> {
    for argument in arguments {
        match *argument {
            W(value) => {
                e.stack = e.stack.wrapping_sub(2);
                e.word(e.stack, value)?;
            }
            L(value) | P(value) => {
                e.stack = e.stack.wrapping_sub(4);
                e.long(e.stack, value)?;
            }
            FeedbackArg::Byte(value) => {
                e.stack = e.stack.wrapping_sub(2);
                e.byte(e.stack, value)?;
            }
        }
    }
    e.call(entry, e.stack.wrapping_sub(8))?;
    e.stack = e.stack.wrapping_add(cleanup);
    Ok(())
}

fn trap(
    e: &mut Execution<'_>,
    opcode: u16,
    arguments: &[FeedbackArg],
    returned: u32,
) -> application::Result<u32> {
    let top = e.stack;
    e.stack = e.stack.wrapping_sub(returned);
    let slot = e.stack;
    for argument in arguments {
        match *argument {
            W(value) => {
                e.stack = e.stack.wrapping_sub(2);
                e.word(e.stack, value)?;
            }
            L(value) | P(value) => {
                e.stack = e.stack.wrapping_sub(4);
                e.long(e.stack, value)?;
            }
            FeedbackArg::Byte(value) => {
                e.stack = e.stack.wrapping_sub(2);
                e.byte(e.stack, value)?;
            }
        }
    }
    let value = e.trap(opcode, arguments.to_vec())?;
    if returned == 4 {
        e.long(slot, value)?;
        e.registers.address[0] = RegisterBits::full(value);
    }
    e.stack = top;
    Ok(value)
}

fn warrior(member: u16) -> u32 {
    0x448aa0u32.wrapping_add(u32::from(member) * 3138)
}
fn monster(member: u16) -> u32 {
    0x439d3cu32.wrapping_add(u32::from(member.wrapping_sub(10)) * 288)
}
fn coordinate(base: u32, index: u16, vertical: bool) -> u32 {
    base.wrapping_add((i32::from(index as i16) * 4) as u32)
        .wrapping_add(if vertical { 2 } else { 0 })
}
fn queued(e: &Execution<'_>) -> application::Result<u32> {
    Ok(0x437a94u32.wrapping_add(u32::from(e.w(0x43800c)?) * 14))
}

pub fn kill_body_body(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    e.stack = frame.wrapping_sub(72);
    let member = e.w(frame.wrapping_add(8))?;
    let deferred = e.w(frame.wrapping_add(10))?;
    e.registers.data[4].assign_word(member);
    e.registers.data[6].assign_word(deferred);
    e.registers.data[5] = RegisterBits::full(0);
    child(e, 0x13154c, &[W(10)], 2)?;
    if deferred == 0 {
        e.word(0x408574, member)?;
    }
    if e.w(0x438776)? == 0 {
        // The repeated test and its statically unreachable arm are retained.
        if e.w(0x438776)? != 0 {
            child(e, 0x15b2f6, &[W(0), W(member)], 4)?;
        }
        e.byte(0x442739, e.b(0x442739)?.wrapping_add(1))?;
        let sound = e.rand(2, e.stack)?.wrapping_add(506);
        child(e, 0x133a18, &[W(sound as u16)], 0)?;
        child(e, 0x1600fc, &[W(1), W(member)], 0)?;
        e.stack = e.stack.wrapping_sub(4);
        e.long(e.stack, 0x0190001e)?;
        e.trap(0xa893, vec![W(30), W(400)])?;
        e.stack = e.stack.wrapping_add(4);
        let dead = e.sw(warrior(member).wrapping_add(0x50))? < -9;
        e.stack = e.stack.wrapping_add(6);
        trap(e, 0xa884, &[P(if dead { 0x403294 } else { 0x403286 })], 0)?;
        child(e, 0x1205f8, &[W(0)], 0)?;
        let selected = e.sb(0x44273f)? as i16 as u16;
        e.stack = e.stack.wrapping_add(2);
        if member == selected {
            child(e, 0x16043e, &[], 0)?;
        }
        child(e, 0x1500d8, &[W(0)], 2)?;
    } else {
        let mut transition = deferred != 0;
        if deferred == 0 {
            kill_record(e, frame, member)?;
            kill_icon(e, frame)?;
            e.registers.data[3] = RegisterBits::full(1);
            for index in 1..110u16 {
                e.step()?;
                let address = 0x4470b8u32.wrapping_add(u32::from(index) * 2);
                if e.w(address)? == member {
                    e.word(address, u16::MAX)?;
                }
                e.registers.data[3].assign_word(index.wrapping_add(1));
            }
            if (member as i16) < 9 {
                let reason = if e.sw(warrior(member).wrapping_add(0x50))? <= -10 {
                    -2i16
                } else {
                    -11
                };
                child(e, 0x158454, &[W(reason as u16), W(member)], 4)?;
            } else {
                let npc = monster(member);
                let reason = if e.b(npc.wrapping_add(0xab))? != 0 {
                    -14i16
                } else if e.sw(npc.wrapping_add(0x110))? <= -10 {
                    -2
                } else {
                    -11
                };
                child(e, 0x158454, &[W(reason as u16), W(member)], 4)?;
                if e.w(npc.wrapping_add(0x60))? != 0 {
                    if e.b(0x438585)? == 0 && e.w(0x438760)? == 0 {
                        transition = true;
                    } else {
                        e.word(queued(e)?, e.sb(npc.wrapping_add(0x103))? as i16 as u16)?;
                        e.word(queued(e)?.wrapping_add(2), e.w(npc.wrapping_add(0xe))?)?;
                        e.word(queued(e)?.wrapping_add(4), e.w(npc.wrapping_add(0x60))?)?;
                        let address = coordinate(0x446f0a, member.wrapping_sub(10), false);
                        let x = e.l(0x442780)?.wrapping_add(e.sw(address)? as i32 as u32);
                        e.long(queued(e)?.wrapping_add(6), x)?;
                        let address = coordinate(0x446f0a, member.wrapping_sub(10), true);
                        let y = e.l(0x442784)?.wrapping_add(e.sw(address)? as i32 as u32);
                        e.long(queued(e)?.wrapping_add(10), y)?;
                        e.addw(0x43800c, 1)?;
                    }
                }
            }
        }
        if transition {
            kill_transition(e, frame, member, deferred)?;
        } else {
            child(e, 0x1205f8, &[W(15)], 0)?;
            child(e, 0x105572, &[W(0), W(member)], 0)?;
            child(e, 0x105366, &[W(member)], 8)?;
        }
        child(
            e,
            0x1382a8,
            &[L(e.l(0x4436fe)?), L(e.l(0x4436fa)?), W(207)],
            10,
        )?;
    }
    kill_finish(e, member, deferred)?;
    child(e, 0x13154c, &[W(11)], 2)?;
    e.moveq_return(0);
    Ok(())
}

fn kill_record(e: &mut Execution<'_>, frame: u32, member: u16) -> application::Result<()> {
    if (member as i16) > 9 {
        let npc = monster(member);
        e.word(frame.wrapping_sub(52), e.w(npc.wrapping_add(0x110))?)?;
        e.word(npc.wrapping_add(0x110), 1)?;
        child(e, 0x13a2c4, &[W(member)], 0)?;
        e.word(npc.wrapping_add(0x110), e.w(frame.wrapping_sub(52))?)?;
        let size = e.w(npc.wrapping_add(0x10))?;
        e.stack = e.stack.wrapping_sub(2);
        e.word(e.stack, size)?;
        let observation = e.lib(0x400038, vec![W(size)])?;
        e.word(0x4435ce, observation.d0 as u16)?;
        let y = e
            .w(coordinate(0x446f0a, member.wrapping_sub(10), true))?
            .wrapping_shl(5);
        e.word(frame.wrapping_sub(50), y)?;
        let x = e
            .w(coordinate(0x446f0a, member.wrapping_sub(10), false))?
            .wrapping_shl(5);
        e.word(frame.wrapping_sub(48), x)?;
        let exists = e.w(npc.wrapping_add(0xe))? != 0;
        e.stack = e.stack.wrapping_add(4);
        if exists && e.b(npc.wrapping_add(0xb0))? != 255 {
            e.addw(0x44374c, 1)?;
        }
        e.byte(npc.wrapping_add(0x106), 0)?;
        e.registers.data[3] = RegisterBits::full(0);
        for index in 0..40u16 {
            e.step()?;
            e.byte(npc.wrapping_add(0xb2).wrapping_add(u32::from(index)), 0)?;
            e.registers.data[3].assign_word(index.wrapping_add(1));
        }
    } else {
        let actor = warrior(member);
        let health = e.w(actor.wrapping_add(0x50))?;
        e.registers.data[7].assign_word(health);
        e.word(actor.wrapping_add(0x50), 1)?;
        e.word(
            frame.wrapping_sub(50),
            e.w(coordinate(0x44709a, member, true))?.wrapping_shl(5),
        )?;
        e.word(
            frame.wrapping_sub(48),
            e.w(coordinate(0x44709a, member, false))?.wrapping_shl(5),
        )?;
        e.word(actor.wrapping_add(0x50), health)?;
        e.byte(0x442742u32.wrapping_add(i32::from(member as i16) as u32), 0)?;
        e.byte(actor.wrapping_add(0x9df), 0)?;
        e.word(0x4435ce, 0)?;
        if health as i16 <= -10 {
            e.addl(actor.wrapping_add(0xa5e), 1)?;
            child(e, 0x12382a, &[L(0x00770077)], 4)?;
        } else {
            e.byte(actor.wrapping_add(0x9de), 1)?;
            e.addl(actor.wrapping_add(0xa62), 1)?;
            child(e, 0x12382a, &[L(0x004f004f)], 4)?;
        }
        e.word(actor.wrapping_add(0x20), u16::MAX)?;
        child(e, 0x160fb2, &[W(0), W(member)], 4)?;
        if e.w(actor.wrapping_add(0x10))? != 0 {
            e.addw(0x44374c, 1)?;
        } else if e.w(actor.wrapping_add(0x10))? == 0 && e.sw(actor.wrapping_add(0x90))? < 1 {
            e.byte(0x442739, e.b(0x442739)?.wrapping_add(1))?;
        }
        e.word(actor.wrapping_add(0x90), 0)?;
        e.registers.data[3] = RegisterBits::full(0);
        for index in 0..40u16 {
            e.step()?;
            let address = actor.wrapping_add(0x5e).wrapping_add(u32::from(index) * 2);
            if e.sw(address)? > 0 {
                e.word(address, 0)?;
            }
            e.registers.data[3].assign_word(index.wrapping_add(1));
        }
    }
    Ok(())
}

fn kill_icon(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    e.word(
        frame.wrapping_sub(46),
        e.w(frame.wrapping_sub(50))?.wrapping_add(32),
    )?;
    e.word(
        frame.wrapping_sub(44),
        e.w(frame.wrapping_sub(48))?.wrapping_add(32),
    )?;
    if e.w(0x4435ce)? == 1 || e.w(0x4435ce)? == 3 {
        e.addw(frame.wrapping_sub(50), -32)?;
    }
    if e.sw(0x4435ce)? > 1 {
        e.addw(frame.wrapping_sub(48), -32)?;
    }
    trap(e, 0xa873, &[L(e.l(0x43879c)?)], 0)?;
    e.long(0x439598, 0)?;
    let icon = trap(e, 0xaa1e, &[W(e.w(0x4435ce)?.wrapping_add(2015))], 4)?;
    e.long(0x439598, icon)?;
    if e.l(0x439598)? != 0 {
        trap(
            e,
            0xaa1f,
            &[P(frame.wrapping_sub(50)), L(e.l(0x439598)?)],
            0,
        )?;
        trap(e, 0xaa25, &[L(e.l(0x439598)?)], 0)?;
    }
    Ok(())
}

fn kill_transition(
    e: &mut Execution<'_>,
    frame: u32,
    member: u16,
    deferred: u16,
) -> application::Result<()> {
    e.byte(0x438585, 1)?;
    e.registers.data[3] = RegisterBits::full(0);
    for index in 0..8u16 {
        e.step()?;
        let offset = u32::from(index) * 2;
        e.word(
            frame.wrapping_sub(34).wrapping_add(offset),
            e.w(0x479eb2 + offset)?,
        )?;
        e.word(
            frame.wrapping_sub(18).wrapping_add(offset),
            e.w(0x479ec2 + offset)?,
        )?;
        e.registers.data[3].assign_word(index.wrapping_add(1));
    }
    if deferred == 0 {
        let npc = monster(member);
        child(e, 0x123c60, &[W(e.w(npc.wrapping_add(0x60))?)], 0)?;
        child(e, 0x105572, &[W(0), W(member)], 0)?;
        child(e, 0x105366, &[W(member)], 0)?;
        e.word(0x43800e, member.wrapping_sub(10))?;
        child(
            e,
            0x10e6b6,
            &[
                W(e.w(coordinate(0x446f0a, member.wrapping_sub(10), true))?),
                W(e.w(coordinate(0x446f0a, member.wrapping_sub(10), false))?),
            ],
            0,
        )?;
        child(
            e,
            0x1400e4,
            &[
                W(member),
                W(e.w(npc.wrapping_add(0x60))?),
                W(u16::MAX),
                L(0),
                L(0),
            ],
            0,
        )?;
        e.word(0x43800e, 0)?;
        e.stack = e.stack.wrapping_add(26);
    } else {
        let node = 0x437a94u32.wrapping_add(u32::from(deferred.wrapping_sub(1)) * 14);
        child(e, 0x123c60, &[W(e.w(node.wrapping_add(4))?)], 0)?;
        child(
            e,
            0x1400e4,
            &[
                W(deferred.wrapping_neg()),
                W(e.w(node.wrapping_add(4))?),
                W(u16::MAX),
                L(0),
                L(0),
            ],
            16,
        )?;
    }
    let selected = 0x4470b8u32.wrapping_add((i32::from(e.sw(0x447194)?) * 2) as u32);
    if e.w(selected)? != u16::MAX {
        let selected = 0x4470b8u32.wrapping_add((i32::from(e.sw(0x447194)?) * 2) as u32);
        child(e, 0x11f1a8, &[W(e.w(selected)?)], 2)?;
    }
    e.registers.data[3] = RegisterBits::full(0);
    for index in 0..8u16 {
        e.step()?;
        let offset = u32::from(index) * 2;
        e.word(
            0x479eb2 + offset,
            e.w(frame.wrapping_sub(34).wrapping_add(offset))?,
        )?;
        e.word(
            0x479ec2 + offset,
            e.w(frame.wrapping_sub(18).wrapping_add(offset))?,
        )?;
        e.registers.data[3].assign_word(index.wrapping_add(1));
    }
    e.byte(0x438582, 0)?;
    e.byte(0x438585, 0)?;
    Ok(())
}

fn kill_finish(e: &mut Execution<'_>, member: u16, deferred: u16) -> application::Result<()> {
    if e.sb(0x442739)? > e.sb(0x44273a)? {
        child(e, 0x14913e, &[W(0)], 2)?;
    }
    if e.w(0x438776)? != 0 {
        if deferred == 0 {
            let mut unavailable = 0u16;
            let mut index = 0u16;
            e.registers.data[3] = RegisterBits::full(0);
            while index as i16 <= i16::from(e.sb(0x44273a)?) {
                e.step()?;
                let actor = warrior(index);
                if e.w(actor.wrapping_add(0x20))? == u16::MAX
                    || e.sw(actor.wrapping_add(0x50))? < 1
                    || e.w(actor.wrapping_add(0x10))? != 0
                    || e.sw(actor.wrapping_add(0x90))? < 0
                {
                    unavailable = unavailable.wrapping_add(1);
                    e.registers.data[5].assign_word(unavailable);
                }
                index = index.wrapping_add(1);
                e.registers.data[3].assign_word(index);
            }
            if unavailable as i16 > i16::from(e.sb(0x44273a)?) {
                e.word(0x44374c, e.w(0x4435c4)?)?;
                e.byte(0x4386fc, 1)?;
                child(e, 0x125a3a, &[W(0)], 2)?;
            }
        }
        let mut index = 0u16;
        e.registers.data[3] = RegisterBits::full(0);
        while index as i16 <= i16::from(e.sb(0x44273a)?) {
            e.step()?;
            let address = warrior(index).wrapping_add(0x18);
            if e.w(address)? == member {
                e.word(address, u16::MAX)?;
            }
            index = index.wrapping_add(1);
            e.registers.data[3].assign_word(index);
        }
        let mut index = 0u16;
        e.registers.data[3] = RegisterBits::full(0);
        while (index as i16) < e.sw(0x43874a)? {
            e.step()?;
            let address = 0x439d3cu32
                .wrapping_add(u32::from(index) * 288)
                .wrapping_add(0x10c);
            if e.w(address)? == member {
                e.word(address, u16::MAX)?;
            }
            index = index.wrapping_add(1);
            e.registers.data[3].assign_word(index);
        }
    }
    Ok(())
}

fn sane(e: &mut Execution<'_>, pointers: &[u32], selector: u16) -> application::Result<()> {
    let top = e.stack;
    for pointer in pointers {
        e.stack = e.stack.wrapping_sub(4);
        e.long(e.stack, *pointer)?;
    }
    e.stack = e.stack.wrapping_sub(2);
    e.word(e.stack, selector)?;
    let mut arguments = pointers.iter().copied().map(P).collect::<Vec<_>>();
    arguments.push(W(selector));
    e.sane(UpgradeSaneRequest {
        opcode: 0xa9eb,
        arguments,
    })?;
    e.stack = top;
    Ok(())
}

pub fn empty_queue_body(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    e.stack = frame.wrapping_sub(4);
    e.byte(0x438584, 1)?;
    let mut index = 0u16;
    e.registers.data[3] = RegisterBits::full(0);
    while (index as i16) < e.sw(0x43800c)? {
        e.step()?;
        let node = 0x437a98u32.wrapping_add(u32::from(index) * 14);
        if e.w(node)? != 0 {
            e.word(e.stack.wrapping_sub(2), index.wrapping_add(1))?;
            e.word(e.stack.wrapping_sub(4), 0)?;
            e.call(0x12e4e4, e.stack.wrapping_sub(12))?;
        }
        e.word(node, 0)?;
        index = index.wrapping_add(1);
        e.registers.data[3].assign_word(index);
    }
    e.word(0x43800c, 0)?;
    e.word(0x437a90, 0)?;
    e.byte(0x438584, 0)?;
    e.moveq_return(0);
    // The final saved-D3 MOVE is outside this projected register bookkeeping.
    e.registers.ccr_known_mask = 0;
    Ok(())
}

pub fn build_monster_body(e: &mut Execution<'_>, frame: u32) -> application::Result<()> {
    let stack = frame.wrapping_sub(46);
    e.stack = stack;
    e.long(frame.wrapping_sub(8), 0)?;
    e.word(0x440ecc, e.w(0x440dbe)?)?;
    let mut index = 0u16;
    e.registers.data[3] = RegisterBits::full(0);
    while (index as i16) < e.sw(0x440dbc)? {
        e.step()?;
        let draw = e.rand(8, stack)?;
        e.addw(0x440ecc, draw)?;
        index = index.wrapping_add(1);
        e.registers.data[3].assign_word(index);
    }
    e.word(0x440ece, e.w(0x440ecc)?)?;
    if e.sw(frame.wrapping_add(8))? > -1 {
        e.word(0x440dca, e.w(frame.wrapping_add(8))?)?;
    }
    e.byte(0x440ec4, 0)?;
    e.byte(0x440ec3, 0)?;
    if e.w(0x440dc4)? != 0 {
        let draw = e.rand(3, stack)?.wrapping_sub(2);
        e.addw(0x440dc4, draw)?;
    }
    if e.sw(0x440dc4)? < 0 {
        e.word(0x440dc4, 0)?;
    }
    let draw = e.rand(3, stack)?.wrapping_sub(2);
    e.addw(0x440dc0, draw)?;
    if e.sw(0x440dc0)? < 1 {
        e.word(0x440dc0, 1)?;
    }
    let tenth = e.sw(0x440eca)? / 10;
    e.word(0x4434b6, tenth as u16)?;
    let maximum = e.sw(0x4434b6)?;
    let minimum = e.sw(0x4434b6)?.wrapping_neg();
    let draw = e.range(minimum, maximum, stack)?;
    e.addw(0x440eca, draw)?;
    if e.sw(0x440dc6)? < 99 {
        e.addw(0x440dc6, e.sw(0x442576)?.wrapping_mul(3))?;
    }
    e.registers.data[3] = RegisterBits::full(0);
    for index in 0..6u16 {
        e.step()?;
        let addition = e.sw(0x442576)?.wrapping_mul(7) as u8;
        let address = 0x440e47u32.wrapping_add(u32::from(index));
        e.byte(address, e.b(address)?.wrapping_add(addition))?;
        e.registers.data[3].assign_word(index.wrapping_add(1));
    }
    e.addw(0x440dc4, e.sw(0x442576)?.wrapping_mul(2).wrapping_neg())?;
    e.addw(0x440dc0, e.sw(0x442576)?)?;

    let count = frame.wrapping_sub(42);
    let constant = frame.wrapping_sub(32);
    let extended = frame.wrapping_sub(22);
    let single = frame.wrapping_sub(12);
    let factor = frame.wrapping_sub(8);
    let work = frame.wrapping_sub(4);
    e.word(count, e.w(0x442576)?)?;
    sane(e, &[count, count], 0x200e)?;
    e.long(constant, 0x3ffda8f5)?;
    e.long(constant.wrapping_add(4), 0xc28f5c28)?;
    e.word(constant.wrapping_add(8), 0xf5c3)?;
    sane(e, &[count, constant], 4)?;
    e.long(extended, 0x3fff8000)?;
    e.long(extended.wrapping_add(4), 0)?;
    e.word(extended.wrapping_add(8), 0)?;
    sane(e, &[constant, extended], 0)?;
    sane(e, &[extended, single], 0x1010)?;
    e.long(factor, e.l(single)?)?;
    e.word(extended, e.w(0x440eca)?)?;
    scale(e, extended, factor, work)?;
    e.word(0x440eca, e.w(extended.wrapping_add(2))?)?;
    e.word(0x440e0e, e.w(0x440eca)?)?;
    e.word(extended, e.w(0x440ecc)?)?;
    scale(e, extended, factor, work)?;
    e.word(0x440ecc, e.w(extended.wrapping_add(2))?)?;
    if e.sw(0x440ecc)? < 1 {
        e.word(0x440ecc, 1)?;
    }
    e.word(0x440ece, e.w(0x440ecc)?)?;
    if e.sw(0x440e0a)? < 0 {
        e.word(stack.wrapping_sub(2), 0)?;
        e.word(stack.wrapping_sub(4), e.w(0x440e0a)?)?;
        e.call(0x11d406, stack.wrapping_sub(12))?;
        e.word(0x440de2, e.d0_word()?)?;
        e.word(0x440e0a, e.d0_word()?)?;
    }
    e.moveq_return(0);
    e.registers.ccr_known_mask = 0;
    Ok(())
}

fn scale(e: &mut Execution<'_>, extended: u32, factor: u32, work: u32) -> application::Result<()> {
    sane(e, &[extended, extended], 0x200e)?;
    sane(e, &[extended, extended], 0x1010)?;
    e.long(work, e.l(extended)?)?;
    sane(e, &[work, extended], 0x100e)?;
    sane(e, &[factor, extended], 0x1004)?;
    sane(e, &[extended, work], 0x1010)?;
    sane(e, &[work, extended], 0x100e)?;
    sane(e, &[extended], 0x16)?;
    sane(e, &[extended, extended], 0x2810)?;
    Ok(())
}

crate::recovered_application_control::entry!(kill_body, 0x12e4e4);
crate::recovered_application_control::entry!(empty_queue, 0x121524);
crate::recovered_application_control::entry!(build_monster, 0x109210);
