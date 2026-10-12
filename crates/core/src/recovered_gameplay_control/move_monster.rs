//! Complete provisional gameplay control from canonical CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{ApplicationControlServices, Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};

pub(crate) fn move_monster_body(
    e: &mut Execution<'_, impl ApplicationControlServices>,
    frame: u32,
) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x13e276_u32;
        loop {
            match pc {
                0x13e276 => {
                    // 0013e276: link.w a6, #$ffec
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -20)?;
                    // 0013e27a: movem.l d3-d7, -(a7)
                    e.step()?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 0013e27e: move.w #$1, -$e(a6)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e284: clr.w -$14(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e288: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e28e: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e292: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e298: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e29a: tst.b $b2(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xb2_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0013e29e: beq.b $13e2b2
                    e.step()?;
                    pc = if r.equal()? {
                        0x13e2b2_u32
                    } else {
                        0x13e2a0_u32
                    };
                }
                0x13e2a0 => {
                    // 0013e2a0: move.w #$ffff, -$e(a6)
                    e.step()?;
                    let value = 0xffff_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e2a6: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e2aa: bsr.l $110e96
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e2b0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x110e96_u32)?;
                    // 0013e2b0: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x13e2b2_u32;
                }
                0x13e2b2 => {
                    // 0013e2b2: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0013e2b4: move.w d5, d6
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 0013e2b6: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e2bc: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e2c0: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e2c6: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e2c8: move.w $10c(a0), d7
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x10c_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(7, 16, value);
                    r.move_flags(value, 16);
                    // 0013e2cc: cmpi.w #$9, d7
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(7, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e2d0: bge.b $13e2ec
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x13e2ec_u32
                    } else {
                        0x13e2d2_u32
                    };
                }
                0x13e2d2 => {
                    // 0013e2d2: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0013e2d8: move.w (a0, d7.w * 4), -$10(a6)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(7, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e2de: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0013e2e4: move.w $2(a0, d7.w * 4), -$12(a6)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(7, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e2ea: bra.b $13e30c
                    e.step()?;
                    pc = 0x13e30c_u32;
                }
                0x13e2ec => {
                    // 0013e2ec: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 0013e2ee: add.w d7, d0
                    e.step()?;
                    let source = r.data(7, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e2f0: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013e2f6: move.w (a0, d0.w * 4), -$10(a6)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e2fc: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 0013e2fe: add.w d7, d0
                    e.step()?;
                    let source = r.data(7, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e300: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013e306: move.w $2(a0, d0.w * 4), -$12(a6)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x13e30c_u32;
                }
                0x13e30c => {
                    // 0013e30c: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013e312: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e318: move.w (a0, d0.w * 4), -$a(a6)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e31e: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013e324: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e32a: move.w $2(a0, d0.w * 4), -$c(a6)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e330: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e336: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e33a: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e340: move.w $10(a0, d0.l), -(a7)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x10_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e344: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e34a_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 0013e34a: move.w d0, $4435ce.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4435ce_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e350: clr.b $442737.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0013e356: clr.b $442736.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0013e35c: move.w -$a(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e360: cmp.w -$10(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e364: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0013e366: bge.b $13e370
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x13e370_u32
                    } else {
                        0x13e368_u32
                    };
                }
                0x13e368 => {
                    // 0013e368: move.b -$d(a6), $442736.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff3_u32);
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x13e370_u32;
                }
                0x13e370 => {
                    // 0013e370: move.w -$a(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e374: cmp.w -$10(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e378: ble.b $13e386
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13e386_u32
                    } else {
                        0x13e37a_u32
                    };
                }
                0x13e37a => {
                    // 0013e37a: moveq #$ff, d0
                    e.step()?;
                    r.put_data(0, 32, 0xffffffff_u32);
                    r.move_flags(0xffffffff_u32, 32);
                    // 0013e37c: muls.w -$e(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e380: move.b d0, $442736.l
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x13e386_u32;
                }
                0x13e386 => {
                    // 0013e386: move.w -$c(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e38a: cmp.w -$12(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xffffffee_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e38e: bge.b $13e398
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x13e398_u32
                    } else {
                        0x13e390_u32
                    };
                }
                0x13e390 => {
                    // 0013e390: move.b -$d(a6), $442737.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff3_u32);
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x13e398_u32;
                }
                0x13e398 => {
                    // 0013e398: move.w -$c(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e39c: cmp.w -$12(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xffffffee_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e3a0: ble.b $13e3ae
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13e3ae_u32
                    } else {
                        0x13e3a2_u32
                    };
                }
                0x13e3a2 => {
                    // 0013e3a2: moveq #$ff, d0
                    e.step()?;
                    r.put_data(0, 32, 0xffffffff_u32);
                    r.move_flags(0xffffffff_u32, 32);
                    // 0013e3a4: muls.w -$e(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e3a8: move.b d0, $442737.l
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x13e3ae_u32;
                }
                0x13e3ae => {
                    // 0013e3ae: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e3b4: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e3b8: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e3be: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e3c0: move.b $442736.l, $107(a0)
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = r.address(0)?.wrapping_add(0x107_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0013e3c8: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e3ce: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e3d2: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e3d8: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e3da: move.b $442737.l, $108(a0)
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = r.address(0)?.wrapping_add(0x108_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x13e3e2_u32;
                }
                0x13e3e2 => {
                    // 0013e3e2: move.w -$14(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffec_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e3e6: addq.w #$1, -$14(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0013e3ea: cmpi.w #$14, d0
                    e.step()?;
                    let source = 0x14_u32;
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e3ee: ble.b $13e424
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13e424_u32
                    } else {
                        0x13e3f0_u32
                    };
                }
                0x13e3f0 => {
                    // 0013e3f0: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e3f6: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e3fa: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e400: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e402: move.w #$ffff, $10c(a0)
                    e.step()?;
                    let value = 0xffff_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e408: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e40e: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e412: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e418: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e41a: clr.b $106(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x106_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0013e41e: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0013e420: bra.w $13ea6c
                    e.step()?;
                    pc = 0x13ea6c_u32;
                }
                0x13e424 => {
                    // 0013e424: move.b $402f43.l, d0
                    e.step()?;
                    let value_address = 0x402f43_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013e42a: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e42c: lea.l $435508.l, a0
                    e.step()?;
                    let value = 0x435508_u32;
                    r.put_address(0, value);
                    // 0013e432: move.w (a0, d0.l * 2), -$2(a6)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e438: move.w (a0, d0.l * 2), -$4(a6)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e43e: move.w (a0, d0.l * 2), -$6(a6)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e444: move.w (a0, d0.l * 2), -$8(a6)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e44a: movea.w -$a(a6), a0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0013e44e: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e450: add.l $442780.l, d0
                    e.step()?;
                    let source_address = 0x442780_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e456: move.b $442736.l, d1
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e45c: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e45e: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e460: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e462: muls.l #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 0013e46a: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 0013e470: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e472: movea.w -$c(a6), a1
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(1, value);
                    // 0013e476: move.l a1, d0
                    e.step()?;
                    let value = r.address(1)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e478: add.l $442784.l, d0
                    e.step()?;
                    let source_address = 0x442784_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e47e: move.b $442737.l, d1
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e484: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e486: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e488: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e48a: move.w (a0, d1.l * 2), -(a7)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e48e: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e494_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 0013e494: move.w d0, -$8(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e498: cmpi.w #$1, $4435ce.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4435ce_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e4a0: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0013e4a2: beq.b $13e4ae
                    e.step()?;
                    pc = if r.equal()? {
                        0x13e4ae_u32
                    } else {
                        0x13e4a4_u32
                    };
                }
                0x13e4a4 => {
                    // 0013e4a4: cmpi.w #$3, $4435ce.l
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = 0x4435ce_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e4ac: bne.b $13e500
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e500_u32
                    } else {
                        0x13e4ae_u32
                    };
                }
                0x13e4ae => {
                    // 0013e4ae: movea.w -$a(a6), a0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0013e4b2: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e4b4: add.l $442780.l, d0
                    e.step()?;
                    let source_address = 0x442780_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e4ba: move.b $442736.l, d1
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e4c0: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e4c2: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e4c4: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e4c6: muls.l #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 0013e4ce: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 0013e4d4: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e4d6: movea.w -$c(a6), a1
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(1, value);
                    // 0013e4da: move.l a1, d0
                    e.step()?;
                    let value = r.address(1)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e4dc: add.l $442784.l, d0
                    e.step()?;
                    let source_address = 0x442784_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e4e2: move.b $442737.l, d1
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e4e8: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e4ea: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e4ec: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e4ee: subq.l #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e4f0: move.w (a0, d1.l * 2), -(a7)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e4f4: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e4fa_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 0013e4fa: move.w d0, -$6(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e4fe: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x13e500_u32;
                }
                0x13e500 => {
                    // 0013e500: cmpi.w #$1, $4435ce.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4435ce_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e508: ble.b $13e55c
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13e55c_u32
                    } else {
                        0x13e50a_u32
                    };
                }
                0x13e50a => {
                    // 0013e50a: movea.w -$a(a6), a0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0013e50e: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e510: add.l $442780.l, d0
                    e.step()?;
                    let source_address = 0x442780_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e516: move.b $442736.l, d1
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e51c: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e51e: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e520: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e522: subq.l #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e524: muls.l #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 0013e52c: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 0013e532: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e534: movea.w -$c(a6), a1
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(1, value);
                    // 0013e538: move.l a1, d0
                    e.step()?;
                    let value = r.address(1)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e53a: add.l $442784.l, d0
                    e.step()?;
                    let source_address = 0x442784_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e540: move.b $442737.l, d1
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e546: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e548: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e54a: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e54c: move.w (a0, d1.l * 2), -(a7)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e550: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e556_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 0013e556: move.w d0, -$4(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e55a: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x13e55c_u32;
                }
                0x13e55c => {
                    // 0013e55c: cmpi.w #$3, $4435ce.l
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = 0x4435ce_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e564: bne.b $13e5ba
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e5ba_u32
                    } else {
                        0x13e566_u32
                    };
                }
                0x13e566 => {
                    // 0013e566: movea.w -$a(a6), a0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0013e56a: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e56c: add.l $442780.l, d0
                    e.step()?;
                    let source_address = 0x442780_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e572: move.b $442736.l, d1
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e578: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e57a: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e57c: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e57e: subq.l #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e580: muls.l #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 0013e588: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 0013e58e: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e590: movea.w -$c(a6), a1
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(1, value);
                    // 0013e594: move.l a1, d0
                    e.step()?;
                    let value = r.address(1)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e596: add.l $442784.l, d0
                    e.step()?;
                    let source_address = 0x442784_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e59c: move.b $442737.l, d1
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e5a2: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e5a4: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e5a6: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e5a8: subq.l #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e5aa: move.w (a0, d1.l * 2), -(a7)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e5ae: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e5b4_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 0013e5b4: move.w d0, -$2(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e5b8: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x13e5ba_u32;
                }
                0x13e5ba => {
                    // 0013e5ba: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0013e5bc: bra.w $13e718
                    e.step()?;
                    pc = 0x13e718_u32;
                }
                0x13e5c0 => {
                    // 0013e5c0: tst.b $442736.l
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0013e5c6: bne.b $13e5d2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e5d2_u32
                    } else {
                        0x13e5c8_u32
                    };
                }
                0x13e5c8 => {
                    // 0013e5c8: tst.b $442737.l
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0013e5ce: beq.w $13e6b2
                    e.step()?;
                    pc = if r.equal()? {
                        0x13e6b2_u32
                    } else {
                        0x13e5d2_u32
                    };
                }
                0x13e5d2 => {
                    // 0013e5d2: cmpi.w #$2, $4435ce.l
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = 0x4435ce_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e5da: bne.b $13e5e4
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e5e4_u32
                    } else {
                        0x13e5dc_u32
                    };
                }
                0x13e5dc => {
                    // 0013e5dc: cmpi.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e5e0: bne.b $13e5e4
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e5e4_u32
                    } else {
                        0x13e5e2_u32
                    };
                }
                0x13e5e2 => {
                    // 0013e5e2: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x13e5e4_u32;
                }
                0x13e5e4 => {
                    // 0013e5e4: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 0013e5ea: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e5f0: lea.l -$8(a6), a1
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.put_address(1, value);
                    // 0013e5f4: move.w (a1, d3.w * 2), d1
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e5f8: cmp.w (a0, d0.w * 2), d1
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e5fc: bne.b $13e61c
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e61c_u32
                    } else {
                        0x13e5fe_u32
                    };
                }
                0x13e5fe => {
                    // 0013e5fe: move.b $402f43.l, d0
                    e.step()?;
                    let value_address = 0x402f43_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013e604: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e606: lea.l $435508.l, a0
                    e.step()?;
                    let value = 0x435508_u32;
                    r.put_address(0, value);
                    // 0013e60c: move.w (a0, d0.l * 2), d0
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e610: addi.w #$3e8, d0
                    e.step()?;
                    let source = 0x3e8_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e614: lea.l -$8(a6), a0
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.put_address(0, value);
                    // 0013e618: move.w d0, (a0, d3.w * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x13e61c_u32;
                }
                0x13e61c => {
                    // 0013e61c: lea.l -$8(a6), a0
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.put_address(0, value);
                    // 0013e620: cmpi.w #$3e7, (a0, d3.w * 2)
                    e.step()?;
                    let source = 0x3e7_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e626: ble.b $13e644
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13e644_u32
                    } else {
                        0x13e628_u32
                    };
                }
                0x13e628 => {
                    // 0013e628: lea.l -$8(a6), a0
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.put_address(0, value);
                    // 0013e62c: move.w (a0, d3.w * 2), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e630: addi.w #$fc18, d0
                    e.step()?;
                    let source = 0xfc18_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e634: mulu.w #$28, d0
                    e.step()?;
                    let source = 0x28_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e638: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 0013e63e: tst.w $4(a0, d0.l)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x4_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0013e642: bne.b $13e652
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e652_u32
                    } else {
                        0x13e644_u32
                    };
                }
                0x13e644 => {
                    // 0013e644: lea.l -$8(a6), a0
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.put_address(0, value);
                    // 0013e648: cmpi.w #$3e8, (a0, d3.w * 2)
                    e.step()?;
                    let source = 0x3e8_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e64e: bge.w $13e6e0
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x13e6e0_u32
                    } else {
                        0x13e652_u32
                    };
                }
                0x13e652 => {
                    // 0013e652: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0013e654: lea.l -$8(a6), a0
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.put_address(0, value);
                    // 0013e658: cmpi.w #$3e8, (a0, d3.w * 2)
                    e.step()?;
                    let source = 0x3e8_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e65e: bge.b $13e662
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x13e662_u32
                    } else {
                        0x13e660_u32
                    };
                }
                0x13e660 => {
                    // 0013e660: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x13e662_u32;
                }
                0x13e662 => {
                    // 0013e662: tst.w $4435ce.l
                    e.step()?;
                    let value_address = 0x4435ce_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0013e668: bne.b $13e682
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e682_u32
                    } else {
                        0x13e66a_u32
                    };
                }
                0x13e66a => {
                    // 0013e66a: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e670: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e674: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e67a: tst.b $7f(a0, d0.l)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x7f_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0013e67e: bne.b $13e682
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e682_u32
                    } else {
                        0x13e680_u32
                    };
                }
                0x13e680 => {
                    // 0013e680: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x13e682_u32;
                }
                0x13e682 => {
                    // 0013e682: lea.l -$8(a6), a0
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.put_address(0, value);
                    // 0013e686: cmpi.w #$3e8, (a0, d3.w * 2)
                    e.step()?;
                    let source = 0x3e8_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e68c: ble.b $13e6ae
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13e6ae_u32
                    } else {
                        0x13e68e_u32
                    };
                }
                0x13e68e => {
                    // 0013e68e: lea.l -$8(a6), a0
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.put_address(0, value);
                    // 0013e692: move.w (a0, d3.w * 2), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e696: addi.w #$fc18, d0
                    e.step()?;
                    let source = 0xfc18_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e69a: mulu.w #$28, d0
                    e.step()?;
                    let source = 0x28_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e69e: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 0013e6a4: cmpi.w #$1, $4(a0, d0.l)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x4_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e6aa: ble.b $13e6ae
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13e6ae_u32
                    } else {
                        0x13e6ac_u32
                    };
                }
                0x13e6ac => {
                    // 0013e6ac: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x13e6ae_u32;
                }
                0x13e6ae => {
                    // 0013e6ae: tst.b d4
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.move_flags(value, 8);
                    // 0013e6b0: beq.b $13e6e0
                    e.step()?;
                    pc = if r.equal()? {
                        0x13e6e0_u32
                    } else {
                        0x13e6b2_u32
                    };
                }
                0x13e6b2 => {
                    // 0013e6b2: move.w d7, -(a7)
                    e.step()?;
                    let value = r.data(7, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e6b4: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013e6ba: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e6c0: move.w $2(a0, d0.w * 4), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e6c4: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013e6ca: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e6d0: move.w (a0, d0.w * 4), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e6d4: bsr.l $153e44
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e6da_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153e44_u32)?;
                    // 0013e6da: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0013e6dc: bra.w $13e3e2
                    e.step()?;
                    pc = 0x13e3e2_u32;
                }
                0x13e6e0 => {
                    // 0013e6e0: lea.l -$8(a6), a0
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.put_address(0, value);
                    // 0013e6e4: move.w (a0, d3.w * 2), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e6e8: bsr.l $153de6
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e6ee_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153de6_u32)?;
                    // 0013e6ee: move.w d0, d6
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 0013e6f0: cmp.w d5, d6
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e6f2: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0013e6f4: ble.b $13e6f8
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13e6f8_u32
                    } else {
                        0x13e6f6_u32
                    };
                }
                0x13e6f6 => {
                    // 0013e6f6: move.w d6, d5
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x13e6f8_u32;
                }
                0x13e6f8 => {
                    // 0013e6f8: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e6fe: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e702: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e708: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e70a: move.b $106(a0), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x106_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013e70e: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e710: cmp.w d0, d5
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(5, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e712: bgt.w $13e3e2
                    e.step()?;
                    pc = if r.greater()? {
                        0x13e3e2_u32
                    } else {
                        0x13e716_u32
                    };
                }
                0x13e716 => {
                    // 0013e716: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x13e718_u32;
                }
                0x13e718 => {
                    // 0013e718: cmp.w $4435ce.l, d3
                    e.step()?;
                    let source_address = 0x4435ce_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e71e: ble.w $13e5c0
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13e5c0_u32
                    } else {
                        0x13e722_u32
                    };
                }
                0x13e722 => {
                    // 0013e722: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e728: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e72c: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e732: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e734: tst.b $b2(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xb2_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0013e738: beq.b $13e746
                    e.step()?;
                    pc = if r.equal()? {
                        0x13e746_u32
                    } else {
                        0x13e73a_u32
                    };
                }
                0x13e73a => {
                    // 0013e73a: move.w #$2, -(a7)
                    e.step()?;
                    let value = 0x2_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e73e: bsr.l $110e96
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e744_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x110e96_u32)?;
                    // 0013e744: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x13e746_u32;
                }
                0x13e746 => {
                    // 0013e746: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e748: moveq #$a, d0
                    e.step()?;
                    r.put_data(0, 32, 0xa_u32);
                    r.move_flags(0xa_u32, 32);
                    // 0013e74a: add.w $447196.l, d0
                    e.step()?;
                    let source_address = 0x447196_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e750: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e752: bsr.l $105572
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e758_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x105572_u32)?;
                    // 0013e758: moveq #$a, d0
                    e.step()?;
                    r.put_data(0, 32, 0xa_u32);
                    r.move_flags(0xa_u32, 32);
                    // 0013e75a: add.w $447196.l, d0
                    e.step()?;
                    let source_address = 0x447196_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e760: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e762: bsr.l $105366
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e768_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x105366_u32)?;
                    // 0013e768: move.b $442736.l, d0
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013e76e: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e770: add.w -$a(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e774: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013e77a: move.w $447196.l, d1
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e780: move.w d0, (a0, d1.w * 4)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e784: move.b $442737.l, d0
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013e78a: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e78c: add.w -$c(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e790: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013e796: move.w $447196.l, d1
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e79c: move.w d0, $2(a0, d1.w * 4)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e7a0: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e7a6: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e7aa: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e7b0: move.w $447196.l, d1
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e7b6: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e7ba: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 0013e7c0: adda.l d1, a1
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0013e7c2: move.b $106(a1), d1
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x106_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e7c6: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e7c8: cmp.w $6(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x6_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e7cc: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0013e7ce: bne.b $13e802
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e802_u32
                    } else {
                        0x13e7d0_u32
                    };
                }
                0x13e7d0 => {
                    // 0013e7d0: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e7d6: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e7da: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e7e0: tst.w $6c(a0, d0.l)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x6c_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0013e7e4: beq.b $13e802
                    e.step()?;
                    pc = if r.equal()? {
                        0x13e802_u32
                    } else {
                        0x13e7e6_u32
                    };
                }
                0x13e7e6 => {
                    // 0013e7e6: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e7ec: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e7f0: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e7f6: move.w $6c(a0, d0.l), -(a7)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x6c_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e7fa: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e800_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0013e800: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x13e802_u32;
                }
                0x13e802 => {
                    // 0013e802: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e808: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e80c: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e812: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e814: sub.b d5, $106(a0)
                    e.step()?;
                    let source = r.data(5, 8)?;
                    let destination_address = r.address(0)?.wrapping_add(0x106_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.subtract_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    // 0013e818: move.w $447196.l, -(a7)
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e81e: move.b $442737.l, d0
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013e824: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e826: add.w -$c(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e82a: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e82c: move.b $442736.l, d0
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013e832: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e834: add.w -$a(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e838: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e83a: bsr.l $149e0e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e840_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x149e0e_u32)?;
                    // 0013e840: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e846: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e84a: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e850: cmpi.w #$fffd, $10(a0, d0.l)
                    e.step()?;
                    let source = 0xfffd_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x10_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e856: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0013e858: bne.b $13e872
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e872_u32
                    } else {
                        0x13e85a_u32
                    };
                }
                0x13e85a => {
                    // 0013e85a: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e85e: moveq #$a, d0
                    e.step()?;
                    r.put_data(0, 32, 0xa_u32);
                    r.move_flags(0xa_u32, 32);
                    // 0013e860: add.w $447196.l, d0
                    e.step()?;
                    let source_address = 0x447196_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e866: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e868: bsr.l $1107cc
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e86e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1107cc_u32)?;
                    // 0013e86e: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0013e870: bra.b $13e886
                    e.step()?;
                    pc = 0x13e886_u32;
                }
                0x13e872 => {
                    // 0013e872: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0013e874: moveq #$a, d0
                    e.step()?;
                    r.put_data(0, 32, 0xa_u32);
                    r.move_flags(0xa_u32, 32);
                    // 0013e876: add.w $447196.l, d0
                    e.step()?;
                    let source_address = 0x447196_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e87c: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e87e: bsr.l $120c14
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e884_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x120c14_u32)?;
                    // 0013e884: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x13e886_u32;
                }
                0x13e886 => {
                    // 0013e886: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e88c: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e890: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e896: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e898: move.w $11e(a0), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x11e_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e89c: addi.w #$fc18, d0
                    e.step()?;
                    let source = 0xfc18_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e8a0: mulu.w #$28, d0
                    e.step()?;
                    let source = 0x28_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e8a4: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 0013e8aa: move.w (a0, d0.l), -(a7)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e8ae: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e8b4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0013e8b4: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e8b6: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 0013e8bc: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e8c2: move.w (a0, d0.w * 2), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e8c6: bsr.l $1140ae
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e8cc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1140ae_u32)?;
                    // 0013e8cc: tst.w $439568.l
                    e.step()?;
                    let value_address = 0x439568_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0013e8d2: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0013e8d4: beq.b $13e8f2
                    e.step()?;
                    pc = if r.equal()? {
                        0x13e8f2_u32
                    } else {
                        0x13e8d6_u32
                    };
                }
                0x13e8d6 => {
                    // 0013e8d6: move.w $439568.l, d0
                    e.step()?;
                    let value_address = 0x439568_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e8dc: lsr.w #$8, d0
                    e.step()?;
                    let count = 0x8_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(0, 16, value);
                    // 0013e8de: lsr.w #$7, d0
                    e.step()?;
                    let count = 0x7_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(0, 16, value);
                    // 0013e8e0: add.w $439568.l, d0
                    e.step()?;
                    let source_address = 0x439568_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e8e6: asr.w #$1, d0
                    e.step()?;
                    let count = 0x1_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(0, 16, value);
                    // 0013e8e8: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013e8ea: bsr.l $1205f8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13e8f0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1205f8_u32)?;
                    // 0013e8f0: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x13e8f2_u32;
                }
                0x13e8f2 => {
                    // 0013e8f2: movea.w -$a(a6), a0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0013e8f6: move.l $442780.l, d0
                    e.step()?;
                    let value_address = 0x442780_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e8fc: add.l a0, d0
                    e.step()?;
                    let source = r.address(0)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e8fe: move.b $442736.l, d1
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e904: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e906: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e908: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e90a: moveq #$2, d0
                    e.step()?;
                    r.put_data(0, 32, 0x2_u32);
                    r.move_flags(0x2_u32, 32);
                    // 0013e90c: cmp.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0013e90e: blt.b $13e96c
                    e.step()?;
                    pc = if r.less()? {
                        0x13e96c_u32
                    } else {
                        0x13e910_u32
                    };
                }
                0x13e910 => {
                    // 0013e910: movea.w -$c(a6), a0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0013e914: move.l $442784.l, d0
                    e.step()?;
                    let value_address = 0x442784_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e91a: add.l a0, d0
                    e.step()?;
                    let source = r.address(0)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e91c: move.b $442737.l, d1
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e922: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e924: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e926: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e928: moveq #$2, d0
                    e.step()?;
                    r.put_data(0, 32, 0x2_u32);
                    r.move_flags(0x2_u32, 32);
                    // 0013e92a: cmp.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0013e92c: blt.b $13e96c
                    e.step()?;
                    pc = if r.less()? {
                        0x13e96c_u32
                    } else {
                        0x13e92e_u32
                    };
                }
                0x13e92e => {
                    // 0013e92e: movea.w -$a(a6), a0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0013e932: move.l $442780.l, d0
                    e.step()?;
                    let value_address = 0x442780_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e938: add.l a0, d0
                    e.step()?;
                    let source = r.address(0)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e93a: move.b $442736.l, d1
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e940: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e942: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e944: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e946: moveq #$57, d0
                    e.step()?;
                    r.put_data(0, 32, 0x57_u32);
                    r.move_flags(0x57_u32, 32);
                    // 0013e948: cmp.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0013e94a: bgt.b $13e96c
                    e.step()?;
                    pc = if r.greater()? {
                        0x13e96c_u32
                    } else {
                        0x13e94c_u32
                    };
                }
                0x13e94c => {
                    // 0013e94c: movea.w -$c(a6), a0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0013e950: move.l $442784.l, d0
                    e.step()?;
                    let value_address = 0x442784_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e956: add.l a0, d0
                    e.step()?;
                    let source = r.address(0)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013e958: move.b $442737.l, d1
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0013e95e: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013e960: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013e962: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0013e964: moveq #$57, d0
                    e.step()?;
                    r.put_data(0, 32, 0x57_u32);
                    r.move_flags(0x57_u32, 32);
                    // 0013e966: cmp.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0013e968: ble.w $13e9f2
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13e9f2_u32
                    } else {
                        0x13e96c_u32
                    };
                }
                0x13e96c => {
                    // 0013e96c: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e972: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e976: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e97c: cmpi.w #$ffff, $5a(a0, d0.l)
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x5a_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013e982: bne.b $13e9aa
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13e9aa_u32
                    } else {
                        0x13e984_u32
                    };
                }
                0x13e984 => {
                    // 0013e984: move.b $442736.l, d0
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013e98a: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e98c: muls.w #$ffff, d0
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e990: move.b d0, $442736.l
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0013e996: move.b $442737.l, d0
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013e99c: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e99e: muls.w #$ffff, d0
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e9a2: move.b d0, $442737.l
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0013e9a8: bra.b $13e9f2
                    e.step()?;
                    pc = 0x13e9f2_u32;
                }
                0x13e9aa => {
                    // 0013e9aa: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e9b0: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e9b4: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e9ba: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e9bc: clr.b $104(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x104_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0013e9c0: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e9c6: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e9ca: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e9d0: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013e9d2: clr.b $106(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x106_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0013e9d6: subq.w #$1, $447194.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x447194_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0013e9dc: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e9e2: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013e9e6: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013e9ec: move.w #$65, $46(a0, d0.l)
                    e.step()?;
                    let value = 0x65_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x46_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x13e9f2_u32;
                }
                0x13e9f2 => {
                    // 0013e9f2: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013e9f8: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013e9fe: cmpi.w #$1, (a0, d0.w * 4)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013ea04: blt.b $13ea42
                    e.step()?;
                    pc = if r.less()? {
                        0x13ea42_u32
                    } else {
                        0x13ea06_u32
                    };
                }
                0x13ea06 => {
                    // 0013ea06: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013ea0c: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013ea12: cmpi.w #$8, (a0, d0.w * 4)
                    e.step()?;
                    let source = 0x8_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013ea18: bgt.b $13ea42
                    e.step()?;
                    pc = if r.greater()? {
                        0x13ea42_u32
                    } else {
                        0x13ea1a_u32
                    };
                }
                0x13ea1a => {
                    // 0013ea1a: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013ea20: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013ea26: cmpi.w #$1, $2(a0, d0.w * 4)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013ea2c: blt.b $13ea42
                    e.step()?;
                    pc = if r.less()? {
                        0x13ea42_u32
                    } else {
                        0x13ea2e_u32
                    };
                }
                0x13ea2e => {
                    // 0013ea2e: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013ea34: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013ea3a: cmpi.w #$8, $2(a0, d0.w * 4)
                    e.step()?;
                    let source = 0x8_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0013ea40: ble.b $13ea6a
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13ea6a_u32
                    } else {
                        0x13ea42_u32
                    };
                }
                0x13ea42 => {
                    // 0013ea42: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013ea48: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013ea4e: move.w $2(a0, d0.w * 4), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013ea52: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0013ea58: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013ea5e: move.w (a0, d0.w * 4), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013ea62: bsr.l $10e6b6
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13ea68_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10e6b6_u32)?;
                    // 0013ea68: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x13ea6a_u32;
                }
                0x13ea6a => {
                    // 0013ea6a: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x13ea6c_u32;
                }
                0x13ea6c => {
                    // 0013ea6c: movem.l (a7)+, d3-d7
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    // 0013ea70: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0013ea72: rts
                    e.step()?;
                    return finish(e, &mut r);
                }
                _ => {
                    return Err(RuleError::Invalid(format!(
                        "source transition outside recovered body: {pc:08x}"
                    ))
                    .into());
                }
            }
        }
    })();
    result.map_err(|signal| match signal {
        Signal::Rule(error) => Signal::Rule(r.opaque_error(error)),
        stopped => stopped,
    })
}
crate::recovered_application_control::entry!(move_monster, 0x13e276);
