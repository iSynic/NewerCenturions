//! Complete provisional gameplay control from canonical CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{ApplicationControlServices, Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};

pub(crate) fn advance_time_body(
    e: &mut Execution<'_, impl ApplicationControlServices>,
    frame: u32,
) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x15e148_u32;
        loop {
            match pc {
                0x15e148 => {
                    // 0015e148: link.w a6, #$ffda
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -38)?;
                    // 0015e14c: movem.l d3-d7/a2, -(a7)
                    e.step()?;
                    r.save_register(e, 10)?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 0015e150: suba.l a2, a2
                    e.step()?;
                    r.put_address(2, 0);
                    // 0015e152: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015e154: move.b #$5, -$b(a6)
                    e.step()?;
                    let value = 0x5_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff5_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015e15a: clr.b -$5(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015e15e: move.b $402f43.l, d0
                    e.step()?;
                    let value_address = 0x402f43_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015e164: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e166: lea.l $435530.l, a0
                    e.step()?;
                    let value = 0x435530_u32;
                    r.put_address(0, value);
                    // 0015e16c: tst.w (a0, d0.l * 2)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015e170: beq.b $15e178
                    e.step()?;
                    pc = if r.equal()? {
                        0x15e178_u32
                    } else {
                        0x15e172_u32
                    };
                }
                0x15e172 => {
                    // 0015e172: move.b #$1, -$b(a6)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff5_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x15e178_u32;
                }
                0x15e178 => {
                    // 0015e178: pea.l -$24(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e17c: MacToolbox 0xa874
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa874_u32 as u16))?;
                    // 0015e17e: tst.b $9(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x9_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015e182: bne.b $15e19a
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e19a_u32
                    } else {
                        0x15e184_u32
                    };
                }
                0x15e184 => {
                    // 0015e184: move.b -$b(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff5_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015e188: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e18a: sub.w d0, $4427c8.l
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination_address = 0x4427c8_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0015e190: move.b #$1, -$5(a6)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015e196: addq.b #$1, $9(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0x9_u32);
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.add_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    pc = 0x15e19a_u32;
                }
                0x15e19a => {
                    // 0015e19a: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015e19c: move.b $9(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x9_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015e1a0: move.w d0, -$a(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e1a4: bra.w $15e752
                    e.step()?;
                    pc = 0x15e752_u32;
                }
                0x15e1a8 => {
                    // 0015e1a8: bsr.l $15ec56
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e1ae_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15ec56_u32)?;
                    // 0015e1ae: move.b -$b(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff5_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015e1b2: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e1b4: add.w d0, $4427c8.l
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination_address = 0x4427c8_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0015e1ba: tst.b -$5(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015e1be: bne.w $15e3a4
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e3a4_u32
                    } else {
                        0x15e1c2_u32
                    };
                }
                0x15e1c2 => {
                    // 0015e1c2: cmpi.w #$3c, $4427c8.l
                    e.step()?;
                    let source = 0x3c_u32;
                    let destination_address = 0x4427c8_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e1ca: blt.w $15e74e
                    e.step()?;
                    pc = if r.less()? {
                        0x15e74e_u32
                    } else {
                        0x15e1ce_u32
                    };
                }
                0x15e1ce => {
                    // 0015e1ce: tst.b -$5(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015e1d2: bne.b $15e1da
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e1da_u32
                    } else {
                        0x15e1d4_u32
                    };
                }
                0x15e1d4 => {
                    // 0015e1d4: bsr.l $14de04
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e1da_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14de04_u32)?;
                    pc = 0x15e1da_u32;
                }
                0x15e1da => {
                    // 0015e1da: subi.w #$3c, $4427c8.l
                    e.step()?;
                    let source = 0x3c_u32;
                    let destination_address = 0x4427c8_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0015e1e2: addq.w #$1, $4427ca.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4427ca_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0015e1e8: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e1ea: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e1ee: bsr.l $160a8e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e1f4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x160a8e_u32)?;
                    // 0015e1f4: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015e1f6: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015e1f8: bra.w $15e2da
                    e.step()?;
                    pc = 0x15e2da_u32;
                }
                0x15e1fc => {
                    // 0015e1fc: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e1fe: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e202: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e208: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e20a: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e20e: lea.l $448aa0.l, a1
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(1, value);
                    // 0015e214: move.w $58(a1, d1.l), d1
                    e.step()?;
                    let value_address = r
                        .address(1)?
                        .wrapping_add(0x58_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e218: cmp.w $5a(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x5a_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e21c: bge.w $15e2d8
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15e2d8_u32
                    } else {
                        0x15e220_u32
                    };
                }
                0x15e220 => {
                    // 0015e220: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e222: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e226: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e22c: tst.w $50(a0, d0.l)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x50_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015e230: ble.w $15e2d8
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e2d8_u32
                    } else {
                        0x15e234_u32
                    };
                }
                0x15e234 => {
                    // 0015e234: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e236: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e23a: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e240: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e242: tst.w $90(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x90_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015e246: bne.w $15e2d8
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e2d8_u32
                    } else {
                        0x15e24a_u32
                    };
                }
                0x15e24a => {
                    // 0015e24a: tst.b $44274a.l
                    e.step()?;
                    let value_address = 0x44274a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015e250: bne.w $15e2d8
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e2d8_u32
                    } else {
                        0x15e254_u32
                    };
                }
                0x15e254 => {
                    // 0015e254: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e256: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e25a: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e260: move.w $3a(a0, d0.l), d4
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x3a_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 0015e264: lsr.w #$8, d4
                    e.step()?;
                    let count = 0x8_u32;
                    let source = r.data(4, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(4, 16, value);
                    // 0015e266: lsr.w #$7, d4
                    e.step()?;
                    let count = 0x7_u32;
                    let source = r.data(4, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(4, 16, value);
                    // 0015e268: add.w $3a(a0, d0.l), d4
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x3a_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    // 0015e26c: asr.w #$1, d4
                    e.step()?;
                    let count = 0x1_u32;
                    let source = r.data(4, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(4, 16, value);
                    // 0015e26e: cmpi.b #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 0015e272: bge.b $15e276
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15e276_u32
                    } else {
                        0x15e274_u32
                    };
                }
                0x15e274 => {
                    // 0015e274: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x15e276_u32;
                }
                0x15e276 => {
                    // 0015e276: move.b d4, d0
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015e278: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e27a: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e27c: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e280: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e286: lea.l $58(a0, d1.l), a0
                    e.step()?;
                    let value = r
                        .address(0)?
                        .wrapping_add(0x58_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    r.put_address(0, value);
                    // 0015e28a: add.w d0, (a0)
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0015e28c: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e28e: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e292: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e298: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e29a: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e29e: lea.l $448aa0.l, a1
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(1, value);
                    // 0015e2a4: move.w $58(a1, d1.l), d1
                    e.step()?;
                    let value_address = r
                        .address(1)?
                        .wrapping_add(0x58_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e2a8: cmp.w $5a(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x5a_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e2ac: ble.b $15e2cc
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e2cc_u32
                    } else {
                        0x15e2ae_u32
                    };
                }
                0x15e2ae => {
                    // 0015e2ae: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e2b0: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e2b4: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e2ba: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e2bc: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e2c0: lea.l $448aa0.l, a1
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(1, value);
                    // 0015e2c6: move.w $5a(a0, d0.l), $58(a1, d1.l)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x5a_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r
                        .address(1)?
                        .wrapping_add(0x58_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x15e2cc_u32;
                }
                0x15e2cc => {
                    // 0015e2cc: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e2ce: move.w d3, -(a7)
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e2d0: bsr.l $1600fc
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e2d6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1600fc_u32)?;
                    // 0015e2d6: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x15e2d8_u32;
                }
                0x15e2d8 => {
                    // 0015e2d8: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x15e2da_u32;
                }
                0x15e2da => {
                    // 0015e2da: move.b $44273a.l, d0
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015e2e0: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e2e2: cmp.w d0, d3
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e2e4: ble.w $15e1fc
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e1fc_u32
                    } else {
                        0x15e2e8_u32
                    };
                }
                0x15e2e8 => {
                    // 0015e2e8: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015e2ea: bra.w $15e39a
                    e.step()?;
                    pc = 0x15e39a_u32;
                }
                0x15e2ee => {
                    // 0015e2ee: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e2f0: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e2f4: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0015e2fa: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e2fc: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e300: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 0015e306: adda.l d1, a1
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0015e308: move.w $10e(a1), d1
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x10e_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e30c: cmp.w $52(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x52_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e310: bge.w $15e398
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15e398_u32
                    } else {
                        0x15e314_u32
                    };
                }
                0x15e314 => {
                    // 0015e314: tst.b $44274a.l
                    e.step()?;
                    let value_address = 0x44274a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015e31a: bne.b $15e398
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e398_u32
                    } else {
                        0x15e31c_u32
                    };
                }
                0x15e31c => {
                    // 0015e31c: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e31e: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e322: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0015e328: move.w (a0, d0.l), d4
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 0015e32c: lsr.w #$8, d4
                    e.step()?;
                    let count = 0x8_u32;
                    let source = r.data(4, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(4, 16, value);
                    // 0015e32e: lsr.w #$7, d4
                    e.step()?;
                    let count = 0x7_u32;
                    let source = r.data(4, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(4, 16, value);
                    // 0015e330: add.w (a0, d0.l), d4
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    // 0015e334: asr.w #$1, d4
                    e.step()?;
                    let count = 0x1_u32;
                    let source = r.data(4, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(4, 16, value);
                    // 0015e336: cmpi.b #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 0015e33a: bge.b $15e33e
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15e33e_u32
                    } else {
                        0x15e33c_u32
                    };
                }
                0x15e33c => {
                    // 0015e33c: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x15e33e_u32;
                }
                0x15e33e => {
                    // 0015e33e: move.b d4, d0
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015e340: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e342: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e344: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e348: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0015e34e: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e350: add.w d0, $10e(a0)
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x10e_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0015e354: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e356: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e35a: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0015e360: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e362: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e366: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 0015e36c: adda.l d1, a1
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0015e36e: move.w $10e(a1), d1
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x10e_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e372: cmp.w $52(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x52_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e376: ble.b $15e398
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e398_u32
                    } else {
                        0x15e378_u32
                    };
                }
                0x15e378 => {
                    // 0015e378: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e37a: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e37e: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0015e384: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e386: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e38a: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 0015e390: adda.l d1, a1
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0015e392: move.w $52(a0, d0.l), $10e(a1)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x52_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(1)?.wrapping_add(0x10e_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x15e398_u32;
                }
                0x15e398 => {
                    // 0015e398: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x15e39a_u32;
                }
                0x15e39a => {
                    // 0015e39a: cmp.w $44257c.l, d3
                    e.step()?;
                    let source_address = 0x44257c_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e3a0: blt.w $15e2ee
                    e.step()?;
                    pc = if r.less()? {
                        0x15e2ee_u32
                    } else {
                        0x15e3a4_u32
                    };
                }
                0x15e3a4 => {
                    // 0015e3a4: cmpi.w #$c, $4427ca.l
                    e.step()?;
                    let source = 0xc_u32;
                    let destination_address = 0x4427ca_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e3ac: beq.b $15e3ba
                    e.step()?;
                    pc = if r.equal()? {
                        0x15e3ba_u32
                    } else {
                        0x15e3ae_u32
                    };
                }
                0x15e3ae => {
                    // 0015e3ae: cmpi.w #$18, $4427ca.l
                    e.step()?;
                    let source = 0x18_u32;
                    let destination_address = 0x4427ca_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e3b6: bne.w $15e74e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e74e_u32
                    } else {
                        0x15e3ba_u32
                    };
                }
                0x15e3ba => {
                    // 0015e3ba: cmpi.w #$18, $4427ca.l
                    e.step()?;
                    let source = 0x18_u32;
                    let destination_address = 0x4427ca_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e3c2: bne.w $15e5c0
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e5c0_u32
                    } else {
                        0x15e3c6_u32
                    };
                }
                0x15e3c6 => {
                    // 0015e3c6: addq.w #$1, $4427d4.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4427d4_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0015e3cc: subi.w #$18, $4427ca.l
                    e.step()?;
                    let source = 0x18_u32;
                    let destination_address = 0x4427ca_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0015e3d4: moveq #$ff, d6
                    e.step()?;
                    r.put_data(6, 32, 0xffffffff_u32);
                    r.move_flags(0xffffffff_u32, 32);
                    pc = 0x15e3d6_u32;
                }
                0x15e3d6 => {
                    // 0015e3d6: pea.l $400a86.l
                    e.step()?;
                    let value = 0x400a86_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e3dc: bsr.l $136d86
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e3e2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x136d86_u32)?;
                    // 0015e3e2: pea.l $400a98.l
                    e.step()?;
                    let value = 0x400a98_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e3e8: pea.l $4389a4.l
                    e.step()?;
                    let value = 0x4389a4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e3ee: jsr $400070.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e3f4_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400070_u32))?;
                    // 0015e3f4: movea.l a0, a2
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_address(2, value);
                    // 0015e3f6: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e3f8: lea.l $c(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xc_u32);
                    r.put_address(7, value);
                    // 0015e3fc: bne.b $15e40a
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e40a_u32
                    } else {
                        0x15e3fe_u32
                    };
                }
                0x15e3fe => {
                    // 0015e3fe: move.w #$c9, -(a7)
                    e.step()?;
                    let value = 0xc9_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e402: bsr.l $138be8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e408_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x138be8_u32)?;
                    // 0015e408: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x15e40a_u32;
                }
                0x15e40a => {
                    // 0015e40a: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e40c: move.w d6, d0
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e40e: addq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015e410: mulu.w #$28, d0
                    e.step()?;
                    let source = 0x28_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e414: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e416: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e418: jsr $400078.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e41e_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400078_u32))?;
                    // 0015e41e: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015e420: lea.l $a(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xa_u32);
                    r.put_address(7, value);
                    pc = 0x15e424_u32;
                }
                0x15e424 => {
                    // 0015e424: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e426: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e42a: pea.l $28.w
                    e.step()?;
                    let value = 0x28_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e42e: pea.l $439d14.l
                    e.step()?;
                    let value = 0x439d14_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e434: jsr $400048.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e43a_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400048_u32))?;
                    // 0015e43a: addq.w #$1, d6
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(6, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(6, 16, value);
                    // 0015e43c: cmpi.w #$96, d6
                    e.step()?;
                    let source = 0x96_u32;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e440: lea.l $10(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x10_u32);
                    r.put_address(7, value);
                    // 0015e444: bgt.w $15e5b6
                    e.step()?;
                    pc = if r.greater()? {
                        0x15e5b6_u32
                    } else {
                        0x15e448_u32
                    };
                }
                0x15e448 => {
                    // 0015e448: move.w $439d14.l, d0
                    e.step()?;
                    let value_address = 0x439d14_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e44e: cmp.w $4427d4.l, d0
                    e.step()?;
                    let source_address = 0x4427d4_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e454: bne.w $15e582
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e582_u32
                    } else {
                        0x15e458_u32
                    };
                }
                0x15e458 => {
                    // 0015e458: move.w $439d16.l, d0
                    e.step()?;
                    let value_address = 0x439d16_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e45e: add.w d0, $439d14.l
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination_address = 0x439d14_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0015e464: move.w #$64, -(a7)
                    e.step()?;
                    let value = 0x64_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e468: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e46e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 0015e46e: cmp.w $439d18.l, d0
                    e.step()?;
                    let source_address = 0x439d18_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e474: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015e476: bgt.w $15e58c
                    e.step()?;
                    pc = if r.greater()? {
                        0x15e58c_u32
                    } else {
                        0x15e47a_u32
                    };
                }
                0x15e47a => {
                    // 0015e47a: tst.w $439d24.l
                    e.step()?;
                    let value_address = 0x439d24_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015e480: ble.b $15e49e
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e49e_u32
                    } else {
                        0x15e482_u32
                    };
                }
                0x15e482 => {
                    // 0015e482: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e484: move.l #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e48a: move.w $439d24.l, -(a7)
                    e.step()?;
                    let value_address = 0x439d24_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e490: bsr.l $112650
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e496_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x112650_u32)?;
                    // 0015e496: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0015e498: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015e49a: beq.w $15e58c
                    e.step()?;
                    pc = if r.equal()? {
                        0x15e58c_u32
                    } else {
                        0x15e49e_u32
                    };
                }
                0x15e49e => {
                    // 0015e49e: cmpi.w #$ffff, $439d26.l
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = 0x439d26_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e4a6: ble.b $15e4bc
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e4bc_u32
                    } else {
                        0x15e4a8_u32
                    };
                }
                0x15e4a8 => {
                    // 0015e4a8: lea.l $44258c.l, a0
                    e.step()?;
                    let value = 0x44258c_u32;
                    r.put_address(0, value);
                    // 0015e4ae: move.w $439d26.l, d0
                    e.step()?;
                    let value_address = 0x439d26_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e4b4: tst.b (a0, d0.w)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015e4b8: beq.w $15e58c
                    e.step()?;
                    pc = if r.equal()? {
                        0x15e58c_u32
                    } else {
                        0x15e4bc_u32
                    };
                }
                0x15e4bc => {
                    // 0015e4bc: tst.w $439d28.l
                    e.step()?;
                    let value_address = 0x439d28_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015e4c2: ble.w $15e57e
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e57e_u32
                    } else {
                        0x15e4c6_u32
                    };
                }
                0x15e4c6 => {
                    // 0015e4c6: cmpi.w #$1, $439d28.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x439d28_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e4ce: bne.w $15e582
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e582_u32
                    } else {
                        0x15e4d2_u32
                    };
                }
                0x15e4d2 => {
                    // 0015e4d2: movea.w $439d1c.l, a0
                    e.step()?;
                    let value_address = 0x439d1c_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015e4d8: cmpa.l $442774.l, a0
                    e.step()?;
                    let source_address = 0x442774_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0015e4de: bne.w $15e582
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e582_u32
                    } else {
                        0x15e4e2_u32
                    };
                }
                0x15e4e2 => {
                    // 0015e4e2: moveq #$1, d5
                    e.step()?;
                    r.put_data(5, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 0015e4e4: move.l $442778.l, d0
                    e.step()?;
                    let value_address = 0x442778_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e4ea: add.l $442764.l, d0
                    e.step()?;
                    let source_address = 0x442764_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015e4f0: move.b $442736.l, d1
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0015e4f6: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e4f8: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e4fa: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0015e4fc: move.w d1, -$2(a6)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e500: move.l $44277c.l, d0
                    e.step()?;
                    let value_address = 0x44277c_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e506: add.l $442768.l, d0
                    e.step()?;
                    let source_address = 0x442768_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015e50c: move.b $442737.l, d1
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0015e512: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e514: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e516: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0015e518: move.w d1, -$4(a6)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e51c: cmpi.w #$ffff, $439d1e.l
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = 0x439d1e_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e524: ble.b $15e548
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e548_u32
                    } else {
                        0x15e526_u32
                    };
                }
                0x15e526 => {
                    // 0015e526: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 0015e528: move.l -$4(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e52c: movea.w $439d1e.l, a0
                    e.step()?;
                    let value_address = 0x439d1e_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015e532: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e534: lsl.l #$3, d0
                    e.step()?;
                    let count = 0x3_u32;
                    let source = r.data(0, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(0, 32, value);
                    // 0015e536: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015e53c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e53e: pea.l (a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x0_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e540: MacToolbox 0xa8ad
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
                    // 0015e542: move.b (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.b(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015e544: bne.b $15e548
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e548_u32
                    } else {
                        0x15e546_u32
                    };
                }
                0x15e546 => {
                    // 0015e546: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x15e548_u32;
                }
                0x15e548 => {
                    // 0015e548: cmpi.w #$ffff, $439d20.l
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = 0x439d20_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e550: ble.b $15e560
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e560_u32
                    } else {
                        0x15e552_u32
                    };
                }
                0x15e552 => {
                    // 0015e552: move.w -$2(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e556: cmp.w $439d20.l, d0
                    e.step()?;
                    let source_address = 0x439d20_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e55c: beq.b $15e560
                    e.step()?;
                    pc = if r.equal()? {
                        0x15e560_u32
                    } else {
                        0x15e55e_u32
                    };
                }
                0x15e55e => {
                    // 0015e55e: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x15e560_u32;
                }
                0x15e560 => {
                    // 0015e560: cmpi.w #$ffff, $439d20.l
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = 0x439d20_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e568: ble.b $15e578
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e578_u32
                    } else {
                        0x15e56a_u32
                    };
                }
                0x15e56a => {
                    // 0015e56a: move.w -$4(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e56e: cmp.w $439d22.l, d0
                    e.step()?;
                    let source_address = 0x439d22_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e574: beq.b $15e578
                    e.step()?;
                    pc = if r.equal()? {
                        0x15e578_u32
                    } else {
                        0x15e576_u32
                    };
                }
                0x15e576 => {
                    // 0015e576: moveq #$0, d5
                    e.step()?;
                    r.put_data(5, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x15e578_u32;
                }
                0x15e578 => {
                    // 0015e578: tst.b d5
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.move_flags(value, 8);
                    // 0015e57a: bne.b $15e58c
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e58c_u32
                    } else {
                        0x15e57c_u32
                    };
                }
                0x15e57c => {
                    // 0015e57c: bra.b $15e582
                    e.step()?;
                    pc = 0x15e582_u32;
                }
                0x15e57e => {
                    // 0015e57e: moveq #$1, d5
                    e.step()?;
                    r.put_data(5, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 0015e580: bra.b $15e58c
                    e.step()?;
                    pc = 0x15e58c_u32;
                }
                0x15e582 => {
                    // 0015e582: tst.w $439d14.l
                    e.step()?;
                    let value_address = 0x439d14_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015e588: bne.w $15e424
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e424_u32
                    } else {
                        0x15e58c_u32
                    };
                }
                0x15e58c => {
                    // 0015e58c: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e58e: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 0015e590: mulu.w d6, d0
                    e.step()?;
                    let source = r.data(6, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e592: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e594: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e596: jsr $400078.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e59c_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400078_u32))?;
                    // 0015e59c: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e59e: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e5a2: pea.l $28.w
                    e.step()?;
                    let value = 0x28_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e5a6: pea.l $439d14.l
                    e.step()?;
                    let value = 0x439d14_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e5ac: jsr $400050.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e5b2_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400050_u32))?;
                    // 0015e5b2: lea.l $1a(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x1a_u32);
                    r.put_address(7, value);
                    pc = 0x15e5b6_u32;
                }
                0x15e5b6 => {
                    // 0015e5b6: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e5b8: jsr $400068.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e5be_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400068_u32))?;
                    // 0015e5be: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x15e5c0_u32;
                }
                0x15e5c0 => {
                    // 0015e5c0: tst.b -$5(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015e5c4: bne.w $15e74e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e74e_u32
                    } else {
                        0x15e5c8_u32
                    };
                }
                0x15e5c8 => {
                    // 0015e5c8: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015e5ca: bra.w $15e690
                    e.step()?;
                    pc = 0x15e690_u32;
                }
                0x15e5ce => {
                    // 0015e5ce: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e5d0: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e5d4: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e5da: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e5dc: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e5e0: lea.l $448aa0.l, a1
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(1, value);
                    // 0015e5e6: move.w $50(a1, d1.l), d1
                    e.step()?;
                    let value_address = r
                        .address(1)?
                        .wrapping_add(0x50_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e5ea: cmp.w $52(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x52_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e5ee: bge.w $15e68e
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15e68e_u32
                    } else {
                        0x15e5f2_u32
                    };
                }
                0x15e5f2 => {
                    // 0015e5f2: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e5f4: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e5f8: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e5fe: move.w $3a(a0, d0.l), d4
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x3a_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 0015e602: asr.w #$1, d4
                    e.step()?;
                    let count = 0x1_u32;
                    let source = r.data(4, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(4, 16, value);
                    // 0015e604: lsr.w #$8, d4
                    e.step()?;
                    let count = 0x8_u32;
                    let source = r.data(4, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(4, 16, value);
                    // 0015e606: lsr.w #$6, d4
                    e.step()?;
                    let count = 0x6_u32;
                    let source = r.data(4, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(4, 16, value);
                    // 0015e608: add.w $3a(a0, d0.l), d4
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x3a_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    // 0015e60c: asr.w #$2, d4
                    e.step()?;
                    let count = 0x2_u32;
                    let source = r.data(4, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(4, 16, value);
                    // 0015e60e: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e610: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e614: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e61a: cmpi.w #$3, $32(a0, d0.l)
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x32_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e620: bne.b $15e632
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e632_u32
                    } else {
                        0x15e622_u32
                    };
                }
                0x15e622 => {
                    // 0015e622: move.w d3, d4
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    // 0015e624: mulu.w #$c42, d4
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(4, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(4, 32, value);
                    r.move_flags(value, 32);
                    // 0015e628: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e62e: move.w $3a(a0, d4.l), d4
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x3a_u32)
                        .wrapping_add((r.data(4, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(4, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x15e632_u32;
                }
                0x15e632 => {
                    // 0015e632: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e634: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e638: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e63e: cmpi.w #$fff6, $50(a0, d0.l)
                    e.step()?;
                    let source = 0xfff6_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x50_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e644: ble.b $15e68e
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e68e_u32
                    } else {
                        0x15e646_u32
                    };
                }
                0x15e646 => {
                    // 0015e646: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e64c: move.l #$3200001, -(a7)
                    e.step()?;
                    let value = 0x3200001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e652: bsr.l $112650
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e658_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x112650_u32)?;
                    // 0015e658: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0015e65a: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015e65c: bne.b $15e664
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e664_u32
                    } else {
                        0x15e65e_u32
                    };
                }
                0x15e65e => {
                    // 0015e65e: extb.l d4
                    e.step()?;
                    let source = r.data(4, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(4, 32, value);
                    r.move_flags(value, 32);
                    // 0015e660: divs.w #$2, d4
                    e.step()?;
                    let divisor = 0x2_u32;
                    let dividend = r.data(4, 32)?;
                    let value = r.divide_signed(dividend, divisor)?;
                    r.put_data(4, 32, value);
                    pc = 0x15e664_u32;
                }
                0x15e664 => {
                    // 0015e664: cmpi.b #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 0015e668: bge.b $15e66c
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15e66c_u32
                    } else {
                        0x15e66a_u32
                    };
                }
                0x15e66a => {
                    // 0015e66a: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    pc = 0x15e66c_u32;
                }
                0x15e66c => {
                    // 0015e66c: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e66e: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e670: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e674: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0015e67a: move.b d4, d1
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0015e67c: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e67e: add.w $66(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x66_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e682: move.w d1, -(a7)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e684: move.w d3, -(a7)
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e686: bsr.l $1287c2
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e68c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1287c2_u32)?;
                    // 0015e68c: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x15e68e_u32;
                }
                0x15e68e => {
                    // 0015e68e: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x15e690_u32;
                }
                0x15e690 => {
                    // 0015e690: move.b $44273a.l, d0
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015e696: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e698: cmp.w d0, d3
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e69a: ble.w $15e5ce
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e5ce_u32
                    } else {
                        0x15e69e_u32
                    };
                }
                0x15e69e => {
                    // 0015e69e: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015e6a0: bra.w $15e744
                    e.step()?;
                    pc = 0x15e744_u32;
                }
                0x15e6a4 => {
                    // 0015e6a4: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e6a6: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e6aa: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0015e6b0: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e6b2: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e6b4: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e6b8: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 0015e6be: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0015e6c0: move.w $110(a1), d0
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x110_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e6c4: cmp.w $112(a0), d0
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x112_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e6c8: bge.b $15e742
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15e742_u32
                    } else {
                        0x15e6ca_u32
                    };
                }
                0x15e6ca => {
                    // 0015e6ca: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e6cc: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e6d0: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0015e6d6: move.w (a0, d0.l), d1
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e6da: asr.w #$1, d1
                    e.step()?;
                    let count = 0x1_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e6dc: lsr.w #$8, d1
                    e.step()?;
                    let count = 0x8_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(1, 16, value);
                    // 0015e6de: lsr.w #$6, d1
                    e.step()?;
                    let count = 0x6_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_right(source, count & 63, 16, false);
                    r.put_data(1, 16, value);
                    // 0015e6e0: add.w (a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e6e4: asr.w #$2, d1
                    e.step()?;
                    let count = 0x2_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_right(source, count & 63, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e6e6: addq.w #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e6e8: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e6ea: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e6ee: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0015e6f4: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e6f6: add.w d1, $110(a0)
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x110_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0015e6fa: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e6fc: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e700: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0015e706: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e708: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e70a: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e70e: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 0015e714: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0015e716: move.w $110(a1), d0
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x110_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e71a: cmp.w $112(a0), d0
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x112_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e71e: ble.b $15e742
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15e742_u32
                    } else {
                        0x15e720_u32
                    };
                }
                0x15e720 => {
                    // 0015e720: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e722: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e726: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0015e72c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e72e: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e730: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e734: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 0015e73a: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0015e73c: move.w $112(a0), $110(a1)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x112_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(1)?.wrapping_add(0x110_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x15e742_u32;
                }
                0x15e742 => {
                    // 0015e742: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x15e744_u32;
                }
                0x15e744 => {
                    // 0015e744: cmp.w $44257c.l, d3
                    e.step()?;
                    let source_address = 0x44257c_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e74a: blt.w $15e6a4
                    e.step()?;
                    pc = if r.less()? {
                        0x15e6a4_u32
                    } else {
                        0x15e74e_u32
                    };
                }
                0x15e74e => {
                    // 0015e74e: subq.w #$1, -$a(a6)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    pc = 0x15e752_u32;
                }
                0x15e752 => {
                    // 0015e752: tst.w -$a(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015e756: bgt.w $15e1a8
                    e.step()?;
                    pc = if r.greater()? {
                        0x15e1a8_u32
                    } else {
                        0x15e75a_u32
                    };
                }
                0x15e75a => {
                    // 0015e75a: move.l $43878c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43878c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e760: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 0015e762: move.l $4358ce.l, -(a7)
                    e.step()?;
                    let value_address = 0x4358ce_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e768: MacToolbox 0xaa0b
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa0b_u32 as u16))?;
                    // 0015e76a: move.w $439582.l, d0
                    e.step()?;
                    let value_address = 0x439582_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e770: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 0015e772: move.w $40856e.l, d1
                    e.step()?;
                    let value_address = 0x40856e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e778: addi.w #$178, d1
                    e.step()?;
                    let source = 0x178_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e77c: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e77e: move.w d1, $4434ba.l
                    e.step()?;
                    let value = r.data(1, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e784: moveq #$e, d0
                    e.step()?;
                    r.put_data(0, 32, 0xe_u32);
                    r.move_flags(0xe_u32, 32);
                    // 0015e786: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015e78c: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e792: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e798: addi.w #$230, d0
                    e.step()?;
                    let source = 0x230_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015e79c: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e7a2: moveq #$46, d0
                    e.step()?;
                    r.put_data(0, 32, 0x46_u32);
                    r.move_flags(0x46_u32, 32);
                    // 0015e7a4: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015e7aa: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e7b0: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e7b6: MacToolbox 0xa8a3
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8a3_u32 as u16))?;
                    // 0015e7b8: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e7bc: MacToolbox 0xa889
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa889_u32 as u16))?;
                    // 0015e7be: move.w $402f44.l, -(a7)
                    e.step()?;
                    let value_address = 0x402f44_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e7c4: MacToolbox 0xa887
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa887_u32 as u16))?;
                    // 0015e7c6: move.w #$c, -(a7)
                    e.step()?;
                    let value = 0xc_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e7ca: MacToolbox 0xa88a
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa88a_u32 as u16))?;
                    // 0015e7cc: pea.l $45.w
                    e.step()?;
                    let value = 0x45_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e7d0: MacToolbox 0xa862
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa862_u32 as u16))?;
                    // 0015e7d2: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e7d8: addi.w #$230, d0
                    e.step()?;
                    let source = 0x230_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015e7dc: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e7de: move.w $439582.l, d0
                    e.step()?;
                    let value_address = 0x439582_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e7e4: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 0015e7e6: move.w $40856e.l, d1
                    e.step()?;
                    let value_address = 0x40856e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e7ec: addi.w #$182, d1
                    e.step()?;
                    let source = 0x182_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e7f0: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e7f2: move.w d1, -(a7)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e7f4: MacToolbox 0xa893
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa893_u32 as u16))?;
                    // 0015e7f6: pea.l $4427c6.l
                    e.step()?;
                    let value = 0x4427c6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e7fc: pea.l $400a7c.l
                    e.step()?;
                    let value = 0x400a7c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e802: pea.l $14.w
                    e.step()?;
                    let value = 0x14_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e806: pea.l -$20(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e80a: jsr $4000d0.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e810_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000d0_u32))?;
                    // 0015e810: pea.l -$20(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e814: jsr $400188.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e81a_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400188_u32))?;
                    // 0015e81a: pea.l -$20(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e81e: MacToolbox 0xa884
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa884_u32 as u16))?;
                    // 0015e820: move.w $439582.l, d0
                    e.step()?;
                    let value_address = 0x439582_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e826: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 0015e828: move.w $40856e.l, d1
                    e.step()?;
                    let value_address = 0x40856e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e82e: addi.w #$186, d1
                    e.step()?;
                    let source = 0x186_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e832: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e834: move.w d1, $4434ba.l
                    e.step()?;
                    let value = r.data(1, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e83a: moveq #$c, d0
                    e.step()?;
                    r.put_data(0, 32, 0xc_u32);
                    r.move_flags(0xc_u32, 32);
                    // 0015e83c: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015e842: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e848: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e84e: addi.w #$249, d0
                    e.step()?;
                    let source = 0x249_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015e852: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e858: moveq #$26, d0
                    e.step()?;
                    r.put_data(0, 32, 0x26_u32);
                    r.move_flags(0x26_u32, 32);
                    // 0015e85a: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015e860: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e866: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0015e86c: MacToolbox 0xa8a3
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8a3_u32 as u16))?;
                    // 0015e86e: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e874: addi.w #$249, d0
                    e.step()?;
                    let source = 0x249_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015e878: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e87a: move.w $439582.l, d0
                    e.step()?;
                    let value_address = 0x439582_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e880: lsl.w #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(0, 16, value);
                    // 0015e882: move.w $40856e.l, d1
                    e.step()?;
                    let value_address = 0x40856e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0015e888: addi.w #$191, d1
                    e.step()?;
                    let source = 0x191_u32;
                    let destination = r.data(1, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e88c: sub.w d0, d1
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(1, 16, value);
                    // 0015e88e: move.w d1, -(a7)
                    e.step()?;
                    let value = r.data(1, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e890: MacToolbox 0xa893
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa893_u32 as u16))?;
                    // 0015e892: move.w $4427d4.l, -(a7)
                    e.step()?;
                    let value_address = 0x4427d4_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e898: bsr.l $1382d0
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e89e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1382d0_u32)?;
                    // 0015e89e: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e8a0: MacToolbox 0xa888
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa888_u32 as u16))?;
                    // 0015e8a2: move.l -$24(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e8a6: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 0015e8a8: move.w $402f3c.l, -(a7)
                    e.step()?;
                    let value_address = 0x402f3c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e8ae: MacToolbox 0xa887
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa887_u32 as u16))?;
                    // 0015e8b0: move.l $442778.l, d0
                    e.step()?;
                    let value_address = 0x442778_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e8b6: add.l $442764.l, d0
                    e.step()?;
                    let source_address = 0x442764_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015e8bc: move.w d0, -$2(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e8c0: move.l $44277c.l, d0
                    e.step()?;
                    let value_address = 0x44277c_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e8c6: add.l $442768.l, d0
                    e.step()?;
                    let source_address = 0x442768_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015e8cc: move.w d0, -$4(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e8d0: tst.b d5
                    e.step()?;
                    let value = r.data(5, 8)?;
                    r.move_flags(value, 8);
                    // 0015e8d2: lea.l $16(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x16_u32);
                    r.put_address(7, value);
                    // 0015e8d6: beq.b $15e8f6
                    e.step()?;
                    pc = if r.equal()? {
                        0x15e8f6_u32
                    } else {
                        0x15e8d8_u32
                    };
                }
                0x15e8d8 => {
                    // 0015e8d8: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e8da: move.w $439d1a.l, -(a7)
                    e.step()?;
                    let value_address = 0x439d1a_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e8e0: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e8e4: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e8e6: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015e8e8: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e8ee_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 0015e8ee: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    // 0015e8f2: bra.w $15e3d6
                    e.step()?;
                    pc = 0x15e3d6_u32;
                }
                0x15e8f6 => {
                    // 0015e8f6: tst.b -$5(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015e8fa: bne.w $15ec40
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15ec40_u32
                    } else {
                        0x15e8fe_u32
                    };
                }
                0x15e8fe => {
                    // 0015e8fe: tst.w $a(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015e902: beq.w $15ec40
                    e.step()?;
                    pc = if r.equal()? {
                        0x15ec40_u32
                    } else {
                        0x15e906_u32
                    };
                }
                0x15e906 => {
                    // 0015e906: moveq #$13, d3
                    e.step()?;
                    r.put_data(3, 32, 0x13_u32);
                    r.move_flags(0x13_u32, 32);
                    // 0015e908: bra.w $15ec34
                    e.step()?;
                    pc = 0x15ec34_u32;
                }
                0x15e90c => {
                    // 0015e90c: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015e912: move.w $6(a0, d3.w * 8), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x6_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(8),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e916: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015e91c: move.w $2(a0, d3.w * 8), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x2_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(8),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e920: move.w -$2(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e924: bsr.l $13846a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e92a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13846a_u32)?;
                    // 0015e92a: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0015e92c: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015e92e: beq.w $15ec32
                    e.step()?;
                    pc = if r.equal()? {
                        0x15ec32_u32
                    } else {
                        0x15e932_u32
                    };
                }
                0x15e932 => {
                    // 0015e932: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015e938: move.w $4(a0, d3.w * 8), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x4_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(8),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e93c: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015e942: move.w (a0, d3.w * 8), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(8),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e946: move.w -$4(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e94a: bsr.l $13846a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e950_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13846a_u32)?;
                    // 0015e950: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0015e952: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015e954: beq.w $15ec32
                    e.step()?;
                    pc = if r.equal()? {
                        0x15ec32_u32
                    } else {
                        0x15e958_u32
                    };
                }
                0x15e958 => {
                    // 0015e958: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015e95a: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e95c: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015e95e: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015e964: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e966: move.w $a0(a0), -$8(a6)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xa0_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e96c: cmpi.w #$ffff, $a(a6)
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xa_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e972: bne.b $15e982
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15e982_u32
                    } else {
                        0x15e974_u32
                    };
                }
                0x15e974 => {
                    // 0015e974: move.w -$8(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e978: ext.l d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e97a: divs.w #$4, d0
                    e.step()?;
                    let divisor = 0x4_u32;
                    let dividend = r.data(0, 32)?;
                    let value = r.divide_signed(dividend, divisor)?;
                    r.put_data(0, 32, value);
                    // 0015e97e: move.w d0, -$8(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x15e982_u32;
                }
                0x15e982 => {
                    // 0015e982: move.w #$2710, -(a7)
                    e.step()?;
                    let value = 0x2710_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e986: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e98c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 0015e98c: move.w d0, $4434b6.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e992: move.w $4434b6.l, d0
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015e998: cmp.w -$8(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e99c: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015e99e: bgt.w $15ec20
                    e.step()?;
                    pc = if r.greater()? {
                        0x15ec20_u32
                    } else {
                        0x15e9a2_u32
                    };
                }
                0x15e9a2 => {
                    // 0015e9a2: clr.w $4434ac.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x4434ac_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e9a8: bra.w $15eaac
                    e.step()?;
                    pc = 0x15eaac_u32;
                }
                0x15e9ac => {
                    // 0015e9ac: movea.w $4434ac.l, a0
                    e.step()?;
                    let value_address = 0x4434ac_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015e9b2: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e9b4: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015e9b6: moveq #$6, d1
                    e.step()?;
                    r.put_data(1, 32, 0x6_u32);
                    r.move_flags(0x6_u32, 32);
                    // 0015e9b8: mulu.w d3, d1
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e9ba: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015e9c0: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e9c2: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e9c4: move.w $190(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x190_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e9c8: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e9ce_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 0015e9ce: move.w d0, -$26(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffda_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e9d2: move.w #$64, -(a7)
                    e.step()?;
                    let value = 0x64_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015e9d6: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15e9dc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 0015e9dc: cmp.w -$26(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xffffffda_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015e9e0: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015e9e2: bgt.w $15eaa6
                    e.step()?;
                    pc = if r.greater()? {
                        0x15eaa6_u32
                    } else {
                        0x15e9e6_u32
                    };
                }
                0x15e9e6 => {
                    // 0015e9e6: movea.w $4434ac.l, a0
                    e.step()?;
                    let value_address = 0x4434ac_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015e9ec: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015e9ee: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015e9f0: moveq #$6, d1
                    e.step()?;
                    r.put_data(1, 32, 0x6_u32);
                    r.move_flags(0x6_u32, 32);
                    // 0015e9f2: mulu.w d3, d1
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015e9f4: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015e9fa: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e9fc: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015e9fe: tst.w $190(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x190_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015ea02: ble.b $15ea2e
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x15ea2e_u32
                    } else {
                        0x15ea04_u32
                    };
                }
                0x15ea04 => {
                    // 0015ea04: movea.w $4434ac.l, a0
                    e.step()?;
                    let value_address = 0x4434ac_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015ea0a: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015ea0c: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015ea0e: moveq #$6, d1
                    e.step()?;
                    r.put_data(1, 32, 0x6_u32);
                    r.move_flags(0x6_u32, 32);
                    // 0015ea10: mulu.w d3, d1
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015ea12: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015ea18: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015ea1a: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015ea1c: clr.w $190(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x190_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ea20: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ea26: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ea2c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 0015ea2c: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x15ea2e_u32;
                }
                0x15ea2e => {
                    // 0015ea2e: clr.b $44273b.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x44273b_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015ea34: clr.b $442753.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442753_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015ea3a: clr.b $442756.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442756_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015ea40: clr.b $442755.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442755_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015ea46: clr.b $44274f.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x44274f_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015ea4c: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ea4e: bsr.l $13ea82
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ea54_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13ea82_u32)?;
                    // 0015ea54: bsr.l $16043e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ea5a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16043e_u32)?;
                    // 0015ea5a: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ea5c: movea.w $4434ac.l, a0
                    e.step()?;
                    let value_address = 0x4434ac_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015ea62: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015ea64: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015ea66: moveq #$6, d1
                    e.step()?;
                    r.put_data(1, 32, 0x6_u32);
                    r.move_flags(0x6_u32, 32);
                    // 0015ea68: mulu.w d3, d1
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0015ea6a: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015ea70: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015ea72: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015ea74: move.w $118(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x118_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ea78: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ea7c: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ea7e: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ea80: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ea86_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 0015ea86: move.w d0, $4471ea.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4471ea_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ea8c: clr.b $442737.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015ea92: clr.b $442736.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015ea98: move.w $4471ea.l, d0
                    e.step()?;
                    let value_address = 0x4471ea_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015ea9e: lea.l $10(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x10_u32);
                    r.put_address(7, value);
                    // 0015eaa2: bra.w $15ec42
                    e.step()?;
                    pc = 0x15ec42_u32;
                }
                0x15eaa6 => {
                    // 0015eaa6: addq.w #$1, $4434ac.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4434ac_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    pc = 0x15eaac_u32;
                }
                0x15eaac => {
                    // 0015eaac: cmpi.w #$3, $4434ac.l
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = 0x4434ac_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0015eab4: blt.w $15e9ac
                    e.step()?;
                    pc = if r.less()? {
                        0x15e9ac_u32
                    } else {
                        0x15eab8_u32
                    };
                }
                0x15eab8 => {
                    // 0015eab8: tst.w $442570.l
                    e.step()?;
                    let value_address = 0x442570_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015eabe: bne.w $15ec20
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x15ec20_u32
                    } else {
                        0x15eac2_u32
                    };
                }
                0x15eac2 => {
                    // 0015eac2: tst.b $402f42.l
                    e.step()?;
                    let value_address = 0x402f42_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015eac8: beq.w $15ec20
                    e.step()?;
                    pc = if r.equal()? {
                        0x15ec20_u32
                    } else {
                        0x15eacc_u32
                    };
                }
                0x15eacc => {
                    // 0015eacc: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015eace: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015ead0: lsl.l #$2, d0
                    e.step()?;
                    let count = 0x2_u32;
                    let source = r.data(0, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(0, 32, value);
                    // 0015ead2: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015ead8: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015eada: tst.w $c8(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015eade: beq.w $15ec20
                    e.step()?;
                    pc = if r.equal()? {
                        0x15ec20_u32
                    } else {
                        0x15eae2_u32
                    };
                }
                0x15eae2 => {
                    // 0015eae2: moveq #$0, d7
                    e.step()?;
                    r.put_data(7, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015eae4: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015eaea: adda.w d3, a0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.address(0)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015eaec: move.b $220(a0), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x220_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0015eaf0: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0015eaf2: move.w d0, -$26(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffda_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eaf6: move.w #$64, -(a7)
                    e.step()?;
                    let value = 0x64_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eafa: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15eb00_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 0015eb00: cmp.w -$26(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xffffffda_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015eb04: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015eb06: bge.b $15eb72
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15eb72_u32
                    } else {
                        0x15eb08_u32
                    };
                }
                0x15eb08 => {
                    // 0015eb08: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015eb0a: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015eb0c: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015eb0e: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015eb14: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015eb16: move.w $234(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x234_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eb1a: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15eb20_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0015eb20: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eb22: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015eb24: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015eb26: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015eb28: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0015eb2a: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015eb30: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015eb32: move.w $25c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x25c_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eb36: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15eb3c_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 0015eb3c: neg.w d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = r.subtract_flags(0, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0015eb3e: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015eb40: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eb42: move.w #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eb46: jsr $15dc34(pc)
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15eb4a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15dc34_u32)?;
                    // 0015eb4a: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015eb4c: bsr.l $14d21a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15eb52_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14d21a_u32)?;
                    // 0015eb52: move.w d0, $4471ea.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4471ea_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eb58: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eb5a: bsr.l $1500d8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15eb60_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1500d8_u32)?;
                    // 0015eb60: tst.w $4471ea.l
                    e.step()?;
                    let value_address = 0x4471ea_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015eb66: lea.l $12(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x12_u32);
                    r.put_address(7, value);
                    // 0015eb6a: beq.w $15ec20
                    e.step()?;
                    pc = if r.equal()? {
                        0x15ec20_u32
                    } else {
                        0x15eb6e_u32
                    };
                }
                0x15eb6e => {
                    // 0015eb6e: moveq #$1, d7
                    e.step()?;
                    r.put_data(7, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 0015eb70: bra.b $15eb86
                    e.step()?;
                    pc = 0x15eb86_u32;
                }
                0x15eb72 => {
                    // 0015eb72: move.w #$64, -(a7)
                    e.step()?;
                    let value = 0x64_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eb76: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15eb7c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 0015eb7c: cmpi.w #$a, d0
                    e.step()?;
                    let source = 0xa_u32;
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015eb80: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015eb82: bge.b $15eb86
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x15eb86_u32
                    } else {
                        0x15eb84_u32
                    };
                }
                0x15eb84 => {
                    // 0015eb84: moveq #$ff, d7
                    e.step()?;
                    r.put_data(7, 32, 0xffffffff_u32);
                    r.move_flags(0xffffffff_u32, 32);
                    pc = 0x15eb86_u32;
                }
                0x15eb86 => {
                    // 0015eb86: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015eb88: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015eb8a: lsl.l #$2, d0
                    e.step()?;
                    let count = 0x2_u32;
                    let source = r.data(0, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(0, 32, value);
                    // 0015eb8c: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015eb92: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015eb94: move.w $ca(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xca_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015eb98: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0015eb9a: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0015eb9c: lsl.l #$2, d0
                    e.step()?;
                    let count = 0x2_u32;
                    let source = r.data(0, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(0, 32, value);
                    // 0015eb9e: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015eba4: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015eba6: move.w $c8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ebaa: bsr.l $13849e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ebb0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13849e_u32)?;
                    // 0015ebb0: move.w d0, $43874e.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x43874e_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ebb6: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ebb8: move.w d7, -(a7)
                    e.step()?;
                    let value = r.data(7, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ebba: bsr.l $114378
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ebc0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x114378_u32)?;
                    // 0015ebc0: tst.b $4386b2.l
                    e.step()?;
                    let value_address = 0x4386b2_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015ebc6: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0015ebc8: beq.b $15ebce
                    e.step()?;
                    pc = if r.equal()? {
                        0x15ebce_u32
                    } else {
                        0x15ebca_u32
                    };
                }
                0x15ebca => {
                    // 0015ebca: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015ebcc: bra.b $15ec42
                    e.step()?;
                    pc = 0x15ec42_u32;
                }
                0x15ebce => {
                    // 0015ebce: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ebd0: bsr.l $105748
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ebd6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x105748_u32)?;
                    // 0015ebd6: move.b #$ff, $402f43.l
                    e.step()?;
                    let value = 0xff_u32;
                    let destination_address = 0x402f43_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0015ebde: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ebe4: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ebea_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 0015ebea: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ebec: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ebf2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 0015ebf2: tst.w $439cf4.l
                    e.step()?;
                    let value_address = 0x439cf4_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0015ebf8: lea.l $c(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xc_u32);
                    r.put_address(7, value);
                    // 0015ebfc: beq.b $15ec20
                    e.step()?;
                    pc = if r.equal()? {
                        0x15ec20_u32
                    } else {
                        0x15ebfe_u32
                    };
                }
                0x15ebfe => {
                    // 0015ebfe: move.w $439cf4.l, -(a7)
                    e.step()?;
                    let value_address = 0x439cf4_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ec04: bsr.l $123c60
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ec0a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123c60_u32)?;
                    // 0015ec0a: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0015ec0c: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ec12: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ec14: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0015ec16: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x15ec1c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 0015ec1c: lea.l $10(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x10_u32);
                    r.put_address(7, value);
                    pc = 0x15ec20_u32;
                }
                0x15ec20 => {
                    // 0015ec20: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0015ec26: adda.w d3, a0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.address(0)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0015ec28: tst.b $20c(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x20c_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0015ec2c: beq.b $15ec32
                    e.step()?;
                    pc = if r.equal()? {
                        0x15ec32_u32
                    } else {
                        0x15ec2e_u32
                    };
                }
                0x15ec2e => {
                    // 0015ec2e: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015ec30: bra.b $15ec42
                    e.step()?;
                    pc = 0x15ec42_u32;
                }
                0x15ec32 => {
                    // 0015ec32: subq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x15ec34_u32;
                }
                0x15ec34 => {
                    // 0015ec34: cmpi.w #$ffff, d3
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0015ec38: bgt.w $15e90c
                    e.step()?;
                    pc = if r.greater()? {
                        0x15e90c_u32
                    } else {
                        0x15ec3c_u32
                    };
                }
                0x15ec3c => {
                    // 0015ec3c: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0015ec3e: bra.b $15ec42
                    e.step()?;
                    pc = 0x15ec42_u32;
                }
                0x15ec40 => {
                    // 0015ec40: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x15ec42_u32;
                }
                0x15ec42 => {
                    // 0015ec42: movem.l (a7)+, d3-d7/a2
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    r.restore_register(e, 10)?;
                    // 0015ec46: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0015ec48: rts
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
crate::recovered_application_control::entry!(advance_time, 0x15e148);
