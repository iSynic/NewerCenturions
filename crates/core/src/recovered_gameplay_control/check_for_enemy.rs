//! Complete provisional gameplay control from canonical CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{ApplicationControlServices, Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};

pub(crate) fn check_for_enemy_body(
    e: &mut Execution<'_, impl ApplicationControlServices>,
    frame: u32,
) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x110e96_u32;
        loop {
            match pc {
                0x110e96 => {
                    // 00110e96: link.w a6, #$fff6
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -10)?;
                    // 00110e9a: movem.l d3-d7, -(a7)
                    e.step()?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 00110e9e: move.w $8(a6), d6
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 00110ea2: move.w #$c, -(a7)
                    e.step()?;
                    let value = 0xc_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00110ea6: bsr.l $13154c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x110eac_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13154c_u32)?;
                    // 00110eac: clr.b -$5(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00110eb0: clr.b -$8(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00110eb4: clr.b -$6(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00110eb8: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00110eba: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00110ebc: move.w d0, $4434c2.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c2_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00110ec2: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 00110ec8: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110ece: tst.w (a0, d0.w * 2)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00110ed2: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00110ed4: bge.b $110edc
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x110edc_u32
                    } else {
                        0x110ed6_u32
                    };
                }
                0x110ed6 => {
                    // 00110ed6: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00110ed8: bra.w $111766
                    e.step()?;
                    pc = 0x111766_u32;
                }
                0x110edc => {
                    // 00110edc: move.w $44374c.l, d0
                    e.step()?;
                    let value_address = 0x44374c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110ee2: cmp.w $4435c4.l, d0
                    e.step()?;
                    let source_address = 0x4435c4_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00110ee8: bne.b $110ef0
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x110ef0_u32
                    } else {
                        0x110eea_u32
                    };
                }
                0x110eea => {
                    // 00110eea: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00110eec: bra.w $111766
                    e.step()?;
                    pc = 0x111766_u32;
                }
                0x110ef0 => {
                    // 00110ef0: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 00110ef6: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110efc: cmpi.w #$9, (a0, d0.w * 2)
                    e.step()?;
                    let source = 0x9_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00110f02: ble.b $110f4e
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x110f4e_u32
                    } else {
                        0x110f04_u32
                    };
                }
                0x110f04 => {
                    // 00110f04: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 00110f0a: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110f10: move.w (a0, d0.w * 4), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110f14: move.b d0, -$3(a6)
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffd_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00110f18: lea.l $446f0a.l, a0
                    e.step()?;
                    let value = 0x446f0a_u32;
                    r.put_address(0, value);
                    // 00110f1e: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110f24: move.w $2(a0, d0.w * 4), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110f28: move.b d0, -$4(a6)
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00110f2c: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110f32: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110f36: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00110f3c: move.w $10(a0, d0.l), -(a7)
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
                    // 00110f40: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x110f46_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00110f46: move.b d0, -$8(a6)
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00110f4a: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00110f4c: bra.b $110f76
                    e.step()?;
                    pc = 0x110f76_u32;
                }
                0x110f4e => {
                    // 00110f4e: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 00110f54: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110f5a: move.w (a0, d0.w * 4), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110f5e: move.b d0, -$3(a6)
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffd_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00110f62: lea.l $44709a.l, a0
                    e.step()?;
                    let value = 0x44709a_u32;
                    r.put_address(0, value);
                    // 00110f68: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110f6e: move.w $2(a0, d0.w * 4), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110f72: move.b d0, -$4(a6)
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x110f76_u32;
                }
                0x110f76 => {
                    // 00110f76: cmpi.w #$2, d6
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00110f7a: bne.b $110f90
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x110f90_u32
                    } else {
                        0x110f7c_u32
                    };
                }
                0x110f7c => {
                    // 00110f7c: move.b $442736.l, d0
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00110f82: add.b d0, -$3(a6)
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffd_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.add_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    // 00110f86: move.b $442737.l, d0
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00110f8c: add.b d0, -$4(a6)
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.add_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    pc = 0x110f90_u32;
                }
                0x110f90 => {
                    // 00110f90: clr.b -$1(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00110f94: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00110f96: bra.b $110fa8
                    e.step()?;
                    pc = 0x110fa8_u32;
                }
                0x110f98 => {
                    // 00110f98: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00110f9a: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110f9c: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 00110fa2: clr.w (a0, d0.l * 2)
                    e.step()?;
                    let value = 0;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00110fa6: addq.b #$1, d5
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(5, 8)?;
                    let value = r.add_flags(destination, source, 8, true);
                    r.put_data(5, 8, value);
                    pc = 0x110fa8_u32;
                }
                0x110fa8 => {
                    // 00110fa8: cmpi.b #$c, d5
                    e.step()?;
                    let source = 0xc_u32;
                    let destination = r.data(5, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 00110fac: blt.b $110f98
                    e.step()?;
                    pc = if r.less()? {
                        0x110f98_u32
                    } else {
                        0x110fae_u32
                    };
                }
                0x110fae => {
                    // 00110fae: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00110fb0: bra.b $110fca
                    e.step()?;
                    pc = 0x110fca_u32;
                }
                0x110fb2 => {
                    // 00110fb2: move.w d6, d0
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00110fb4: mulu.w #$dc, d0
                    e.step()?;
                    let source = 0xdc_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110fb8: lea.l $442934.l, a0
                    e.step()?;
                    let value = 0x442934_u32;
                    r.put_address(0, value);
                    // 00110fbe: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00110fc0: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00110fc2: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00110fc4: clr.w (a0, d0.l * 2)
                    e.step()?;
                    let value = 0;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00110fc8: addq.b #$1, d5
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(5, 8)?;
                    let value = r.add_flags(destination, source, 8, true);
                    r.put_data(5, 8, value);
                    pc = 0x110fca_u32;
                }
                0x110fca => {
                    // 00110fca: cmpi.b #$6e, d5
                    e.step()?;
                    let source = 0x6e_u32;
                    let destination = r.data(5, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 00110fce: blt.b $110fb2
                    e.step()?;
                    pc = if r.less()? {
                        0x110fb2_u32
                    } else {
                        0x110fd0_u32
                    };
                }
                0x110fd0 => {
                    // 00110fd0: move.b #$ff, -$9(a6)
                    e.step()?;
                    let value = 0xff_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff7_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00110fd6: move.b #$ff, -$7(a6)
                    e.step()?;
                    let value = 0xff_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff9_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00110fdc: cmpi.b #$1, -$8(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00110fe2: beq.b $110fec
                    e.step()?;
                    pc = if r.equal()? {
                        0x110fec_u32
                    } else {
                        0x110fe4_u32
                    };
                }
                0x110fe4 => {
                    // 00110fe4: cmpi.b #$3, -$8(a6)
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00110fea: bne.b $110ff2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x110ff2_u32
                    } else {
                        0x110fec_u32
                    };
                }
                0x110fec => {
                    // 00110fec: move.b #$fe, -$9(a6)
                    e.step()?;
                    let value = 0xfe_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff7_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x110ff2_u32;
                }
                0x110ff2 => {
                    // 00110ff2: cmpi.b #$2, -$8(a6)
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00110ff8: beq.b $111002
                    e.step()?;
                    pc = if r.equal()? {
                        0x111002_u32
                    } else {
                        0x110ffa_u32
                    };
                }
                0x110ffa => {
                    // 00110ffa: cmpi.b #$3, -$8(a6)
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00111000: bne.b $111008
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x111008_u32
                    } else {
                        0x111002_u32
                    };
                }
                0x111002 => {
                    // 00111002: move.b #$fe, -$7(a6)
                    e.step()?;
                    let value = 0xfe_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff9_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x111008_u32;
                }
                0x111008 => {
                    // 00111008: move.b -$9(a6), -$2(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff7_u32);
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0011100e: bra.w $11121e
                    e.step()?;
                    pc = 0x11121e_u32;
                }
                0x111012 => {
                    // 00111012: move.b -$7(a6), d5
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff9_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(5, 8, value);
                    r.move_flags(value, 8);
                    // 00111016: bra.w $111212
                    e.step()?;
                    pc = 0x111212_u32;
                }
                0x11101a => {
                    // 0011101a: move.b -$3(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffd_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0011101e: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111020: ext.l d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111022: add.l $442780.l, d0
                    e.step()?;
                    let source_address = 0x442780_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00111028: move.b d5, d1
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0011102a: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0011102c: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0011102e: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 00111030: muls.l #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 00111038: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 0011103e: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111040: move.b -$4(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00111044: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111046: ext.l d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111048: add.l $442784.l, d0
                    e.step()?;
                    let source_address = 0x442784_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0011104e: move.b -$2(a6), d1
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 00111052: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00111054: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00111056: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 00111058: move.w (a0, d1.l * 2), -(a7)
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
                    // 0011105c: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x111062_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00111062: move.w d0, d3
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(3, 16, value);
                    r.move_flags(value, 16);
                    // 00111064: cmpi.w #$6e, d3
                    e.step()?;
                    let source = 0x6e_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00111068: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0011106a: bge.w $111210
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x111210_u32
                    } else {
                        0x11106e_u32
                    };
                }
                0x11106e => {
                    // 0011106e: cmpi.w #$ffff, d3
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00111072: ble.w $111210
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x111210_u32
                    } else {
                        0x111076_u32
                    };
                }
                0x111076 => {
                    // 00111076: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00111078: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 0011107e: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111084: cmpi.w #$9, (a0, d0.w * 2)
                    e.step()?;
                    let source = 0x9_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0011108a: bge.w $111120
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x111120_u32
                    } else {
                        0x11108e_u32
                    };
                }
                0x11108e => {
                    // 0011108e: cmpi.w #$9, d3
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00111092: bge.b $1110d8
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x1110d8_u32
                    } else {
                        0x111094_u32
                    };
                }
                0x111094 => {
                    // 00111094: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0011109a: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011109e: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 001110a4: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001110a6: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001110aa: lea.l $448aa0.l, a1
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(1, value);
                    // 001110b0: move.w $10(a1, d1.l), d1
                    e.step()?;
                    let value_address = r
                        .address(1)?
                        .wrapping_add(0x10_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001110b4: cmp.w $10(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x10_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001110b8: beq.w $1111d0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1111d0_u32
                    } else {
                        0x1110bc_u32
                    };
                }
                0x1110bc => {
                    // 001110bc: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001110be: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001110c2: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 001110c8: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001110ca: tst.b $9df(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9df_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001110ce: beq.w $1111d0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1111d0_u32
                    } else {
                        0x1110d2_u32
                    };
                }
                0x1110d2 => {
                    // 001110d2: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 001110d4: bra.w $1111d0
                    e.step()?;
                    pc = 0x1111d0_u32;
                }
                0x1110d8 => {
                    // 001110d8: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001110de: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001110e2: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 001110e8: moveq #$f6, d1
                    e.step()?;
                    r.put_data(1, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 001110ea: add.w d3, d1
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 001110ec: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001110f0: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 001110f6: move.w $e(a1, d1.l), d1
                    e.step()?;
                    let value_address = r
                        .address(1)?
                        .wrapping_add(0xe_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001110fa: cmp.w $10(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x10_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001110fe: beq.w $1111d0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1111d0_u32
                    } else {
                        0x111102_u32
                    };
                }
                0x111102 => {
                    // 00111102: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00111104: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00111106: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011110a: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00111110: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111112: tst.w $110(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x110_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00111116: ble.w $1111d0
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1111d0_u32
                    } else {
                        0x11111a_u32
                    };
                }
                0x11111a => {
                    // 0011111a: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 0011111c: bra.w $1111d0
                    e.step()?;
                    pc = 0x1111d0_u32;
                }
                0x111120 => {
                    // 00111120: cmpi.w #$9, d3
                    e.step()?;
                    let source = 0x9_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00111124: bge.b $111166
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x111166_u32
                    } else {
                        0x111126_u32
                    };
                }
                0x111126 => {
                    // 00111126: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111128: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011112c: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00111132: move.w $447196.l, d1
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00111138: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0011113c: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 00111142: move.w $e(a1, d1.l), d1
                    e.step()?;
                    let value_address = r
                        .address(1)?
                        .wrapping_add(0xe_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00111146: cmp.w $10(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x10_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0011114a: beq.w $1111d0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1111d0_u32
                    } else {
                        0x11114e_u32
                    };
                }
                0x11114e => {
                    // 0011114e: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111150: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111154: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0011115a: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0011115c: tst.b $9df(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9df_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00111160: beq.b $1111d0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1111d0_u32
                    } else {
                        0x111162_u32
                    };
                }
                0x111162 => {
                    // 00111162: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 00111164: bra.b $1111d0
                    e.step()?;
                    pc = 0x1111d0_u32;
                }
                0x111166 => {
                    // 00111166: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00111168: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0011116a: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011116e: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00111174: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111176: cmpi.b #$ff, $b0(a0)
                    e.step()?;
                    let source = 0xff_u32;
                    let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 0011117c: beq.b $1111d0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1111d0_u32
                    } else {
                        0x11117e_u32
                    };
                }
                0x11117e => {
                    // 0011117e: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00111180: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00111182: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111186: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0011118c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0011118e: tst.w $110(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x110_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00111192: ble.b $1111d0
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1111d0_u32
                    } else {
                        0x111194_u32
                    };
                }
                0x111194 => {
                    // 00111194: moveq #$f6, d0
                    e.step()?;
                    r.put_data(0, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00111196: add.w d3, d0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00111198: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011119c: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 001111a2: move.w $447196.l, d1
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001111a8: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001111ac: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 001111b2: move.w $e(a1, d1.l), d1
                    e.step()?;
                    let value_address = r
                        .address(1)?
                        .wrapping_add(0xe_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001111b6: cmp.w $e(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0xe_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001111ba: beq.b $1111d0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1111d0_u32
                    } else {
                        0x1111bc_u32
                    };
                }
                0x1111bc => {
                    // 001111bc: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 001111c2: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001111c8: cmp.w (a0, d0.w * 2), d3
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001111cc: beq.b $1111d0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1111d0_u32
                    } else {
                        0x1111ce_u32
                    };
                }
                0x1111ce => {
                    // 001111ce: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x1111d0_u32;
                }
                0x1111d0 => {
                    // 001111d0: tst.b d4
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.move_flags(value, 8);
                    // 001111d2: beq.b $111210
                    e.step()?;
                    pc = if r.equal()? {
                        0x111210_u32
                    } else {
                        0x1111d4_u32
                    };
                }
                0x1111d4 => {
                    // 001111d4: move.w d6, d0
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001111d6: mulu.w #$dc, d0
                    e.step()?;
                    let source = 0xdc_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001111da: lea.l $442934.l, a0
                    e.step()?;
                    let value = 0x442934_u32;
                    r.put_address(0, value);
                    // 001111e0: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001111e2: tst.w (a0, d3.w * 2)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001111e6: bne.b $111210
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x111210_u32
                    } else {
                        0x1111e8_u32
                    };
                }
                0x1111e8 => {
                    // 001111e8: move.b -$1(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001111ec: addq.b #$1, -$1(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.add_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    // 001111f0: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001111f2: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 001111f8: move.w d3, (a0, d0.l * 2)
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001111fc: move.w d6, d0
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001111fe: mulu.w #$dc, d0
                    e.step()?;
                    let source = 0xdc_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111202: lea.l $442934.l, a0
                    e.step()?;
                    let value = 0x442934_u32;
                    r.put_address(0, value);
                    // 00111208: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0011120a: move.w #$1, (a0, d3.w * 2)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x111210_u32;
                }
                0x111210 => {
                    // 00111210: addq.b #$1, d5
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(5, 8)?;
                    let value = r.add_flags(destination, source, 8, true);
                    r.put_data(5, 8, value);
                    pc = 0x111212_u32;
                }
                0x111212 => {
                    // 00111212: cmpi.b #$2, d5
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(5, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 00111216: blt.w $11101a
                    e.step()?;
                    pc = if r.less()? {
                        0x11101a_u32
                    } else {
                        0x11121a_u32
                    };
                }
                0x11121a => {
                    // 0011121a: addq.b #$1, -$2(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.add_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    pc = 0x11121e_u32;
                }
                0x11121e => {
                    // 0011121e: cmpi.b #$2, -$2(a6)
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00111224: blt.w $111012
                    e.step()?;
                    pc = if r.less()? {
                        0x111012_u32
                    } else {
                        0x111228_u32
                    };
                }
                0x111228 => {
                    // 00111228: cmpi.w #$1, d6
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0011122c: bne.b $111238
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x111238_u32
                    } else {
                        0x11122e_u32
                    };
                }
                0x11122e => {
                    // 0011122e: move.b -$1(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00111232: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111234: bra.w $111766
                    e.step()?;
                    pc = 0x111766_u32;
                }
                0x111238 => {
                    // 00111238: cmpi.w #$2, d6
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0011123c: bne.w $11136e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x11136e_u32
                    } else {
                        0x111240_u32
                    };
                }
                0x111240 => {
                    // 00111240: clr.b -$1(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00111244: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 0011124a: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111250: cmpi.w #$9, (a0, d0.w * 2)
                    e.step()?;
                    let source = 0x9_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00111256: ble.b $1112d2
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1112d2_u32
                    } else {
                        0x111258_u32
                    };
                }
                0x111258 => {
                    // 00111258: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0011125e: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111262: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00111268: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0011126a: tst.b $ca(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xca_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0011126e: bne.w $11136e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x11136e_u32
                    } else {
                        0x111272_u32
                    };
                }
                0x111272 => {
                    // 00111272: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00111274: bra.b $1112b8
                    e.step()?;
                    pc = 0x1112b8_u32;
                }
                0x111276 => {
                    // 00111276: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00111278: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011127a: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0011127c: lea.l $442934.l, a0
                    e.step()?;
                    let value = 0x442934_u32;
                    r.put_address(0, value);
                    // 00111282: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111284: tst.w $dc(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xdc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00111288: beq.b $1112b6
                    e.step()?;
                    pc = if r.equal()? {
                        0x1112b6_u32
                    } else {
                        0x11128a_u32
                    };
                }
                0x11128a => {
                    // 0011128a: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0011128c: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011128e: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00111290: lea.l $442934.l, a0
                    e.step()?;
                    let value = 0x442934_u32;
                    r.put_address(0, value);
                    // 00111296: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111298: tst.w $1b8(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x1b8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0011129c: bne.b $1112b6
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1112b6_u32
                    } else {
                        0x11129e_u32
                    };
                }
                0x11129e => {
                    // 0011129e: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001112a0: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001112a2: move.b -$1(a6), d1
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 001112a6: addq.b #$1, -$1(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.add_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    // 001112aa: extb.l d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001112ac: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 001112b2: move.w d0, (a0, d1.l * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x1112b6_u32;
                }
                0x1112b6 => {
                    // 001112b6: addq.b #$1, d5
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(5, 8)?;
                    let value = r.add_flags(destination, source, 8, true);
                    r.put_data(5, 8, value);
                    pc = 0x1112b8_u32;
                }
                0x1112b8 => {
                    // 001112b8: cmpi.b #$6e, d5
                    e.step()?;
                    let source = 0x6e_u32;
                    let destination = r.data(5, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 001112bc: blt.b $111276
                    e.step()?;
                    pc = if r.less()? {
                        0x111276_u32
                    } else {
                        0x1112be_u32
                    };
                }
                0x1112be => {
                    // 001112be: tst.b -$1(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001112c2: beq.w $11136e
                    e.step()?;
                    pc = if r.equal()? {
                        0x11136e_u32
                    } else {
                        0x1112c6_u32
                    };
                }
                0x1112c6 => {
                    // 001112c6: move.w #$1, $4434c2.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x4434c2_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001112ce: bra.w $11136e
                    e.step()?;
                    pc = 0x11136e_u32;
                }
                0x1112d2 => {
                    // 001112d2: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 001112d8: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001112de: cmpi.w #$9, (a0, d0.w * 2)
                    e.step()?;
                    let source = 0x9_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 001112e4: bge.w $11136e
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x11136e_u32
                    } else {
                        0x1112e8_u32
                    };
                }
                0x1112e8 => {
                    // 001112e8: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 001112ee: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001112f4: cmpi.w #$ffff, (a0, d0.w * 2)
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 001112fa: ble.b $11136e
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x11136e_u32
                    } else {
                        0x1112fc_u32
                    };
                }
                0x1112fc => {
                    // 001112fc: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111302: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111306: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0011130c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0011130e: tst.w $8e(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8e_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00111312: bne.b $11136e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x11136e_u32
                    } else {
                        0x111314_u32
                    };
                }
                0x111314 => {
                    // 00111314: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00111316: bra.b $11135a
                    e.step()?;
                    pc = 0x11135a_u32;
                }
                0x111318 => {
                    // 00111318: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0011131a: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011131c: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0011131e: lea.l $442934.l, a0
                    e.step()?;
                    let value = 0x442934_u32;
                    r.put_address(0, value);
                    // 00111324: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111326: tst.w $dc(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xdc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0011132a: beq.b $111358
                    e.step()?;
                    pc = if r.equal()? {
                        0x111358_u32
                    } else {
                        0x11132c_u32
                    };
                }
                0x11132c => {
                    // 0011132c: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0011132e: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111330: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00111332: lea.l $442934.l, a0
                    e.step()?;
                    let value = 0x442934_u32;
                    r.put_address(0, value);
                    // 00111338: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0011133a: tst.w $1b8(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x1b8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0011133e: bne.b $111358
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x111358_u32
                    } else {
                        0x111340_u32
                    };
                }
                0x111340 => {
                    // 00111340: move.b d5, d0
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00111342: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111344: move.b -$1(a6), d1
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 00111348: addq.b #$1, -$1(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.add_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    // 0011134c: extb.l d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0011134e: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 00111354: move.w d0, (a0, d1.l * 2)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x111358_u32;
                }
                0x111358 => {
                    // 00111358: addq.b #$1, d5
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(5, 8)?;
                    let value = r.add_flags(destination, source, 8, true);
                    r.put_data(5, 8, value);
                    pc = 0x11135a_u32;
                }
                0x11135a => {
                    // 0011135a: cmpi.b #$6e, d5
                    e.step()?;
                    let source = 0x6e_u32;
                    let destination = r.data(5, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 0011135e: blt.b $111318
                    e.step()?;
                    pc = if r.less()? {
                        0x111318_u32
                    } else {
                        0x111360_u32
                    };
                }
                0x111360 => {
                    // 00111360: tst.b -$1(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00111364: beq.b $11136e
                    e.step()?;
                    pc = if r.equal()? {
                        0x11136e_u32
                    } else {
                        0x111366_u32
                    };
                }
                0x111366 => {
                    // 00111366: move.w #$1, $4434c2.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x4434c2_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x11136e_u32;
                }
                0x11136e => {
                    // 0011136e: moveq #$0, d7
                    e.step()?;
                    r.put_data(7, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00111370: bra.w $11158a
                    e.step()?;
                    pc = 0x11158a_u32;
                }
                0x111374 => {
                    // 00111374: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00111376: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111378: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 0011137e: cmpi.w #$9, (a0, d0.l * 2)
                    e.step()?;
                    let source = 0x9_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00111384: bge.w $111456
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x111456_u32
                    } else {
                        0x111388_u32
                    };
                }
                0x111388 => {
                    // 00111388: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0011138a: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011138c: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 00111392: cmpi.w #$ffff, (a0, d0.l * 2)
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00111398: ble.w $111456
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x111456_u32
                    } else {
                        0x11139c_u32
                    };
                }
                0x11139c => {
                    // 0011139c: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0011139e: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001113a0: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 001113a6: move.w (a0, d0.l * 2), d0
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001113aa: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001113ae: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 001113b4: tst.w $16(a0, d0.l)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x16_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001113b8: bne.b $1113c2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1113c2_u32
                    } else {
                        0x1113ba_u32
                    };
                }
                0x1113ba => {
                    // 001113ba: cmpi.w #$2, d6
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001113be: bne.w $111588
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x111588_u32
                    } else {
                        0x1113c2_u32
                    };
                }
                0x1113c2 => {
                    // 001113c2: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001113c4: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001113c6: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 001113cc: move.w (a0, d0.l * 2), d0
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001113d0: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001113d4: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 001113da: tst.w $60(a0, d0.l)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x60_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001113de: bne.w $111588
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x111588_u32
                    } else {
                        0x1113e2_u32
                    };
                }
                0x1113e2 => {
                    // 001113e2: move.b #$1, -$5(a6)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 001113e8: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001113ea: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001113ec: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 001113f2: move.w (a0, d0.l * 2), -(a7)
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
                    // 001113f6: bsr.l $11f1a8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1113fc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x11f1a8_u32)?;
                    // 001113fc: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001113fe: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111400: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 00111406: move.w (a0, d0.l * 2), d0
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0011140a: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011140e: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00111414: clr.w $16(a0, d0.l)
                    e.step()?;
                    let value = 0;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x16_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00111418: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 0011141e: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111424: move.w (a0, d0.w * 2), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00111428: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0011142a: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011142c: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 00111432: move.w (a0, d0.l * 2), -(a7)
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
                    // 00111436: bsr.l $100af0
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x11143c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x100af0_u32)?;
                    // 0011143c: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0011143e: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00111440: beq.w $111588
                    e.step()?;
                    pc = if r.equal()? {
                        0x111588_u32
                    } else {
                        0x111444_u32
                    };
                }
                0x111444 => {
                    // 00111444: clr.b -$5(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00111448: clr.b -$1(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0011144c: move.b #$1, -$6(a6)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00111452: bra.w $111588
                    e.step()?;
                    pc = 0x111588_u32;
                }
                0x111456 => {
                    // 00111456: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00111458: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011145a: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 00111460: moveq #$f6, d1
                    e.step()?;
                    r.put_data(1, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00111462: add.w (a0, d0.l * 2), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00111466: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0011146a: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00111470: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111472: tst.b $104(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x104_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00111476: bne.b $111480
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x111480_u32
                    } else {
                        0x111478_u32
                    };
                }
                0x111478 => {
                    // 00111478: cmpi.w #$2, d6
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0011147c: bne.w $111588
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x111588_u32
                    } else {
                        0x111480_u32
                    };
                }
                0x111480 => {
                    // 00111480: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00111482: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111484: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 0011148a: moveq #$f6, d1
                    e.step()?;
                    r.put_data(1, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 0011148c: add.w (a0, d0.l * 2), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00111490: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00111494: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0011149a: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0011149c: tst.b $b3(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xb3_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001114a0: bne.w $111588
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x111588_u32
                    } else {
                        0x1114a4_u32
                    };
                }
                0x1114a4 => {
                    // 001114a4: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001114a6: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001114a8: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 001114ae: moveq #$f6, d1
                    e.step()?;
                    r.put_data(1, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 001114b0: add.w (a0, d0.l * 2), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 001114b4: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001114b8: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 001114be: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001114c0: cmpi.b #$ff, $b0(a0)
                    e.step()?;
                    let source = 0xff_u32;
                    let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 001114c6: beq.w $111588
                    e.step()?;
                    pc = if r.equal()? {
                        0x111588_u32
                    } else {
                        0x1114ca_u32
                    };
                }
                0x1114ca => {
                    // 001114ca: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001114d0: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001114d4: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 001114da: cmpi.w #$4, $34(a0, d0.l)
                    e.step()?;
                    let source = 0x4_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x34_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 001114e0: beq.b $1114fa
                    e.step()?;
                    pc = if r.equal()? {
                        0x1114fa_u32
                    } else {
                        0x1114e2_u32
                    };
                }
                0x1114e2 => {
                    // 001114e2: move.w $4435d2.l, d0
                    e.step()?;
                    let value_address = 0x4435d2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001114e8: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001114ec: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 001114f2: cmpi.w #$5, $34(a0, d0.l)
                    e.step()?;
                    let source = 0x5_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x34_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 001114f8: bne.b $11150e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x11150e_u32
                    } else {
                        0x1114fa_u32
                    };
                }
                0x1114fa => {
                    // 001114fa: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 00111500: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111506: cmpi.w #$6, (a0, d0.w * 2)
                    e.step()?;
                    let source = 0x6_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0011150c: ble.b $111588
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x111588_u32
                    } else {
                        0x11150e_u32
                    };
                }
                0x11150e => {
                    // 0011150e: move.b #$1, -$5(a6)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00111514: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00111516: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111518: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 0011151e: move.w (a0, d0.l * 2), -(a7)
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
                    // 00111522: bsr.l $11f1a8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x111528_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x11f1a8_u32)?;
                    // 00111528: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0011152a: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011152c: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 00111532: moveq #$f6, d1
                    e.step()?;
                    r.put_data(1, 32, 0xfffffff6_u32);
                    r.move_flags(0xfffffff6_u32, 32);
                    // 00111534: add.w (a0, d0.l * 2), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00111538: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0011153c: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00111542: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111544: clr.b $104(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x104_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00111548: clr.w $4435d4.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x4435d4_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0011154e: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00111550: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 00111556: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0011155c: move.w (a0, d0.w * 2), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00111560: move.b d7, d0
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00111562: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111564: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 0011156a: move.w (a0, d0.l * 2), -(a7)
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
                    // 0011156e: bsr.l $101eb6
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x111574_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x101eb6_u32)?;
                    // 00111574: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00111576: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00111578: beq.b $111588
                    e.step()?;
                    pc = if r.equal()? {
                        0x111588_u32
                    } else {
                        0x11157a_u32
                    };
                }
                0x11157a => {
                    // 0011157a: clr.b -$5(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0011157e: clr.b -$1(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00111582: move.b #$1, -$6(a6)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x111588_u32;
                }
                0x111588 => {
                    // 00111588: addq.b #$1, d7
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(7, 8)?;
                    let value = r.add_flags(destination, source, 8, true);
                    r.put_data(7, 8, value);
                    pc = 0x11158a_u32;
                }
                0x11158a => {
                    // 0011158a: cmp.b -$1(a6), d7
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let source = u32::from(e.b(source_address)?);
                    let destination = r.data(7, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 0011158e: blt.w $111374
                    e.step()?;
                    pc = if r.less()? {
                        0x111374_u32
                    } else {
                        0x111592_u32
                    };
                }
                0x111592 => {
                    // 00111592: tst.b -$5(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00111596: beq.b $1115b0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1115b0_u32
                    } else {
                        0x111598_u32
                    };
                }
                0x111598 => {
                    // 00111598: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 0011159e: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001115a4: move.w (a0, d0.w * 2), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001115a8: bsr.l $11f1a8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1115ae_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x11f1a8_u32)?;
                    // 001115ae: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x1115b0_u32;
                }
                0x1115b0 => {
                    // 001115b0: tst.b -$1(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001115b4: beq.w $111754
                    e.step()?;
                    pc = if r.equal()? {
                        0x111754_u32
                    } else {
                        0x1115b8_u32
                    };
                }
                0x1115b8 => {
                    // 001115b8: lea.l $4470b8.l, a0
                    e.step()?;
                    let value = 0x4470b8_u32;
                    r.put_address(0, value);
                    // 001115be: move.w $447194.l, d0
                    e.step()?;
                    let value_address = 0x447194_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001115c4: cmpi.w #$9, (a0, d0.w * 2)
                    e.step()?;
                    let source = 0x9_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 001115ca: ble.w $111754
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x111754_u32
                    } else {
                        0x1115ce_u32
                    };
                }
                0x1115ce => {
                    // 001115ce: cmpi.w #$2, d6
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001115d2: beq.w $111754
                    e.step()?;
                    pc = if r.equal()? {
                        0x111754_u32
                    } else {
                        0x1115d6_u32
                    };
                }
                0x1115d6 => {
                    // 001115d6: move.b -$1(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001115da: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001115dc: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001115de: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1115e4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 001115e4: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001115e6: lea.l $4434c4.l, a0
                    e.step()?;
                    let value = 0x4434c4_u32;
                    r.put_address(0, value);
                    // 001115ec: move.w $447196.l, d1
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001115f2: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001115f6: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 001115fc: adda.l d1, a1
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 001115fe: move.w (a0, d0.w * 2), $10c(a1)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(1)?.wrapping_add(0x10c_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00111604: clr.w $4435d4.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x4435d4_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0011160a: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0011160c: bra.w $111728
                    e.step()?;
                    pc = 0x111728_u32;
                }
                0x111610 => {
                    // 00111610: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111616: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0011161a: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00111620: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111622: subq.b #$2, $109(a0)
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x109_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.subtract_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    // 00111626: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0011162c: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111630: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00111636: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111638: tst.b $109(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x109_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0011163c: bge.b $111676
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x111676_u32
                    } else {
                        0x11163e_u32
                    };
                }
                0x11163e => {
                    // 0011163e: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111644: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111648: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0011164e: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111650: cmpi.b #$ff, $109(a0)
                    e.step()?;
                    let source = 0xff_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x109_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00111656: bne.w $111754
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x111754_u32
                    } else {
                        0x11165a_u32
                    };
                }
                0x11165a => {
                    // 0011165a: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111660: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111664: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0011166a: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0011166c: move.b #$1, $104(a0)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x104_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00111672: bra.w $111754
                    e.step()?;
                    pc = 0x111754_u32;
                }
                0x111676 => {
                    // 00111676: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0011167c: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111680: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00111686: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111688: subq.b #$3, $106(a0)
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x106_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.subtract_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    // 0011168c: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111692: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111696: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0011169c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0011169e: cmpi.b #$2, $109(a0)
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x109_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 001116a4: bge.b $1116ee
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x1116ee_u32
                    } else {
                        0x1116a6_u32
                    };
                }
                0x1116a6 => {
                    // 001116a6: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001116ac: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001116b0: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 001116b6: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001116b8: tst.b $109(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x109_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001116bc: ble.b $1116d8
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1116d8_u32
                    } else {
                        0x1116be_u32
                    };
                }
                0x1116be => {
                    // 001116be: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001116c4: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001116c8: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 001116ce: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001116d0: move.b #$1, $104(a0)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x104_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 001116d6: bra.b $1116ee
                    e.step()?;
                    pc = 0x1116ee_u32;
                }
                0x1116d8 => {
                    // 001116d8: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001116de: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001116e2: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 001116e8: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001116ea: clr.b $104(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x104_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x1116ee_u32;
                }
                0x1116ee => {
                    // 001116ee: move.w $4435d4.l, -(a7)
                    e.step()?;
                    let value_address = 0x4435d4_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001116f4: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001116fa: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001116fe: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00111704: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00111706: move.w $10c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x10c_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0011170a: moveq #$a, d0
                    e.step()?;
                    r.put_data(0, 32, 0xa_u32);
                    r.move_flags(0xa_u32, 32);
                    // 0011170c: add.w $447196.l, d0
                    e.step()?;
                    let source_address = 0x447196_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00111712: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00111714: bsr.l $101eb6
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x11171a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x101eb6_u32)?;
                    // 0011171a: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0011171c: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0011171e: bne.w $110edc
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x110edc_u32
                    } else {
                        0x111722_u32
                    };
                }
                0x111722 => {
                    // 00111722: addq.w #$1, $4435d4.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4435d4_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    pc = 0x111728_u32;
                }
                0x111728 => {
                    // 00111728: move.w $447196.l, d0
                    e.step()?;
                    let value_address = 0x447196_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0011172e: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00111732: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 00111738: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0011173a: move.b $8a(a0), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8a_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0011173e: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00111740: move.w d0, d1
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00111742: lsr.w #$8, d1
                    e.step()?;
                    let count = 0x8_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(1, 16, value);
                    // 00111744: lsr.w #$7, d1
                    e.step()?;
                    let count = 0x7_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(1, 16, value);
                    // 00111746: add.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 00111748: asr.w #$1, d1
                    e.step()?;
                    let count = 0x1_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(1, 16, value);
                    // 0011174a: cmp.w $4435d4.l, d1
                    e.step()?;
                    let source_address = 0x4435d4_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00111750: bgt.w $111610
                    e.step()?;
                    pc = if r.greater()? {
                        0x111610_u32
                    } else {
                        0x111754_u32
                    };
                }
                0x111754 => {
                    // 00111754: move.w #$d, -(a7)
                    e.step()?;
                    let value = 0xd_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00111758: bsr.l $13154c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x11175e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13154c_u32)?;
                    // 0011175e: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00111760: move.b -$6(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00111764: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x111766_u32;
                }
                0x111766 => {
                    // 00111766: movem.l (a7)+, d3-d7
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    // 0011176a: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0011176c: rts
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
crate::recovered_application_control::entry!(check_for_enemy, 0x110e96);
