//! Native leaf control used by combat, movement, sound and spell callers.
use crate::RuleError;
use crate::recovered_application_control::{Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};
pub fn restore_body_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x105366_u32;
        loop {
            match pc {
                0x105366 => {
                    // 00105366: link.w a6, #$fffe
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -2)?;
                    // 0010536a: move.l d3, -(a7)
                    e.step()?;
                    let value = r.data(3, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010536c: move.w $8(a6), d3
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00105370: cmpi.w #$9, d3
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00105374: ble.w $105518
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x105518_u32
                    } else {
                        0x105378_u32
                    };
                }
                0x105378 => {
                    // 00105378: subi.w #$a, d3
                    e.step()?;
                    let source = 0xa_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    // 0010537c: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010537e: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00105382: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00105388: move.w $10(a0, d0.l), -(a7)
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
                    // 0010538c: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x105392_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00105392: subq.w #$3, d0
                    e.step()?;
                    let source = 0x3_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00105394: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00105396: bne.b $1053e6
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1053e6_u32
                    } else {
                        0x105398_u32
                    };
                }
                0x105398 => {
                    // 00105398: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010539a: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010539e: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 001053a4: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001053a6: lea.l $446f0a.l, a1
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(1, value);
                    // 001053ac: move.w (a1, d3.w * 4), d0
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001053b0: ext.l d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001053b2: add.l $442780.l, d0
                    e.step()?;
                    let source_address = 0x442780_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001053b8: subq.l #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001053ba: muls.l #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 001053c2: lea.l $4085b8.l, a1
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(1, value);
                    // 001053c8: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 001053ca: move.w $118(a0), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x118_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001053ce: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 001053d4: move.w $2(a0, d3.w * 4), d1
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001053d8: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001053da: add.l $442784.l, d1
                    e.step()?;
                    let source_address = 0x442784_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 001053e0: subq.l #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 001053e2: move.w d0, (a1, d1.l * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r
                        .address(1)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x1053e6_u32;
                }
                0x1053e6 => {
                    // 001053e6: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001053e8: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001053ec: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 001053f2: move.w $10(a0, d0.l), -(a7)
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
                    // 001053f6: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1053fc_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 001053fc: cmpi.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00105400: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00105402: ble.b $105450
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x105450_u32
                    } else {
                        0x105404_u32
                    };
                }
                0x105404 => {
                    // 00105404: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00105406: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010540a: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00105410: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00105412: lea.l $446f0a.l, a1
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(1, value);
                    // 00105418: move.w (a1, d3.w * 4), d0
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010541c: ext.l d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010541e: add.l $442780.l, d0
                    e.step()?;
                    let source_address = 0x442780_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00105424: subq.l #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00105426: muls.l #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 0010542e: lea.l $4085b8.l, a1
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(1, value);
                    // 00105434: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 00105436: move.w $11a(a0), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x11a_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010543a: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 00105440: move.w $2(a0, d3.w * 4), d1
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00105444: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00105446: add.l $442784.l, d1
                    e.step()?;
                    let source_address = 0x442784_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0010544c: move.w d0, (a1, d1.l * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r
                        .address(1)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x105450_u32;
                }
                0x105450 => {
                    // 00105450: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00105452: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00105456: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0010545c: move.w $10(a0, d0.l), -(a7)
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
                    // 00105460: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x105466_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00105466: subq.w #$3, d0
                    e.step()?;
                    let source = 0x3_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00105468: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010546a: beq.b $105480
                    e.step()?;
                    pc = if r.equal()? {
                        0x105480_u32
                    } else {
                        0x10546c_u32
                    };
                }
                0x10546c => {
                    // 0010546c: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010546e: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00105472: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00105478: cmpi.w #$1, $10(a0, d0.l)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x10_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0010547e: bne.b $1054cc
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1054cc_u32
                    } else {
                        0x105480_u32
                    };
                }
                0x105480 => {
                    // 00105480: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00105482: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00105486: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0010548c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010548e: lea.l $446f0a.l, a1
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(1, value);
                    // 00105494: move.w (a1, d3.w * 4), d0
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00105498: ext.l d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010549a: add.l $442780.l, d0
                    e.step()?;
                    let source_address = 0x442780_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001054a0: muls.l #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 001054a8: lea.l $4085b8.l, a1
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(1, value);
                    // 001054ae: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 001054b0: move.w $11c(a0), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x11c_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001054b4: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 001054ba: move.w $2(a0, d3.w * 4), d1
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001054be: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001054c0: add.l $442784.l, d1
                    e.step()?;
                    let source_address = 0x442784_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 001054c6: subq.l #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 001054c8: move.w d0, (a1, d1.l * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r
                        .address(1)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x1054cc_u32;
                }
                0x1054cc => {
                    // 001054cc: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001054ce: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001054d2: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 001054d8: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001054da: lea.l $446f0a.l, a1
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(1, value);
                    // 001054e0: move.w (a1, d3.w * 4), d0
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001054e4: ext.l d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001054e6: add.l $442780.l, d0
                    e.step()?;
                    let source_address = 0x442780_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001054ec: muls.l #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 001054f4: lea.l $4085b8.l, a1
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(1, value);
                    // 001054fa: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 001054fc: move.w $11e(a0), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x11e_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00105500: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 00105506: move.w $2(a0, d3.w * 4), d1
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0010550a: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0010550c: add.l $442784.l, d1
                    e.step()?;
                    let source_address = 0x442784_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 00105512: move.w d0, (a1, d1.l * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r
                        .address(1)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00105516: bra.b $10555e
                    e.step()?;
                    pc = 0x10555e_u32;
                }
                0x105518 => {
                    // 00105518: lea.l $437a6e.l, a0
                    e.step()?;
                    let value = 0x437a6e_u32;
                    r.put_address(0, value);
                    // 0010551e: lea.l $44709a.l, a1
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(1, value);
                    // 00105524: move.w (a1, d3.w * 4), d0
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00105528: ext.l d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010552a: add.l $442780.l, d0
                    e.step()?;
                    let source_address = 0x442780_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00105530: muls.l #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 00105538: lea.l $4085b8.l, a1
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(1, value);
                    // 0010553e: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 00105540: move.w (a0, d3.w * 2), -$2(a6)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00105546: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0010554c: move.w $2(a0, d3.w * 4), d1
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00105550: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00105552: add.l $442784.l, d1
                    e.step()?;
                    let source_address = 0x442784_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 00105558: move.w -$2(a6), (a1, d1.l * 2)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r
                        .address(1)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x10555e_u32;
                }
                0x10555e => {
                    // 0010555e: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00105560: move.l (a7)+, d3
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_data(3, 32, value);
                    r.move_flags(value, 32);
                    // 00105562: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00105564: rts
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
crate::recovered_application_control::entry!(restore_body, 0x105366);

pub fn bodyground_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x105572_u32;
        loop {
            match pc {
                0x105572 => {
                    // 00105572: link.w a6, #$fff8
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -8)?;
                    // 00105576: movem.l d3-d7, -(a7)
                    e.step()?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 0010557a: move.w $8(a6), d5
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 0010557e: tst.w $438776.l
                    e.step()?;
                    let value_address = 0x438776_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00105584: bne.b $10558c
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10558c_u32
                    } else {
                        0x105586_u32
                    };
                }
                0x105586 => {
                    // 00105586: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00105588: bra.w $1056be
                    e.step()?;
                    pc = 0x1056be_u32;
                }
                0x10558c => {
                    // 0010558c: move.l $43879c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43879c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00105592: MacToolbox a873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 00105594: cmpi.w #$9, d5
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(5, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00105598: ble.w $105666
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x105666_u32
                    } else {
                        0x10559c_u32
                    };
                }
                0x10559c => {
                    // 0010559c: subi.w #$a, d5
                    e.step()?;
                    let source = 0xa_u32;
                    let destination = r.data(5, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(5, 16, value);
                    // 001055a0: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 001055a6: move.w (a0, d5.w * 4), d6
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(5, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 001055aa: subq.w #$1, d6
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(6, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(6, 16, value);
                    // 001055ac: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 001055b2: move.w $2(a0, d5.w * 4), d7
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(5, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(7, 16, value);
                    r.move_flags(value, 16);
                    // 001055b6: subq.w #$1, d7
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(7, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(7, 16, value);
                    // 001055b8: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 001055ba: bra.w $10565c
                    e.step()?;
                    pc = 0x10565c_u32;
                }
                0x1055be => {
                    // 001055be: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 001055c0: bra.w $105652
                    e.step()?;
                    pc = 0x105652_u32;
                }
                0x1055c4 => {
                    // 001055c4: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 001055c6: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001055c8: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001055ca: movea.w d4, a0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 001055cc: move.l a0, d1
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001055ce: lsl.l #$2, d1
                    e.step()?;
                    let count = 0x2_u32;
                    let source = r.data(1, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(1, 32, value);
                    // 001055d0: move.w d5, d2
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(2, 16, value);
                    r.move_flags(value, 16);
                    // 001055d2: mulu.w #$120, d2
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(2, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(2, 32, value);
                    r.move_flags(value, 32);
                    // 001055d6: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 001055dc: adda.l d2, a0
                    e.step()?;
                    let source = r.data(2, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001055de: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001055e0: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001055e2: tst.w $118(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x118_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001055e6: beq.b $105650
                    e.step()?;
                    pc = if r.equal()? {
                        0x105650_u32
                    } else {
                        0x1055e8_u32
                    };
                }
                0x1055e8 => {
                    // 001055e8: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001055ea: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001055ec: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001055ee: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 001055f0: move.w d0, -$8(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001055f4: move.b d6, d0
                    e.step()?;
                    let value = r.data(6, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001055f6: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001055f8: add.w d4, d0
                    e.step()?;
                    let source = r.data(4, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001055fa: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 001055fc: move.w d0, -$6(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00105600: moveq #$20, d0
                    e.step()?;
                    r.put_data(0, 32, 0x20_u32);
                    r.move_flags(0x20_u32, 32);
                    // 00105602: add.w -$6(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00105606: move.w d0, -$2(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010560a: moveq #$20, d0
                    e.step()?;
                    r.put_data(0, 32, 0x20_u32);
                    r.move_flags(0x20_u32, 32);
                    // 0010560c: add.w -$8(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00105610: move.w d0, -$4(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00105614: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00105616: move.w $a(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010561a: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010561c: move.l -$4(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00105620: move.l -$8(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00105624: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 00105626: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00105628: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010562a: movea.w d4, a0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010562c: move.l a0, d1
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0010562e: lsl.l #$2, d1
                    e.step()?;
                    let count = 0x2_u32;
                    let source = r.data(1, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(1, 32, value);
                    // 00105630: move.w d5, d2
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(2, 16, value);
                    r.move_flags(value, 16);
                    // 00105632: mulu.w #$120, d2
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(2, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(2, 32, value);
                    r.move_flags(value, 32);
                    // 00105636: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0010563c: adda.l d2, a0
                    e.step()?;
                    let source = r.data(2, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010563e: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00105640: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00105642: move.w $118(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x118_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00105646: bsr.l $122bec
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10564c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x122bec_u32)?;
                    // 0010564c: lea.l $10(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x10_u32);
                    r.put_address(7, value);
                    pc = 0x105650_u32;
                }
                0x105650 => {
                    // 00105650: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x105652_u32;
                }
                0x105652 => {
                    // 00105652: cmpi.w #$2, d3
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00105656: blt.w $1055c4
                    e.step()?;
                    pc = if r.less()? {
                        0x1055c4_u32
                    } else {
                        0x10565a_u32
                    };
                }
                0x10565a => {
                    // 0010565a: addq.w #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    pc = 0x10565c_u32;
                }
                0x10565c => {
                    // 0010565c: cmpi.w #$2, d4
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(4, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00105660: blt.w $1055be
                    e.step()?;
                    pc = if r.less()? {
                        0x1055be_u32
                    } else {
                        0x105664_u32
                    };
                }
                0x105664 => {
                    // 00105664: bra.b $1056bc
                    e.step()?;
                    pc = 0x1056bc_u32;
                }
                0x105666 => {
                    // 00105666: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0010566c: move.w $2(a0, d5.w * 4), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(5, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00105670: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00105672: move.w d0, -$8(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00105676: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0010567c: move.w (a0, d5.w * 4), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(5, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00105680: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00105682: move.w d0, -$6(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00105686: moveq #$20, d0
                    e.step()?;
                    r.put_data(0, 32, 0x20_u32);
                    r.move_flags(0x20_u32, 32);
                    // 00105688: add.w -$6(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010568c: move.w d0, -$2(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00105690: moveq #$20, d0
                    e.step()?;
                    r.put_data(0, 32, 0x20_u32);
                    r.move_flags(0x20_u32, 32);
                    // 00105692: add.w -$8(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00105696: move.w d0, -$4(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010569a: move.w $a(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010569e: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001056a0: move.l -$4(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001056a4: move.l -$8(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001056a8: lea.l $437a6e.l, a0
                    e.step()?;
                    let value = 0x437a6e_u32;
                    r.put_address(0, value);
                    // 001056ae: move.w (a0, d5.w * 2), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(5, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001056b2: bsr.l $122bec
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1056b8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x122bec_u32)?;
                    // 001056b8: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x1056bc_u32;
                }
                0x1056bc => {
                    // 001056bc: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x1056be_u32;
                }
                0x1056be => {
                    // 001056be: movem.l (a7)+, d3-d7
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    // 001056c2: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 001056c4: rts
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
crate::recovered_application_control::entry!(bodyground, 0x105572);

pub fn cansee_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x10ae90_u32;
        loop {
            match match pc {
                0x10ae90 => cansee_body_block_0(e, &mut r, frame, pc)?,
                0x10b008 => cansee_body_block_1(e, &mut r, frame, pc)?,
                0x10b056 => cansee_body_block_2(e, &mut r, frame, pc)?,
                0x10b06c => cansee_body_block_3(e, &mut r, frame, pc)?,
                0x10b072 => cansee_body_block_4(e, &mut r, frame, pc)?,
                0x10b120 => cansee_body_block_5(e, &mut r, frame, pc)?,
                0x10b128 => cansee_body_block_6(e, &mut r, frame, pc)?,
                0x10b12a => cansee_body_block_7(e, &mut r, frame, pc)?,
                _ => cansee_body_block_8(e, &mut r, frame, pc)?,
            } {
                std::ops::ControlFlow::Continue(next) => pc = next,
                std::ops::ControlFlow::Break(()) => return Ok(()),
            }
        }
    })();
    result.map_err(|signal| match signal {
        Signal::Rule(error) => Signal::Rule(r.opaque_error(error)),
        stopped => stopped,
    })
}
crate::recovered_application_control::entry!(cansee, 0x10ae90);

pub fn checkformechanicalunits_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x110d12_u32;
        loop {
            match pc {
                0x110d12 => {
                    // 00110d12: link.w a6, #$fffe
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -2)?;
                    // 00110d16: movem.l d3-d7, -(a7)
                    e.step()?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 00110d1a: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 00110d20: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110d26: move.w (a0, d0.w * 4), d7
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(7, 16, value);
                    r.move_flags(value, 16);
                    // 00110d2a: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 00110d30: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110d36: move.w $2(a0, d0.w * 4), -$2(a6)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00110d3c: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110d42: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110d46: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00110d4c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00110d4e: tst.w $f6(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xf6_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00110d52: beq.b $110d58
                    e.step()?;
                    pc = if r.equal()? {
                        0x110d58_u32
                    } else {
                        0x110d54_u32
                    };
                }
                0x110d54 => {
                    // 00110d54: moveq #$2, d5
                    e.step()?;
                    r.put_data(5, 32, 0x2_u32);
                    r.move_flags(0x2_u32, 32);
                    // 00110d56: bra.b $110d72
                    e.step()?;
                    pc = 0x110d72_u32;
                }
                0x110d58 => {
                    // 00110d58: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110d5e: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110d62: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00110d68: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00110d6a: tst.w $f4(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xf4_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00110d6e: beq.b $110d72
                    e.step()?;
                    pc = if r.equal()? {
                        0x110d72_u32
                    } else {
                        0x110d70_u32
                    };
                }
                0x110d70 => {
                    // 00110d70: moveq #$1, d5
                    e.step()?;
                    r.put_data(5, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x110d72_u32;
                }
                0x110d72 => {
                    // 00110d72: moveq #$ff, d6
                    e.step()?;
                    r.put_data(6, 32, 0xffffffff_u32);
                    r.move_flags(0xffffffff_u32, 32);
                    // 00110d74: bra.w $110e6a
                    e.step()?;
                    pc = 0x110e6a_u32;
                }
                0x110d78 => {
                    // 00110d78: moveq #$ff, d4
                    e.step()?;
                    r.put_data(4, 32, 0xffffffff_u32);
                    r.move_flags(0xffffffff_u32, 32);
                    // 00110d7a: bra.w $110e60
                    e.step()?;
                    pc = 0x110e60_u32;
                }
                0x110d7e => {
                    // 00110d7e: movea.w d7, a0
                    e.step()?;
                    let value = r.data(7, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 00110d80: move.l $442780.l, d0
                    e.step()?;
                    let value_address = 0x442780_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110d86: add.l a0, d0
                    e.step()?;
                    let source = r.address(0)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00110d88: movea.w d4, a0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 00110d8a: add.l a0, d0
                    e.step()?;
                    let source = r.address(0)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00110d8c: muls.l #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 00110d94: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 00110d9a: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00110d9c: movea.w -$2(a6), a1
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(1, value);
                    // 00110da0: move.l $442784.l, d0
                    e.step()?;
                    let value_address = 0x442784_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110da6: add.l a1, d0
                    e.step()?;
                    let source = r.address(1)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00110da8: movea.w d6, a1
                    e.step()?;
                    let value = r.data(6, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(1, value);
                    // 00110daa: add.l a1, d0
                    e.step()?;
                    let source = r.address(1)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00110dac: move.w (a0, d0.l * 2), -(a7)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00110db0: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x110db6_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00110db6: move.w d0, d3
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00110db8: cmpi.w #$6e, d3
                    e.step()?;
                    let source = 0x6e_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00110dbc: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00110dbe: bge.w $110e5e
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x110e5e_u32
                    } else {
                        0x110dc2_u32
                    };
                }
                0x110dc2 => {
                    // 00110dc2: cmpi.w #$9, d3
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00110dc6: ble.w $110e5e
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x110e5e_u32
                    } else {
                        0x110dca_u32
                    };
                }
                0x110dca => {
                    // 00110dca: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110dd0: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110dd4: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00110dda: moveq #$f6, d1
                    e.step()?;
                    r.put_data(1, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00110ddc: add.w d3, d1
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00110dde: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00110de2: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 00110de8: move.w $e(a1, d1.l), d1
                    e.step()?;
                    let value_address = r
                        .address(1)?
                        .wrapping_add(0xe_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00110dec: cmp.w $10(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x10_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00110df0: beq.b $110e5e
                    e.step()?;
                    pc = if r.equal()? {
                        0x110e5e_u32
                    } else {
                        0x110df2_u32
                    };
                }
                0x110df2 => {
                    // 00110df2: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00110df4: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00110df6: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110dfa: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00110e00: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00110e02: tst.w $110(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x110_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00110e06: ble.b $110e5e
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x110e5e_u32
                    } else {
                        0x110e08_u32
                    };
                }
                0x110e08 => {
                    // 00110e08: cmpi.w #$1, d5
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(5, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00110e0c: bne.b $110e26
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x110e26_u32
                    } else {
                        0x110e0e_u32
                    };
                }
                0x110e0e => {
                    // 00110e0e: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00110e10: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00110e12: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110e16: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00110e1c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00110e1e: cmpi.b #$1, $b0(a0)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00110e24: beq.b $110e5a
                    e.step()?;
                    pc = if r.equal()? {
                        0x110e5a_u32
                    } else {
                        0x110e26_u32
                    };
                }
                0x110e26 => {
                    // 00110e26: cmpi.w #$2, d5
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(5, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00110e2a: bne.b $110e5e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x110e5e_u32
                    } else {
                        0x110e2c_u32
                    };
                }
                0x110e2c => {
                    // 00110e2c: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00110e2e: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00110e30: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110e34: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00110e3a: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00110e3c: cmpi.b #$1, $b0(a0)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00110e42: beq.b $110e5a
                    e.step()?;
                    pc = if r.equal()? {
                        0x110e5a_u32
                    } else {
                        0x110e44_u32
                    };
                }
                0x110e44 => {
                    // 00110e44: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00110e46: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00110e48: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110e4c: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00110e52: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00110e54: tst.b $b1(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xb1_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00110e58: beq.b $110e5e
                    e.step()?;
                    pc = if r.equal()? {
                        0x110e5e_u32
                    } else {
                        0x110e5a_u32
                    };
                }
                0x110e5a => {
                    // 00110e5a: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 00110e5c: bra.b $110e74
                    e.step()?;
                    pc = 0x110e74_u32;
                }
                0x110e5e => {
                    // 00110e5e: addq.w #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    pc = 0x110e60_u32;
                }
                0x110e60 => {
                    // 00110e60: cmpi.w #$2, d4
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(4, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00110e64: blt.w $110d7e
                    e.step()?;
                    pc = if r.less()? {
                        0x110d7e_u32
                    } else {
                        0x110e68_u32
                    };
                }
                0x110e68 => {
                    // 00110e68: addq.w #$1, d6
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(6, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(6, 16, value);
                    pc = 0x110e6a_u32;
                }
                0x110e6a => {
                    // 00110e6a: cmpi.w #$2, d6
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00110e6e: blt.w $110d78
                    e.step()?;
                    pc = if r.less()? {
                        0x110d78_u32
                    } else {
                        0x110e72_u32
                    };
                }
                0x110e72 => {
                    // 00110e72: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x110e74_u32;
                }
                0x110e74 => {
                    // 00110e74: movem.l (a7)+, d3-d7
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    // 00110e78: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00110e7a: rts
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
crate::recovered_application_control::entry!(checkformechanicalunits, 0x110d12);

pub fn checkrange_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x11235e_u32;
        loop {
            match pc {
                0x11235e => {
                    // 0011235e: link.w a6, #$ffe4
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -28)?;
                    // 00112362: movem.l d3-d5, -(a7)
                    e.step()?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 00112366: move.w $8(a6), d3
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 0011236a: cmpi.w #$9, d3
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0011236e: ble.b $11238e
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x11238e_u32
                    } else {
                        0x112370_u32
                    };
                }
                0x112370 => {
                    // 00112370: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00112372: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00112374: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0011237a: move.w (a0, d0.w * 4), d4
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 0011237e: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00112380: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00112382: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 00112388: move.w $2(a0, d0.w * 4), d5
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 0011238c: bra.b $1123a2
                    e.step()?;
                    pc = 0x1123a2_u32;
                }
                0x11238e => {
                    // 0011238e: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 00112394: move.w (a0, d3.w * 4), d4
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 00112398: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0011239e: move.w $2(a0, d3.w * 4), d5
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x1123a2_u32;
                }
                0x1123a2 => {
                    // 001123a2: move.w $a(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001123a6: sub.w d4, d0
                    e.step()?;
                    let source = r.data(4, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001123a8: move.w d0, -$1c(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001123ac: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001123b0: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001123b4: move.w #$200e, -(a7)
                    e.step()?;
                    let value = 0x200e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001123b8: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 001123ba: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001123be: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001123c2: move.w #$1010, -(a7)
                    e.step()?;
                    let value = 0x1010_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001123c6: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 001123c8: move.l -$1c(a6), -$4(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001123ce: move.w $c(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001123d2: sub.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001123d4: move.w d0, -$1c(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001123d8: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001123dc: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001123e0: move.w #$200e, -(a7)
                    e.step()?;
                    let value = 0x200e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001123e4: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 001123e6: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001123ea: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001123ee: move.w #$1010, -(a7)
                    e.step()?;
                    let value = 0x1010_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001123f2: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 001123f4: move.l -$1c(a6), -$8(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001123fa: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001123fe: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112402: move.w #$100e, -(a7)
                    e.step()?;
                    let value = 0x100e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112406: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112408: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011240c: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112410: move.w #$1004, -(a7)
                    e.step()?;
                    let value = 0x1004_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112414: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112416: pea.l -$4(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011241a: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011241e: move.w #$100e, -(a7)
                    e.step()?;
                    let value = 0x100e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112422: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112424: pea.l -$4(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112428: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011242c: move.w #$1004, -(a7)
                    e.step()?;
                    let value = 0x1004_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112430: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112432: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112436: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011243a: move.w #$0, -(a7)
                    e.step()?;
                    let value = 0x0_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0011243e: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112440: move.l -$12(a6), $44349c.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffee_u32);
                    let value = e.l(value_address)?;
                    let destination_address = 0x44349c_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00112448: move.l -$e(a6), $4434a0.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
                    let value = e.l(value_address)?;
                    let destination_address = 0x4434a0_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00112450: move.w -$a(a6), $4434a4.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x4434a4_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112458: move.l $4434a2.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434a2_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0011245e: move.l $44349e.l, -(a7)
                    e.step()?;
                    let value_address = 0x44349e_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00112464: move.w $44349c.l, -(a7)
                    e.step()?;
                    let value_address = 0x44349c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0011246a: pea.l $44349c.l
                    e.step()?;
                    let value = 0x44349c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112470: jsr $4001a0.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x112476_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4001a0_u32))?;
                    // 00112476: move.l $44349c.l, -$12(a6)
                    e.step()?;
                    let value_address = 0x44349c_u32;
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0011247e: move.l $4434a0.l, -$e(a6)
                    e.step()?;
                    let value_address = 0x4434a0_u32;
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00112486: move.w $4434a4.l, -$a(a6)
                    e.step()?;
                    let value_address = 0x4434a4_u32;
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0011248e: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112492: move.w #$16, -(a7)
                    e.step()?;
                    let value = 0x16_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112496: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112498: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011249c: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001124a0: move.w #$2810, -(a7)
                    e.step()?;
                    let value = 0x2810_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001124a4: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 001124a6: move.w -$10(a6), $4434b6.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001124ae: move.w $4434b6.l, d0
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001124b4: cmp.w $4435bc.l, d0
                    e.step()?;
                    let source_address = 0x4435bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001124ba: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    // 001124be: ble.b $1124c4
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1124c4_u32
                    } else {
                        0x1124c0_u32
                    };
                }
                0x1124c0 => {
                    // 001124c0: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 001124c2: bra.b $1124c6
                    e.step()?;
                    pc = 0x1124c6_u32;
                }
                0x1124c4 => {
                    // 001124c4: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x1124c6_u32;
                }
                0x1124c6 => {
                    // 001124c6: movem.l (a7)+, d3-d5
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    // 001124ca: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 001124cc: rts
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
crate::recovered_application_control::entry!(checkrange, 0x11235e);

pub fn getrangetobody_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x1124dc_u32;
        loop {
            match pc {
                0x1124dc => {
                    // 001124dc: link.w a6, #$ffe4
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -28)?;
                    // 001124e0: movem.l d3-d5, -(a7)
                    e.step()?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 001124e4: move.w $8(a6), d3
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 001124e8: cmpi.w #$9, d3
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001124ec: ble.b $11250c
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x11250c_u32
                    } else {
                        0x1124ee_u32
                    };
                }
                0x1124ee => {
                    // 001124ee: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 001124f0: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001124f2: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 001124f8: move.w (a0, d0.w * 4), d4
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 001124fc: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 001124fe: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00112500: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 00112506: move.w $2(a0, d0.w * 4), d5
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 0011250a: bra.b $112520
                    e.step()?;
                    pc = 0x112520_u32;
                }
                0x11250c => {
                    // 0011250c: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 00112512: move.w (a0, d3.w * 4), d4
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 00112516: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0011251c: move.w $2(a0, d3.w * 4), d5
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x112520_u32;
                }
                0x112520 => {
                    // 00112520: move.w $a(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00112524: sub.w d4, d0
                    e.step()?;
                    let source = r.data(4, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00112526: move.w d0, -$1c(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0011252a: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011252e: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112532: move.w #$200e, -(a7)
                    e.step()?;
                    let value = 0x200e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112536: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112538: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011253c: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112540: move.w #$1010, -(a7)
                    e.step()?;
                    let value = 0x1010_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112544: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112546: move.l -$1c(a6), -$4(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0011254c: move.w $c(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00112550: sub.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00112552: move.w d0, -$1c(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112556: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011255a: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011255e: move.w #$200e, -(a7)
                    e.step()?;
                    let value = 0x200e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112562: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112564: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112568: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011256c: move.w #$1010, -(a7)
                    e.step()?;
                    let value = 0x1010_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112570: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112572: move.l -$1c(a6), -$8(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00112578: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011257c: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112580: move.w #$100e, -(a7)
                    e.step()?;
                    let value = 0x100e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112584: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112586: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011258a: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011258e: move.w #$1004, -(a7)
                    e.step()?;
                    let value = 0x1004_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112592: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112594: pea.l -$4(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112598: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011259c: move.w #$100e, -(a7)
                    e.step()?;
                    let value = 0x100e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001125a0: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 001125a2: pea.l -$4(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001125a6: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001125aa: move.w #$1004, -(a7)
                    e.step()?;
                    let value = 0x1004_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001125ae: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 001125b0: pea.l -$1c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001125b4: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001125b8: move.w #$0, -(a7)
                    e.step()?;
                    let value = 0x0_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001125bc: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 001125be: move.l -$12(a6), $44349c.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffee_u32);
                    let value = e.l(value_address)?;
                    let destination_address = 0x44349c_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001125c6: move.l -$e(a6), $4434a0.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
                    let value = e.l(value_address)?;
                    let destination_address = 0x4434a0_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001125ce: move.w -$a(a6), $4434a4.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x4434a4_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001125d6: move.l $4434a2.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434a2_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001125dc: move.l $44349e.l, -(a7)
                    e.step()?;
                    let value_address = 0x44349e_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001125e2: move.w $44349c.l, -(a7)
                    e.step()?;
                    let value_address = 0x44349c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001125e8: pea.l $44349c.l
                    e.step()?;
                    let value = 0x44349c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001125ee: jsr $4001a0.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1125f4_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4001a0_u32))?;
                    // 001125f4: move.l $44349c.l, -$12(a6)
                    e.step()?;
                    let value_address = 0x44349c_u32;
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001125fc: move.l $4434a0.l, -$e(a6)
                    e.step()?;
                    let value_address = 0x4434a0_u32;
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00112604: move.w $4434a4.l, -$a(a6)
                    e.step()?;
                    let value_address = 0x4434a4_u32;
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0011260c: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00112610: move.w #$16, -(a7)
                    e.step()?;
                    let value = 0x16_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112614: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112616: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011261a: pea.l -$12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffee_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0011261e: move.w #$2810, -(a7)
                    e.step()?;
                    let value = 0x2810_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00112622: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 00112624: move.w -$10(a6), $4434b6.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0011262c: move.w $4434b6.l, d0
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00112632: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    // 00112636: movem.l (a7)+, d3-d5
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    // 0011263a: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0011263c: rts
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
crate::recovered_application_control::entry!(getrangetobody, 0x1124dc);

pub fn item_charges_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x11279a_u32;
        loop {
            match pc {
                0x11279a => {
                    // 0011279a: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 0011279e: movem.l d4-d6, -(a7)
                    e.step()?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    // 001127a2: move.w $8(a6), d6
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 001127a6: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 001127a8: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 001127aa: bra.b $1127f8
                    e.step()?;
                    pc = 0x1127f8_u32;
                }
                0x1127ac => {
                    // 001127ac: moveq #$0, d2
                    e.step()?;
                    r.put_data(2, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 001127ae: bra.b $1127e4
                    e.step()?;
                    pc = 0x1127e4_u32;
                }
                0x1127b0 => {
                    // 001127b0: moveq #$16, d0
                    e.step()?;
                    r.put_data(0, 32, 0x16_u32);
                    r.move_flags(0x16_u32, 32);
                    // 001127b2: mulu.w d2, d0
                    e.step()?;
                    let source = r.data(2, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001127b4: move.w d4, d1
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001127b6: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001127ba: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 001127c0: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001127c2: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001127c4: cmp.w $132(a0), d6
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x132_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001127c8: bne.b $1127e2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1127e2_u32
                    } else {
                        0x1127ca_u32
                    };
                }
                0x1127ca => {
                    // 001127ca: moveq #$16, d0
                    e.step()?;
                    r.put_data(0, 32, 0x16_u32);
                    r.move_flags(0x16_u32, 32);
                    // 001127cc: mulu.w d2, d0
                    e.step()?;
                    let source = r.data(2, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001127ce: move.w d4, d1
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001127d0: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001127d4: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 001127da: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001127dc: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001127de: add.w $134(a0), d5
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x134_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(5, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(5, 16, value);
                    pc = 0x1127e2_u32;
                }
                0x1127e2 => {
                    // 001127e2: addq.w #$1, d2
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(2, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(2, 16, value);
                    pc = 0x1127e4_u32;
                }
                0x1127e4 => {
                    // 001127e4: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001127e6: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001127ea: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 001127f0: cmp.w $1a(a0, d0.l), d2
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x1a_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(2, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001127f4: blt.b $1127b0
                    e.step()?;
                    pc = if r.less()? {
                        0x1127b0_u32
                    } else {
                        0x1127f6_u32
                    };
                }
                0x1127f6 => {
                    // 001127f6: addq.w #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    pc = 0x1127f8_u32;
                }
                0x1127f8 => {
                    // 001127f8: move.b $44273a.l, d0
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001127fe: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00112800: cmp.w d0, d4
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(4, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00112802: ble.b $1127ac
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1127ac_u32
                    } else {
                        0x112804_u32
                    };
                }
                0x112804 => {
                    // 00112804: move.w d5, d0
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00112806: movem.l (a7)+, d4-d6
                    e.step()?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    // 0011280a: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0011280c: rts
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
crate::recovered_application_control::entry!(item_charges, 0x11279a);

pub fn destroymonster_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x12063c_u32;
        loop {
            match match pc {
                0x12063c => destroymonster_body_block_0(e, &mut r, frame, pc)?,
                0x1206c6 => destroymonster_body_block_1(e, &mut r, frame, pc)?,
                0x1206dc => destroymonster_body_block_2(e, &mut r, frame, pc)?,
                0x1206e2 => destroymonster_body_block_3(e, &mut r, frame, pc)?,
                0x12070a => destroymonster_body_block_4(e, &mut r, frame, pc)?,
                0x120720 => destroymonster_body_block_5(e, &mut r, frame, pc)?,
                0x120726 => destroymonster_body_block_6(e, &mut r, frame, pc)?,
                0x120740 => destroymonster_body_block_7(e, &mut r, frame, pc)?,
                0x120746 => destroymonster_body_block_8(e, &mut r, frame, pc)?,
                0x12074c => destroymonster_body_block_9(e, &mut r, frame, pc)?,
                0x120752 => destroymonster_body_block_a(e, &mut r, frame, pc)?,
                0x120758 => destroymonster_body_block_b(e, &mut r, frame, pc)?,
                0x120782 => destroymonster_body_block_c(e, &mut r, frame, pc)?,
                0x120796 => destroymonster_body_block_d(e, &mut r, frame, pc)?,
                0x12079a => destroymonster_body_block_e(e, &mut r, frame, pc)?,
                0x1207aa => destroymonster_body_block_f(e, &mut r, frame, pc)?,
                0x1207b6 => destroymonster_body_block_10(e, &mut r, frame, pc)?,
                0x1207b8 => destroymonster_body_block_11(e, &mut r, frame, pc)?,
                0x1207be => destroymonster_body_block_12(e, &mut r, frame, pc)?,
                0x1207d2 => destroymonster_body_block_13(e, &mut r, frame, pc)?,
                0x1207ec => destroymonster_body_block_14(e, &mut r, frame, pc)?,
                0x120804 => destroymonster_body_block_15(e, &mut r, frame, pc)?,
                0x120824 => destroymonster_body_block_16(e, &mut r, frame, pc)?,
                0x120838 => destroymonster_body_block_17(e, &mut r, frame, pc)?,
                0x12084c => destroymonster_body_block_18(e, &mut r, frame, pc)?,
                0x12084e => destroymonster_body_block_19(e, &mut r, frame, pc)?,
                0x120856 => destroymonster_body_block_1a(e, &mut r, frame, pc)?,
                _ => destroymonster_body_block_1b(e, &mut r, frame, pc)?,
            } {
                std::ops::ControlFlow::Continue(next) => pc = next,
                std::ops::ControlFlow::Break(()) => return Ok(()),
            }
        }
    })();
    result.map_err(|signal| match signal {
        Signal::Rule(error) => Signal::Rule(r.opaque_error(error)),
        stopped => stopped,
    })
}
crate::recovered_application_control::entry!(destroymonster, 0x12063c);

pub fn fastplot_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x122bec_u32;
        loop {
            match pc {
                0x122bec => {
                    // 00122bec: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 00122bf0: movem.l d3-d5, -(a7)
                    e.step()?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 00122bf4: move.w $8(a6), d3
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00122bf8: move.b $13(a6), d5
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x13_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(5, 8, value);
                    r.move_flags(value, 8);
                    // 00122bfc: move.b $15(a6), d4
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x15_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(4, 8, value);
                    r.move_flags(value, 8);
                    // 00122c00: tst.w d3
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.move_flags(value, 16);
                    // 00122c02: bge.b $122c0a
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x122c0a_u32
                    } else {
                        0x122c04_u32
                    };
                }
                0x122c04 => {
                    // 00122c04: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00122c06: bra.w $122db0
                    e.step()?;
                    pc = 0x122db0_u32;
                }
                0x122c0a => {
                    // 00122c0a: move.l $43879c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43879c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122c10: MacToolbox a873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 00122c12: pea.l $21.w
                    e.step()?;
                    let value = 0x21_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122c16: MacToolbox a862
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa862_u32 as u16))?;
                    // 00122c18: pea.l $1e.w
                    e.step()?;
                    let value = 0x1e_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122c1c: MacToolbox a863
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa863_u32 as u16))?;
                    // 00122c1e: cmpi.w #$59d7, d3
                    e.step()?;
                    let source = 0x59d7_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00122c22: ble.w $122cbc
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x122cbc_u32
                    } else {
                        0x122c26_u32
                    };
                }
                0x122c26 => {
                    // 00122c26: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122c28: addi.w #$a628, d0
                    e.step()?;
                    let source = 0xa628_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122c2c: lea.l $41c220.l, a0
                    e.step()?;
                    let value = 0x41c220_u32;
                    r.put_address(0, value);
                    // 00122c32: tst.l (a0, d0.w * 4)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = e.l(value_address)?;
                    r.move_flags(value, 32);
                    // 00122c36: bne.b $122c46
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122c46_u32
                    } else {
                        0x122c38_u32
                    };
                }
                0x122c38 => {
                    // 00122c38: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122c3a: addi.w #$a628, d0
                    e.step()?;
                    let source = 0xa628_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122c3e: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122c40: jsr $122b18(pc)
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122c44_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x122b18_u32)?;
                    // 00122c44: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x122c46_u32;
                }
                0x122c46 => {
                    // 00122c46: move.l $4471f2.l, $4434ba.l
                    e.step()?;
                    let value_address = 0x4471f2_u32;
                    let value = e.l(value_address)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122c50: move.l $4471f6.l, $4434be.l
                    e.step()?;
                    let value_address = 0x4471f6_u32;
                    let value = e.l(value_address)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122c5a: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122c5c: addi.w #$a628, d0
                    e.step()?;
                    let source = 0xa628_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122c60: lea.l $41c220.l, a0
                    e.step()?;
                    let value = 0x41c220_u32;
                    r.put_address(0, value);
                    // 00122c66: movea.l (a0, d0.w * 4), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00122c6a: move.w $4434be.l, d0
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122c70: sub.w $14(a0), d0
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x14_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122c74: move.w d0, $4434ba.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122c7a: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122c7c: addi.w #$a628, d0
                    e.step()?;
                    let source = 0xa628_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122c80: lea.l $41c220.l, a0
                    e.step()?;
                    let value = 0x41c220_u32;
                    r.put_address(0, value);
                    // 00122c86: movea.l (a0, d0.w * 4), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00122c8a: move.w $4434c0.l, d0
                    e.step()?;
                    let value_address = 0x4434c0_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122c90: sub.w $16(a0), d0
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x16_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122c94: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122c9a: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122ca0: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122ca6: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122ca8: addi.w #$a628, d0
                    e.step()?;
                    let source = 0xa628_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122cac: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122cae: bsr.l $122dc4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122cb4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x122dc4_u32)?;
                    // 00122cb4: lea.l $a(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xa_u32);
                    r.put_address(7, value);
                    // 00122cb8: bra.w $122dae
                    e.step()?;
                    pc = 0x122dae_u32;
                }
                0x122cbc => {
                    // 00122cbc: cmpi.w #$3e7, d3
                    e.step()?;
                    let source = 0x3e7_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00122cc0: ble.b $122cc6
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x122cc6_u32
                    } else {
                        0x122cc2_u32
                    };
                }
                0x122cc2 => {
                    // 00122cc2: subi.w #$3e8, d3
                    e.step()?;
                    let source = 0x3e8_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x122cc6_u32;
                }
                0x122cc6 => {
                    // 00122cc6: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122cc8: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122cca: ext.l d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00122ccc: divs.w #$14, d0
                    e.step()?;
                    let divisor = 0x14_u32;
                    let dividend = r.data(0, 32)?;
                    let value = r.divide_signed(dividend, divisor)?;
                    r.put_data(0, 32, value);
                    // 00122cd0: move.w d0, $4434b4.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434b4_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122cd6: moveq #$14, d0
                    e.step()?;
                    r.put_data(0, 32, 0x14_u32);
                    r.move_flags(0x14_u32, 32);
                    // 00122cd8: muls.w $4434b4.l, d0
                    e.step()?;
                    let source_address = 0x4434b4_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00122cde: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00122ce0: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00122ce2: subq.w #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00122ce4: move.w d1, $4434b6.l
                    e.step()?;
                    let value = r.data(1, 16)?;
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122cea: move.w $4434b4.l, d0
                    e.step()?;
                    let value_address = 0x4434b4_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122cf0: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00122cf2: move.w d0, $4434ba.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122cf8: move.w $4434b6.l, d0
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122cfe: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00122d00: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122d06: moveq #$20, d0
                    e.step()?;
                    r.put_data(0, 32, 0x20_u32);
                    r.move_flags(0x20_u32, 32);
                    // 00122d08: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122d0e: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122d14: moveq #$20, d0
                    e.step()?;
                    r.put_data(0, 32, 0x20_u32);
                    r.move_flags(0x20_u32, 32);
                    // 00122d16: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122d1c: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122d22: tst.b d4
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.move_flags(value, 8);
                    // 00122d24: bne.b $122d50
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122d50_u32
                    } else {
                        0x122d26_u32
                    };
                }
                0x122d26 => {
                    // 00122d26: movea.l $4386c4.l, a0
                    e.step()?;
                    let value_address = 0x4386c4_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00122d2c: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122d30: movea.l $43879c.l, a0
                    e.step()?;
                    let value_address = 0x43879c_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00122d36: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122d3a: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122d40: pea.l $a(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xa_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122d44: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00122d46: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122d48: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122d4a: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122d4c: MacToolbox a8ec
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
                    // 00122d4e: bra.b $122dae
                    e.step()?;
                    pc = 0x122dae_u32;
                }
                0x122d50 => {
                    // 00122d50: cmpi.b #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 00122d54: bne.b $122d80
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122d80_u32
                    } else {
                        0x122d56_u32
                    };
                }
                0x122d56 => {
                    // 00122d56: movea.l $4386c4.l, a0
                    e.step()?;
                    let value_address = 0x4386c4_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00122d5c: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122d60: movea.l $4386c0.l, a0
                    e.step()?;
                    let value_address = 0x4386c0_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00122d66: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122d6a: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122d70: pea.l $a(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xa_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122d74: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00122d76: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122d78: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122d7a: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122d7c: MacToolbox a8ec
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
                    // 00122d7e: bra.b $122dae
                    e.step()?;
                    pc = 0x122dae_u32;
                }
                0x122d80 => {
                    // 00122d80: cmpi.b #$2, d4
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(4, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 00122d84: bne.b $122dae
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122dae_u32
                    } else {
                        0x122d86_u32
                    };
                }
                0x122d86 => {
                    // 00122d86: movea.l $4386c4.l, a0
                    e.step()?;
                    let value_address = 0x4386c4_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00122d8c: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122d90: movea.l $4386bc.l, a0
                    e.step()?;
                    let value_address = 0x4386bc_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00122d96: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122d9a: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122da0: pea.l $a(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xa_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122da4: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00122da6: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122da8: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122daa: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122dac: MacToolbox a8ec
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
                    pc = 0x122dae_u32;
                }
                0x122dae => {
                    // 00122dae: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x122db0_u32;
                }
                0x122db0 => {
                    // 00122db0: movem.l (a7)+, d3-d5
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    // 00122db4: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00122db6: rts
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
crate::recovered_application_control::entry!(fastplot, 0x122bec);

pub fn getrange_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x125538_u32;
        loop {
            match match pc {
                0x125538 => getrange_body_block_0(e, &mut r, frame, pc)?,
                0x125556 => getrange_body_block_1(e, &mut r, frame, pc)?,
                0x125578 => getrange_body_block_2(e, &mut r, frame, pc)?,
                0x1255a0 => getrange_body_block_3(e, &mut r, frame, pc)?,
                0x1255a4 => getrange_body_block_4(e, &mut r, frame, pc)?,
                0x1255b6 => getrange_body_block_5(e, &mut r, frame, pc)?,
                0x1255bc => getrange_body_block_6(e, &mut r, frame, pc)?,
                0x1255c2 => getrange_body_block_7(e, &mut r, frame, pc)?,
                0x1255dc => getrange_body_block_8(e, &mut r, frame, pc)?,
                0x1255f2 => getrange_body_block_9(e, &mut r, frame, pc)?,
                0x12560a => getrange_body_block_a(e, &mut r, frame, pc)?,
                0x125610 => getrange_body_block_b(e, &mut r, frame, pc)?,
                0x12562a => getrange_body_block_c(e, &mut r, frame, pc)?,
                0x125632 => getrange_body_block_d(e, &mut r, frame, pc)?,
                0x125662 => getrange_body_block_e(e, &mut r, frame, pc)?,
                0x125678 => getrange_body_block_f(e, &mut r, frame, pc)?,
                0x1256a8 => getrange_body_block_10(e, &mut r, frame, pc)?,
                0x1256aa => getrange_body_block_11(e, &mut r, frame, pc)?,
                0x1256ae => getrange_body_block_12(e, &mut r, frame, pc)?,
                0x1256f2 => getrange_body_block_13(e, &mut r, frame, pc)?,
                0x12571a => getrange_body_block_14(e, &mut r, frame, pc)?,
                0x125752 => getrange_body_block_15(e, &mut r, frame, pc)?,
                0x125754 => getrange_body_block_16(e, &mut r, frame, pc)?,
                0x12575e => getrange_body_block_17(e, &mut r, frame, pc)?,
                0x125764 => getrange_body_block_18(e, &mut r, frame, pc)?,
                0x12577e => getrange_body_block_19(e, &mut r, frame, pc)?,
                0x125796 => getrange_body_block_1a(e, &mut r, frame, pc)?,
                0x1257ae => getrange_body_block_1b(e, &mut r, frame, pc)?,
                0x1257b4 => getrange_body_block_1c(e, &mut r, frame, pc)?,
                0x1257ce => getrange_body_block_1d(e, &mut r, frame, pc)?,
                0x1257e8 => getrange_body_block_1e(e, &mut r, frame, pc)?,
                0x1257f0 => getrange_body_block_1f(e, &mut r, frame, pc)?,
                0x125830 => getrange_body_block_20(e, &mut r, frame, pc)?,
                0x125860 => getrange_body_block_21(e, &mut r, frame, pc)?,
                0x125862 => getrange_body_block_22(e, &mut r, frame, pc)?,
                0x12588a => getrange_body_block_23(e, &mut r, frame, pc)?,
                0x12588e => getrange_body_block_24(e, &mut r, frame, pc)?,
                0x1258d6 => getrange_body_block_25(e, &mut r, frame, pc)?,
                0x1258fe => getrange_body_block_26(e, &mut r, frame, pc)?,
                0x125936 => getrange_body_block_27(e, &mut r, frame, pc)?,
                0x125938 => getrange_body_block_28(e, &mut r, frame, pc)?,
                0x125946 => getrange_body_block_29(e, &mut r, frame, pc)?,
                _ => getrange_body_block_2a(e, &mut r, frame, pc)?,
            } {
                std::ops::ControlFlow::Continue(next) => pc = next,
                std::ops::ControlFlow::Break(()) => return Ok(()),
            }
        }
    })();
    result.map_err(|signal| match signal {
        Signal::Rule(error) => Signal::Rule(r.opaque_error(error)),
        stopped => stopped,
    })
}
crate::recovered_application_control::entry!(getrange, 0x125538);

pub fn getclosest_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x125960_u32;
        loop {
            match pc {
                0x125960 => {
                    // 00125960: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 00125964: movem.l d3-d7, -(a7)
                    e.step()?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 00125968: move.w $a(a6), d7
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(7, 16, value);
                    r.move_flags(value, 16);
                    // 0012596c: move.w $c(a6), d6
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 00125970: moveq #$ff, d5
                    e.step()?;
                    r.put_data(5, 32, 0xffffffff_u32);
                    r.move_flags(0xffffffff_u32, 32);
                    // 00125972: move.w d6, d4
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 00125974: tst.w d4
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.move_flags(value, 16);
                    // 00125976: bne.b $12597e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x12597e_u32
                    } else {
                        0x125978_u32
                    };
                }
                0x125978 => {
                    // 00125978: move.w #$3e8, d6
                    e.step()?;
                    let value = 0x3e8_u32;
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 0012597c: move.w d6, d4
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x12597e_u32;
                }
                0x12597e => {
                    // 0012597e: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00125980: bra.b $1259e8
                    e.step()?;
                    pc = 0x1259e8_u32;
                }
                0x125982 => {
                    // 00125982: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00125984: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00125988: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0012598e: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00125990: tst.w $110(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x110_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00125994: ble.b $1259e6
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1259e6_u32
                    } else {
                        0x125996_u32
                    };
                }
                0x125996 => {
                    // 00125996: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00125998: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0012599c: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 001259a2: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001259a4: move.b $103(a0), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x103_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001259a8: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001259aa: cmp.w d0, d7
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(7, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001259ac: bne.b $1259e6
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1259e6_u32
                    } else {
                        0x1259ae_u32
                    };
                }
                0x1259ae => {
                    // 001259ae: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 001259b4: move.w $2(a0, d3.w * 4), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001259b8: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 001259be: move.w (a0, d3.w * 4), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001259c2: move.w $8(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001259c6: bsr.l $1124dc
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1259cc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1124dc_u32)?;
                    // 001259cc: cmp.w $4434b6.l, d4
                    e.step()?;
                    let source_address = 0x4434b6_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(4, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001259d2: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001259d4: ble.b $1259e6
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1259e6_u32
                    } else {
                        0x1259d6_u32
                    };
                }
                0x1259d6 => {
                    // 001259d6: cmp.w $4434b6.l, d6
                    e.step()?;
                    let source_address = 0x4434b6_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001259dc: ble.b $1259e6
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1259e6_u32
                    } else {
                        0x1259de_u32
                    };
                }
                0x1259de => {
                    // 001259de: move.w $4434b6.l, d4
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 001259e4: move.w d3, d5
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x1259e6_u32;
                }
                0x1259e6 => {
                    // 001259e6: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x1259e8_u32;
                }
                0x1259e8 => {
                    // 001259e8: cmp.w $43874a.l, d3
                    e.step()?;
                    let source_address = 0x43874a_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001259ee: blt.b $125982
                    e.step()?;
                    pc = if r.less()? {
                        0x125982_u32
                    } else {
                        0x1259f0_u32
                    };
                }
                0x1259f0 => {
                    // 001259f0: move.w d5, d0
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001259f2: movem.l (a7)+, d3-d7
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    // 001259f6: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 001259f8: rts
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
crate::recovered_application_control::entry!(getclosest, 0x125960);

pub fn start_sound_cycle_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x133bb8_u32;
        loop {
            match pc {
                0x133bb8 => {
                    // 00133bb8: link.w a6, #$ffe8
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -24)?;
                    // 00133bbc: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133bbe: move.b $402f43.l, d0
                    e.step()?;
                    let value_address = 0x402f43_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00133bc4: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00133bc6: lea.l $435530.l, a0
                    e.step()?;
                    let value = 0x435530_u32;
                    r.put_address(0, value);
                    // 00133bcc: tst.w (a0, d0.l * 2)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00133bd0: beq.b $133bd6
                    e.step()?;
                    pc = if r.equal()? {
                        0x133bd6_u32
                    } else {
                        0x133bd2_u32
                    };
                }
                0x133bd2 => {
                    // 00133bd2: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00133bd4: bra.b $133c3e
                    e.step()?;
                    pc = 0x133c3e_u32;
                }
                0x133bd6 => {
                    // 00133bd6: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00133bd8: move.l $4436ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4436ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133bde: move.w #$18, -(a7)
                    e.step()?;
                    let value = 0x18_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00133be2: pea.l -$18(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00133be6: move.l #$5100008, d0
                    e.step()?;
                    let value = 0x5100008_u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00133bec: MacToolbox a800
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa800_u32 as u16))?;
                    // 00133bee: move.w (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00133bf0: move.w d0, $4386ac.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4386ac_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00133bf6: movea.l a2, a0
                    e.step()?;
                    let value = r.address(2)?;
                    r.put_address(0, value);
                    // 00133bf8: MacToolbox a02a
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa02a_u32 as u16))?;
                    // 00133bfa: subq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00133bfc: move.l #$736e6420, -(a7)
                    e.step()?;
                    let value = 0x736e6420_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133c02: move.w #$2710, -(a7)
                    e.step()?;
                    let value = 0x2710_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00133c06: MacToolbox a9a0
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9a0_u32 as u16))?;
                    // 00133c08: movea.l (a7)+, a0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_address(0, value);
                    // 00133c0a: movea.l a0, a2
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_address(2, value);
                    // 00133c0c: movea.l a2, a0
                    e.step()?;
                    let value = r.address(2)?;
                    r.put_address(0, value);
                    // 00133c0e: MacToolbox a029
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa029_u32 as u16))?;
                    // 00133c10: move.l a2, d0
                    e.step()?;
                    let value = r.address(2)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00133c12: beq.b $133c3c
                    e.step()?;
                    pc = if r.equal()? {
                        0x133c3c_u32
                    } else {
                        0x133c14_u32
                    };
                }
                0x133c14 => {
                    // 00133c14: cmpi.b #$1, -$c(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00133c1a: beq.b $133c36
                    e.step()?;
                    pc = if r.equal()? {
                        0x133c36_u32
                    } else {
                        0x133c1c_u32
                    };
                }
                0x133c1c => {
                    // 00133c1c: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00133c1e: move.l $4436ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4436ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133c24: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133c26: move.b #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00133c2a: MacToolbox a805
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa805_u32 as u16))?;
                    // 00133c2c: move.w (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00133c2e: bsr.l $133c56
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x133c34_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133c56_u32)?;
                    // 00133c34: bra.b $133c3c
                    e.step()?;
                    pc = 0x133c3c_u32;
                }
                0x133c36 => {
                    // 00133c36: bsr.l $133c56
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x133c3c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133c56_u32)?;
                    pc = 0x133c3c_u32;
                }
                0x133c3c => {
                    // 00133c3c: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x133c3e_u32;
                }
                0x133c3e => {
                    // 00133c3e: movea.l (a7)+, a2
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_address(2, value);
                    // 00133c40: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00133c42: rts
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
crate::recovered_application_control::entry!(start_sound_cycle, 0x133bb8);

pub fn sustain_sound_cycle_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x133c56_u32;
        loop {
            match pc {
                0x133c56 => {
                    // 00133c56: link.w a6, #$ffe0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -32)?;
                    // 00133c5a: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133c5c: move.b $402f43.l, d0
                    e.step()?;
                    let value_address = 0x402f43_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00133c62: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00133c64: lea.l $435530.l, a0
                    e.step()?;
                    let value = 0x435530_u32;
                    r.put_address(0, value);
                    // 00133c6a: tst.w (a0, d0.l * 2)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00133c6e: beq.b $133c76
                    e.step()?;
                    pc = if r.equal()? {
                        0x133c76_u32
                    } else {
                        0x133c70_u32
                    };
                }
                0x133c70 => {
                    // 00133c70: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00133c72: bra.w $133d2c
                    e.step()?;
                    pc = 0x133d2c_u32;
                }
                0x133c76 => {
                    // 00133c76: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00133c78: move.l $4436ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4436ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133c7e: move.w #$18, -(a7)
                    e.step()?;
                    let value = 0x18_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00133c82: pea.l -$20(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00133c86: move.l #$5100008, d0
                    e.step()?;
                    let value = 0x5100008_u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00133c8c: MacToolbox a800
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa800_u32 as u16))?;
                    // 00133c8e: move.w (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00133c90: move.w d0, $4386ac.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4386ac_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00133c96: movea.l a2, a0
                    e.step()?;
                    let value = r.address(2)?;
                    r.put_address(0, value);
                    // 00133c98: MacToolbox a02a
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa02a_u32 as u16))?;
                    // 00133c9a: subq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00133c9c: move.l #$736e6420, -(a7)
                    e.step()?;
                    let value = 0x736e6420_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133ca2: move.w #$2711, -(a7)
                    e.step()?;
                    let value = 0x2711_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00133ca6: MacToolbox a9a0
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9a0_u32 as u16))?;
                    // 00133ca8: movea.l (a7)+, a0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_address(0, value);
                    // 00133caa: movea.l a0, a2
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_address(2, value);
                    // 00133cac: movea.l a2, a0
                    e.step()?;
                    let value = r.address(2)?;
                    r.put_address(0, value);
                    // 00133cae: MacToolbox a029
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa029_u32 as u16))?;
                    // 00133cb0: move.w #$4, -$8(a6)
                    e.step()?;
                    let value = 0x4_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00133cb6: clr.w -$6(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00133cba: clr.l -$4(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133cbe: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00133cc0: move.l $4436ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4436ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133cc6: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00133cca: MacToolbox a804
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa804_u32 as u16))?;
                    // 00133ccc: move.w (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00133cce: move.w d0, $4386ac.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4386ac_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00133cd4: move.l a2, d0
                    e.step()?;
                    let value = r.address(2)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00133cd6: beq.b $133cf2
                    e.step()?;
                    pc = if r.equal()? {
                        0x133cf2_u32
                    } else {
                        0x133cd8_u32
                    };
                }
                0x133cd8 => {
                    // 00133cd8: cmpi.b #$1, -$14(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00133cde: bne.b $133cf2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x133cf2_u32
                    } else {
                        0x133ce0_u32
                    };
                }
                0x133ce0 => {
                    // 00133ce0: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00133ce2: move.l $4436ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4436ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133ce8: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133cea: move.b #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00133cee: MacToolbox a805
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa805_u32 as u16))?;
                    // 00133cf0: move.w (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x133cf2_u32;
                }
                0x133cf2 => {
                    // 00133cf2: movea.l a2, a0
                    e.step()?;
                    let value = r.address(2)?;
                    r.put_address(0, value);
                    // 00133cf4: MacToolbox a02a
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa02a_u32 as u16))?;
                    // 00133cf6: subq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00133cf8: move.l #$736e6420, -(a7)
                    e.step()?;
                    let value = 0x736e6420_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133cfe: move.w #$2712, -(a7)
                    e.step()?;
                    let value = 0x2712_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00133d02: MacToolbox a9a0
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9a0_u32 as u16))?;
                    // 00133d04: movea.l (a7)+, a0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_address(0, value);
                    // 00133d06: movea.l a0, a2
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_address(2, value);
                    // 00133d08: movea.l a2, a0
                    e.step()?;
                    let value = r.address(2)?;
                    r.put_address(0, value);
                    // 00133d0a: MacToolbox a029
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa029_u32 as u16))?;
                    // 00133d0c: move.l a2, d0
                    e.step()?;
                    let value = r.address(2)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00133d0e: beq.b $133d2a
                    e.step()?;
                    pc = if r.equal()? {
                        0x133d2a_u32
                    } else {
                        0x133d10_u32
                    };
                }
                0x133d10 => {
                    // 00133d10: cmpi.b #$1, -$14(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00133d16: bne.b $133d2a
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x133d2a_u32
                    } else {
                        0x133d18_u32
                    };
                }
                0x133d18 => {
                    // 00133d18: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00133d1a: move.l $4436ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4436ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133d20: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00133d22: move.b #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00133d26: MacToolbox a805
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa805_u32 as u16))?;
                    // 00133d28: move.w (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x133d2a_u32;
                }
                0x133d2a => {
                    // 00133d2a: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x133d2c_u32;
                }
                0x133d2c => {
                    // 00133d2c: movea.l (a7)+, a2
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_address(2, value);
                    // 00133d2e: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00133d30: rts
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
crate::recovered_application_control::entry!(sustain_sound_cycle, 0x133c56);

pub fn nobrassnoammo_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x136398_u32;
        loop {
            match pc {
                0x136398 => {
                    // 00136398: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 0013639c: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013639e: bsr.l $130d38
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1363a4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x130d38_u32)?;
                    // 001363a4: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001363a6: bsr.l $130e54
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1363ac_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x130e54_u32)?;
                    // 001363ac: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001363ae: bsr.l $130afe
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1363b4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x130afe_u32)?;
                    // 001363b4: lea.l $443788.l, a0
                    e.step()?;
                    let value = 0x443788_u32;
                    r.put_address(0, value);
                    // 001363ba: lea.l $21c2(a0), a0
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x21c2_u32);
                    r.put_address(0, value);
                    // 001363be: lea.l $44374e.l, a1
                    e.step()?;
                    let value = 0x44374e_u32;
                    r.put_address(1, value);
                    // 001363c4: moveq #$d, d0
                    e.step()?;
                    r.put_data(0, 32, 0xd_u32);
                    r.move_flags(0xd_u32, 32);
                    pc = 0x1363c6_u32;
                }
                0x1363c6 => {
                    // 001363c6: move.l (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = e.l(value_address)?;
                    r.offset_address(0, 4)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.offset_address(1, 4)?;
                    r.move_flags(value, 32);
                    // 001363c8: dbra d0, $1363c6
                    e.step()?;
                    let counter = r.data(0, 16)?;
                    let value = counter.wrapping_sub(1) & 0xffff;
                    r.put_data(0, 16, value);
                    pc = if value != 0xffff {
                        0x1363c6_u32
                    } else {
                        0x1363cc_u32
                    };
                }
                0x1363cc => {
                    // 001363cc: move.w (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(0, 2)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.offset_address(1, 2)?;
                    r.move_flags(value, 16);
                    // 001363ce: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 001363d0: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 001363d2: rts
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
crate::recovered_application_control::entry!(nobrassnoammo, 0x136398);

pub fn best_choice_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x1363e4_u32;
        loop {
            match match pc {
                0x1363e4 => best_choice_body_block_0(e, &mut r, frame, pc)?,
                0x13640a => best_choice_body_block_1(e, &mut r, frame, pc)?,
                0x136422 => best_choice_body_block_2(e, &mut r, frame, pc)?,
                0x13642a => best_choice_body_block_3(e, &mut r, frame, pc)?,
                0x136434 => best_choice_body_block_4(e, &mut r, frame, pc)?,
                0x13643a => best_choice_body_block_5(e, &mut r, frame, pc)?,
                0x136442 => best_choice_body_block_6(e, &mut r, frame, pc)?,
                0x13645a => best_choice_body_block_7(e, &mut r, frame, pc)?,
                0x136498 => best_choice_body_block_8(e, &mut r, frame, pc)?,
                0x1364a8 => best_choice_body_block_9(e, &mut r, frame, pc)?,
                0x1364c0 => best_choice_body_block_a(e, &mut r, frame, pc)?,
                0x1364f8 => best_choice_body_block_b(e, &mut r, frame, pc)?,
                0x1364fe => best_choice_body_block_c(e, &mut r, frame, pc)?,
                0x136504 => best_choice_body_block_d(e, &mut r, frame, pc)?,
                0x136524 => best_choice_body_block_e(e, &mut r, frame, pc)?,
                0x13655a => best_choice_body_block_f(e, &mut r, frame, pc)?,
                0x136564 => best_choice_body_block_10(e, &mut r, frame, pc)?,
                0x13657a => best_choice_body_block_11(e, &mut r, frame, pc)?,
                0x13657e => best_choice_body_block_12(e, &mut r, frame, pc)?,
                0x136582 => best_choice_body_block_13(e, &mut r, frame, pc)?,
                0x13659a => best_choice_body_block_14(e, &mut r, frame, pc)?,
                0x1365b6 => best_choice_body_block_15(e, &mut r, frame, pc)?,
                0x1365ce => best_choice_body_block_16(e, &mut r, frame, pc)?,
                0x1365d2 => best_choice_body_block_17(e, &mut r, frame, pc)?,
                0x1365ea => best_choice_body_block_18(e, &mut r, frame, pc)?,
                0x1365ec => best_choice_body_block_19(e, &mut r, frame, pc)?,
                0x1365ee => best_choice_body_block_1a(e, &mut r, frame, pc)?,
                0x1365f6 => best_choice_body_block_1b(e, &mut r, frame, pc)?,
                0x1365f8 => best_choice_body_block_1c(e, &mut r, frame, pc)?,
                0x136600 => best_choice_body_block_1d(e, &mut r, frame, pc)?,
                0x136604 => best_choice_body_block_1e(e, &mut r, frame, pc)?,
                0x136606 => best_choice_body_block_1f(e, &mut r, frame, pc)?,
                0x13660c => best_choice_body_block_20(e, &mut r, frame, pc)?,
                0x136618 => best_choice_body_block_21(e, &mut r, frame, pc)?,
                0x13661e => best_choice_body_block_22(e, &mut r, frame, pc)?,
                0x136624 => best_choice_body_block_23(e, &mut r, frame, pc)?,
                0x136628 => best_choice_body_block_24(e, &mut r, frame, pc)?,
                0x136638 => best_choice_body_block_25(e, &mut r, frame, pc)?,
                0x136642 => best_choice_body_block_26(e, &mut r, frame, pc)?,
                0x136646 => best_choice_body_block_27(e, &mut r, frame, pc)?,
                0x136656 => best_choice_body_block_28(e, &mut r, frame, pc)?,
                0x13665c => best_choice_body_block_29(e, &mut r, frame, pc)?,
                0x136662 => best_choice_body_block_2a(e, &mut r, frame, pc)?,
                0x136668 => best_choice_body_block_2b(e, &mut r, frame, pc)?,
                0x136676 => best_choice_body_block_2c(e, &mut r, frame, pc)?,
                0x136680 => best_choice_body_block_2d(e, &mut r, frame, pc)?,
                0x136684 => best_choice_body_block_2e(e, &mut r, frame, pc)?,
                0x13668e => best_choice_body_block_2f(e, &mut r, frame, pc)?,
                _ => best_choice_body_block_30(e, &mut r, frame, pc)?,
            } {
                std::ops::ControlFlow::Continue(next) => pc = next,
                std::ops::ControlFlow::Break(()) => return Ok(()),
            }
        }
    })();
    result.map_err(|signal| match signal {
        Signal::Rule(error) => Signal::Rule(r.opaque_error(error)),
        stopped => stopped,
    })
}
crate::recovered_application_control::entry!(best_choice, 0x1363e4);

pub fn drawspell_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x1377f4_u32;
        loop {
            match pc {
                0x1377f4 => {
                    // 001377f4: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 001377f8: cmpi.b #$5, $44374f.l
                    e.step()?;
                    let source = 0x5_u32;
                    let destination_address = 0x44374f_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00137800: beq.b $13780a
                    e.step()?;
                    pc = if r.equal()? {
                        0x13780a_u32
                    } else {
                        0x137802_u32
                    };
                }
                0x137802 => {
                    // 00137802: tst.w $4435bc.l
                    e.step()?;
                    let value_address = 0x4435bc_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00137808: bne.b $137810
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x137810_u32
                    } else {
                        0x13780a_u32
                    };
                }
                0x13780a => {
                    // 0013780a: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0013780c: bra.w $137892
                    e.step()?;
                    pc = 0x137892_u32;
                }
                0x137810 => {
                    // 00137810: move.l $43879c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43879c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00137816: MacToolbox a873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 00137818: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0013781e: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00137824: move.w (a0, d0.w * 4), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00137828: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 0013782a: addi.w #$10, d0
                    e.step()?;
                    let source = 0x10_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013782e: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00137830: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 00137836: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013783c: move.w $2(a0, d0.w * 4), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00137840: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00137842: addi.w #$10, d0
                    e.step()?;
                    let source = 0x10_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00137846: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00137848: MacToolbox a893
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa893_u32 as u16))?;
                    // 0013784a: move.w $44338e.l, d0
                    e.step()?;
                    let value_address = 0x44338e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00137850: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00137852: addi.w #$10, d0
                    e.step()?;
                    let source = 0x10_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00137856: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00137858: move.w $443390.l, d0
                    e.step()?;
                    let value_address = 0x443390_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013785e: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00137860: addi.w #$10, d0
                    e.step()?;
                    let source = 0x10_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00137864: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00137866: MacToolbox a891
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa891_u32 as u16))?;
                    // 00137868: movea.l $4386b8.l, a0
                    e.step()?;
                    let value_address = 0x4386b8_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0013786e: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00137872: movea.l $43879c.l, a0
                    e.step()?;
                    let value_address = 0x43879c_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00137878: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0013787c: pea.l $439d0c.l
                    e.step()?;
                    let value = 0x439d0c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00137882: pea.l $443712.l
                    e.step()?;
                    let value = 0x443712_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00137888: move.w #$2, -(a7)
                    e.step()?;
                    let value = 0x2_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013788c: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0013788e: MacToolbox a8ec
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
                    // 00137890: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x137892_u32;
                }
                0x137892 => {
                    // 00137892: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00137894: rts
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
crate::recovered_application_control::entry!(drawspell, 0x1377f4);

pub fn string_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        // 001382d0: link.w a6, #$0
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, 0)?;
        // 001382d4: movea.w $8(a6), a0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0x8_u32);
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 001382d8: move.l a0, -(a7)
        e.step()?;
        let value = r.address(0)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001382da: pea.l $479ed8.l
        e.step()?;
        let value = 0x479ed8_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001382e0: jsr $400168.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1382e6_u32)?;
        imported(e, &mut r, NativeCall::Library(0x400168_u32))?;
        // 001382e6: pea.l $479ed8.l
        e.step()?;
        let value = 0x479ed8_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001382ec: MacToolbox a884
        e.step()?;
        imported(e, &mut r, NativeCall::Toolbox(0xa884_u32 as u16))?;
        // 001382ee: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 001382f0: rts
        e.step()?;
        finish(e, &mut r)
    })();
    result.map_err(|signal| match signal {
        Signal::Rule(error) => Signal::Rule(r.opaque_error(error)),
        stopped => stopped,
    })
}
crate::recovered_application_control::entry!(string, 0x1382d0);

pub fn xy_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x138a40_u32;
        loop {
            match pc {
                0x138a40 => {
                    // 00138a40: link.w a6, #$fff8
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -8)?;
                    // 00138a44: move.l $43878c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43878c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00138a4a: MacToolbox a873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 00138a4c: move.l $4358ce.l, -(a7)
                    e.step()?;
                    let value_address = 0x4358ce_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00138a52: MacToolbox aa0b
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa0b_u32 as u16))?;
                    // 00138a54: move.w $439582.l, d0
                    e.step()?;
                    let value_address = 0x439582_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00138a5a: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00138a5c: move.w $40856e.l, d1
                    e.step()?;
                    let value_address = 0x40856e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00138a62: addi.w #$130, d1
                    e.step()?;
                    let source = 0x130_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00138a66: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00138a68: move.w d1, -$8(a6)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138a6c: moveq #$10, d0
                    e.step()?;
                    r.put_data(0, 32, 0x10_u32);
                    r.move_flags(0x10_u32, 32);
                    // 00138a6e: add.w -$8(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00138a72: move.w d0, -$4(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138a76: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00138a7c: addi.w #$15e, d0
                    e.step()?;
                    let source = 0x15e_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00138a80: move.w d0, -$6(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138a84: moveq #$19, d0
                    e.step()?;
                    r.put_data(0, 32, 0x19_u32);
                    r.move_flags(0x19_u32, 32);
                    // 00138a86: add.w -$6(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00138a8a: move.w d0, -$2(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138a8e: pea.l $44337a.l
                    e.step()?;
                    let value = 0x44337a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138a94: MacToolbox aa14
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa14_u32 as u16))?;
                    // 00138a96: move.w $402f3c.l, -(a7)
                    e.step()?;
                    let value_address = 0x402f3c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138a9c: MacToolbox a887
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa887_u32 as u16))?;
                    // 00138a9e: move.w #$c, -(a7)
                    e.step()?;
                    let value = 0xc_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138aa2: MacToolbox a88a
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa88a_u32 as u16))?;
                    // 00138aa4: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138aa8: MacToolbox a889
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa889_u32 as u16))?;
                    // 00138aaa: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138aac: MacToolbox a888
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa888_u32 as u16))?;
                    // 00138aae: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138ab2: MacToolbox a8a3
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8a3_u32 as u16))?;
                    // 00138ab4: addi.w #$32, -$6(a6)
                    e.step()?;
                    let source = 0x32_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 00138aba: addi.w #$32, -$2(a6)
                    e.step()?;
                    let source = 0x32_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 00138ac0: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138ac4: MacToolbox a8a3
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8a3_u32 as u16))?;
                    // 00138ac6: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00138acc: addi.w #$160, d0
                    e.step()?;
                    let source = 0x160_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00138ad0: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138ad2: move.w $439582.l, d0
                    e.step()?;
                    let value_address = 0x439582_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00138ad8: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00138ada: move.w $40856e.l, d1
                    e.step()?;
                    let value_address = 0x40856e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00138ae0: addi.w #$13c, d1
                    e.step()?;
                    let source = 0x13c_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00138ae4: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00138ae6: move.w d1, -(a7)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138ae8: MacToolbox a893
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa893_u32 as u16))?;
                    // 00138aea: tst.w $8(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00138aee: beq.b $138b50
                    e.step()?;
                    pc = if r.equal()? {
                        0x138b50_u32
                    } else {
                        0x138af0_u32
                    };
                }
                0x138af0 => {
                    // 00138af0: movea.w $442814.l, a0
                    e.step()?;
                    let value_address = 0x442814_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 00138af6: move.l a0, -(a7)
                    e.step()?;
                    let value = r.address(0)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00138af8: pea.l $479ed8.l
                    e.step()?;
                    let value = 0x479ed8_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138afe: jsr $400168.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x138b04_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400168_u32))?;
                    // 00138b04: pea.l $479ed8.l
                    e.step()?;
                    let value = 0x479ed8_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138b0a: MacToolbox a884
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa884_u32 as u16))?;
                    // 00138b0c: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00138b12: addi.w #$196, d0
                    e.step()?;
                    let source = 0x196_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00138b16: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138b18: move.w $439582.l, d0
                    e.step()?;
                    let value_address = 0x439582_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00138b1e: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00138b20: move.w $40856e.l, d1
                    e.step()?;
                    let value_address = 0x40856e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00138b26: addi.w #$13c, d1
                    e.step()?;
                    let source = 0x13c_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00138b2a: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00138b2c: move.w d1, -(a7)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138b2e: MacToolbox a893
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa893_u32 as u16))?;
                    // 00138b30: movea.w $442816.l, a0
                    e.step()?;
                    let value_address = 0x442816_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 00138b36: move.l a0, -(a7)
                    e.step()?;
                    let value = r.address(0)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00138b38: pea.l $479ed8.l
                    e.step()?;
                    let value = 0x479ed8_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138b3e: jsr $400168.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x138b44_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400168_u32))?;
                    // 00138b44: pea.l $479ed8.l
                    e.step()?;
                    let value = 0x479ed8_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138b4a: MacToolbox a884
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa884_u32 as u16))?;
                    // 00138b4c: bra.w $138bdc
                    e.step()?;
                    pc = 0x138bdc_u32;
                }
                0x138b50 => {
                    // 00138b50: move.l $442778.l, d0
                    e.step()?;
                    let value_address = 0x442778_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00138b56: add.l $442764.l, d0
                    e.step()?;
                    let source_address = 0x442764_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00138b5c: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00138b5e: pea.l $479ed8.l
                    e.step()?;
                    let value = 0x479ed8_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138b64: jsr $400168.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x138b6a_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400168_u32))?;
                    // 00138b6a: tst.w $4354fe.l
                    e.step()?;
                    let value_address = 0x4354fe_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00138b70: beq.b $138b7c
                    e.step()?;
                    pc = if r.equal()? {
                        0x138b7c_u32
                    } else {
                        0x138b72_u32
                    };
                }
                0x138b72 => {
                    // 00138b72: pea.l $40226a.l
                    e.step()?;
                    let value = 0x40226a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138b78: MacToolbox a884
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa884_u32 as u16))?;
                    // 00138b7a: bra.b $138b84
                    e.step()?;
                    pc = 0x138b84_u32;
                }
                0x138b7c => {
                    // 00138b7c: pea.l $479ed8.l
                    e.step()?;
                    let value = 0x479ed8_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138b82: MacToolbox a884
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa884_u32 as u16))?;
                    pc = 0x138b84_u32;
                }
                0x138b84 => {
                    // 00138b84: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00138b8a: addi.w #$196, d0
                    e.step()?;
                    let source = 0x196_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00138b8e: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138b90: move.w $439582.l, d0
                    e.step()?;
                    let value_address = 0x439582_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00138b96: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 00138b98: move.w $40856e.l, d1
                    e.step()?;
                    let value_address = 0x40856e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00138b9e: addi.w #$13c, d1
                    e.step()?;
                    let source = 0x13c_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00138ba2: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00138ba4: move.w d1, -(a7)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00138ba6: MacToolbox a893
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa893_u32 as u16))?;
                    // 00138ba8: move.l $44277c.l, d0
                    e.step()?;
                    let value_address = 0x44277c_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00138bae: add.l $442768.l, d0
                    e.step()?;
                    let source_address = 0x442768_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00138bb4: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00138bb6: pea.l $479ed8.l
                    e.step()?;
                    let value = 0x479ed8_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138bbc: jsr $400168.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x138bc2_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400168_u32))?;
                    // 00138bc2: tst.w $4354fe.l
                    e.step()?;
                    let value_address = 0x4354fe_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00138bc8: beq.b $138bd4
                    e.step()?;
                    pc = if r.equal()? {
                        0x138bd4_u32
                    } else {
                        0x138bca_u32
                    };
                }
                0x138bca => {
                    // 00138bca: pea.l $40226a.l
                    e.step()?;
                    let value = 0x40226a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138bd0: MacToolbox a884
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa884_u32 as u16))?;
                    // 00138bd2: bra.b $138bdc
                    e.step()?;
                    pc = 0x138bdc_u32;
                }
                0x138bd4 => {
                    // 00138bd4: pea.l $479ed8.l
                    e.step()?;
                    let value = 0x479ed8_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00138bda: MacToolbox a884
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa884_u32 as u16))?;
                    pc = 0x138bdc_u32;
                }
                0x138bdc => {
                    // 00138bdc: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00138bde: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00138be0: rts
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
crate::recovered_application_control::entry!(xy, 0x138a40);

pub fn save_against_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x13a3ae_u32;
        loop {
            match pc {
                0x13a3ae => {
                    // 0013a3ae: link.w a6, #$fffe
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -2)?;
                    // 0013a3b2: movem.l d3-d6, -(a7)
                    e.step()?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 0013a3b6: move.w $8(a6), d5
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 0013a3ba: move.w $a(a6), d3
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 0013a3be: clr.w -$2(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013a3c2: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0013a3c4: tst.w $438760.l
                    e.step()?;
                    let value_address = 0x438760_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0013a3ca: beq.b $13a3dc
                    e.step()?;
                    pc = if r.equal()? {
                        0x13a3dc_u32
                    } else {
                        0x13a3cc_u32
                    };
                }
                0x13a3cc => {
                    // 0013a3cc: cmpi.b #$1, $443757.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x443757_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 0013a3d4: ble.b $13a3dc
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13a3dc_u32
                    } else {
                        0x13a3d6_u32
                    };
                }
                0x13a3d6 => {
                    // 0013a3d6: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0013a3d8: bra.w $13a486
                    e.step()?;
                    pc = 0x13a486_u32;
                }
                0x13a3dc => {
                    // 0013a3dc: move.b $443756.l, d0
                    e.step()?;
                    let value_address = 0x443756_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013a3e2: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013a3e4: muls.w $4435cc.l, d0
                    e.step()?;
                    let source_address = 0x4435cc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013a3ea: move.b $443754.l, d4
                    e.step()?;
                    let value_address = 0x443754_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(4, 8, value);
                    r.move_flags(value, 8);
                    // 0013a3f0: ext.w d4
                    e.step()?;
                    let source = r.data(4, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 0013a3f2: add.w d0, d4
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    // 0013a3f4: move.w #$64, -(a7)
                    e.step()?;
                    let value = 0x64_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0013a3f8: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x13a3fe_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 0013a3fe: move.w d0, d6
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 0013a400: cmpi.w #$9, d3
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013a404: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0013a406: bge.b $13a442
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x13a442_u32
                    } else {
                        0x13a408_u32
                    };
                }
                0x13a408 => {
                    // 0013a408: cmpi.w #$ffff, d3
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013a40c: ble.b $13a442
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13a442_u32
                    } else {
                        0x13a40e_u32
                    };
                }
                0x13a40e => {
                    // 0013a40e: tst.w d5
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.move_flags(value, 16);
                    // 0013a410: bne.b $13a41e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x13a41e_u32
                    } else {
                        0x13a412_u32
                    };
                }
                0x13a412 => {
                    // 0013a412: tst.w $442570.l
                    e.step()?;
                    let value_address = 0x442570_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0013a418: beq.b $13a41e
                    e.step()?;
                    pc = if r.equal()? {
                        0x13a41e_u32
                    } else {
                        0x13a41a_u32
                    };
                }
                0x13a41a => {
                    // 0013a41a: addi.w #$32, d4
                    e.step()?;
                    let source = 0x32_u32;
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    pc = 0x13a41e_u32;
                }
                0x13a41e => {
                    // 0013a41e: movea.w d5, a0
                    e.step()?;
                    let value = r.data(5, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0013a420: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013a422: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0013a424: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013a426: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0013a42a: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0013a430: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013a432: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013a434: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013a436: add.w $112(a0), d0
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x112_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013a43a: cmp.w d0, d6
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013a43c: bgt.b $13a484
                    e.step()?;
                    pc = if r.greater()? {
                        0x13a484_u32
                    } else {
                        0x13a43e_u32
                    };
                }
                0x13a43e => {
                    // 0013a43e: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 0013a440: bra.b $13a486
                    e.step()?;
                    pc = 0x13a486_u32;
                }
                0x13a442 => {
                    // 0013a442: cmpi.w #$9, d3
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013a446: ble.b $13a484
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13a484_u32
                    } else {
                        0x13a448_u32
                    };
                }
                0x13a448 => {
                    // 0013a448: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 0013a44a: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013a44c: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013a450: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013a456: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013a458: adda.w d5, a0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.address(0)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013a45a: move.b $8b(a0), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8b_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0013a45e: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013a460: add.w d4, d0
                    e.step()?;
                    let source = r.data(4, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013a462: cmp.w d0, d6
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013a464: ble.b $13a480
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13a480_u32
                    } else {
                        0x13a466_u32
                    };
                }
                0x13a466 => {
                    // 0013a466: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 0013a468: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0013a46a: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0013a46e: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0013a474: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013a476: adda.w d5, a0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.address(0)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0013a478: cmpi.b #$63, $8b(a0)
                    e.step()?;
                    let source = 0x63_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x8b_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 0013a47e: ble.b $13a484
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13a484_u32
                    } else {
                        0x13a480_u32
                    };
                }
                0x13a480 => {
                    // 0013a480: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 0013a482: bra.b $13a486
                    e.step()?;
                    pc = 0x13a486_u32;
                }
                0x13a484 => {
                    // 0013a484: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x13a486_u32;
                }
                0x13a486 => {
                    // 0013a486: movem.l (a7)+, d3-d6
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    // 0013a48a: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0013a48c: rts
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
crate::recovered_application_control::entry!(save_against, 0x13a3ae);

pub fn pin_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x13f65e_u32;
        loop {
            match pc {
                0x13f65e => {
                    // 0013f65e: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 0013f662: move.w $8(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0013f666: move.w $a(a6), d2
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(2, 16, value);
                    r.move_flags(value, 16);
                    // 0013f66a: move.w $c(a6), d1
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0013f66e: cmp.w d2, d0
                    e.step()?;
                    let source = r.data(2, 16)?;
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013f670: bge.b $13f674
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x13f674_u32
                    } else {
                        0x13f672_u32
                    };
                }
                0x13f672 => {
                    // 0013f672: move.w d2, d0
                    e.step()?;
                    let value = r.data(2, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x13f674_u32;
                }
                0x13f674 => {
                    // 0013f674: cmp.w d1, d0
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0013f676: ble.b $13f67a
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x13f67a_u32
                    } else {
                        0x13f678_u32
                    };
                }
                0x13f678 => {
                    // 0013f678: move.w d1, d0
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x13f67a_u32;
                }
                0x13f67a => {
                    // 0013f67a: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0013f67c: rts
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
crate::recovered_application_control::entry!(pin, 0x13f65e);

pub fn placemonster_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x149e0e_u32;
        loop {
            match pc {
                0x149e0e => {
                    // 00149e0e: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 00149e12: movem.l d3-d5, -(a7)
                    e.step()?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 00149e16: move.w $8(a6), d5
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 00149e1a: move.w $a(a6), d4
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 00149e1e: move.w $c(a6), d3
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00149e22: add.w $442782.l, d5
                    e.step()?;
                    let source_address = 0x442782_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(5, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(5, 16, value);
                    // 00149e28: add.w $442786.l, d4
                    e.step()?;
                    let source_address = 0x442786_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    // 00149e2e: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149e30: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00149e34: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00149e3a: move.w $10(a0, d0.l), -(a7)
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
                    // 00149e3e: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x149e44_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00149e44: move.w d0, $4435ce.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4435ce_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00149e4a: move.w d5, d0
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149e4c: mulu.w #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00149e50: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 00149e56: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149e58: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149e5a: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00149e5e: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 00149e64: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 00149e66: move.w (a0, d4.w * 2), $11e(a1)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(1)?.wrapping_add(0x11e_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00149e6c: moveq #$a, d0
                    e.step()?;
                    r.put_data(0, 32, 0xa_u32);
                    r.move_flags(0xa_u32, 32);
                    // 00149e6e: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149e70: move.w d5, d1
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00149e72: mulu.w #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00149e76: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 00149e7c: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149e7e: move.w d0, (a0, d4.w * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00149e82: cmpi.w #$3, $4435ce.l
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = 0x4435ce_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00149e8a: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00149e8c: bne.b $149ed6
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x149ed6_u32
                    } else {
                        0x149e8e_u32
                    };
                }
                0x149e8e => {
                    // 00149e8e: move.w d5, d0
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149e90: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149e92: mulu.w #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00149e96: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 00149e9c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149e9e: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149ea0: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149ea2: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00149ea4: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00149ea8: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 00149eae: adda.l d1, a1
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 00149eb0: move.w (a0, d0.w * 2), $118(a1)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(1)?.wrapping_add(0x118_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00149eb6: moveq #$a, d0
                    e.step()?;
                    r.put_data(0, 32, 0xa_u32);
                    r.move_flags(0xa_u32, 32);
                    // 00149eb8: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149eba: neg.w d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = r.subtract_flags(0, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149ebc: move.w d5, d1
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00149ebe: subq.w #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00149ec0: mulu.w #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00149ec4: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 00149eca: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149ecc: move.w d4, d1
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00149ece: subq.w #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00149ed0: move.w d0, (a0, d1.w * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00149ed4: bra.b $149ee8
                    e.step()?;
                    pc = 0x149ee8_u32;
                }
                0x149ed6 => {
                    // 00149ed6: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149ed8: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00149edc: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00149ee2: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149ee4: clr.w $118(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x118_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x149ee8_u32;
                }
                0x149ee8 => {
                    // 00149ee8: cmpi.w #$1, $4435ce.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4435ce_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00149ef0: beq.b $149efc
                    e.step()?;
                    pc = if r.equal()? {
                        0x149efc_u32
                    } else {
                        0x149ef2_u32
                    };
                }
                0x149ef2 => {
                    // 00149ef2: cmpi.w #$3, $4435ce.l
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = 0x4435ce_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00149efa: bne.b $149f40
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x149f40_u32
                    } else {
                        0x149efc_u32
                    };
                }
                0x149efc => {
                    // 00149efc: move.w d5, d0
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149efe: mulu.w #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00149f02: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 00149f08: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149f0a: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149f0c: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149f0e: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00149f10: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00149f14: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 00149f1a: adda.l d1, a1
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 00149f1c: move.w (a0, d0.w * 2), $11c(a1)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(1)?.wrapping_add(0x11c_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00149f22: moveq #$a, d0
                    e.step()?;
                    r.put_data(0, 32, 0xa_u32);
                    r.move_flags(0xa_u32, 32);
                    // 00149f24: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149f26: neg.w d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = r.subtract_flags(0, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149f28: move.w d5, d1
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00149f2a: mulu.w #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00149f2e: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 00149f34: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149f36: move.w d4, d1
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00149f38: subq.w #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00149f3a: move.w d0, (a0, d1.w * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00149f3e: bra.b $149f52
                    e.step()?;
                    pc = 0x149f52_u32;
                }
                0x149f40 => {
                    // 00149f40: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149f42: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00149f46: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00149f4c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149f4e: clr.w $11c(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x11c_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x149f52_u32;
                }
                0x149f52 => {
                    // 00149f52: cmpi.w #$1, $4435ce.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4435ce_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00149f5a: ble.b $149f9c
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x149f9c_u32
                    } else {
                        0x149f5c_u32
                    };
                }
                0x149f5c => {
                    // 00149f5c: move.w d5, d0
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149f5e: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149f60: mulu.w #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00149f64: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 00149f6a: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149f6c: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149f6e: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00149f72: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 00149f78: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 00149f7a: move.w (a0, d4.w * 2), $11a(a1)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(1)?.wrapping_add(0x11a_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00149f80: moveq #$a, d0
                    e.step()?;
                    r.put_data(0, 32, 0xa_u32);
                    r.move_flags(0xa_u32, 32);
                    // 00149f82: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149f84: neg.w d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = r.subtract_flags(0, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00149f86: move.w d5, d1
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00149f88: subq.w #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00149f8a: mulu.w #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00149f8e: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 00149f94: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149f96: move.w d0, (a0, d4.w * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00149f9a: bra.b $149fae
                    e.step()?;
                    pc = 0x149fae_u32;
                }
                0x149f9c => {
                    // 00149f9c: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00149f9e: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00149fa2: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00149fa8: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00149faa: clr.w $11a(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x11a_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x149fae_u32;
                }
                0x149fae => {
                    // 00149fae: movem.l (a7)+, d3-d5
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    // 00149fb2: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00149fb4: rts
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
crate::recovered_application_control::entry!(placemonster, 0x149e0e);

pub fn direction_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x151f30_u32;
        loop {
            match match pc {
                0x151f30 => direction_body_block_0(e, &mut r, frame, pc)?,
                0x151f7e => direction_body_block_1(e, &mut r, frame, pc)?,
                0x151fa0 => direction_body_block_2(e, &mut r, frame, pc)?,
                0x151fa2 => direction_body_block_3(e, &mut r, frame, pc)?,
                0x151fc4 => direction_body_block_4(e, &mut r, frame, pc)?,
                0x151fe0 => direction_body_block_5(e, &mut r, frame, pc)?,
                0x151fe2 => direction_body_block_6(e, &mut r, frame, pc)?,
                0x152004 => direction_body_block_7(e, &mut r, frame, pc)?,
                0x152026 => direction_body_block_8(e, &mut r, frame, pc)?,
                0x152028 => direction_body_block_9(e, &mut r, frame, pc)?,
                0x152044 => direction_body_block_a(e, &mut r, frame, pc)?,
                0x152066 => direction_body_block_b(e, &mut r, frame, pc)?,
                0x152068 => direction_body_block_c(e, &mut r, frame, pc)?,
                0x15208a => direction_body_block_d(e, &mut r, frame, pc)?,
                0x1520ac => direction_body_block_e(e, &mut r, frame, pc)?,
                0x1520ae => direction_body_block_f(e, &mut r, frame, pc)?,
                0x1520d0 => direction_body_block_10(e, &mut r, frame, pc)?,
                0x1520ec => direction_body_block_11(e, &mut r, frame, pc)?,
                0x1520ee => direction_body_block_12(e, &mut r, frame, pc)?,
                0x152110 => direction_body_block_13(e, &mut r, frame, pc)?,
                0x152132 => direction_body_block_14(e, &mut r, frame, pc)?,
                0x152134 => direction_body_block_15(e, &mut r, frame, pc)?,
                0x152150 => direction_body_block_16(e, &mut r, frame, pc)?,
                0x152172 => direction_body_block_17(e, &mut r, frame, pc)?,
                0x152174 => direction_body_block_18(e, &mut r, frame, pc)?,
                _ => direction_body_block_19(e, &mut r, frame, pc)?,
            } {
                std::ops::ControlFlow::Continue(next) => pc = next,
                std::ops::ControlFlow::Break(()) => return Ok(()),
            }
        }
    })();
    result.map_err(|signal| match signal {
        Signal::Rule(error) => Signal::Rule(r.opaque_error(error)),
        stopped => stopped,
    })
}
crate::recovered_application_control::entry!(direction, 0x151f30);

pub fn movement_cost_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x153de6_u32;
        loop {
            match pc {
                0x153de6 => {
                    // 00153de6: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 00153dea: move.w $8(a6), d2
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(2, 16, value);
                    r.move_flags(value, 16);
                    // 00153dee: moveq #$1, d1
                    e.step()?;
                    r.put_data(1, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 00153df0: cmpi.w #$3e7, d2
                    e.step()?;
                    let source = 0x3e7_u32;
                    let destination = r.data(2, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153df4: ble.b $153dfa
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x153dfa_u32
                    } else {
                        0x153df6_u32
                    };
                }
                0x153df6 => {
                    // 00153df6: subi.w #$3e8, d2
                    e.step()?;
                    let source = 0x3e8_u32;
                    let destination = r.data(2, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(2, 16, value);
                    pc = 0x153dfa_u32;
                }
                0x153dfa => {
                    // 00153dfa: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 00153dfc: mulu.w d2, d0
                    e.step()?;
                    let source = r.data(2, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00153dfe: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 00153e04: move.w $2(a0, d0.l), d1
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x2_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00153e08: lsr.w #$8, d1
                    e.step()?;
                    let count = 0x8_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(1, 16, value);
                    // 00153e0a: lsr.w #$7, d1
                    e.step()?;
                    let count = 0x7_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(1, 16, value);
                    // 00153e0c: add.w $2(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x2_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00153e10: asr.w #$1, d1
                    e.step()?;
                    let count = 0x1_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(1, 16, value);
                    // 00153e12: subq.w #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00153e14: cmpi.w #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153e18: bge.b $153e1c
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x153e1c_u32
                    } else {
                        0x153e1a_u32
                    };
                }
                0x153e1a => {
                    // 00153e1a: moveq #$0, d1
                    e.step()?;
                    r.put_data(1, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x153e1c_u32;
                }
                0x153e1c => {
                    // 00153e1c: tst.b $442736.l
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00153e22: beq.b $153e26
                    e.step()?;
                    pc = if r.equal()? {
                        0x153e26_u32
                    } else {
                        0x153e24_u32
                    };
                }
                0x153e24 => {
                    // 00153e24: addq.w #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    pc = 0x153e26_u32;
                }
                0x153e26 => {
                    // 00153e26: tst.b $442737.l
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00153e2c: beq.b $153e30
                    e.step()?;
                    pc = if r.equal()? {
                        0x153e30_u32
                    } else {
                        0x153e2e_u32
                    };
                }
                0x153e2e => {
                    // 00153e2e: addq.w #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    pc = 0x153e30_u32;
                }
                0x153e30 => {
                    // 00153e30: move.w d1, d0
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00153e32: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00153e34: rts
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
crate::recovered_application_control::entry!(movement_cost, 0x153de6);

pub fn shift_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x153e44_u32;
        loop {
            match pc {
                0x153e44 => {
                    // 00153e44: link.w a6, #$fffe
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -2)?;
                    // 00153e48: movem.l d3-d7, -(a7)
                    e.step()?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 00153e4c: move.b $d(a6), d4
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xd_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(4, 8, value);
                    r.move_flags(value, 8);
                    // 00153e50: cmpi.b #$9, d4
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(4, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 00153e54: bge.b $153e74
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x153e74_u32
                    } else {
                        0x153e56_u32
                    };
                }
                0x153e56 => {
                    // 00153e56: move.b d4, d5
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(5, 8, value);
                    r.move_flags(value, 8);
                    // 00153e58: extb.l d5
                    e.step()?;
                    let source = r.data(5, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(5, 32, value);
                    r.move_flags(value, 32);
                    // 00153e5a: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 00153e60: move.w (a0, d5.l * 4), d5
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(5, 32)?).wrapping_mul(4));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 00153e64: move.b d4, d6
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(6, 8, value);
                    r.move_flags(value, 8);
                    // 00153e66: extb.l d6
                    e.step()?;
                    let source = r.data(6, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(6, 32, value);
                    r.move_flags(value, 32);
                    // 00153e68: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 00153e6e: move.w $2(a0, d6.l * 4), d6
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x2_u32)
                        .wrapping_add((r.data(6, 32)?).wrapping_mul(4));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 00153e72: bra.b $153e98
                    e.step()?;
                    pc = 0x153e98_u32;
                }
                0x153e74 => {
                    // 00153e74: move.b d4, d5
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(5, 8, value);
                    r.move_flags(value, 8);
                    // 00153e76: ext.w d5
                    e.step()?;
                    let source = r.data(5, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 00153e78: addi.w #$fff6, d5
                    e.step()?;
                    let source = 0xfff6_u32;
                    let destination = r.data(5, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(5, 16, value);
                    // 00153e7c: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 00153e82: move.w (a0, d5.w * 4), d5
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(5, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 00153e86: move.b d4, d6
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(6, 8, value);
                    r.move_flags(value, 8);
                    // 00153e88: ext.w d6
                    e.step()?;
                    let source = r.data(6, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 00153e8a: addi.w #$fff6, d6
                    e.step()?;
                    let source = 0xfff6_u32;
                    let destination = r.data(6, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(6, 16, value);
                    // 00153e8e: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 00153e94: move.w $2(a0, d6.w * 4), d6
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(6, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x153e98_u32;
                }
                0x153e98 => {
                    // 00153e98: move.b $9(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x9_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00153e9c: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00153e9e: move.b d5, d1
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 00153ea0: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00153ea2: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00153ea4: move.w d1, -(a7)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00153ea6: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x153eac_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00153eac: move.b d0, d7
                    e.step()?;
                    let value = r.data(0, 8)?;
                    r.put_data(7, 8, value);
                    r.move_flags(value, 8);
                    // 00153eae: move.b $b(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xb_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00153eb2: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00153eb4: move.b d6, d1
                    e.step()?;
                    let value = r.data(6, 8)?;
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 00153eb6: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00153eb8: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00153eba: move.w d1, -(a7)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00153ebc: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x153ec2_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00153ec2: move.b d0, -$1(a6)
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00153ec6: cmp.b -$1(a6), d7
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let source = u32::from(e.b(source_address)?);
                    let destination = r.data(7, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 00153eca: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00153ecc: ble.w $153f90
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x153f90_u32
                    } else {
                        0x153ed0_u32
                    };
                }
                0x153ed0 => {
                    // 00153ed0: tst.b $442737.l
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00153ed6: beq.b $153ef2
                    e.step()?;
                    pc = if r.equal()? {
                        0x153ef2_u32
                    } else {
                        0x153ed8_u32
                    };
                }
                0x153ed8 => {
                    // 00153ed8: move.w #$2, -(a7)
                    e.step()?;
                    let value = 0x2_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00153edc: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x153ee2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 00153ee2: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00153ee4: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00153ee6: bne.b $153ef2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x153ef2_u32
                    } else {
                        0x153ee8_u32
                    };
                }
                0x153ee8 => {
                    // 00153ee8: clr.b $442737.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00153eee: bra.w $154006
                    e.step()?;
                    pc = 0x154006_u32;
                }
                0x153ef2 => {
                    // 00153ef2: move.b #$ff, $442737.l
                    e.step()?;
                    let value = 0xff_u32;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00153efa: move.w #$3, -(a7)
                    e.step()?;
                    let value = 0x3_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00153efe: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x153f04_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 00153f04: move.w d0, d3
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00153f06: cmpi.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153f0a: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00153f0c: bne.b $153f16
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x153f16_u32
                    } else {
                        0x153f0e_u32
                    };
                }
                0x153f0e => {
                    // 00153f0e: move.b #$1, $442737.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x153f16_u32;
                }
                0x153f16 => {
                    // 00153f16: cmpi.w #$2, d3
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153f1a: bne.b $153f22
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x153f22_u32
                    } else {
                        0x153f1c_u32
                    };
                }
                0x153f1c => {
                    // 00153f1c: clr.b $442737.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x153f22_u32;
                }
                0x153f22 => {
                    // 00153f22: move.w #$3, -(a7)
                    e.step()?;
                    let value = 0x3_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00153f26: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x153f2c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 00153f2c: move.w d0, d3
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00153f2e: cmpi.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153f32: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00153f34: bne.b $153f3e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x153f3e_u32
                    } else {
                        0x153f36_u32
                    };
                }
                0x153f36 => {
                    // 00153f36: move.b #$ff, $442736.l
                    e.step()?;
                    let value = 0xff_u32;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x153f3e_u32;
                }
                0x153f3e => {
                    // 00153f3e: cmpi.w #$2, d3
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153f42: bne.b $153f4c
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x153f4c_u32
                    } else {
                        0x153f44_u32
                    };
                }
                0x153f44 => {
                    // 00153f44: move.b #$1, $442736.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x153f4c_u32;
                }
                0x153f4c => {
                    // 00153f4c: cmpi.w #$3, d3
                    e.step()?;
                    let source = 0x3_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153f50: bne.b $153f64
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x153f64_u32
                    } else {
                        0x153f52_u32
                    };
                }
                0x153f52 => {
                    // 00153f52: tst.b $442737.l
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00153f58: beq.b $153f64
                    e.step()?;
                    pc = if r.equal()? {
                        0x153f64_u32
                    } else {
                        0x153f5a_u32
                    };
                }
                0x153f5a => {
                    // 00153f5a: clr.b $442736.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00153f60: bra.w $154006
                    e.step()?;
                    pc = 0x154006_u32;
                }
                0x153f64 => {
                    // 00153f64: move.b #$1, $442736.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00153f6c: move.w #$2, -(a7)
                    e.step()?;
                    let value = 0x2_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00153f70: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x153f76_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 00153f76: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00153f78: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00153f7a: bne.w $154006
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x154006_u32
                    } else {
                        0x153f7e_u32
                    };
                }
                0x153f7e => {
                    // 00153f7e: move.b $442736.l, d0
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00153f84: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00153f86: neg.w d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = r.subtract_flags(0, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00153f88: move.b d0, $442736.l
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00153f8e: bra.b $154006
                    e.step()?;
                    pc = 0x154006_u32;
                }
                0x153f90 => {
                    // 00153f90: tst.b $442736.l
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00153f96: beq.b $153fa0
                    e.step()?;
                    pc = if r.equal()? {
                        0x153fa0_u32
                    } else {
                        0x153f98_u32
                    };
                }
                0x153f98 => {
                    // 00153f98: clr.b $442736.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00153f9e: bra.b $154006
                    e.step()?;
                    pc = 0x154006_u32;
                }
                0x153fa0 => {
                    // 00153fa0: move.b #$ff, $442736.l
                    e.step()?;
                    let value = 0xff_u32;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00153fa8: move.w #$3, -(a7)
                    e.step()?;
                    let value = 0x3_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00153fac: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x153fb2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 00153fb2: move.w d0, d3
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00153fb4: cmpi.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153fb8: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00153fba: bne.b $153fc4
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x153fc4_u32
                    } else {
                        0x153fbc_u32
                    };
                }
                0x153fbc => {
                    // 00153fbc: move.b #$1, $442736.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x153fc4_u32;
                }
                0x153fc4 => {
                    // 00153fc4: cmpi.w #$2, d3
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153fc8: bne.b $153fd0
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x153fd0_u32
                    } else {
                        0x153fca_u32
                    };
                }
                0x153fca => {
                    // 00153fca: clr.b $442736.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x153fd0_u32;
                }
                0x153fd0 => {
                    // 00153fd0: move.w #$3, -(a7)
                    e.step()?;
                    let value = 0x3_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00153fd4: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x153fda_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 00153fda: move.w d0, d3
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00153fdc: cmpi.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153fe0: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00153fe2: bne.b $153fec
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x153fec_u32
                    } else {
                        0x153fe4_u32
                    };
                }
                0x153fe4 => {
                    // 00153fe4: move.b #$ff, $442737.l
                    e.step()?;
                    let value = 0xff_u32;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x153fec_u32;
                }
                0x153fec => {
                    // 00153fec: cmpi.w #$2, d3
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153ff0: bne.b $153ffa
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x153ffa_u32
                    } else {
                        0x153ff2_u32
                    };
                }
                0x153ff2 => {
                    // 00153ff2: move.b #$1, $442737.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x153ffa_u32;
                }
                0x153ffa => {
                    // 00153ffa: cmpi.w #$3, d3
                    e.step()?;
                    let source = 0x3_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00153ffe: bne.b $154006
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x154006_u32
                    } else {
                        0x154000_u32
                    };
                }
                0x154000 => {
                    // 00154000: clr.b $442737.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x154006_u32;
                }
                0x154006 => {
                    // 00154006: movem.l (a7)+, d3-d7
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    // 0015400a: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0015400c: rts
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
crate::recovered_application_control::entry!(shift, 0x153e44);

pub fn save_against_attribute_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x157c68_u32;
        loop {
            match pc {
                0x157c68 => {
                    // 00157c68: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 00157c6c: movem.l d3-d4, -(a7)
                    e.step()?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 00157c70: move.w $a(a6), d4
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 00157c74: move.w $8(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00157c78: cmpi.w #$5, d0
                    e.step()?;
                    let source = 0x5_u32;
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00157c7c: bhi.w $157d0a
                    e.step()?;
                    pc = if r.higher()? {
                        0x157d0a_u32
                    } else {
                        0x157c80_u32
                    };
                }
                0x157c80 => {
                    // 00157c80: move.w $157c88(pc, d0.w * 2), d0
                    e.step()?;
                    let selector = r.data(0, 16)? as usize;
                    let offsets: [u16; 6] = [0x000c, 0x0020, 0x0034, 0x0048, 0x005c, 0x0070];
                    let value = u32::from(*offsets.get(selector).ok_or_else(|| {
                        RuleError::Invalid("source event selector outside recovered table".into())
                    })?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00157c84: jmp $157c88(pc, d0.w)
                    e.step()?;
                    pc = 0x157c88_u32.wrapping_add(i32::from(r.data(0, 16)? as u16 as i16) as u32);
                }
                0x157c94 => {
                    // 00157c94: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00157c96: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00157c9a: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00157ca0: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00157ca2: move.w $9e2(a0), d3
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9e2_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00157ca6: bra.b $157d0a
                    e.step()?;
                    pc = 0x157d0a_u32;
                }
                0x157ca8 => {
                    // 00157ca8: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00157caa: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00157cae: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00157cb4: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00157cb6: move.w $9e4(a0), d3
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9e4_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00157cba: bra.b $157d0a
                    e.step()?;
                    pc = 0x157d0a_u32;
                }
                0x157cbc => {
                    // 00157cbc: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00157cbe: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00157cc2: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00157cc8: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00157cca: move.w $9e6(a0), d3
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9e6_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00157cce: bra.b $157d0a
                    e.step()?;
                    pc = 0x157d0a_u32;
                }
                0x157cd0 => {
                    // 00157cd0: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00157cd2: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00157cd6: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00157cdc: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00157cde: move.w $9e8(a0), d3
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9e8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00157ce2: bra.b $157d0a
                    e.step()?;
                    pc = 0x157d0a_u32;
                }
                0x157ce4 => {
                    // 00157ce4: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00157ce6: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00157cea: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00157cf0: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00157cf2: move.w $9ec(a0), d3
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9ec_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00157cf6: bra.b $157d0a
                    e.step()?;
                    pc = 0x157d0a_u32;
                }
                0x157cf8 => {
                    // 00157cf8: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00157cfa: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00157cfe: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00157d04: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00157d06: move.w $9ee(a0), d3
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9ee_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x157d0a_u32;
                }
                0x157d0a => {
                    // 00157d0a: lsl.w #$2, d3
                    e.step()?;
                    let count = 0x2_u32;
                    let source = r.data(3, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(3, 16, value);
                    // 00157d0c: move.w #$64, -(a7)
                    e.step()?;
                    let value = 0x64_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00157d10: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x157d16_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 00157d16: cmp.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00157d18: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00157d1a: bge.b $157d20
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x157d20_u32
                    } else {
                        0x157d1c_u32
                    };
                }
                0x157d1c => {
                    // 00157d1c: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 00157d1e: bra.b $157d22
                    e.step()?;
                    pc = 0x157d22_u32;
                }
                0x157d20 => {
                    // 00157d20: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x157d22_u32;
                }
                0x157d22 => {
                    // 00157d22: movem.l (a7)+, d3-d4
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    // 00157d26: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00157d28: rts
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
crate::recovered_application_control::entry!(save_against_attribute, 0x157c68);

pub fn spelleffectoutdoors_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x15b84c_u32;
        loop {
            match pc {
                0x15b84c => {
                    // 0015b84c: link.w a6, #$fff8
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -8)?;
                    // 0015b850: movem.l d3-d7, -(a7)
                    e.step()?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 0015b854: move.w $c(a6), d7
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(7, 16, value);
                    r.move_flags(value, 16);
                    // 0015b858: move.w $e(a6), d6
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xe_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 0015b85c: move.w $10(a6), d4
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x10_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 0015b860: clr.w -$6(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b864: clr.w -$8(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b868: move.w #$40, -$2(a6)
                    e.step()?;
                    let value = 0x40_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b86e: move.w #$40, -$4(a6)
                    e.step()?;
                    let value = 0x40_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b874: move.w $8(a6), d5
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 0015b878: lsl.w #$3, d5
                    e.step()?;
                    let count = 0x3_u32;
                    let source = r.data(5, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(5, 16, value);
                    // 0015b87a: addi.w #$2ee0, d5
                    e.step()?;
                    let source = 0x2ee0_u32;
                    let destination = r.data(5, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(5, 16, value);
                    // 0015b87e: movea.w $a(a6), a0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015b882: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015b884: sub.l $442778.l, d0
                    e.step()?;
                    let source_address = 0x442778_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015b88a: move.w d0, $a(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b88e: movea.w d7, a0
                    e.step()?;
                    let value = r.data(7, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015b890: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015b892: sub.l $44277c.l, d0
                    e.step()?;
                    let source_address = 0x44277c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015b898: move.w d0, d7
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(7, 16, value);
                    r.move_flags(value, 16);
                    // 0015b89a: move.w d7, d0
                    e.step()?;
                    let value = r.data(7, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015b89c: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 0015b89e: move.w d0, $44371a.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x44371a_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b8a4: move.w $a(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015b8a8: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 0015b8aa: move.w d0, $44371c.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x44371c_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b8b0: move.w $44371a.l, d0
                    e.step()?;
                    let value_address = 0x44371a_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015b8b6: add.w d6, d0
                    e.step()?;
                    let source = r.data(6, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b8b8: move.w d0, $44371e.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x44371e_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b8be: move.w $44371c.l, d0
                    e.step()?;
                    let value_address = 0x44371c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015b8c4: add.w d6, d0
                    e.step()?;
                    let source = r.data(6, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b8c6: move.w d0, $443720.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x443720_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b8cc: pea.l $44371a.l
                    e.step()?;
                    let value = 0x44371a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b8d2: move.w d6, d0
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015b8d4: lsr.w #$8, d0
                    e.step()?;
                    let count = 0x8_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(0, 16, value);
                    // 0015b8d6: lsr.w #$7, d0
                    e.step()?;
                    let count = 0x7_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(0, 16, value);
                    // 0015b8d8: add.w d6, d0
                    e.step()?;
                    let source = r.data(6, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b8da: asr.w #$1, d0
                    e.step()?;
                    let count = 0x1_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b8dc: neg.w d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = r.subtract_flags(0, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b8de: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b8e0: move.w d6, d0
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015b8e2: lsr.w #$8, d0
                    e.step()?;
                    let count = 0x8_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(0, 16, value);
                    // 0015b8e4: lsr.w #$7, d0
                    e.step()?;
                    let count = 0x7_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(0, 16, value);
                    // 0015b8e6: add.w d6, d0
                    e.step()?;
                    let source = r.data(6, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b8e8: asr.w #$1, d0
                    e.step()?;
                    let count = 0x1_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b8ea: neg.w d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = r.subtract_flags(0, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b8ec: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b8ee: MacToolbox a8a8
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8a8_u32 as u16))?;
                    // 0015b8f0: move.l $43879c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43879c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015b8f6: MacToolbox a873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 0015b8f8: move.w $44371e.l, d0
                    e.step()?;
                    let value_address = 0x44371e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015b8fe: sub.w $44371a.l, d0
                    e.step()?;
                    let source_address = 0x44371a_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b904: move.w d0, -$4(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b908: move.w $443720.l, d0
                    e.step()?;
                    let value_address = 0x443720_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015b90e: sub.w $44371c.l, d0
                    e.step()?;
                    let source_address = 0x44371c_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b914: move.w d0, -$2(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b918: movea.l $43879c.l, a0
                    e.step()?;
                    let value_address = 0x43879c_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0015b91e: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b922: movea.l $4386b4.l, a0
                    e.step()?;
                    let value_address = 0x4386b4_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0015b928: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b92c: pea.l $44371a.l
                    e.step()?;
                    let value = 0x44371a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b932: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b936: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b938: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015b93a: MacToolbox a8ec
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
                    // 0015b93c: move.w d5, d3
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 0015b93e: bra.b $15b98c
                    e.step()?;
                    pc = 0x15b98c_u32;
                }
                0x15b940 => {
                    // 0015b940: movea.l $4386b4.l, a0
                    e.step()?;
                    let value_address = 0x4386b4_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0015b946: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b94a: movea.l $43879c.l, a0
                    e.step()?;
                    let value_address = 0x43879c_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0015b950: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b954: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b958: pea.l $44371a.l
                    e.step()?;
                    let value = 0x44371a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b95e: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b960: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015b962: MacToolbox a8ec
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
                    // 0015b964: move.l $44371e.l, -(a7)
                    e.step()?;
                    let value_address = 0x44371e_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015b96a: move.l $44371a.l, -(a7)
                    e.step()?;
                    let value_address = 0x44371a_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015b970: move.w d3, -(a7)
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b972: bsr.l $14a02c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15b978_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14a02c_u32)?;
                    // 0015b978: tst.w d4
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.move_flags(value, 16);
                    // 0015b97a: lea.l $a(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xa_u32);
                    r.put_address(7, value);
                    // 0015b97e: beq.b $15b98a
                    e.step()?;
                    pc = if r.equal()? {
                        0x15b98a_u32
                    } else {
                        0x15b980_u32
                    };
                }
                0x15b980 => {
                    // 0015b980: move.w d4, -(a7)
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b982: bsr.l $1205f8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15b988_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1205f8_u32)?;
                    // 0015b988: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x15b98a_u32;
                }
                0x15b98a => {
                    // 0015b98a: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x15b98c_u32;
                }
                0x15b98c => {
                    // 0015b98c: move.w d5, d0
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015b98e: addq.w #$8, d0
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015b990: cmp.w d0, d3
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015b992: blt.b $15b940
                    e.step()?;
                    pc = if r.less()? {
                        0x15b940_u32
                    } else {
                        0x15b994_u32
                    };
                }
                0x15b994 => {
                    // 0015b994: movea.l $4386b4.l, a0
                    e.step()?;
                    let value_address = 0x4386b4_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0015b99a: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b99e: movea.l $43879c.l, a0
                    e.step()?;
                    let value_address = 0x43879c_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0015b9a4: pea.l $2(a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b9a8: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b9ac: pea.l $44371a.l
                    e.step()?;
                    let value = 0x44371a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015b9b2: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015b9b4: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015b9b6: MacToolbox a8ec
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
                    // 0015b9b8: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015b9ba: movem.l (a7)+, d3-d7
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    // 0015b9be: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0015b9c0: rts
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
crate::recovered_application_control::entry!(spelleffectoutdoors, 0x15b84c);

pub fn takegold_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x15d210_u32;
        loop {
            match pc {
                0x15d210 => {
                    // 0015d210: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 0015d214: movem.l d3-d7, -(a7)
                    e.step()?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 0015d218: move.w $8(a6), d7
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(7, 16, value);
                    r.move_flags(value, 16);
                    // 0015d21c: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015d21e: move.w d7, d6
                    e.step()?;
                    let value = r.data(7, 16)?;
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 0015d220: movea.w d7, a0
                    e.step()?;
                    let value = r.data(7, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015d222: cmpa.l $442788.l, a0
                    e.step()?;
                    let source_address = 0x442788_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0015d228: bge.b $15d23a
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15d23a_u32
                    } else {
                        0x15d22a_u32
                    };
                }
                0x15d22a => {
                    // 0015d22a: movea.w d7, a0
                    e.step()?;
                    let value = r.data(7, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015d22c: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015d22e: sub.l d0, $442788.l
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination_address = 0x442788_u32;
                    let destination = e.l(destination_address)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.write_memory(e, destination_address, 32, value)?;
                    // 0015d234: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 0015d236: bra.w $15d2dc
                    e.step()?;
                    pc = 0x15d2dc_u32;
                }
                0x15d23a => {
                    // 0015d23a: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015d23c: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015d23e: bra.b $15d266
                    e.step()?;
                    pc = 0x15d266_u32;
                }
                0x15d240 => {
                    // 0015d240: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015d242: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015d246: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015d24c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015d24e: add.w $9da(a0), d5
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x9da_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(5, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(5, 16, value);
                    // 0015d252: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015d254: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015d258: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015d25e: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015d260: add.w $9da(a0), d5
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x9da_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(5, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(5, 16, value);
                    // 0015d264: addq.w #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    pc = 0x15d266_u32;
                }
                0x15d266 => {
                    // 0015d266: move.b $44273a.l, d0
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015d26c: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015d26e: cmp.w d0, d4
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(4, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015d270: ble.b $15d240
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15d240_u32
                    } else {
                        0x15d272_u32
                    };
                }
                0x15d272 => {
                    // 0015d272: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015d274: move.w d5, d0
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015d276: add.l $442788.l, d0
                    e.step()?;
                    let source_address = 0x442788_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015d27c: movea.w d7, a0
                    e.step()?;
                    let value = r.data(7, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015d27e: cmpa.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0015d280: ble.b $15d292
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15d292_u32
                    } else {
                        0x15d282_u32
                    };
                }
                0x15d282 => {
                    // 0015d282: move.w #$32, -(a7)
                    e.step()?;
                    let value = 0x32_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015d286: bsr.l $16474a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15d28c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16474a_u32)?;
                    // 0015d28c: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015d28e: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015d290: bra.b $15d2dc
                    e.step()?;
                    pc = 0x15d2dc_u32;
                }
                0x15d292 => {
                    // 0015d292: sub.w $44278a.l, d6
                    e.step()?;
                    let source_address = 0x44278a_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(6, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(6, 16, value);
                    // 0015d298: clr.l $442788.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442788_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    pc = 0x15d29e_u32;
                }
                0x15d29e => {
                    // 0015d29e: move.b d3, d0
                    e.step()?;
                    let value = r.data(3, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015d2a0: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015d2a2: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015d2a6: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015d2ac: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015d2ae: tst.w $9da(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9da_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015d2b2: beq.b $15d2ca
                    e.step()?;
                    pc = if r.equal()? {
                        0x15d2ca_u32
                    } else {
                        0x15d2b4_u32
                    };
                }
                0x15d2b4 => {
                    // 0015d2b4: subq.w #$1, d6
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(6, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(6, 16, value);
                    // 0015d2b6: move.b d3, d0
                    e.step()?;
                    let value = r.data(3, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015d2b8: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015d2ba: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015d2be: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015d2c4: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015d2c6: subq.w #$1, $9da(a0)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x9da_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    pc = 0x15d2ca_u32;
                }
                0x15d2ca => {
                    // 0015d2ca: addq.b #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 8)?;
                    let value = r.add_flags(destination, source, 8, true);
                    r.put_data(3, 8, value);
                    // 0015d2cc: cmp.b $44273a.l, d3
                    e.step()?;
                    let source_address = 0x44273a_u32;
                    let source = u32::from(e.b(source_address)?);
                    let destination = r.data(3, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 0015d2d2: ble.b $15d2d6
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15d2d6_u32
                    } else {
                        0x15d2d4_u32
                    };
                }
                0x15d2d4 => {
                    // 0015d2d4: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x15d2d6_u32;
                }
                0x15d2d6 => {
                    // 0015d2d6: tst.w d6
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.move_flags(value, 16);
                    // 0015d2d8: bne.b $15d29e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15d29e_u32
                    } else {
                        0x15d2da_u32
                    };
                }
                0x15d2da => {
                    // 0015d2da: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x15d2dc_u32;
                }
                0x15d2dc => {
                    // 0015d2dc: movem.l (a7)+, d3-d7
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    // 0015d2e0: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0015d2e2: rts
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
crate::recovered_application_control::entry!(takegold, 0x15d210);

pub fn finddxdy_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x15edfa_u32;
        loop {
            match pc {
                0x15edfa => {
                    // 0015edfa: link.w a6, #$ffe2
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -30)?;
                    // 0015edfe: movem.l d3-d7, -(a7)
                    e.step()?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 0015ee02: move.b $9(a6), d4
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x9_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(4, 8, value);
                    r.move_flags(value, 8);
                    // 0015ee06: move.b $b(a6), d3
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xb_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(3, 8, value);
                    r.move_flags(value, 8);
                    // 0015ee0a: clr.l -$6(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ee0e: clr.l -$a(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ee12: cmpi.b #$9, d4
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(4, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 0015ee16: ble.b $15ee3e
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15ee3e_u32
                    } else {
                        0x15ee18_u32
                    };
                }
                0x15ee18 => {
                    // 0015ee18: subi.b #$a, d4
                    e.step()?;
                    let source = 0xa_u32;
                    let destination = r.data(4, 8)?;
                    let value = r.subtract_flags(destination, source, 8, true);
                    r.put_data(4, 8, value);
                    // 0015ee1c: move.b d4, d7
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(7, 8, value);
                    r.move_flags(value, 8);
                    // 0015ee1e: extb.l d7
                    e.step()?;
                    let source = r.data(7, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(7, 32, value);
                    r.move_flags(value, 32);
                    // 0015ee20: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0015ee26: move.w (a0, d7.l * 4), d7
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(7, 32)?).wrapping_mul(4));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(7, 16, value);
                    r.move_flags(value, 16);
                    // 0015ee2a: move.b d4, d0
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015ee2c: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015ee2e: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0015ee34: move.w $2(a0, d0.l * 4), d0
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x2_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015ee38: move.b d0, -$1(a6)
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015ee3c: bra.b $15ee5e
                    e.step()?;
                    pc = 0x15ee5e_u32;
                }
                0x15ee3e => {
                    // 0015ee3e: move.b d4, d7
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(7, 8, value);
                    r.move_flags(value, 8);
                    // 0015ee40: extb.l d7
                    e.step()?;
                    let source = r.data(7, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(7, 32, value);
                    r.move_flags(value, 32);
                    // 0015ee42: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0015ee48: move.w (a0, d7.l * 4), d7
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(7, 32)?).wrapping_mul(4));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(7, 16, value);
                    r.move_flags(value, 16);
                    // 0015ee4c: move.b d4, d0
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015ee4e: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015ee50: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0015ee56: move.w $2(a0, d0.l * 4), d0
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x2_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015ee5a: move.b d0, -$1(a6)
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x15ee5e_u32;
                }
                0x15ee5e => {
                    // 0015ee5e: cmpi.b #$9, d3
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(3, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 0015ee62: bge.b $15ee82
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15ee82_u32
                    } else {
                        0x15ee64_u32
                    };
                }
                0x15ee64 => {
                    // 0015ee64: move.b d3, d5
                    e.step()?;
                    let value = r.data(3, 8)?;
                    r.put_data(5, 8, value);
                    r.move_flags(value, 8);
                    // 0015ee66: extb.l d5
                    e.step()?;
                    let source = r.data(5, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(5, 32, value);
                    r.move_flags(value, 32);
                    // 0015ee68: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0015ee6e: move.w (a0, d5.l * 4), d5
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(5, 32)?).wrapping_mul(4));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 0015ee72: move.b d3, d6
                    e.step()?;
                    let value = r.data(3, 8)?;
                    r.put_data(6, 8, value);
                    r.move_flags(value, 8);
                    // 0015ee74: extb.l d6
                    e.step()?;
                    let source = r.data(6, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(6, 32, value);
                    r.move_flags(value, 32);
                    // 0015ee76: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0015ee7c: move.w $2(a0, d6.l * 4), d6
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x2_u32)
                        .wrapping_add((r.data(6, 32)?).wrapping_mul(4));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 0015ee80: bra.b $15eea2
                    e.step()?;
                    pc = 0x15eea2_u32;
                }
                0x15ee82 => {
                    // 0015ee82: subi.b #$a, d3
                    e.step()?;
                    let source = 0xa_u32;
                    let destination = r.data(3, 8)?;
                    let value = r.subtract_flags(destination, source, 8, true);
                    r.put_data(3, 8, value);
                    // 0015ee86: move.b d3, d5
                    e.step()?;
                    let value = r.data(3, 8)?;
                    r.put_data(5, 8, value);
                    r.move_flags(value, 8);
                    // 0015ee88: extb.l d5
                    e.step()?;
                    let source = r.data(5, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(5, 32, value);
                    r.move_flags(value, 32);
                    // 0015ee8a: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0015ee90: move.w (a0, d5.l * 4), d5
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(5, 32)?).wrapping_mul(4));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 0015ee94: move.b d3, d6
                    e.step()?;
                    let value = r.data(3, 8)?;
                    r.put_data(6, 8, value);
                    r.move_flags(value, 8);
                    // 0015ee96: extb.l d6
                    e.step()?;
                    let source = r.data(6, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(6, 32, value);
                    r.move_flags(value, 32);
                    // 0015ee98: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 0015ee9e: move.w $2(a0, d6.l * 4), d6
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x2_u32)
                        .wrapping_add((r.data(6, 32)?).wrapping_mul(4));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x15eea2_u32;
                }
                0x15eea2 => {
                    // 0015eea2: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015eea4: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015eea6: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015eea8: move.b d5, d1
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0015eeaa: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015eeac: cmp.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015eeae: bge.b $15eeb8
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15eeb8_u32
                    } else {
                        0x15eeb0_u32
                    };
                }
                0x15eeb0 => {
                    // 0015eeb0: move.l #$bf800000, -$6(a6)
                    e.step()?;
                    let value = 0xbf800000_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    pc = 0x15eeb8_u32;
                }
                0x15eeb8 => {
                    // 0015eeb8: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015eeba: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015eebc: addq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015eebe: move.b d5, d1
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0015eec0: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015eec2: cmp.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015eec4: ble.b $15eece
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15eece_u32
                    } else {
                        0x15eec6_u32
                    };
                }
                0x15eec6 => {
                    // 0015eec6: move.l #$3f800000, -$6(a6)
                    e.step()?;
                    let value = 0x3f800000_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    pc = 0x15eece_u32;
                }
                0x15eece => {
                    // 0015eece: move.b -$1(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015eed2: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015eed4: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015eed6: move.b d6, d1
                    e.step()?;
                    let value = r.data(6, 8)?;
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0015eed8: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015eeda: cmp.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015eedc: bge.b $15eee6
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15eee6_u32
                    } else {
                        0x15eede_u32
                    };
                }
                0x15eede => {
                    // 0015eede: move.l #$bf800000, -$a(a6)
                    e.step()?;
                    let value = 0xbf800000_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    pc = 0x15eee6_u32;
                }
                0x15eee6 => {
                    // 0015eee6: move.b -$1(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015eeea: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015eeec: addq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015eeee: move.b d6, d1
                    e.step()?;
                    let value = r.data(6, 8)?;
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0015eef0: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015eef2: cmp.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015eef4: ble.b $15eefe
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15eefe_u32
                    } else {
                        0x15eef6_u32
                    };
                }
                0x15eef6 => {
                    // 0015eef6: move.l #$3f800000, -$a(a6)
                    e.step()?;
                    let value = 0x3f800000_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    pc = 0x15eefe_u32;
                }
                0x15eefe => {
                    // 0015eefe: pea.l -$a(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015ef02: pea.l -$14(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffec_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015ef06: move.w #$100e, -(a7)
                    e.step()?;
                    let value = 0x100e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ef0a: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 0015ef0c: move.l -$e(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ef10: move.l -$12(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffee_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ef14: move.w -$14(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffec_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ef18: pea.l -$6(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015ef1c: pea.l -$1e(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe2_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015ef20: move.w #$100e, -(a7)
                    e.step()?;
                    let value = 0x100e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ef24: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 0015ef26: move.l -$18(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ef2a: move.l -$1c(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffe4_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ef2e: move.w -$1e(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffe2_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ef32: bsr.l $15f048
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ef38_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15f048_u32)?;
                    // 0015ef38: lea.l $14(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x14_u32);
                    r.put_address(7, value);
                    // 0015ef3c: movem.l (a7)+, d3-d7
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    // 0015ef40: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0015ef42: rts
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
crate::recovered_application_control::entry!(finddxdy, 0x15edfa);

pub fn to_weapon_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x15ef50_u32;
        loop {
            match pc {
                0x15ef50 => {
                    // 0015ef50: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 0015ef54: move.l d3, -(a7)
                    e.step()?;
                    let value = r.data(3, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ef56: move.w $8(a6), d3
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 0015ef5a: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015ef5c: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015ef60: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015ef66: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015ef68: clr.b $9dd(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x9dd_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015ef6c: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015ef6e: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015ef72: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015ef78: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015ef7a: tst.w $ca(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xca_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015ef7e: beq.b $15efb4
                    e.step()?;
                    pc = if r.equal()? {
                        0x15efb4_u32
                    } else {
                        0x15ef80_u32
                    };
                }
                0x15ef80 => {
                    // 0015ef80: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015ef82: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015ef86: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015ef8c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015ef8e: move.w $ca(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xca_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ef92: bsr.l $130afe
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ef98_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x130afe_u32)?;
                    // 0015ef98: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015ef9a: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015ef9e: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015efa4: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015efa6: move.w $45364a.l, $128(a0)
                    e.step()?;
                    let value_address = 0x45364a_u32;
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(0)?.wrapping_add(0x128_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015efae: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 0015efb0: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015efb2: bra.b $15efde
                    e.step()?;
                    pc = 0x15efde_u32;
                }
                0x15efb4 => {
                    // 0015efb4: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015efb6: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015efba: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015efc0: move.w $38(a0, d0.l), d0
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x38_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015efc4: lsl.w #$3, d0
                    e.step()?;
                    let count = 0x3_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 0015efc6: addi.w #$1e, d0
                    e.step()?;
                    let source = 0x1e_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015efca: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015efcc: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015efd0: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015efd6: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015efd8: move.w d0, $128(a0)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x128_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015efdc: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x15efde_u32;
                }
                0x15efde => {
                    // 0015efde: move.l (a7)+, d3
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_data(3, 32, value);
                    r.move_flags(value, 32);
                    // 0015efe0: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0015efe2: rts
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
crate::recovered_application_control::entry!(to_weapon, 0x15ef50);

pub fn to_missile_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x15efee_u32;
        loop {
            match pc {
                0x15efee => {
                    // 0015efee: link.w a6, #$0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, 0)?;
                    // 0015eff2: move.b $9(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x9_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015eff6: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015eff8: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015effc: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015f002: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015f004: tst.w $e4(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xe4_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015f008: bne.b $15f00e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15f00e_u32
                    } else {
                        0x15f00a_u32
                    };
                }
                0x15f00a => {
                    // 0015f00a: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015f00c: bra.b $15f038
                    e.step()?;
                    pc = 0x15f038_u32;
                }
                0x15f00e => {
                    // 0015f00e: move.b $9(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x9_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015f012: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015f014: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015f018: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015f01e: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015f020: move.w $e4(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xe4_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f024: bsr.l $130afe
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15f02a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x130afe_u32)?;
                    // 0015f02a: move.w $453668.l, -(a7)
                    e.step()?;
                    let value_address = 0x453668_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f030: bsr.l $13145c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15f036_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13145c_u32)?;
                    // 0015f036: moveq #$1, d0
                    e.step()?;
                    r.put_data(0, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x15f038_u32;
                }
                0x15f038 => {
                    // 0015f038: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0015f03a: rts
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
crate::recovered_application_control::entry!(to_missile, 0x15efee);

pub fn missileset_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x15f048_u32;
        loop {
            match pc {
                0x15f048 => {
                    // 0015f048: link.w a6, #$ffd8
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -40)?;
                    // 0015f04c: pea.l $8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0x8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f050: pea.l -$28(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffd8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f054: move.w #$1010, -(a7)
                    e.step()?;
                    let value = 0x1010_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f058: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 0015f05a: move.l -$28(a6), -$c(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffd8_u32);
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f060: pea.l $12(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0x12_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f064: pea.l -$28(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffd8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f068: move.w #$1010, -(a7)
                    e.step()?;
                    let value = 0x1010_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f06c: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 0015f06e: move.l -$28(a6), -$10(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffd8_u32);
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f074: move.l $43878c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43878c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f07a: MacToolbox a873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 0015f07c: move.w #$140, -$8(a6)
                    e.step()?;
                    let value = 0x140_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f082: move.w #$20, -$2(a6)
                    e.step()?;
                    let value = 0x20_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f088: clr.w -$6(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f08c: move.w #$160, -$4(a6)
                    e.step()?;
                    let value = 0x160_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f092: cmpi.w #$10, $453788.l
                    e.step()?;
                    let source = 0x10_u32;
                    let destination_address = 0x453788_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015f09a: beq.b $15f0a8
                    e.step()?;
                    pc = if r.equal()? {
                        0x15f0a8_u32
                    } else {
                        0x15f09c_u32
                    };
                }
                0x15f09c => {
                    // 0015f09c: cmpi.w #$12, $453788.l
                    e.step()?;
                    let source = 0x12_u32;
                    let destination_address = 0x453788_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015f0a4: bne.w $15f184
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15f184_u32
                    } else {
                        0x15f0a8_u32
                    };
                }
                0x15f0a8 => {
                    // 0015f0a8: clr.l $439598.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x439598_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f0ae: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0015f0b4: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015f0ba: move.w $44719e.l, d1
                    e.step()?;
                    let value_address = 0x44719e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015f0c0: sub.w (a0, d0.w * 4), d1
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015f0c4: move.w d1, -$24(a6)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f0c8: pea.l -$24(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f0cc: pea.l -$24(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f0d0: move.w #$200e, -(a7)
                    e.step()?;
                    let value = 0x200e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f0d4: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 0015f0d6: pea.l -$24(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f0da: pea.l -$24(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f0de: move.w #$1010, -(a7)
                    e.step()?;
                    let value = 0x1010_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f0e2: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 0015f0e4: move.l -$24(a6), -$c(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f0ea: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 0015f0f0: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015f0f6: move.w $4471a0.l, d1
                    e.step()?;
                    let value_address = 0x4471a0_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015f0fc: sub.w $2(a0, d0.w * 4), d1
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015f100: move.w d1, -$24(a6)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f104: pea.l -$24(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f108: pea.l -$24(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f10c: move.w #$200e, -(a7)
                    e.step()?;
                    let value = 0x200e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f110: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 0015f112: pea.l -$24(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f116: pea.l -$24(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f11a: move.w #$1010, -(a7)
                    e.step()?;
                    let value = 0x1010_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f11e: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 0015f120: move.l -$24(a6), -$10(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    let value = e.l(value_address)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f126: subq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 0015f128: pea.l -$10(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f12c: pea.l -$1a(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe6_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f130: move.w #$100e, -(a7)
                    e.step()?;
                    let value = 0x100e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f134: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 0015f136: move.l -$14(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffec_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f13a: move.l -$18(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f13e: move.w -$1a(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffe6_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f142: pea.l -$c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f146: pea.l -$24(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f14a: move.w #$100e, -(a7)
                    e.step()?;
                    let value = 0x100e_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f14e: MacToolbox a9eb
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
                    // 0015f150: move.l -$1e(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffe2_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f154: move.l -$22(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffde_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f158: move.w -$24(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f15c: bsr.l $151f30
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15f162_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151f30_u32)?;
                    // 0015f162: move.b $44375f.l, d1
                    e.step()?;
                    let value_address = 0x44375f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0015f168: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015f16a: lsl.w #$3, d1
                    e.step()?;
                    let count = 0x3_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(1, 16, value);
                    // 0015f16c: addi.w #$2ed8, d1
                    e.step()?;
                    let source = 0x2ed8_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015f170: add.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015f172: lea.l $14(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x14_u32);
                    r.put_address(7, value);
                    // 0015f176: move.w d1, -(a7)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015f178: MacToolbox aa1e
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa1e_u32 as u16))?;
                    // 0015f17a: movea.l (a7)+, a0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_address(0, value);
                    // 0015f17c: move.l a0, $439598.l
                    e.step()?;
                    let value = r.address(0)?;
                    let destination_address = 0x439598_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f182: bra.b $15f188
                    e.step()?;
                    pc = 0x15f188_u32;
                }
                0x15f184 => {
                    // 0015f184: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015f186: bra.b $15f1fe
                    e.step()?;
                    pc = 0x15f1fe_u32;
                }
                0x15f188 => {
                    // 0015f188: tst.l $439598.l
                    e.step()?;
                    let value_address = 0x439598_u32;
                    let value = e.l(value_address)?;
                    r.move_flags(value, 32);
                    // 0015f18e: beq.b $15f1fc
                    e.step()?;
                    pc = if r.equal()? {
                        0x15f1fc_u32
                    } else {
                        0x15f190_u32
                    };
                }
                0x15f190 => {
                    // 0015f190: pea.l $21.w
                    e.step()?;
                    let value = 0x21_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f194: MacToolbox a862
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa862_u32 as u16))?;
                    // 0015f196: pea.l $1e.w
                    e.step()?;
                    let value = 0x1e_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f19a: MacToolbox a863
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa863_u32 as u16))?;
                    // 0015f19c: pea.l $4436f2.l
                    e.step()?;
                    let value = 0x4436f2_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f1a2: move.l $439598.l, -(a7)
                    e.step()?;
                    let value_address = 0x439598_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f1a8: MacToolbox aa1f
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa1f_u32 as u16))?;
                    // 0015f1aa: pea.l $4046da.l
                    e.step()?;
                    let value = 0x4046da_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f1b0: pea.l $4046de.l
                    e.step()?;
                    let value = 0x4046de_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f1b6: move.l #$80005, d0
                    e.step()?;
                    let value = 0x80005_u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015f1bc: MacToolbox ab1d
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xab1d_u32 as u16))?;
                    // 0015f1be: move.l $4386c8.l, -(a7)
                    e.step()?;
                    let value_address = 0x4386c8_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f1c4: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f1c6: move.l #$80006, d0
                    e.step()?;
                    let value = 0x80006_u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015f1cc: MacToolbox ab1d
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xab1d_u32 as u16))?;
                    // 0015f1ce: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f1d2: MacToolbox a8a3
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8a3_u32 as u16))?;
                    // 0015f1d4: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015f1d8: move.l $439598.l, -(a7)
                    e.step()?;
                    let value_address = 0x439598_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f1de: MacToolbox aa1f
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa1f_u32 as u16))?;
                    // 0015f1e0: move.l $439598.l, -(a7)
                    e.step()?;
                    let value_address = 0x439598_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f1e6: MacToolbox aa25
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa25_u32 as u16))?;
                    // 0015f1e8: move.l $4046da.l, -(a7)
                    e.step()?;
                    let value_address = 0x4046da_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f1ee: move.l $4046de.l, -(a7)
                    e.step()?;
                    let value_address = 0x4046de_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015f1f4: move.l #$80006, d0
                    e.step()?;
                    let value = 0x80006_u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015f1fa: MacToolbox ab1d
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xab1d_u32 as u16))?;
                    pc = 0x15f1fc_u32;
                }
                0x15f1fc => {
                    // 0015f1fc: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x15f1fe_u32;
                }
                0x15f1fe => {
                    // 0015f1fe: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0015f200: rts
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
crate::recovered_application_control::entry!(missileset, 0x15f048);

pub fn updatelight_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x160fb2_u32;
        loop {
            match pc {
                0x160fb2 => {
                    // 00160fb2: link.w a6, #$fff8
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -8)?;
                    // 00160fb6: move.l d3, -(a7)
                    e.step()?;
                    let value = r.data(3, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00160fb8: move.b $9(a6), d3
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x9_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(3, 8, value);
                    r.move_flags(value, 8);
                    // 00160fbc: move.l $43878c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43878c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00160fc2: MacToolbox a873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 00160fc4: move.b d3, d0
                    e.step()?;
                    let value = r.data(3, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00160fc6: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00160fc8: muls.w #$32, d0
                    e.step()?;
                    let source = 0x32_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00160fcc: addi.w #$9, d0
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00160fd0: move.w d0, -$8(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00160fd4: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00160fda: addi.w #$183, d0
                    e.step()?;
                    let source = 0x183_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00160fde: move.w d0, -$6(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00160fe2: moveq #$9, d0
                    e.step()?;
                    r.put_data(0, 32, 0x9_u32);
                    r.move_flags(0x9_u32, 32);
                    // 00160fe4: add.w -$6(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00160fe8: move.w d0, -$2(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00160fec: moveq #$9, d0
                    e.step()?;
                    r.put_data(0, 32, 0x9_u32);
                    r.move_flags(0x9_u32, 32);
                    // 00160fee: add.w -$8(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00160ff2: move.w d0, -$4(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00160ff6: tst.b $b(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xb_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00160ffa: beq.b $16100a
                    e.step()?;
                    pc = if r.equal()? {
                        0x16100a_u32
                    } else {
                        0x160ffc_u32
                    };
                }
                0x160ffc => {
                    // 00160ffc: move.l $439554.l, -(a7)
                    e.step()?;
                    let value_address = 0x439554_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00161002: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00161006: MacToolbox a8f6
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8f6_u32 as u16))?;
                    // 00161008: bra.b $161054
                    e.step()?;
                    pc = 0x161054_u32;
                }
                0x16100a => {
                    // 0016100a: move.b d3, d0
                    e.step()?;
                    let value = r.data(3, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0016100c: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0016100e: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00161012: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00161018: move.w $447194.l, d1
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0016101e: cmp.w $20(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x20_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00161022: bgt.b $161048
                    e.step()?;
                    pc = if r.greater()? {
                        0x161048_u32
                    } else {
                        0x161024_u32
                    };
                }
                0x161024 => {
                    // 00161024: move.b d3, d0
                    e.step()?;
                    let value = r.data(3, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00161026: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00161028: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0016102c: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00161032: cmpi.w #$ffff, $20(a0, d0.l)
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x20_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00161038: beq.b $161048
                    e.step()?;
                    pc = if r.equal()? {
                        0x161048_u32
                    } else {
                        0x16103a_u32
                    };
                }
                0x16103a => {
                    // 0016103a: move.l $439550.l, -(a7)
                    e.step()?;
                    let value_address = 0x439550_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00161040: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00161044: MacToolbox a8f6
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8f6_u32 as u16))?;
                    // 00161046: bra.b $161054
                    e.step()?;
                    pc = 0x161054_u32;
                }
                0x161048 => {
                    // 00161048: move.l $439558.l, -(a7)
                    e.step()?;
                    let value_address = 0x439558_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0016104e: pea.l -$8(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00161052: MacToolbox a8f6
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8f6_u32 as u16))?;
                    pc = 0x161054_u32;
                }
                0x161054 => {
                    // 00161054: move.l (a7)+, d3
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_data(3, 32, value);
                    r.move_flags(value, 32);
                    // 00161056: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00161058: rts
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
crate::recovered_application_control::entry!(updatelight, 0x160fb2);

pub fn owns_frame(entry: u32) -> bool {
    matches!(
        entry,
        0x105366
            | 0x105572
            | 0x10ae90
            | 0x110d12
            | 0x11235e
            | 0x11279a
            | 0x12063c
            | 0x125538
            | 0x125960
            | 0x133bb8
            | 0x133c56
            | 0x136398
            | 0x1363e4
            | 0x1377f4
            | 0x1382d0
            | 0x138a40
            | 0x13a3ae
            | 0x13f65e
            | 0x149e0e
            | 0x151f30
            | 0x153de6
            | 0x153e44
            | 0x157c68
            | 0x15b84c
            | 0x15d210
            | 0x15edfa
            | 0x15ef50
            | 0x15efee
            | 0x15f048
            | 0x160fb2
            | 0x1124dc
            | 0x122bec
    )
}
mod cansee_body_block_0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cansee_body_block_0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010ae90: link.w a6, #$ffc0
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, -64)?;
        // 0010ae94: move.l d4, -(a7)
        e.step()?;
        let value = r.data(4, 32)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010ae96: move.w $a(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xa_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010ae9a: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010ae9c: move.w d0, -$8(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010aea0: move.w $8(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0x8_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010aea4: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010aea6: move.w d0, -$6(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010aeaa: move.w $e(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xe_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010aeae: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010aeb0: move.w d0, -$20(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010aeb4: move.w $c(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xc_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010aeb8: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010aeba: move.w d0, -$1e(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe2_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010aebe: move.w -$1e(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffe2_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010aec2: sub.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010aec6: move.w d0, -$40(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010aeca: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010aece: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010aed2: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010aed6: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010aed8: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010aedc: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010aee0: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010aee4: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010aee6: move.l -$40(a6), -$24(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffc0_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010aeec: move.w -$20(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffe0_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010aef0: sub.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010aef4: move.w d0, -$40(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010aef8: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010aefc: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af00: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af04: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010af06: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af0a: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af0e: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af12: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010af14: move.l -$40(a6), -$28(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffc0_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffd8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010af1a: move.w $439568.l, d0
        e.step()?;
        let value_address = 0x439568_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010af20: addi.w #$80, d0
        e.step()?;
        let source = 0x80_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010af24: move.w d0, -$40(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af28: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af2c: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af30: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af34: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010af36: pea.l -$24(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af3a: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af3e: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af42: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010af44: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af48: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af4c: move.w #$6, -(a7)
        e.step()?;
        let value = 0x6_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af50: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010af52: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af56: pea.l -$2c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af5a: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af5e: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010af60: move.l -$2c(a6), -$14(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffd4_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010af66: move.w $439568.l, d0
        e.step()?;
        let value_address = 0x439568_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010af6c: addi.w #$80, d0
        e.step()?;
        let source = 0x80_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010af70: move.w d0, -$36(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af74: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af78: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af7c: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af80: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010af82: pea.l -$28(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af86: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af8a: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af8e: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010af90: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af94: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010af98: move.w #$6, -(a7)
        e.step()?;
        let value = 0x6_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010af9c: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010af9e: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010afa2: pea.l -$2c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010afa6: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010afaa: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010afac: move.l -$2c(a6), -$18(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffd4_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010afb2: move.w -$6(a6), -$36(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010afb8: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010afbc: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010afc0: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010afc4: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010afc6: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010afca: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010afce: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010afd2: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010afd4: move.l -$36(a6), -$c(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffca_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010afda: move.w -$8(a6), -$36(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010afe0: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010afe4: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010afe8: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010afec: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010afee: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010aff2: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010aff6: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010affa: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010affc: move.l -$36(a6), -$10(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffca_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010b002: moveq #$0, d4
        e.step()?;
        r.put_data(4, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010b004: bra.w $10b120
        e.step()?;
        pc = 0x10b120_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cansee_body_block_0_unit::cansee_body_block_0;

mod cansee_body_block_1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cansee_body_block_1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010b008: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010b00a: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010b00e: move.w d0, d1
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010b010: asr.w #$4, d1
        e.step()?;
        let count = 0x4_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(1, 16, value);
        // 0010b012: lsr.w #$8, d1
        e.step()?;
        let count = 0x8_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(1, 16, value);
        // 0010b014: lsr.w #$3, d1
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(1, 16, value);
        // 0010b016: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010b018: asr.w #$5, d1
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(1, 16, value);
        // 0010b01a: ext.l d1
        e.step()?;
        let source = r.data(1, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010b01c: add.l $442780.l, d1
        e.step()?;
        let source_address = 0x442780_u32;
        let source = e.l(source_address)?;
        let destination = r.data(1, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(1, 32, value);
        // 0010b022: muls.l #$b4, d1
        e.step()?;
        let source = 0xb4_u32;
        let destination = r.data(1, 32)?;
        let product = i64::from(source as i32) * i64::from(destination as i32);
        let value = product as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
        // 0010b02a: lea.l $4085b8.l, a0
        e.step()?;
        let value = 0x4085b8_u32;
        r.put_address(0, value);
        // 0010b030: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010b032: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010b034: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010b038: move.w d0, d1
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010b03a: asr.w #$4, d1
        e.step()?;
        let count = 0x4_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(1, 16, value);
        // 0010b03c: lsr.w #$8, d1
        e.step()?;
        let count = 0x8_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(1, 16, value);
        // 0010b03e: lsr.w #$3, d1
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(1, 16, value);
        // 0010b040: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010b042: asr.w #$5, d1
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(1, 16, value);
        // 0010b044: ext.l d1
        e.step()?;
        let source = r.data(1, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010b046: add.l $442784.l, d1
        e.step()?;
        let source_address = 0x442784_u32;
        let source = e.l(source_address)?;
        let destination = r.data(1, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(1, 32, value);
        // 0010b04c: move.w (a0, d1.l * 2), d2
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(2, 16, value);
        r.move_flags(value, 16);
        // 0010b050: cmpi.w #$3e7, d2
        e.step()?;
        let source = 0x3e7_u32;
        let destination = r.data(2, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010b054: ble.b $10b072
        e.step()?;
        pc = if r.less_equal()? {
            0x10b072_u32
        } else {
            0x10b056_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cansee_body_block_1_unit::cansee_body_block_1;

mod cansee_body_block_2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cansee_body_block_2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010b056: move.w d2, d0
        e.step()?;
        let value = r.data(2, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010b058: addi.w #$fc18, d0
        e.step()?;
        let source = 0xfc18_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010b05c: mulu.w #$28, d0
        e.step()?;
        let source = 0x28_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010b060: lea.l $43167c.l, a0
        e.step()?;
        let value = 0x43167c_u32;
        r.put_address(0, value);
        // 0010b066: tst.w $c(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010b06a: beq.b $10b072
        e.step()?;
        pc = if r.equal()? {
            0x10b072_u32
        } else {
            0x10b06c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cansee_body_block_2_unit::cansee_body_block_2;

mod cansee_body_block_3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cansee_body_block_3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010b06c: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010b06e: bra.w $10b12a
        e.step()?;
        pc = 0x10b12a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cansee_body_block_3_unit::cansee_body_block_3;

mod cansee_body_block_4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cansee_body_block_4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010b072: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b076: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b07a: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b07e: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b080: pea.l -$18(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b084: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b088: move.w #$1000, -(a7)
        e.step()?;
        let value = 0x1000_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b08c: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b08e: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b092: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b096: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b09a: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b09c: pea.l -$c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0a0: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0a4: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b0a8: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b0aa: pea.l -$14(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0ae: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0b2: move.w #$1000, -(a7)
        e.step()?;
        let value = 0x1000_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b0b6: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b0b8: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0bc: pea.l -$c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0c0: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b0c4: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b0c6: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0ca: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0ce: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b0d2: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b0d4: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0d8: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b0dc: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b0de: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0e2: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0e6: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b0ea: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b0ec: move.w -$34(a6), -$8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffcc_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b0f2: pea.l -$c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0f6: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b0fa: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b0fe: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b100: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b104: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b108: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b10a: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b10e: pea.l -$36(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffca_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010b112: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b116: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010b118: move.w -$34(a6), -$6(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffcc_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010b11e: addq.w #$1, d4
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(4, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(4, 16, value);
        pc = 0x10b120_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cansee_body_block_4_unit::cansee_body_block_4;

mod cansee_body_block_5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cansee_body_block_5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010b120: cmpi.w #$80, d4
        e.step()?;
        let source = 0x80_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010b124: blt.w $10b008
        e.step()?;
        pc = if r.less()? {
            0x10b008_u32
        } else {
            0x10b128_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cansee_body_block_5_unit::cansee_body_block_5;

mod cansee_body_block_6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cansee_body_block_6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010b128: moveq #$1, d0
        e.step()?;
        r.put_data(0, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        pc = 0x10b12a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cansee_body_block_6_unit::cansee_body_block_6;

mod cansee_body_block_7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cansee_body_block_7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010b12a: move.l (a7)+, d4
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(4, 32, value);
        r.move_flags(value, 32);
        // 0010b12c: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 0010b12e: rts
        e.step()?;
        return finish(e, r).map(|_| std::ops::ControlFlow::Break(()));

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cansee_body_block_7_unit::cansee_body_block_7;

mod cansee_body_block_8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cansee_body_block_8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        return Err(RuleError::Invalid(format!(
            "source transition outside recovered body: {pc:08x}"
        ))
        .into());

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cansee_body_block_8_unit::cansee_body_block_8;

mod destroymonster_body_block_0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012063c: link.w a6, #$fff8
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, -8)?;
        // 00120640: movem.l d3-d5, -(a7)
        e.step()?;
        r.save_register(e, 5)?;
        r.save_register(e, 4)?;
        r.save_register(e, 3)?;
        // 00120644: move.w $8(a6), d4
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0x8_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 00120648: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012064a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0012064e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00120654: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00120656: move.w #$1, $110(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x110_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012065c: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 0012065e: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00120660: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00120662: bsr.l $13a2c4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x120668_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a2c4_u32)?;
        // 00120668: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012066a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0012066e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00120674: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00120676: clr.w $110(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x110_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012067a: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012067c: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00120680: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00120686: move.w $10(a0, d0.l), -(a7)
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
        // 0012068a: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x120690_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00120690: move.w d0, d5
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(5, 16, value);
        r.move_flags(value, 16);
        // 00120692: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00120698: move.w $2(a0, d4.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012069c: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0012069e: move.w d0, -$8(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001206a2: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 001206a8: move.w (a0, d4.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001206ac: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 001206ae: move.w d0, -$6(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001206b2: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001206b4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001206b8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001206be: tst.w $e(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001206c2: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001206c4: beq.b $1206e2
        e.step()?;
        pc = if r.equal()? {
            0x1206e2_u32
        } else {
            0x1206c6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_0_unit::destroymonster_body_block_0;

mod destroymonster_body_block_1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001206c6: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001206c8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001206cc: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001206d2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001206d4: cmpi.b #$ff, $b0(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001206da: beq.b $1206e2
        e.step()?;
        pc = if r.equal()? {
            0x1206e2_u32
        } else {
            0x1206dc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_1_unit::destroymonster_body_block_1;

mod destroymonster_body_block_2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001206dc: addq.w #$1, $44374c.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x44374c_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x1206e2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_2_unit::destroymonster_body_block_2;

mod destroymonster_body_block_3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001206e2: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001206e4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001206e8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001206ee: clr.w $e(a0, d0.l)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001206f2: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001206f4: move.w d4, d1
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001206f6: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001206fa: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00120700: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00120702: move.b d0, $106(a0)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0x106_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00120706: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00120708: bra.b $120720
        e.step()?;
        pc = 0x120720_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_3_unit::destroymonster_body_block_3;

mod destroymonster_body_block_4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012070a: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012070c: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00120710: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00120716: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00120718: adda.w d3, a0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.address(0)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0012071a: clr.b $b2(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0xb2_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0012071e: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x120720_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_4_unit::destroymonster_body_block_4;

mod destroymonster_body_block_5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120720: cmpi.w #$28, d3
        e.step()?;
        let source = 0x28_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00120724: blt.b $12070a
        e.step()?;
        pc = if r.less()? {
            0x12070a_u32
        } else {
            0x120726_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_5_unit::destroymonster_body_block_5;

mod destroymonster_body_block_6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120726: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 00120728: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0012072c: move.w d0, -$4(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00120730: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 00120732: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00120736: move.w d0, -$2(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012073a: cmpi.w #$1, d5
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0012073e: beq.b $120746
        e.step()?;
        pc = if r.equal()? {
            0x120746_u32
        } else {
            0x120740_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_6_unit::destroymonster_body_block_6;

mod destroymonster_body_block_7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120740: cmpi.w #$3, d5
        e.step()?;
        let source = 0x3_u32;
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00120744: bne.b $12074c
        e.step()?;
        pc = if r.not_equal()? {
            0x12074c_u32
        } else {
            0x120746_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_7_unit::destroymonster_body_block_7;

mod destroymonster_body_block_8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120746: subi.w #$20, -$8(a6)
        e.step()?;
        let source = 0x20_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x12074c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_8_unit::destroymonster_body_block_8;

mod destroymonster_body_block_9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012074c: cmpi.w #$1, d5
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00120750: ble.b $120758
        e.step()?;
        pc = if r.less_equal()? {
            0x120758_u32
        } else {
            0x120752_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_9_unit::destroymonster_body_block_9;
mod destroymonster_body_block_a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120752: subi.w #$20, -$6(a6)
        e.step()?;
        let source = 0x20_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x120758_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_a_unit::destroymonster_body_block_a;

mod destroymonster_body_block_b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120758: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0012075e: MacToolbox a873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 00120760: clr.l $439598.l
        e.step()?;
        let value = 0;
        let destination_address = 0x439598_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00120766: subq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00120768: move.w d5, d0
        e.step()?;
        let value = r.data(5, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012076a: addi.w #$7df, d0
        e.step()?;
        let source = 0x7df_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0012076e: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00120770: MacToolbox aa1e
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xaa1e_u32 as u16))?;
        // 00120772: movea.l (a7)+, a0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_address(0, value);
        // 00120774: move.l a0, $439598.l
        e.step()?;
        let value = r.address(0)?;
        let destination_address = 0x439598_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0012077a: tst.l $439598.l
        e.step()?;
        let value_address = 0x439598_u32;
        let value = e.l(value_address)?;
        r.move_flags(value, 32);
        // 00120780: beq.b $120796
        e.step()?;
        pc = if r.equal()? {
            0x120796_u32
        } else {
            0x120782_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_b_unit::destroymonster_body_block_b;

mod destroymonster_body_block_c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120782: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00120786: move.l $439598.l, -(a7)
        e.step()?;
        let value_address = 0x439598_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0012078c: MacToolbox aa1f
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xaa1f_u32 as u16))?;
        // 0012078e: move.l $439598.l, -(a7)
        e.step()?;
        let value_address = 0x439598_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00120794: MacToolbox aa25
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xaa25_u32 as u16))?;
        pc = 0x120796_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_c_unit::destroymonster_body_block_c;

mod destroymonster_body_block_d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120796: moveq #$1, d3
        e.step()?;
        r.put_data(3, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 00120798: bra.b $1207b8
        e.step()?;
        pc = 0x1207b8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_d_unit::destroymonster_body_block_d;

mod destroymonster_body_block_e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012079a: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 001207a0: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 001207a2: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001207a4: cmp.w (a0, d3.w * 2), d0
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001207a8: bne.b $1207b6
        e.step()?;
        pc = if r.not_equal()? {
            0x1207b6_u32
        } else {
            0x1207aa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_e_unit::destroymonster_body_block_e;

mod destroymonster_body_block_f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001207aa: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 001207b0: move.w #$ffff, (a0, d3.w * 2)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1207b6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_f_unit::destroymonster_body_block_f;

mod destroymonster_body_block_10_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_10(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001207b6: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x1207b8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_10_unit::destroymonster_body_block_10;

mod destroymonster_body_block_11_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_11(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001207b8: cmpi.w #$6e, d3
        e.step()?;
        let source = 0x6e_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001207bc: blt.b $12079a
        e.step()?;
        pc = if r.less()? {
            0x12079a_u32
        } else {
            0x1207be_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_11_unit::destroymonster_body_block_11;

mod destroymonster_body_block_12_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_12(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001207be: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001207c0: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001207c4: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001207ca: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001207cc: tst.b $ab(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xab_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001207d0: beq.b $1207ec
        e.step()?;
        pc = if r.equal()? {
            0x1207ec_u32
        } else {
            0x1207d2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_12_unit::destroymonster_body_block_12;

mod destroymonster_body_block_13_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_13(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001207d2: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 001207d4: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001207d6: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001207d8: move.w #$fff2, -(a7)
        e.step()?;
        let value = 0xfff2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001207dc: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 001207de: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001207e0: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001207e2: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1207e8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001207e8: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001207ea: bra.b $120804
        e.step()?;
        pc = 0x120804_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_13_unit::destroymonster_body_block_13;

mod destroymonster_body_block_14_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_14(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001207ec: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 001207ee: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001207f0: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001207f2: move.w #$fffe, -(a7)
        e.step()?;
        let value = 0xfffe_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001207f6: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 001207f8: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001207fa: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001207fc: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x120802_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00120802: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x120804_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_14_unit::destroymonster_body_block_14;

mod destroymonster_body_block_15_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_15(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120804: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00120806: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00120808: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0012080a: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012080c: bsr.l $105572
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x120812_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x105572_u32)?;
        // 00120812: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00120814: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00120816: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00120818: bsr.l $105366
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x12081e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x105366_u32)?;
        // 0012081e: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00120820: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00120822: bra.b $12084e
        e.step()?;
        pc = 0x12084e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_15_unit::destroymonster_body_block_15;

mod destroymonster_body_block_16_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_16(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120824: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00120826: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0012082a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00120830: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00120832: cmp.w $10c(a0), d4
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0x10c_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00120836: bne.b $12084c
        e.step()?;
        pc = if r.not_equal()? {
            0x12084c_u32
        } else {
            0x120838_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_16_unit::destroymonster_body_block_16;

mod destroymonster_body_block_17_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_17(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120838: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012083a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0012083e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00120844: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00120846: move.w #$ffff, $10c(a0)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x12084c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_17_unit::destroymonster_body_block_17;

mod destroymonster_body_block_18_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_18(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012084c: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x12084e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_18_unit::destroymonster_body_block_18;

mod destroymonster_body_block_19_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_19(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012084e: cmp.w $43874a.l, d3
        e.step()?;
        let source_address = 0x43874a_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00120854: blt.b $120824
        e.step()?;
        pc = if r.less()? {
            0x120824_u32
        } else {
            0x120856_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_19_unit::destroymonster_body_block_19;
mod destroymonster_body_block_1a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_1a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00120856: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00120858: movem.l (a7)+, d3-d5
        e.step()?;
        r.restore_register(e, 3)?;
        r.restore_register(e, 4)?;
        r.restore_register(e, 5)?;
        // 0012085c: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 0012085e: rts
        e.step()?;
        return finish(e, r).map(|_| std::ops::ControlFlow::Break(()));

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_1a_unit::destroymonster_body_block_1a;

mod destroymonster_body_block_1b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn destroymonster_body_block_1b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        return Err(RuleError::Invalid(format!(
            "source transition outside recovered body: {pc:08x}"
        ))
        .into());

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use destroymonster_body_block_1b_unit::destroymonster_body_block_1b;

mod getrange_body_block_0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125538: link.w a6, #$fff0
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, -16)?;
        // 0012553c: movem.l d3-d7, -(a7)
        e.step()?;
        r.save_register(e, 7)?;
        r.save_register(e, 6)?;
        r.save_register(e, 5)?;
        r.save_register(e, 4)?;
        r.save_register(e, 3)?;
        // 00125540: move.b $9(a6), d7
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0x9_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(7, 8, value);
        r.move_flags(value, 8);
        // 00125544: move.b $b(a6), d6
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xb_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(6, 8, value);
        r.move_flags(value, 8);
        // 00125548: move.b $d(a6), d5
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xd_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(5, 8, value);
        r.move_flags(value, 8);
        // 0012554c: clr.b -$1(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00125550: cmpi.b #$a, d7
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(7, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 00125554: bge.b $125578
        e.step()?;
        pc = if r.greater_equal()? {
            0x125578_u32
        } else {
            0x125556_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_0_unit::getrange_body_block_0;

mod getrange_body_block_1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125556: move.b d7, d0
        e.step()?;
        let value = r.data(7, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125558: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0012555a: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00125560: move.w (a0, d0.l * 4), -$4(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125566: move.b d7, d0
        e.step()?;
        let value = r.data(7, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125568: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0012556a: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00125570: move.w $2(a0, d0.l * 4), -$6(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125576: bra.b $1255a0
        e.step()?;
        pc = 0x1255a0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_1_unit::getrange_body_block_1;

mod getrange_body_block_2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125578: move.b d7, d0
        e.step()?;
        let value = r.data(7, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0012557a: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012557c: addi.w #$fff6, d0
        e.step()?;
        let source = 0xfff6_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00125580: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00125586: move.w (a0, d0.w * 4), -$4(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012558c: move.b d7, d0
        e.step()?;
        let value = r.data(7, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0012558e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00125590: addi.w #$fff6, d0
        e.step()?;
        let source = 0xfff6_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00125594: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 0012559a: move.w $2(a0, d0.w * 4), -$6(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1255a0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_2_unit::getrange_body_block_2;

mod getrange_body_block_3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001255a0: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001255a2: bra.b $1255b6
        e.step()?;
        pc = 0x1255b6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_3_unit::getrange_body_block_3;

mod getrange_body_block_4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001255a4: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001255a6: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001255a8: lea.l $4435d6.l, a0
        e.step()?;
        let value = 0x4435d6_u32;
        r.put_address(0, value);
        // 001255ae: move.w #$7f, (a0, d0.l * 2)
        e.step()?;
        let value = 0x7f_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001255b4: addq.b #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 8)?;
        let value = r.add_flags(destination, source, 8, true);
        r.put_data(3, 8, value);
        pc = 0x1255b6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_4_unit::getrange_body_block_4;

mod getrange_body_block_5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001255b6: cmpi.b #$70, d3
        e.step()?;
        let source = 0x70_u32;
        let destination = r.data(3, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 001255ba: blt.b $1255a4
        e.step()?;
        pc = if r.less()? {
            0x1255a4_u32
        } else {
            0x1255bc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_5_unit::getrange_body_block_5;

mod getrange_body_block_6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001255bc: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001255be: bra.w $125754
        e.step()?;
        pc = 0x125754_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_6_unit::getrange_body_block_6;

mod getrange_body_block_7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001255c2: moveq #$0, d4
        e.step()?;
        r.put_data(4, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001255c4: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001255c6: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001255c8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001255cc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001255d2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001255d4: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001255d8: beq.w $125752
        e.step()?;
        pc = if r.equal()? {
            0x125752_u32
        } else {
            0x1255dc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_7_unit::getrange_body_block_7;

mod getrange_body_block_8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001255dc: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001255de: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001255e0: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001255e4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001255ea: tst.w $50(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001255ee: ble.w $125752
        e.step()?;
        pc = if r.less_equal()? {
            0x125752_u32
        } else {
            0x1255f2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_8_unit::getrange_body_block_8;

mod getrange_body_block_9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001255f2: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001255f4: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001255f6: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001255fa: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00125600: move.b d6, d1
        e.step()?;
        let value = r.data(6, 8)?;
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 00125602: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00125604: cmp.w $10(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00125608: beq.b $125610
        e.step()?;
        pc = if r.equal()? {
            0x125610_u32
        } else {
            0x12560a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_9_unit::getrange_body_block_9;
mod getrange_body_block_a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012560a: cmpi.b #$4, d5
        e.step()?;
        let source = 0x4_u32;
        let destination = r.data(5, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 0012560e: bne.b $125632
        e.step()?;
        pc = if r.not_equal()? {
            0x125632_u32
        } else {
            0x125610_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_a_unit::getrange_body_block_a;

mod getrange_body_block_b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125610: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125612: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00125614: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00125618: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0012561e: move.b d6, d1
        e.step()?;
        let value = r.data(6, 8)?;
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 00125620: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00125622: cmp.w $10(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00125626: bne.w $125752
        e.step()?;
        pc = if r.not_equal()? {
            0x125752_u32
        } else {
            0x12562a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_b_unit::getrange_body_block_b;

mod getrange_body_block_c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012562a: cmpi.b #$4, d5
        e.step()?;
        let source = 0x4_u32;
        let destination = r.data(5, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 0012562e: bne.w $125752
        e.step()?;
        pc = if r.not_equal()? {
            0x125752_u32
        } else {
            0x125632_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_c_unit::getrange_body_block_c;

mod getrange_body_block_d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125632: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125634: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00125636: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 0012563c: move.w $2(a0, d0.l * 4), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125640: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125642: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00125644: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 0012564a: move.w (a0, d0.l * 4), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012564e: move.b d7, d0
        e.step()?;
        let value = r.data(7, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125650: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00125652: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125654: bsr.l $11235e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x12565a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11235e_u32)?;
        // 0012565a: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 0012565c: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0012565e: beq.w $1256f2
        e.step()?;
        pc = if r.equal()? {
            0x1256f2_u32
        } else {
            0x125662_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_d_unit::getrange_body_block_d;

mod getrange_body_block_e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125662: move.b $443751.l, d0
        e.step()?;
        let value_address = 0x443751_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125668: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012566a: move.b $443750.l, d1
        e.step()?;
        let value_address = 0x443750_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 00125670: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00125672: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00125674: tst.w d1
        e.step()?;
        let value = r.data(1, 16)?;
        r.move_flags(value, 16);
        // 00125676: ble.b $1256aa
        e.step()?;
        pc = if r.less_equal()? {
            0x1256aa_u32
        } else {
            0x125678_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_e_unit::getrange_body_block_e;

mod getrange_body_block_f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125678: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0012567a: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0012567c: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00125682: move.w $2(a0, d0.l * 4), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125686: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125688: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0012568a: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00125690: move.w (a0, d0.l * 4), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125694: move.w -$6(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125698: move.w -$4(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012569c: bsr.l $10ae90
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1256a2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10ae90_u32)?;
        // 001256a2: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 001256a4: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001256a6: bne.b $1256aa
        e.step()?;
        pc = if r.not_equal()? {
            0x1256aa_u32
        } else {
            0x1256a8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_f_unit::getrange_body_block_f;

mod getrange_body_block_10_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_10(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001256a8: moveq #$1, d4
        e.step()?;
        r.put_data(4, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        pc = 0x1256aa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_10_unit::getrange_body_block_10;

mod getrange_body_block_11_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_11(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001256aa: tst.b d4
        e.step()?;
        let value = r.data(4, 8)?;
        r.move_flags(value, 8);
        // 001256ac: bne.b $1256f2
        e.step()?;
        pc = if r.not_equal()? {
            0x1256f2_u32
        } else {
            0x1256ae_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_11_unit::getrange_body_block_11;

mod getrange_body_block_12_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_12(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001256ae: move.l $44349c.l, -$10(a6)
        e.step()?;
        let value_address = 0x44349c_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001256b6: move.l $4434a0.l, -$c(a6)
        e.step()?;
        let value_address = 0x4434a0_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001256be: move.w $4434a4.l, -$8(a6)
        e.step()?;
        let value_address = 0x4434a4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001256c6: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001256ca: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001256ce: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001256d0: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001256d4: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001256d8: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001256dc: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001256de: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001256e0: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001256e2: lea.l $4435d6.l, a0
        e.step()?;
        let value = 0x4435d6_u32;
        r.put_address(0, value);
        // 001256e8: move.w -$e(a6), (a0, d0.l * 2)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001256ee: addq.b #$1, -$1(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        pc = 0x1256f2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_12_unit::getrange_body_block_12;

mod getrange_body_block_13_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_13(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001256f2: move.w $4436b4.l, -$10(a6)
        e.step()?;
        let value_address = 0x4436b4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001256fa: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001256fe: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00125702: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125706: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00125708: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0012570c: pea.l $44349c.l
        e.step()?;
        let value = 0x44349c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00125712: move.w #$8, -(a7)
        e.step()?;
        let value = 0x8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125716: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00125718: bge.b $125752
        e.step()?;
        pc = if r.greater_equal()? {
            0x125752_u32
        } else {
            0x12571a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_13_unit::getrange_body_block_13;

mod getrange_body_block_14_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_14(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012571a: move.l $44349c.l, -$10(a6)
        e.step()?;
        let value_address = 0x44349c_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00125722: move.l $4434a0.l, -$c(a6)
        e.step()?;
        let value_address = 0x4434a0_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0012572a: move.w $4434a4.l, -$8(a6)
        e.step()?;
        let value_address = 0x4434a4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125732: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00125736: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012573a: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0012573c: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00125740: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00125744: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125748: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0012574a: move.w -$e(a6), $4436b4.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4436b4_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x125752_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_14_unit::getrange_body_block_14;

mod getrange_body_block_15_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_15(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125752: addq.b #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 8)?;
        let value = r.add_flags(destination, source, 8, true);
        r.put_data(3, 8, value);
        pc = 0x125754_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_15_unit::getrange_body_block_15;

mod getrange_body_block_16_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_16(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125754: cmp.b $44273a.l, d3
        e.step()?;
        let source_address = 0x44273a_u32;
        let source = u32::from(e.b(source_address)?);
        let destination = r.data(3, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 0012575a: ble.w $1255c2
        e.step()?;
        pc = if r.less_equal()? {
            0x1255c2_u32
        } else {
            0x12575e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_16_unit::getrange_body_block_16;

mod getrange_body_block_17_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_17(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012575e: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00125760: bra.w $125938
        e.step()?;
        pc = 0x125938_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_17_unit::getrange_body_block_17;

mod getrange_body_block_18_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_18(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125764: moveq #$0, d4
        e.step()?;
        r.put_data(4, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00125766: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125768: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012576a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0012576e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00125774: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00125776: tst.w $110(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x110_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0012577a: ble.w $125936
        e.step()?;
        pc = if r.less_equal()? {
            0x125936_u32
        } else {
            0x12577e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_18_unit::getrange_body_block_18;

mod getrange_body_block_19_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_19(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012577e: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125780: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00125782: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00125786: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0012578c: move.b d6, d1
        e.step()?;
        let value = r.data(6, 8)?;
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 0012578e: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00125790: cmp.w $e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00125794: beq.b $1257b4
        e.step()?;
        pc = if r.equal()? {
            0x1257b4_u32
        } else {
            0x125796_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_19_unit::getrange_body_block_19;
mod getrange_body_block_1a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_1a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125796: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125798: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012579a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0012579e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001257a4: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001257a6: cmpi.b #$ff, $b0(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001257ac: beq.b $1257b4
        e.step()?;
        pc = if r.equal()? {
            0x1257b4_u32
        } else {
            0x1257ae_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_1a_unit::getrange_body_block_1a;

mod getrange_body_block_1b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_1b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001257ae: cmpi.b #$4, d5
        e.step()?;
        let source = 0x4_u32;
        let destination = r.data(5, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 001257b2: bne.b $1257f0
        e.step()?;
        pc = if r.not_equal()? {
            0x1257f0_u32
        } else {
            0x1257b4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_1b_unit::getrange_body_block_1b;

mod getrange_body_block_1c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_1c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001257b4: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001257b6: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001257b8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001257bc: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001257c2: move.b d6, d1
        e.step()?;
        let value = r.data(6, 8)?;
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 001257c4: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001257c6: cmp.w $e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001257ca: bne.w $125936
        e.step()?;
        pc = if r.not_equal()? {
            0x125936_u32
        } else {
            0x1257ce_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_1c_unit::getrange_body_block_1c;

mod getrange_body_block_1d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_1d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001257ce: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001257d0: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001257d2: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001257d6: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001257dc: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001257de: cmpi.b #$ff, $b0(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001257e4: beq.w $125936
        e.step()?;
        pc = if r.equal()? {
            0x125936_u32
        } else {
            0x1257e8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_1d_unit::getrange_body_block_1d;

mod getrange_body_block_1e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_1e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001257e8: cmpi.b #$4, d5
        e.step()?;
        let source = 0x4_u32;
        let destination = r.data(5, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 001257ec: bne.w $125936
        e.step()?;
        pc = if r.not_equal()? {
            0x125936_u32
        } else {
            0x1257f0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_1e_unit::getrange_body_block_1e;

mod getrange_body_block_1f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_1f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001257f0: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001257f2: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001257f4: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 001257fa: move.w $2(a0, d0.l * 4), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001257fe: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125800: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00125802: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00125808: move.w (a0, d0.l * 4), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012580c: move.b d7, d0
        e.step()?;
        let value = r.data(7, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0012580e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00125810: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125812: bsr.l $11235e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x125818_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11235e_u32)?;
        // 00125818: move.b $443751.l, d0
        e.step()?;
        let value_address = 0x443751_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0012581e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00125820: move.b $443750.l, d1
        e.step()?;
        let value_address = 0x443750_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 00125826: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00125828: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0012582a: tst.w d1
        e.step()?;
        let value = r.data(1, 16)?;
        r.move_flags(value, 16);
        // 0012582c: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0012582e: ble.b $125862
        e.step()?;
        pc = if r.less_equal()? {
            0x125862_u32
        } else {
            0x125830_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_1f_unit::getrange_body_block_1f;

mod getrange_body_block_20_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_20(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125830: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125832: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00125834: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 0012583a: move.w $2(a0, d0.l * 4), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012583e: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00125840: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00125842: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00125848: move.w (a0, d0.l * 4), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012584c: move.w -$6(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125850: move.w -$4(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125854: bsr.l $10ae90
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x12585a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10ae90_u32)?;
        // 0012585a: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 0012585c: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0012585e: bne.b $125862
        e.step()?;
        pc = if r.not_equal()? {
            0x125862_u32
        } else {
            0x125860_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_20_unit::getrange_body_block_20;

mod getrange_body_block_21_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_21(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125860: moveq #$1, d4
        e.step()?;
        r.put_data(4, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        pc = 0x125862_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_21_unit::getrange_body_block_21;

mod getrange_body_block_22_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_22(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125862: move.w $4435bc.l, -$10(a6)
        e.step()?;
        let value_address = 0x4435bc_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012586a: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0012586e: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00125872: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125876: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00125878: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0012587c: pea.l $44349c.l
        e.step()?;
        let value = 0x44349c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00125882: move.w #$8, -(a7)
        e.step()?;
        let value = 0x8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125886: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00125888: bgt.b $1258d6
        e.step()?;
        pc = if r.greater()? {
            0x1258d6_u32
        } else {
            0x12588a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_22_unit::getrange_body_block_22;

mod getrange_body_block_23_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_23(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012588a: tst.b d4
        e.step()?;
        let value = r.data(4, 8)?;
        r.move_flags(value, 8);
        // 0012588c: bne.b $1258d6
        e.step()?;
        pc = if r.not_equal()? {
            0x1258d6_u32
        } else {
            0x12588e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_23_unit::getrange_body_block_23;

mod getrange_body_block_24_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_24(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0012588e: move.l $44349c.l, -$10(a6)
        e.step()?;
        let value_address = 0x44349c_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00125896: move.l $4434a0.l, -$c(a6)
        e.step()?;
        let value_address = 0x4434a0_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0012589e: move.w $4434a4.l, -$8(a6)
        e.step()?;
        let value_address = 0x4434a4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001258a6: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001258aa: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001258ae: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001258b0: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001258b4: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001258b8: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001258bc: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001258be: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001258c0: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001258c2: addi.w #$a, d0
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001258c6: lea.l $4435d6.l, a0
        e.step()?;
        let value = 0x4435d6_u32;
        r.put_address(0, value);
        // 001258cc: move.w -$e(a6), (a0, d0.w * 2)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001258d2: addq.b #$1, -$1(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        pc = 0x1258d6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_24_unit::getrange_body_block_24;

mod getrange_body_block_25_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_25(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001258d6: move.w $4436b4.l, -$10(a6)
        e.step()?;
        let value_address = 0x4436b4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001258de: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001258e2: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001258e6: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001258ea: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001258ec: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001258f0: pea.l $44349c.l
        e.step()?;
        let value = 0x44349c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001258f6: move.w #$8, -(a7)
        e.step()?;
        let value = 0x8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001258fa: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001258fc: bge.b $125936
        e.step()?;
        pc = if r.greater_equal()? {
            0x125936_u32
        } else {
            0x1258fe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_25_unit::getrange_body_block_25;

mod getrange_body_block_26_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_26(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001258fe: move.l $44349c.l, -$10(a6)
        e.step()?;
        let value_address = 0x44349c_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00125906: move.l $4434a0.l, -$c(a6)
        e.step()?;
        let value_address = 0x4434a0_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0012590e: move.w $4434a4.l, -$8(a6)
        e.step()?;
        let value_address = 0x4434a4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00125916: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0012591a: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012591e: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00125920: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00125924: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00125928: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0012592c: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0012592e: move.w -$e(a6), $4436b4.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4436b4_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x125936_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_26_unit::getrange_body_block_26;

mod getrange_body_block_27_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_27(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125936: addq.b #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 8)?;
        let value = r.add_flags(destination, source, 8, true);
        r.put_data(3, 8, value);
        pc = 0x125938_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_27_unit::getrange_body_block_27;

mod getrange_body_block_28_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_28(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125938: move.b d3, d0
        e.step()?;
        let value = r.data(3, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0012593a: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012593c: cmp.w $43874a.l, d0
        e.step()?;
        let source_address = 0x43874a_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00125942: blt.w $125764
        e.step()?;
        pc = if r.less()? {
            0x125764_u32
        } else {
            0x125946_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_28_unit::getrange_body_block_28;

mod getrange_body_block_29_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_29(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00125946: move.b -$1(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0012594a: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0012594c: movem.l (a7)+, d3-d7
        e.step()?;
        r.restore_register(e, 3)?;
        r.restore_register(e, 4)?;
        r.restore_register(e, 5)?;
        r.restore_register(e, 6)?;
        r.restore_register(e, 7)?;
        // 00125950: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 00125952: rts
        e.step()?;
        return finish(e, r).map(|_| std::ops::ControlFlow::Break(()));

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_29_unit::getrange_body_block_29;
mod getrange_body_block_2a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn getrange_body_block_2a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        return Err(RuleError::Invalid(format!(
            "source transition outside recovered body: {pc:08x}"
        ))
        .into());

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use getrange_body_block_2a_unit::getrange_body_block_2a;

mod best_choice_body_block_0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001363e4: link.w a6, #$ff16
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, -234)?;
        // 001363e8: movem.l d3-d7, -(a7)
        e.step()?;
        r.save_register(e, 7)?;
        r.save_register(e, 6)?;
        r.save_register(e, 5)?;
        r.save_register(e, 4)?;
        r.save_register(e, 3)?;
        // 001363ec: move.w $a(a6), d7
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xa_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(7, 16, value);
        r.move_flags(value, 16);
        // 001363f0: clr.w -$ea(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffff16_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001363f4: move.w #$ffff, -$e8(a6)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffff18_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001363fa: clr.w -$e6(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1a_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001363fe: clr.w -$e4(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00136402: clr.w -$e2(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00136406: bra.w $136684
        e.step()?;
        pc = 0x136684_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_0_unit::best_choice_body_block_0;

mod best_choice_body_block_1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013640a: lea.l $4435d6.l, a0
        e.step()?;
        let value = 0x4435d6_u32;
        r.put_address(0, value);
        // 00136410: move.w -$e2(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00136414: move.w (a0, d0.w * 2), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00136418: cmp.w $4435bc.l, d0
        e.step()?;
        let source_address = 0x4435bc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0013641e: bgt.w $136680
        e.step()?;
        pc = if r.greater()? {
            0x136680_u32
        } else {
            0x136422_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_1_unit::best_choice_body_block_1;

mod best_choice_body_block_2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136422: moveq #$0, d6
        e.step()?;
        r.put_data(6, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00136424: move.w d6, d5
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(5, 16, value);
        r.move_flags(value, 16);
        // 00136426: moveq #$0, d4
        e.step()?;
        r.put_data(4, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00136428: bra.b $136434
        e.step()?;
        pc = 0x136434_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_2_unit::best_choice_body_block_2;

mod best_choice_body_block_3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013642a: lea.l -$dc(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff24_u32);
        r.put_address(0, value);
        // 0013642e: clr.w (a0, d4.w * 2)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00136432: addq.w #$1, d4
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(4, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(4, 16, value);
        pc = 0x136434_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_3_unit::best_choice_body_block_3;

mod best_choice_body_block_4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136434: cmpi.w #$6e, d4
        e.step()?;
        let source = 0x6e_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00136438: blt.b $13642a
        e.step()?;
        pc = if r.less()? {
            0x13642a_u32
        } else {
            0x13643a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_4_unit::best_choice_body_block_4;

mod best_choice_body_block_5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013643a: cmpi.w #$9, -$e2(a6)
        e.step()?;
        let source = 0x9_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00136440: ble.b $136498
        e.step()?;
        pc = if r.less_equal()? {
            0x136498_u32
        } else {
            0x136442_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_5_unit::best_choice_body_block_5;

mod best_choice_body_block_6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136442: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00136444: add.w -$e2(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00136448: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0013644c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00136452: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00136454: tst.w $110(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x110_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00136458: ble.b $136498
        e.step()?;
        pc = if r.less_equal()? {
            0x136498_u32
        } else {
            0x13645a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_6_unit::best_choice_body_block_6;

mod best_choice_body_block_7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013645a: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0013645c: add.w -$e2(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00136460: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00136466: move.w (a0, d0.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0013646a: subq.w #$3, d0
        e.step()?;
        let source = 0x3_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0013646c: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0013646e: add.l $442780.l, d0
        e.step()?;
        let source_address = 0x442780_u32;
        let source = e.l(source_address)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00136474: move.w d0, -$de(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff22_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00136478: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0013647a: add.w -$e2(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0013647e: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00136484: move.w $2(a0, d0.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00136488: subq.w #$3, d0
        e.step()?;
        let source = 0x3_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0013648a: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0013648c: add.l $442784.l, d0
        e.step()?;
        let source_address = 0x442784_u32;
        let source = e.l(source_address)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00136492: move.w d0, -$e0(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff20_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00136496: bra.b $1364f8
        e.step()?;
        pc = 0x1364f8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_7_unit::best_choice_body_block_7;

mod best_choice_body_block_8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136498: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0013649e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001364a0: cmp.w -$e2(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001364a4: blt.w $136676
        e.step()?;
        pc = if r.less()? {
            0x136676_u32
        } else {
            0x1364a8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_8_unit::best_choice_body_block_8;

mod best_choice_body_block_9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001364a8: move.w -$e2(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001364ac: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001364b0: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001364b6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001364b8: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001364bc: beq.w $136676
        e.step()?;
        pc = if r.equal()? {
            0x136676_u32
        } else {
            0x1364c0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_9_unit::best_choice_body_block_9;
mod best_choice_body_block_a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001364c0: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 001364c6: move.w -$e2(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001364ca: move.w (a0, d0.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001364ce: subq.w #$3, d0
        e.step()?;
        let source = 0x3_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001364d0: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001364d2: add.l $442780.l, d0
        e.step()?;
        let source_address = 0x442780_u32;
        let source = e.l(source_address)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 001364d8: move.w d0, -$de(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff22_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001364dc: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 001364e2: move.w -$e2(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001364e6: move.w $2(a0, d0.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001364ea: subq.w #$3, d0
        e.step()?;
        let source = 0x3_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001364ec: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001364ee: add.l $442784.l, d0
        e.step()?;
        let source_address = 0x442784_u32;
        let source = e.l(source_address)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 001364f4: move.w d0, -$e0(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff20_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1364f8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_a_unit::best_choice_body_block_a;

mod best_choice_body_block_b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001364f8: moveq #$0, d4
        e.step()?;
        r.put_data(4, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001364fa: bra.w $1365f8
        e.step()?;
        pc = 0x1365f8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_b_unit::best_choice_body_block_b;

mod best_choice_body_block_c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001364fe: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00136500: bra.w $1365ee
        e.step()?;
        pc = 0x1365ee_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_c_unit::best_choice_body_block_c;

mod best_choice_body_block_d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136504: moveq #$7, d0
        e.step()?;
        r.put_data(0, 32, 0x7_u32);
        r.move_flags(0x7_u32, 32);
        // 00136506: mulu.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00136508: move.w $8(a6), d1
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0x8_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0013650c: subq.w #$1, d1
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0013650e: mulu.w #$31, d1
        e.step()?;
        let source = 0x31_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00136512: lea.l $438210.l, a0
        e.step()?;
        let value = 0x438210_u32;
        r.put_address(0, value);
        // 00136518: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0013651a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0013651c: tst.b (a0, d3.w)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1));
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00136520: beq.w $1365ec
        e.step()?;
        pc = if r.equal()? {
            0x1365ec_u32
        } else {
            0x136524_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_d_unit::best_choice_body_block_d;

mod best_choice_body_block_e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136524: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00136526: add.w -$de(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff22_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0013652a: mulu.w #$b4, d0
        e.step()?;
        let source = 0xb4_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0013652e: lea.l $4085b8.l, a0
        e.step()?;
        let value = 0x4085b8_u32;
        r.put_address(0, value);
        // 00136534: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00136536: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00136538: add.w -$e0(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff20_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0013653c: move.w (a0, d0.w * 2), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00136540: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x136546_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00136546: move.w d0, $4434b6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0013654c: cmpi.w #$6e, $4434b6.l
        e.step()?;
        let source = 0x6e_u32;
        let destination_address = 0x4434b6_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00136554: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00136556: bge.w $1365ec
        e.step()?;
        pc = if r.greater_equal()? {
            0x1365ec_u32
        } else {
            0x13655a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_e_unit::best_choice_body_block_e;

mod best_choice_body_block_f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013655a: cmpi.w #$9, $4434b6.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x4434b6_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00136562: bge.b $136582
        e.step()?;
        pc = if r.greater_equal()? {
            0x136582_u32
        } else {
            0x136564_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_f_unit::best_choice_body_block_f;

mod best_choice_body_block_10_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_10(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136564: move.w $4434b6.l, d0
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0013656a: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0013656e: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00136574: cmp.w $10(a0, d0.l), d7
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00136578: beq.b $13657e
        e.step()?;
        pc = if r.equal()? {
            0x13657e_u32
        } else {
            0x13657a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_10_unit::best_choice_body_block_10;

mod best_choice_body_block_11_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_11(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013657a: addq.w #$1, d5
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 0013657c: bra.b $1365ec
        e.step()?;
        pc = 0x1365ec_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_11_unit::best_choice_body_block_11;

mod best_choice_body_block_12_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_12(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013657e: addq.w #$1, d6
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 00136580: bra.b $1365ec
        e.step()?;
        pc = 0x1365ec_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_12_unit::best_choice_body_block_12;

mod best_choice_body_block_13_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_13(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136582: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00136584: add.w $4434b6.l, d0
        e.step()?;
        let source_address = 0x4434b6_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0013658a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0013658e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00136594: cmp.w $e(a0, d0.l), d7
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00136598: beq.b $1365d2
        e.step()?;
        pc = if r.equal()? {
            0x1365d2_u32
        } else {
            0x13659a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_13_unit::best_choice_body_block_13;

mod best_choice_body_block_14_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_14(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013659a: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0013659c: add.w $4434b6.l, d0
        e.step()?;
        let source_address = 0x4434b6_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001365a2: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001365a6: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001365ac: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001365ae: cmpi.b #$ff, $b0(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001365b4: beq.b $1365d2
        e.step()?;
        pc = if r.equal()? {
            0x1365d2_u32
        } else {
            0x1365b6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_14_unit::best_choice_body_block_14;

mod best_choice_body_block_15_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_15(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001365b6: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001365b8: add.w $4434b6.l, d0
        e.step()?;
        let source_address = 0x4434b6_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001365be: lea.l -$dc(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff24_u32);
        r.put_address(0, value);
        // 001365c2: move.w (a0, d0.w * 2), d1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001365c6: addq.w #$1, (a0, d0.w * 2)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001365ca: tst.w d1
        e.step()?;
        let value = r.data(1, 16)?;
        r.move_flags(value, 16);
        // 001365cc: bne.b $1365ec
        e.step()?;
        pc = if r.not_equal()? {
            0x1365ec_u32
        } else {
            0x1365ce_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_15_unit::best_choice_body_block_15;

mod best_choice_body_block_16_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_16(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001365ce: addq.w #$1, d5
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 001365d0: bra.b $1365ec
        e.step()?;
        pc = 0x1365ec_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_16_unit::best_choice_body_block_16;

mod best_choice_body_block_17_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_17(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001365d2: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001365d4: add.w $4434b6.l, d0
        e.step()?;
        let source_address = 0x4434b6_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001365da: lea.l -$dc(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff24_u32);
        r.put_address(0, value);
        // 001365de: move.w (a0, d0.w * 2), d1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001365e2: addq.w #$1, (a0, d0.w * 2)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001365e6: tst.w d1
        e.step()?;
        let value = r.data(1, 16)?;
        r.move_flags(value, 16);
        // 001365e8: bne.b $1365ec
        e.step()?;
        pc = if r.not_equal()? {
            0x1365ec_u32
        } else {
            0x1365ea_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_17_unit::best_choice_body_block_17;

mod best_choice_body_block_18_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_18(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001365ea: addq.w #$1, d6
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x1365ec_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_18_unit::best_choice_body_block_18;

mod best_choice_body_block_19_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_19(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001365ec: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x1365ee_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_19_unit::best_choice_body_block_19;
mod best_choice_body_block_1a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_1a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001365ee: cmpi.w #$7, d3
        e.step()?;
        let source = 0x7_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001365f2: blt.w $136504
        e.step()?;
        pc = if r.less()? {
            0x136504_u32
        } else {
            0x1365f6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_1a_unit::best_choice_body_block_1a;

mod best_choice_body_block_1b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_1b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001365f6: addq.w #$1, d4
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(4, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(4, 16, value);
        pc = 0x1365f8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_1b_unit::best_choice_body_block_1b;

mod best_choice_body_block_1c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_1c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001365f8: cmpi.w #$7, d4
        e.step()?;
        let source = 0x7_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001365fc: blt.w $1364fe
        e.step()?;
        pc = if r.less()? {
            0x1364fe_u32
        } else {
            0x136600_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_1c_unit::best_choice_body_block_1c;

mod best_choice_body_block_1d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_1d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136600: move.w d7, d0
        e.step()?;
        let value = r.data(7, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00136602: beq.b $136606
        e.step()?;
        pc = if r.equal()? {
            0x136606_u32
        } else {
            0x136604_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_1d_unit::best_choice_body_block_1d;

mod best_choice_body_block_1e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_1e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136604: bra.b $136656
        e.step()?;
        pc = 0x136656_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_1e_unit::best_choice_body_block_1e;

mod best_choice_body_block_1f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_1f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136606: cmp.w -$e4(a6), d5
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1c_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0013660a: bge.b $136618
        e.step()?;
        pc = if r.greater_equal()? {
            0x136618_u32
        } else {
            0x13660c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_1f_unit::best_choice_body_block_1f;

mod best_choice_body_block_20_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_20(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013660c: move.w -$e4(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff1c_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00136610: sub.w -$e6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1a_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00136614: cmp.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00136616: blt.b $136676
        e.step()?;
        pc = if r.less()? {
            0x136676_u32
        } else {
            0x136618_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_20_unit::best_choice_body_block_20;

mod best_choice_body_block_21_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_21(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136618: cmp.w -$e4(a6), d5
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1c_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0013661c: blt.b $136638
        e.step()?;
        pc = if r.less()? {
            0x136638_u32
        } else {
            0x13661e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_21_unit::best_choice_body_block_21;

mod best_choice_body_block_22_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_22(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013661e: cmp.w -$e6(a6), d6
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1a_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(6, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00136622: bgt.b $136638
        e.step()?;
        pc = if r.greater()? {
            0x136638_u32
        } else {
            0x136624_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_22_unit::best_choice_body_block_22;

mod best_choice_body_block_23_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_23(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136624: cmp.w d5, d6
        e.step()?;
        let source = r.data(5, 16)?;
        let destination = r.data(6, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00136626: bge.b $136638
        e.step()?;
        pc = if r.greater_equal()? {
            0x136638_u32
        } else {
            0x136628_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_23_unit::best_choice_body_block_23;

mod best_choice_body_block_24_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_24(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136628: move.w d6, -$e6(a6)
        e.step()?;
        let value = r.data(6, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1a_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0013662c: move.w d5, -$e4(a6)
        e.step()?;
        let value = r.data(5, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00136630: move.w -$e2(a6), -$e8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffff18_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00136636: bra.b $136676
        e.step()?;
        pc = 0x136676_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_24_unit::best_choice_body_block_24;

mod best_choice_body_block_25_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_25(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136638: move.w d5, d0
        e.step()?;
        let value = r.data(5, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0013663a: sub.w d6, d0
        e.step()?;
        let source = r.data(6, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0013663c: cmp.w -$e4(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1c_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00136640: ble.b $136676
        e.step()?;
        pc = if r.less_equal()? {
            0x136676_u32
        } else {
            0x136642_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_25_unit::best_choice_body_block_25;

mod best_choice_body_block_26_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_26(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136642: cmp.w d5, d6
        e.step()?;
        let source = r.data(5, 16)?;
        let destination = r.data(6, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00136644: bge.b $136676
        e.step()?;
        pc = if r.greater_equal()? {
            0x136676_u32
        } else {
            0x136646_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_26_unit::best_choice_body_block_26;

mod best_choice_body_block_27_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_27(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136646: move.w d6, -$e6(a6)
        e.step()?;
        let value = r.data(6, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1a_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0013664a: move.w d5, -$e4(a6)
        e.step()?;
        let value = r.data(5, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0013664e: move.w -$e2(a6), -$e8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffff18_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00136654: bra.b $136676
        e.step()?;
        pc = 0x136676_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_27_unit::best_choice_body_block_27;

mod best_choice_body_block_28_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_28(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136656: cmp.w -$e4(a6), d5
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1c_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0013665a: blt.b $136676
        e.step()?;
        pc = if r.less()? {
            0x136676_u32
        } else {
            0x13665c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_28_unit::best_choice_body_block_28;

mod best_choice_body_block_29_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_29(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013665c: cmp.w -$e6(a6), d6
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1a_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(6, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00136660: ble.b $136668
        e.step()?;
        pc = if r.less_equal()? {
            0x136668_u32
        } else {
            0x136662_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_29_unit::best_choice_body_block_29;
mod best_choice_body_block_2a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_2a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136662: cmp.w -$e4(a6), d5
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff1c_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00136666: ble.b $136676
        e.step()?;
        pc = if r.less_equal()? {
            0x136676_u32
        } else {
            0x136668_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_2a_unit::best_choice_body_block_2a;

mod best_choice_body_block_2b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_2b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136668: move.w d6, -$e6(a6)
        e.step()?;
        let value = r.data(6, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1a_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0013666c: move.w d5, -$e4(a6)
        e.step()?;
        let value = r.data(5, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00136670: move.w -$e2(a6), -$e8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffff18_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x136676_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_2b_unit::best_choice_body_block_2b;

mod best_choice_body_block_2c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_2c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136676: move.w $4434b6.l, $4434b6.l
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x136680_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_2c_unit::best_choice_body_block_2c;

mod best_choice_body_block_2d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_2d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136680: addq.w #$1, -$e2(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x136684_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_2d_unit::best_choice_body_block_2d;

mod best_choice_body_block_2e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_2e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00136684: cmpi.w #$6e, -$e2(a6)
        e.step()?;
        let source = 0x6e_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffff1e_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0013668a: blt.w $13640a
        e.step()?;
        pc = if r.less()? {
            0x13640a_u32
        } else {
            0x13668e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_2e_unit::best_choice_body_block_2e;

mod best_choice_body_block_2f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_2f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0013668e: move.w -$e8(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff18_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00136692: movem.l (a7)+, d3-d7
        e.step()?;
        r.restore_register(e, 3)?;
        r.restore_register(e, 4)?;
        r.restore_register(e, 5)?;
        r.restore_register(e, 6)?;
        r.restore_register(e, 7)?;
        // 00136696: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 00136698: rts
        e.step()?;
        return finish(e, r).map(|_| std::ops::ControlFlow::Break(()));

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_2f_unit::best_choice_body_block_2f;

mod best_choice_body_block_30_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn best_choice_body_block_30(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        return Err(RuleError::Invalid(format!(
            "source transition outside recovered body: {pc:08x}"
        ))
        .into());

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use best_choice_body_block_30_unit::best_choice_body_block_30;

mod direction_body_block_0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00151f30: link.w a6, #$ffea
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, -22)?;
        // 00151f34: pea.l $8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0x8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f38: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f3c: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151f40: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00151f42: move.l -$16(a6), -$4(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00151f48: pea.l $12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0x12_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f4c: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f50: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151f54: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00151f56: move.l -$16(a6), -$8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00151f5c: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00151f60: pea.l -$4(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f64: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f68: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151f6c: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00151f6e: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f72: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f76: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151f7a: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00151f7c: ble.b $151fa2
        e.step()?;
        pc = if r.less_equal()? {
            0x151fa2_u32
        } else {
            0x151f7e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_0_unit::direction_body_block_0;

mod direction_body_block_1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00151f7e: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00151f82: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f86: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f8a: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151f8e: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00151f90: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f94: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151f98: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151f9c: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00151f9e: bge.b $151fa2
        e.step()?;
        pc = if r.greater_equal()? {
            0x151fa2_u32
        } else {
            0x151fa0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_1_unit::direction_body_block_1;

mod direction_body_block_2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00151fa0: moveq #$1, d1
        e.step()?;
        r.put_data(1, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        pc = 0x151fa2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_2_unit::direction_body_block_2;

mod direction_body_block_3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00151fa2: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00151fa6: pea.l -$4(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151faa: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151fae: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151fb2: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00151fb4: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151fb8: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151fbc: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151fc0: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00151fc2: ble.b $151fe2
        e.step()?;
        pc = if r.less_equal()? {
            0x151fe2_u32
        } else {
            0x151fc4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_3_unit::direction_body_block_3;

mod direction_body_block_4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00151fc4: clr.l -$12(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00151fc8: clr.l -$e(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00151fcc: clr.w -$a(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151fd0: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151fd4: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151fd8: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151fdc: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00151fde: bne.b $151fe2
        e.step()?;
        pc = if r.not_equal()? {
            0x151fe2_u32
        } else {
            0x151fe0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_4_unit::direction_body_block_4;

mod direction_body_block_5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00151fe0: moveq #$2, d1
        e.step()?;
        r.put_data(1, 32, 0x2_u32);
        r.move_flags(0x2_u32, 32);
        pc = 0x151fe2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_5_unit::direction_body_block_5;

mod direction_body_block_6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00151fe2: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00151fe6: pea.l -$4(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151fea: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151fee: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00151ff2: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00151ff4: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151ff8: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00151ffc: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152000: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152002: ble.b $152028
        e.step()?;
        pc = if r.less_equal()? {
            0x152028_u32
        } else {
            0x152004_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_6_unit::direction_body_block_6;

mod direction_body_block_7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152004: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00152008: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015200c: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152010: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152014: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152016: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015201a: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015201e: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152022: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152024: ble.b $152028
        e.step()?;
        pc = if r.less_equal()? {
            0x152028_u32
        } else {
            0x152026_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_7_unit::direction_body_block_7;

mod direction_body_block_8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152026: moveq #$3, d1
        e.step()?;
        r.put_data(1, 32, 0x3_u32);
        r.move_flags(0x3_u32, 32);
        pc = 0x152028_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_8_unit::direction_body_block_8;

mod direction_body_block_9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152028: clr.l -$12(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0015202c: clr.l -$e(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00152030: clr.w -$a(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152034: pea.l -$4(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152038: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015203c: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152040: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152042: bne.b $152068
        e.step()?;
        pc = if r.not_equal()? {
            0x152068_u32
        } else {
            0x152044_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_9_unit::direction_body_block_9;
mod direction_body_block_a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152044: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00152048: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015204c: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152050: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152054: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152056: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015205a: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015205e: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152062: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152064: ble.b $152068
        e.step()?;
        pc = if r.less_equal()? {
            0x152068_u32
        } else {
            0x152066_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_a_unit::direction_body_block_a;

mod direction_body_block_b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152066: moveq #$4, d1
        e.step()?;
        r.put_data(1, 32, 0x4_u32);
        r.move_flags(0x4_u32, 32);
        pc = 0x152068_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_b_unit::direction_body_block_b;

mod direction_body_block_c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152068: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0015206c: pea.l -$4(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152070: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152074: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152078: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0015207a: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015207e: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152082: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152086: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152088: bge.b $1520ae
        e.step()?;
        pc = if r.greater_equal()? {
            0x1520ae_u32
        } else {
            0x15208a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_c_unit::direction_body_block_c;

mod direction_body_block_d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0015208a: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0015208e: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152092: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152096: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0015209a: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0015209c: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001520a0: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001520a4: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001520a8: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001520aa: ble.b $1520ae
        e.step()?;
        pc = if r.less_equal()? {
            0x1520ae_u32
        } else {
            0x1520ac_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_d_unit::direction_body_block_d;

mod direction_body_block_e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001520ac: moveq #$5, d1
        e.step()?;
        r.put_data(1, 32, 0x5_u32);
        r.move_flags(0x5_u32, 32);
        pc = 0x1520ae_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_e_unit::direction_body_block_e;

mod direction_body_block_f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001520ae: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001520b2: pea.l -$4(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001520b6: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001520ba: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001520be: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001520c0: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001520c4: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001520c8: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001520cc: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001520ce: bge.b $1520ee
        e.step()?;
        pc = if r.greater_equal()? {
            0x1520ee_u32
        } else {
            0x1520d0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_f_unit::direction_body_block_f;

mod direction_body_block_10_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_10(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001520d0: clr.l -$12(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001520d4: clr.l -$e(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001520d8: clr.w -$a(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001520dc: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001520e0: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001520e4: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001520e8: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001520ea: bne.b $1520ee
        e.step()?;
        pc = if r.not_equal()? {
            0x1520ee_u32
        } else {
            0x1520ec_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_10_unit::direction_body_block_10;

mod direction_body_block_11_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_11(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001520ec: moveq #$6, d1
        e.step()?;
        r.put_data(1, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        pc = 0x1520ee_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_11_unit::direction_body_block_11;

mod direction_body_block_12_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_12(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001520ee: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001520f2: pea.l -$4(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001520f6: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001520fa: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001520fe: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152100: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152104: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152108: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0015210c: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0015210e: bge.b $152134
        e.step()?;
        pc = if r.greater_equal()? {
            0x152134_u32
        } else {
            0x152110_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_12_unit::direction_body_block_12;

mod direction_body_block_13_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_13(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152110: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00152114: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152118: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015211c: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152120: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152122: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152126: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015212a: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0015212e: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152130: bge.b $152134
        e.step()?;
        pc = if r.greater_equal()? {
            0x152134_u32
        } else {
            0x152132_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_13_unit::direction_body_block_13;

mod direction_body_block_14_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_14(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152132: moveq #$7, d1
        e.step()?;
        r.put_data(1, 32, 0x7_u32);
        r.move_flags(0x7_u32, 32);
        pc = 0x152134_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_14_unit::direction_body_block_14;

mod direction_body_block_15_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_15(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152134: clr.l -$12(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00152138: clr.l -$e(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0015213c: clr.w -$a(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152140: pea.l -$4(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152144: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152148: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0015214c: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0015214e: bne.b $152174
        e.step()?;
        pc = if r.not_equal()? {
            0x152174_u32
        } else {
            0x152150_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_15_unit::direction_body_block_15;

mod direction_body_block_16_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_16(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152150: clr.l -$16(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00152154: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152158: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015215c: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00152160: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152162: pea.l -$16(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffea_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00152166: pea.l -$12(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffee_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0015216a: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0015216e: MacToolbox a9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00152170: bge.b $152174
        e.step()?;
        pc = if r.greater_equal()? {
            0x152174_u32
        } else {
            0x152172_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_16_unit::direction_body_block_16;

mod direction_body_block_17_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_17(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152172: moveq #$0, d1
        e.step()?;
        r.put_data(1, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        pc = 0x152174_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_17_unit::direction_body_block_17;

mod direction_body_block_18_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_18(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00152174: move.b d1, d0
        e.step()?;
        let value = r.data(1, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00152176: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00152178: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 0015217a: rts
        e.step()?;
        return finish(e, r).map(|_| std::ops::ControlFlow::Break(()));

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_18_unit::direction_body_block_18;

mod direction_body_block_19_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn direction_body_block_19(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        return Err(RuleError::Invalid(format!(
            "source transition outside recovered body: {pc:08x}"
        ))
        .into());

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use direction_body_block_19_unit::direction_body_block_19;
