//! Complete provisional gameplay control from canonical CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{ApplicationControlServices, Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};

pub(crate) fn button_choice_body(
    e: &mut Execution<'_, impl ApplicationControlServices>,
    frame: u32,
) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x10950e_u32;
        loop {
            match pc {
                0x10950e => {
                    // 0010950e: link.w a6, #$fff0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -16)?;
                    // 00109512: movem.l d3-d7, -(a7)
                    e.step()?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 00109516: clr.b -$d(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff3_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010951a: clr.b -$1(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010951e: clr.w -$6(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109522: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109528: cmpa.l $439454.l, a0
                    e.step()?;
                    let source_address = 0x439454_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010952e: bne.w $109b6a
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109b6a_u32
                    } else {
                        0x109532_u32
                    };
                }
                0x109532 => {
                    // 00109532: cmpi.w #$86, $442578.l
                    e.step()?;
                    let source = 0x86_u32;
                    let destination_address = 0x442578_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0010953a: ble.b $109562
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109562_u32
                    } else {
                        0x10953c_u32
                    };
                }
                0x10953c => {
                    // 0010953c: tst.w $4394b4.l
                    e.step()?;
                    let value_address = 0x4394b4_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00109542: bne.b $109562
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109562_u32
                    } else {
                        0x109544_u32
                    };
                }
                0x109544 => {
                    // 00109544: move.w #$36, -(a7)
                    e.step()?;
                    let value = 0x36_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109548: bsr.l $16474a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10954e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16474a_u32)?;
                    // 0010954e: move.l #$500050, -(a7)
                    e.step()?;
                    let value = 0x500050_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109554: bsr.l $12382a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10955a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12382a_u32)?;
                    // 0010955a: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010955c: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010955e: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x109562 => {
                    // 00109562: move.w #$14, $4471a4.l
                    e.step()?;
                    let value = 0x14_u32;
                    let destination_address = 0x4471a4_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010956a: tst.b $44274f.l
                    e.step()?;
                    let value_address = 0x44274f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109570: beq.b $1095b0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1095b0_u32
                    } else {
                        0x109572_u32
                    };
                }
                0x109572 => {
                    // 00109572: clr.b $44274f.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x44274f_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00109578: bsr.l $16043e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10957e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16043e_u32)?;
                    // 0010957e: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109580: bsr.l $13ea82
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109586_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13ea82_u32)?;
                    // 00109586: move.l #$50000, -(a7)
                    e.step()?;
                    let value = 0x50000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010958c: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109592_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 00109592: move.l #$a0001, -(a7)
                    e.step()?;
                    let value = 0xa0001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109598: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10959e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 0010959e: tst.b $4386b2.l
                    e.step()?;
                    let value_address = 0x4386b2_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001095a4: lea.l $a(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xa_u32);
                    r.put_address(7, value);
                    // 001095a8: beq.b $1095b0
                    e.step()?;
                    pc = if r.equal()? {
                        0x1095b0_u32
                    } else {
                        0x1095aa_u32
                    };
                }
                0x1095aa => {
                    // 001095aa: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 001095ac: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x1095b0 => {
                    // 001095b0: move.w #$2710, -(a7)
                    e.step()?;
                    let value = 0x2710_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001095b4: bsr.l $133bb8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1095ba_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133bb8_u32)?;
                    // 001095ba: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x1095bc_u32;
                }
                0x1095bc => {
                    // 001095bc: pea.l $4471ee.l
                    e.step()?;
                    let value = 0x4471ee_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001095c2: MacToolbox 0xa972
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa972_u32 as u16))?;
                    // 001095c4: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 001095c6: move.l $4471ee.l, -(a7)
                    e.step()?;
                    let value_address = 0x4471ee_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001095cc: pea.l $4471b4.l
                    e.step()?;
                    let value = 0x4471b4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001095d2: MacToolbox 0xa8ad
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
                    // 001095d4: move.b (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.b(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001095d6: beq.b $1095e6
                    e.step()?;
                    pc = if r.equal()? {
                        0x1095e6_u32
                    } else {
                        0x1095d8_u32
                    };
                }
                0x1095d8 => {
                    // 001095d8: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001095dc: bsr.l $15f210
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1095e2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15f210_u32)?;
                    // 001095e2: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001095e4: bra.b $1095f6
                    e.step()?;
                    pc = 0x1095f6_u32;
                }
                0x1095e6 => {
                    // 001095e6: move.l $43952c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43952c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001095ec: MacToolbox 0xaa1c
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa1c_u32 as u16))?;
                    // 001095ee: move.w #$14, $4471a4.l
                    e.step()?;
                    let value = 0x14_u32;
                    let destination_address = 0x4471a4_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x1095f6_u32;
                }
                0x1095f6 => {
                    // 001095f6: bsr.l $15ec56
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1095fc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15ec56_u32)?;
                    // 001095fc: tst.w $443316.l
                    e.step()?;
                    let value_address = 0x443316_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00109602: bne.w $109690
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109690_u32
                    } else {
                        0x109606_u32
                    };
                }
                0x109606 => {
                    // 00109606: clr.b $442737.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010960c: clr.b $442736.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00109612: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109618: lsl.l #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(0, 32, value);
                    // 0010961a: movea.w $4471ee.l, a0
                    e.step()?;
                    let value_address = 0x4471ee_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 00109620: cmpa.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00109622: bge.b $10962a
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x10962a_u32
                    } else {
                        0x109624_u32
                    };
                }
                0x109624 => {
                    // 00109624: subq.b #$1, $442737.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x442737_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.subtract_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    pc = 0x10962a_u32;
                }
                0x10962a => {
                    // 0010962a: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109630: lsl.l #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(0, 32, value);
                    // 00109632: moveq #$20, d1
                    e.step()?;
                    r.put_data(1, 32, 0x20_u32);
                    r.move_flags(0x20_u32, 32);
                    // 00109634: add.l d1, d0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00109636: movea.w $4471ee.l, a0
                    e.step()?;
                    let value_address = 0x4471ee_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010963c: cmpa.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010963e: ble.b $109646
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109646_u32
                    } else {
                        0x109640_u32
                    };
                }
                0x109640 => {
                    // 00109640: addq.b #$1, $442737.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x442737_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.add_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    pc = 0x109646_u32;
                }
                0x109646 => {
                    // 00109646: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010964c: lsl.l #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(0, 32, value);
                    // 0010964e: movea.w $4471f0.l, a0
                    e.step()?;
                    let value_address = 0x4471f0_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 00109654: cmpa.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00109656: bge.b $10965e
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x10965e_u32
                    } else {
                        0x109658_u32
                    };
                }
                0x109658 => {
                    // 00109658: subq.b #$1, $442736.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x442736_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.subtract_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    pc = 0x10965e_u32;
                }
                0x10965e => {
                    // 0010965e: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109664: lsl.l #$5, d0
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(0, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(0, 32, value);
                    // 00109666: moveq #$20, d1
                    e.step()?;
                    r.put_data(1, 32, 0x20_u32);
                    r.move_flags(0x20_u32, 32);
                    // 00109668: add.l d1, d0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010966a: movea.w $4471f0.l, a0
                    e.step()?;
                    let value_address = 0x4471f0_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 00109670: cmpa.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00109672: ble.b $10967a
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x10967a_u32
                    } else {
                        0x109674_u32
                    };
                }
                0x109674 => {
                    // 00109674: addq.b #$1, $442736.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x442736_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    let value = r.add_flags(destination, source, 8, true);
                    r.write_memory(e, destination_address, 8, value)?;
                    pc = 0x10967a_u32;
                }
                0x10967a => {
                    // 0010967a: tst.b $442736.l
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109680: bne.b $109690
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109690_u32
                    } else {
                        0x109682_u32
                    };
                }
                0x109682 => {
                    // 00109682: tst.b $442737.l
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109688: bne.b $109690
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109690_u32
                    } else {
                        0x10968a_u32
                    };
                }
                0x10968a => {
                    // 0010968a: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010968c: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x109690 => {
                    // 00109690: moveq #$0, d7
                    e.step()?;
                    r.put_data(7, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00109692: cmpi.b #$ff, $442736.l
                    e.step()?;
                    let source = 0xff_u32;
                    let destination_address = 0x442736_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 0010969a: bne.b $1096a6
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1096a6_u32
                    } else {
                        0x10969c_u32
                    };
                }
                0x10969c => {
                    // 0010969c: move.b #$ff, $442740.l
                    e.step()?;
                    let value = 0xff_u32;
                    let destination_address = 0x442740_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 001096a4: bra.b $1096b6
                    e.step()?;
                    pc = 0x1096b6_u32;
                }
                0x1096a6 => {
                    // 001096a6: cmpi.b #$1, $442736.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x442736_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 001096ae: bne.b $1096b6
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1096b6_u32
                    } else {
                        0x1096b0_u32
                    };
                }
                0x1096b0 => {
                    // 001096b0: clr.b $442740.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442740_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x1096b6_u32;
                }
                0x1096b6 => {
                    // 001096b6: tst.b $442736.l
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001096bc: ble.b $1096c8
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1096c8_u32
                    } else {
                        0x1096be_u32
                    };
                }
                0x1096be => {
                    // 001096be: move.b #$1, $435883.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x435883_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 001096c6: bra.b $1096d6
                    e.step()?;
                    pc = 0x1096d6_u32;
                }
                0x1096c8 => {
                    // 001096c8: tst.b $442736.l
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001096ce: bge.b $1096d6
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x1096d6_u32
                    } else {
                        0x1096d0_u32
                    };
                }
                0x1096d0 => {
                    // 001096d0: clr.b $435883.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x435883_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x1096d6_u32;
                }
                0x1096d6 => {
                    // 001096d6: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001096dc: add.l $442778.l, d0
                    e.step()?;
                    let source_address = 0x442778_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001096e2: move.b $442736.l, d1
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 001096e8: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001096ea: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001096ec: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 001096ee: muls.l #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 001096f6: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 001096fc: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001096fe: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109704: add.l $44277c.l, d0
                    e.step()?;
                    let source_address = 0x44277c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010970a: move.b $442737.l, d1
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 00109710: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00109712: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00109714: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 00109716: move.w (a0, d1.l * 2), d5
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 0010971a: move.w d5, -$4(a6)
                    e.step()?;
                    let value = r.data(5, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010971e: cmpi.w #$ffff, d5
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination = r.data(5, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00109722: ble.b $109750
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109750_u32
                    } else {
                        0x109724_u32
                    };
                }
                0x109724 => {
                    // 00109724: tst.b d7
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.move_flags(value, 8);
                    // 00109726: bne.b $109740
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109740_u32
                    } else {
                        0x109728_u32
                    };
                }
                0x109728 => {
                    // 00109728: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 0010972a: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010972c: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 00109732: move.w (a0, d0.l), -(a7)
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
                    // 00109736: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10973c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010973c: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010973e: bra.b $10975e
                    e.step()?;
                    pc = 0x10975e_u32;
                }
                0x109740 => {
                    // 00109740: move.w #$52, -(a7)
                    e.step()?;
                    let value = 0x52_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109744: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10974a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010974a: moveq #$0, d7
                    e.step()?;
                    r.put_data(7, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010974c: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010974e: bra.b $10975e
                    e.step()?;
                    pc = 0x10975e_u32;
                }
                0x109750 => {
                    // 00109750: move.w #$52, -(a7)
                    e.step()?;
                    let value = 0x52_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109754: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10975a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010975a: moveq #$1, d7
                    e.step()?;
                    r.put_data(7, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 0010975c: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x10975e_u32;
                }
                0x10975e => {
                    // 0010975e: move.w d5, -(a7)
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109760: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109766_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00109766: move.w d0, d5
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 00109768: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010976e: add.l $442778.l, d0
                    e.step()?;
                    let source_address = 0x442778_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00109774: move.b $442736.l, d1
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 0010977a: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0010977c: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0010977e: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 00109780: muls.l #$168, d1
                    e.step()?;
                    let source = 0x168_u32;
                    let destination = r.data(1, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 00109788: lea.l $414390.l, a0
                    e.step()?;
                    let value = 0x414390_u32;
                    r.put_address(0, value);
                    // 0010978e: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00109790: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109796: add.l $44277c.l, d0
                    e.step()?;
                    let source_address = 0x44277c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010979c: move.b $442737.l, d1
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 001097a2: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001097a4: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001097a6: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 001097a8: tst.l (a0, d1.l * 4)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(4));
                    let value = e.l(value_address)?;
                    r.move_flags(value, 32);
                    // 001097ac: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001097ae: ble.w $1098ae
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1098ae_u32
                    } else {
                        0x1097b2_u32
                    };
                }
                0x1097b2 => {
                    // 001097b2: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001097b8: add.l $442778.l, d0
                    e.step()?;
                    let source_address = 0x442778_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001097be: move.b $442736.l, d1
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 001097c4: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001097c6: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001097c8: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 001097ca: muls.l #$b4, d1
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(1, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 001097d2: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 001097d8: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001097da: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001097e0: add.l $44277c.l, d0
                    e.step()?;
                    let source_address = 0x44277c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001097e6: move.b $442737.l, d1
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(1, 8, value);
                    r.move_flags(value, 8);
                    // 001097ec: ext.w d1
                    e.step()?;
                    let source = r.data(1, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001097ee: ext.l d1
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let value = i32::from(source as u16 as i16) as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 001097f0: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 001097f2: move.w (a0, d1.l * 2), d6
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 001097f6: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001097fc: add.l $442778.l, d0
                    e.step()?;
                    let source_address = 0x442778_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00109802: move.l d0, $442780.l
                    e.step()?;
                    let value = r.data(0, 32)?;
                    let destination_address = 0x442780_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109808: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010980e: add.l $44277c.l, d0
                    e.step()?;
                    let source_address = 0x44277c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00109814: move.l d0, $442784.l
                    e.step()?;
                    let value = r.data(0, 32)?;
                    let destination_address = 0x442784_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010981a: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010981e: bsr.l $13ea82
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109824_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13ea82_u32)?;
                    // 00109824: cmpi.w #$ffff, d6
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00109828: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010982a: ble.b $109848
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109848_u32
                    } else {
                        0x10982c_u32
                    };
                }
                0x10982c => {
                    // 0010982c: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109830: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 00109832: mulu.w d6, d0
                    e.step()?;
                    let source = r.data(6, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109834: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 0010983a: move.w $2(a0, d0.l), -(a7)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x2_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010983e: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109844_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 00109844: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109846: bra.b $109856
                    e.step()?;
                    pc = 0x109856_u32;
                }
                0x109848 => {
                    // 00109848: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010984e: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109854_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 00109854: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x109856_u32;
                }
                0x109856 => {
                    // 00109856: tst.b $4386b2.l
                    e.step()?;
                    let value_address = 0x4386b2_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010985c: beq.b $109864
                    e.step()?;
                    pc = if r.equal()? {
                        0x109864_u32
                    } else {
                        0x10985e_u32
                    };
                }
                0x10985e => {
                    // 0010985e: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00109860: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x109864 => {
                    // 00109864: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109866: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109868: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010986e: add.l $44277c.l, d0
                    e.step()?;
                    let source_address = 0x44277c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00109874: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109876: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010987c: add.l $442778.l, d0
                    e.step()?;
                    let source_address = 0x442778_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00109882: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109884: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10988a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 0010988a: cmpi.w #$258, d5
                    e.step()?;
                    let source = 0x258_u32;
                    let destination = r.data(5, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010988e: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    // 00109892: bge.w $109afe
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x109afe_u32
                    } else {
                        0x109896_u32
                    };
                }
                0x109896 => {
                    // 00109896: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 00109898: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010989a: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 001098a0: cmpi.w #$1, $8(a0, d0.l)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x8_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 001098a6: beq.w $1099c8
                    e.step()?;
                    pc = if r.equal()? {
                        0x1099c8_u32
                    } else {
                        0x1098aa_u32
                    };
                }
                0x1098aa => {
                    // 001098aa: bra.w $109afe
                    e.step()?;
                    pc = 0x109afe_u32;
                }
                0x1098ae => {
                    // 001098ae: tst.b $442736.l
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001098b4: bne.b $1098c0
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1098c0_u32
                    } else {
                        0x1098b6_u32
                    };
                }
                0x1098b6 => {
                    // 001098b6: tst.b $442737.l
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001098bc: beq.w $109afe
                    e.step()?;
                    pc = if r.equal()? {
                        0x109afe_u32
                    } else {
                        0x1098c0_u32
                    };
                }
                0x1098c0 => {
                    // 001098c0: cmpi.w #$c8, d5
                    e.step()?;
                    let source = 0xc8_u32;
                    let destination = r.data(5, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001098c4: bgt.b $1098cc
                    e.step()?;
                    pc = if r.greater()? {
                        0x1098cc_u32
                    } else {
                        0x1098c6_u32
                    };
                }
                0x1098c6 => {
                    // 001098c6: tst.w -$4(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001098ca: bge.b $1098de
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x1098de_u32
                    } else {
                        0x1098cc_u32
                    };
                }
                0x1098cc => {
                    // 001098cc: move.b $402f43.l, d0
                    e.step()?;
                    let value_address = 0x402f43_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001098d2: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001098d4: lea.l $435508.l, a0
                    e.step()?;
                    let value = 0x435508_u32;
                    r.put_address(0, value);
                    // 001098da: move.w (a0, d0.l * 2), d5
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x1098de_u32;
                }
                0x1098de => {
                    // 001098de: tst.b $442748.l
                    e.step()?;
                    let value_address = 0x442748_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 001098e4: beq.w $1099b6
                    e.step()?;
                    pc = if r.equal()? {
                        0x1099b6_u32
                    } else {
                        0x1098e8_u32
                    };
                }
                0x1098e8 => {
                    // 001098e8: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 001098ea: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001098ec: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 001098f2: tst.w $4(a0, d0.l)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x4_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001098f6: beq.b $109922
                    e.step()?;
                    pc = if r.equal()? {
                        0x109922_u32
                    } else {
                        0x1098f8_u32
                    };
                }
                0x1098f8 => {
                    // 001098f8: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 001098fa: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001098fc: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 00109902: cmpi.w #$2, $8(a0, d0.l)
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x8_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109908: beq.b $109922
                    e.step()?;
                    pc = if r.equal()? {
                        0x109922_u32
                    } else {
                        0x10990a_u32
                    };
                }
                0x10990a => {
                    // 0010990a: move.w #$ff6c, -(a7)
                    e.step()?;
                    let value = 0xff6c_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010990e: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109914_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 00109914: clr.b $442737.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010991a: clr.b $442736.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00109920: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x109922_u32;
                }
                0x109922 => {
                    // 00109922: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 00109924: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109926: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 0010992c: tst.w $6(a0, d0.l)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x6_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00109930: beq.w $109a22
                    e.step()?;
                    pc = if r.equal()? {
                        0x109a22_u32
                    } else {
                        0x109934_u32
                    };
                }
                0x109934 => {
                    // 00109934: move.w #$ff6c, -(a7)
                    e.step()?;
                    let value = 0xff6c_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109938: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10993e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010993e: clr.b $442737.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00109944: clr.b $442736.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010994a: addq.w #$1, $437a86.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x437a86_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 00109950: cmpi.w #$2, $437a86.l
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = 0x437a86_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109958: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010995a: ble.w $109a22
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109a22_u32
                    } else {
                        0x10995e_u32
                    };
                }
                0x10995e => {
                    // 0010995e: clr.w $437a86.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x437a86_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109964: move.l #$e0000, -(a7)
                    e.step()?;
                    let value = 0xe0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010996a: bsr.l $14cff6
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109970_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14cff6_u32)?;
                    // 00109970: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00109972: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109974: beq.w $109a22
                    e.step()?;
                    pc = if r.equal()? {
                        0x109a22_u32
                    } else {
                        0x109978_u32
                    };
                }
                0x109978 => {
                    // 00109978: clr.w $443316.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x443316_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010997e: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00109980: move.b d0, $442748.l
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = 0x442748_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00109986: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010998c: add.l $442778.l, d0
                    e.step()?;
                    let source_address = 0x442778_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00109992: muls.l #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 0010999a: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 001099a0: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001099a2: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001099a8: add.l $44277c.l, d0
                    e.step()?;
                    let source_address = 0x44277c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001099ae: move.w #$93, (a0, d0.l * 2)
                    e.step()?;
                    let value = 0x93_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001099b4: bra.b $109a22
                    e.step()?;
                    pc = 0x109a22_u32;
                }
                0x1099b6 => {
                    // 001099b6: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 001099b8: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001099ba: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 001099c0: cmpi.w #$1, $8(a0, d0.l)
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x8_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 001099c6: bne.b $109a22
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109a22_u32
                    } else {
                        0x1099c8_u32
                    };
                }
                0x1099c8 => {
                    // 001099c8: move.l #$f0000, -(a7)
                    e.step()?;
                    let value = 0xf0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001099ce: bsr.l $14cff6
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1099d4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14cff6_u32)?;
                    // 001099d4: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 001099d6: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001099d8: beq.b $109a22
                    e.step()?;
                    pc = if r.equal()? {
                        0x109a22_u32
                    } else {
                        0x1099da_u32
                    };
                }
                0x1099da => {
                    // 001099da: move.b #$1, $442748.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x442748_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 001099e2: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001099e6: bsr.l $13ea82
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1099ec_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13ea82_u32)?;
                    // 001099ec: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001099f2: add.l $442778.l, d0
                    e.step()?;
                    let source_address = 0x442778_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001099f8: muls.l #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 00109a00: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 00109a06: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00109a08: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109a0e: add.l $44277c.l, d0
                    e.step()?;
                    let source_address = 0x44277c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00109a14: move.w #$3c, (a0, d0.l * 2)
                    e.step()?;
                    let value = 0x3c_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109a1a: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109a20_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 00109a20: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x109a22_u32;
                }
                0x109a22 => {
                    // 00109a22: tst.b $442736.l
                    e.step()?;
                    let value_address = 0x442736_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109a28: bne.b $109a32
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109a32_u32
                    } else {
                        0x109a2a_u32
                    };
                }
                0x109a2a => {
                    // 00109a2a: tst.b $442737.l
                    e.step()?;
                    let value_address = 0x442737_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109a30: beq.b $109a38
                    e.step()?;
                    pc = if r.equal()? {
                        0x109a38_u32
                    } else {
                        0x109a32_u32
                    };
                }
                0x109a32 => {
                    // 00109a32: clr.w $437a86.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x437a86_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x109a38_u32;
                }
                0x109a38 => {
                    // 00109a38: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 00109a3a: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109a3c: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 00109a42: tst.w $4(a0, d0.l)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x4_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00109a46: bne.b $109a76
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109a76_u32
                    } else {
                        0x109a48_u32
                    };
                }
                0x109a48 => {
                    // 00109a48: tst.b $442748.l
                    e.step()?;
                    let value_address = 0x442748_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109a4e: bne.b $109a76
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109a76_u32
                    } else {
                        0x109a50_u32
                    };
                }
                0x109a50 => {
                    // 00109a50: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 00109a52: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109a54: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 00109a5a: cmpi.w #$2, $8(a0, d0.l)
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x8_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109a60: beq.b $109a76
                    e.step()?;
                    pc = if r.equal()? {
                        0x109a76_u32
                    } else {
                        0x109a62_u32
                    };
                }
                0x109a62 => {
                    // 00109a62: tst.b -$1(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109a66: bne.b $109a76
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109a76_u32
                    } else {
                        0x109a68_u32
                    };
                }
                0x109a68 => {
                    // 00109a68: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109a6c: bsr.l $13ea82
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109a72_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13ea82_u32)?;
                    // 00109a72: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109a74: bra.b $109a9c
                    e.step()?;
                    pc = 0x109a9c_u32;
                }
                0x109a76 => {
                    // 00109a76: tst.b $442748.l
                    e.step()?;
                    let value_address = 0x442748_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109a7c: beq.b $109a9c
                    e.step()?;
                    pc = if r.equal()? {
                        0x109a9c_u32
                    } else {
                        0x109a7e_u32
                    };
                }
                0x109a7e => {
                    // 00109a7e: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 00109a80: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109a82: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 00109a88: cmpi.w #$2, $8(a0, d0.l)
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x8_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109a8e: bne.b $109a9c
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109a9c_u32
                    } else {
                        0x109a90_u32
                    };
                }
                0x109a90 => {
                    // 00109a90: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109a94: bsr.l $13ea82
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109a9a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13ea82_u32)?;
                    // 00109a9a: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x109a9c_u32;
                }
                0x109a9c => {
                    // 00109a9c: clr.b -$1(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00109aa0: tst.b d7
                    e.step()?;
                    let value = r.data(7, 8)?;
                    r.move_flags(value, 8);
                    // 00109aa2: bne.b $109ad8
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109ad8_u32
                    } else {
                        0x109aa4_u32
                    };
                }
                0x109aa4 => {
                    // 00109aa4: tst.w d5
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.move_flags(value, 16);
                    // 00109aa6: ble.b $109af0
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109af0_u32
                    } else {
                        0x109aa8_u32
                    };
                }
                0x109aa8 => {
                    // 00109aa8: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 00109aaa: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109aac: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 00109ab2: move.w (a0, d0.l), -(a7)
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
                    // 00109ab6: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109abc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 00109abc: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109ac0: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 00109ac2: mulu.w d5, d0
                    e.step()?;
                    let source = r.data(5, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109ac4: lea.l $43167c.l, a0
                    e.step()?;
                    let value = 0x43167c_u32;
                    r.put_address(0, value);
                    // 00109aca: move.w $2(a0, d0.l), -(a7)
                    e.step()?;
                    let value_address = r
                        .address(0)?
                        .wrapping_add(0x2_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109ace: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109ad4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 00109ad4: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109ad6: bra.b $109af0
                    e.step()?;
                    pc = 0x109af0_u32;
                }
                0x109ad8 => {
                    // 00109ad8: move.w #$52, -(a7)
                    e.step()?;
                    let value = 0x52_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109adc: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109ae2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 00109ae2: move.l #$30001, -(a7)
                    e.step()?;
                    let value = 0x30001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109ae8: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109aee_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 00109aee: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x109af0_u32;
                }
                0x109af0 => {
                    // 00109af0: tst.b $4386b2.l
                    e.step()?;
                    let value_address = 0x4386b2_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109af6: beq.b $109afe
                    e.step()?;
                    pc = if r.equal()? {
                        0x109afe_u32
                    } else {
                        0x109af8_u32
                    };
                }
                0x109af8 => {
                    // 00109af8: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00109afa: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x109afe => {
                    // 00109afe: clr.w $443316.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x443316_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109b04: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109b06: bsr.l $11177e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109b0c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x11177e_u32)?;
                    // 00109b0c: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00109b0e: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109b10: beq.b $109b3e
                    e.step()?;
                    pc = if r.equal()? {
                        0x109b3e_u32
                    } else {
                        0x109b12_u32
                    };
                }
                0x109b12 => {
                    // 00109b12: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109b14: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00109b18: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109b1e: add.l $44277c.l, d0
                    e.step()?;
                    let source_address = 0x44277c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00109b24: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109b26: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109b2c: add.l $442778.l, d0
                    e.step()?;
                    let source_address = 0x442778_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 00109b32: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109b34: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109b3a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 00109b3a: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x109b3e_u32;
                }
                0x109b3e => {
                    // 00109b3e: bsr.l $133c56
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109b44_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133c56_u32)?;
                    // 00109b44: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00109b46: MacToolbox 0xa973
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa973_u32 as u16))?;
                    // 00109b48: move.b (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.b(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00109b4a: bne.b $109b54
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109b54_u32
                    } else {
                        0x109b4c_u32
                    };
                }
                0x109b4c => {
                    // 00109b4c: tst.w $443316.l
                    e.step()?;
                    let value_address = 0x443316_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00109b52: beq.b $109b6a
                    e.step()?;
                    pc = if r.equal()? {
                        0x109b6a_u32
                    } else {
                        0x109b54_u32
                    };
                }
                0x109b54 => {
                    // 00109b54: cmpi.w #$87, $442578.l
                    e.step()?;
                    let source = 0x87_u32;
                    let destination_address = 0x442578_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109b5c: blt.w $1095bc
                    e.step()?;
                    pc = if r.less()? {
                        0x1095bc_u32
                    } else {
                        0x109b60_u32
                    };
                }
                0x109b60 => {
                    // 00109b60: tst.w $4394b4.l
                    e.step()?;
                    let value_address = 0x4394b4_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00109b66: bne.w $1095bc
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1095bc_u32
                    } else {
                        0x109b6a_u32
                    };
                }
                0x109b6a => {
                    // 00109b6a: move.l $43878c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43878c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109b70: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 00109b72: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109b78: cmpa.l $439448.l, a0
                    e.step()?;
                    let source_address = 0x439448_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00109b7e: bne.b $109be2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109be2_u32
                    } else {
                        0x109b80_u32
                    };
                }
                0x109b80 => {
                    // 00109b80: move.l #$70007, -(a7)
                    e.step()?;
                    let value = 0x70007_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109b86: bsr.l $12382a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109b8c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12382a_u32)?;
                    // 00109b8c: tst.w $44256c.l
                    e.step()?;
                    let value_address = 0x44256c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00109b92: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109b94: beq.b $109bce
                    e.step()?;
                    pc = if r.equal()? {
                        0x109bce_u32
                    } else {
                        0x109b96_u32
                    };
                }
                0x109b96 => {
                    // 00109b96: move.w #$cd, -(a7)
                    e.step()?;
                    let value = 0xcd_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109b9a: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109ba0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 00109ba0: clr.w $44256c.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x44256c_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109ba6: move.l $43878c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43878c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109bac: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 00109bae: movea.l $439448.l, a0
                    e.step()?;
                    let value_address = 0x439448_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109bb4: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109bb6: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109bba: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109bbe: move.w #$80, -(a7)
                    e.step()?;
                    let value = 0x80_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109bc2: bsr.l $14a02c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109bc8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14a02c_u32)?;
                    // 00109bc8: lea.l $c(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xc_u32);
                    r.put_address(7, value);
                    // 00109bcc: bra.b $109be2
                    e.step()?;
                    pc = 0x109be2_u32;
                }
                0x109bce => {
                    // 00109bce: move.w #$d0, -(a7)
                    e.step()?;
                    let value = 0xd0_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109bd2: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109bd8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 00109bd8: move.w #$ffff, $44256c.l
                    e.step()?;
                    let value = 0xffff_u32;
                    let destination_address = 0x44256c_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109be0: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x109be2_u32;
                }
                0x109be2 => {
                    // 00109be2: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109be8: cmpa.l $439480.l, a0
                    e.step()?;
                    let source_address = 0x439480_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00109bee: bne.b $109c44
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109c44_u32
                    } else {
                        0x109bf0_u32
                    };
                }
                0x109bf0 => {
                    // 00109bf0: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109bf6: move.l #$3210001, -(a7)
                    e.step()?;
                    let value = 0x3210001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109bfc: bsr.l $112650
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109c02_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x112650_u32)?;
                    // 00109c02: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00109c04: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109c06: beq.b $109c44
                    e.step()?;
                    pc = if r.equal()? {
                        0x109c44_u32
                    } else {
                        0x109c08_u32
                    };
                }
                0x109c08 => {
                    // 00109c08: move.w #$321, -(a7)
                    e.step()?;
                    let value = 0x321_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109c0c: bsr.l $130afe
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109c12_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x130afe_u32)?;
                    // 00109c12: move.w $453668.l, -(a7)
                    e.step()?;
                    let value_address = 0x453668_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109c18: bsr.l $13145c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109c1e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13145c_u32)?;
                    // 00109c1e: move.w $453666.l, -(a7)
                    e.step()?;
                    let value_address = 0x453666_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109c24: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109c2a_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00109c2a: move.w d0, $4435cc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4435cc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109c30: move.w $443774.l, -(a7)
                    e.step()?;
                    let value_address = 0x443774_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109c36: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109c3c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 00109c3c: bsr.l $151022
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109c42_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151022_u32)?;
                    // 00109c42: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x109c44_u32;
                }
                0x109c44 => {
                    // 00109c44: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109c4a: cmpa.l $439478.l, a0
                    e.step()?;
                    let source_address = 0x439478_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00109c50: bne.w $109f26
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109f26_u32
                    } else {
                        0x109c54_u32
                    };
                }
                0x109c54 => {
                    // 00109c54: move.l #$80008, -(a7)
                    e.step()?;
                    let value = 0x80008_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109c5a: bsr.l $12382a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109c60_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12382a_u32)?;
                    // 00109c60: cmpi.w #$86, $442578.l
                    e.step()?;
                    let source = 0x86_u32;
                    let destination_address = 0x442578_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109c68: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109c6a: ble.b $109c8a
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109c8a_u32
                    } else {
                        0x109c6c_u32
                    };
                }
                0x109c6c => {
                    // 00109c6c: move.w #$36, -(a7)
                    e.step()?;
                    let value = 0x36_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109c70: bsr.l $16474a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109c76_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16474a_u32)?;
                    // 00109c76: move.l #$500050, -(a7)
                    e.step()?;
                    let value = 0x500050_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109c7c: bsr.l $12382a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109c82_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12382a_u32)?;
                    // 00109c82: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00109c84: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109c86: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x109c8a => {
                    // 00109c8a: move.w #$2779, -(a7)
                    e.step()?;
                    let value = 0x2779_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109c8e: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109c94_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 00109c94: movea.l $439478.l, a0
                    e.step()?;
                    let value_address = 0x439478_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109c9a: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109c9c: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109ca0: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109ca4: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109ca6: move.l #$60002, -(a7)
                    e.step()?;
                    let value = 0x60002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109cac: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109cb2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 00109cb2: lea.l $10(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x10_u32);
                    r.put_address(7, value);
                    pc = 0x109cb6_u32;
                }
                0x109cb6 => {
                    // 00109cb6: move.w #$2, -(a7)
                    e.step()?;
                    let value = 0x2_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109cba: bsr.l $1205f8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109cc0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1205f8_u32)?;
                    // 00109cc0: move.l #$1ffff, -(a7)
                    e.step()?;
                    let value = 0x1ffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109cc6: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109ccc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 00109ccc: tst.b $4386b2.l
                    e.step()?;
                    let value_address = 0x4386b2_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109cd2: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109cd4: beq.b $109cdc
                    e.step()?;
                    pc = if r.equal()? {
                        0x109cdc_u32
                    } else {
                        0x109cd6_u32
                    };
                }
                0x109cd6 => {
                    // 00109cd6: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00109cd8: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x109cdc => {
                    // 00109cdc: bsr.l $15ec56
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109ce2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15ec56_u32)?;
                    // 00109ce2: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00109ce4: bra.w $109ee2
                    e.step()?;
                    pc = 0x109ee2_u32;
                }
                0x109ce8 => {
                    // 00109ce8: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109cea: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109cee: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00109cf4: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00109cf6: tst.b $9dc(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9dc_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109cfa: beq.w $109ee0
                    e.step()?;
                    pc = if r.equal()? {
                        0x109ee0_u32
                    } else {
                        0x109cfe_u32
                    };
                }
                0x109cfe => {
                    // 00109cfe: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109d00: move.w d4, -(a7)
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109d02: bsr.l $13beea
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109d08_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13beea_u32)?;
                    // 00109d08: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00109d0a: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109d0c: beq.w $109ee0
                    e.step()?;
                    pc = if r.equal()? {
                        0x109ee0_u32
                    } else {
                        0x109d10_u32
                    };
                }
                0x109d10 => {
                    // 00109d10: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00109d12: bra.w $109db0
                    e.step()?;
                    pc = 0x109db0_u32;
                }
                0x109d16 => {
                    // 00109d16: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109d18: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109d1c: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00109d22: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00109d24: mulu.w #$c42, d1
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00109d28: lea.l $448aa0.l, a1
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(1, value);
                    // 00109d2e: move.w $50(a1, d1.l), d1
                    e.step()?;
                    let value_address = r
                        .address(1)?
                        .wrapping_add(0x50_u32)
                        .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00109d32: cmp.w $52(a0, d0.l), d1
                    e.step()?;
                    let source_address = r
                        .address(0)?
                        .wrapping_add(0x52_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00109d36: bge.b $109dae
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x109dae_u32
                    } else {
                        0x109d38_u32
                    };
                }
                0x109d38 => {
                    // 00109d38: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109d3a: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109d3e: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00109d44: cmpi.w #$fff6, $50(a0, d0.l)
                    e.step()?;
                    let source = 0xfff6_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x50_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109d4a: ble.b $109dae
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109dae_u32
                    } else {
                        0x109d4c_u32
                    };
                }
                0x109d4c => {
                    // 00109d4c: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109d4e: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109d52: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00109d58: cmpi.w #$3, $32(a0, d0.l)
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x32_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109d5e: beq.b $109dae
                    e.step()?;
                    pc = if r.equal()? {
                        0x109dae_u32
                    } else {
                        0x109d60_u32
                    };
                }
                0x109d60 => {
                    // 00109d60: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109d62: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109d66: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00109d6c: cmpi.w #$9, $58(a0, d0.l)
                    e.step()?;
                    let source = 0x9_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x58_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109d72: ble.b $109dae
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109dae_u32
                    } else {
                        0x109d74_u32
                    };
                }
                0x109d74 => {
                    // 00109d74: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109d76: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109d7a: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00109d80: lea.l $58(a0, d0.l), a0
                    e.step()?;
                    let value = r
                        .address(0)?
                        .wrapping_add(0x58_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    r.put_address(0, value);
                    // 00109d84: subi.w #$a, (a0)
                    e.step()?;
                    let source = 0xa_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 00109d88: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109d8a: move.w d4, -(a7)
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109d8c: bsr.l $1600fc
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109d92_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1600fc_u32)?;
                    // 00109d92: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109d94: move.w #$8, -(a7)
                    e.step()?;
                    let value = 0x8_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109d98: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109d9e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 00109d9e: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109da0: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109da2: move.w d3, -(a7)
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109da4: bsr.l $1287c2
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109daa_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1287c2_u32)?;
                    // 00109daa: lea.l $a(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xa_u32);
                    r.put_address(7, value);
                    pc = 0x109dae_u32;
                }
                0x109dae => {
                    // 00109dae: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x109db0_u32;
                }
                0x109db0 => {
                    // 00109db0: move.b $44273a.l, d0
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00109db6: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109db8: cmp.w d0, d3
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00109dba: ble.w $109d16
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109d16_u32
                    } else {
                        0x109dbe_u32
                    };
                }
                0x109dbe => {
                    // 00109dbe: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00109dc0: bra.w $109ed6
                    e.step()?;
                    pc = 0x109ed6_u32;
                }
                0x109dc4 => {
                    // 00109dc4: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109dc6: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109dca: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 00109dd0: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00109dd2: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109dd4: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109dd8: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 00109dde: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 00109de0: move.w $110(a1), d0
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x110_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109de4: cmp.w $112(a0), d0
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x112_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00109de8: bge.w $109ed4
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x109ed4_u32
                    } else {
                        0x109dec_u32
                    };
                }
                0x109dec => {
                    // 00109dec: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109dee: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109df2: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 00109df8: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00109dfa: cmpi.w #$fff6, $110(a0)
                    e.step()?;
                    let source = 0xfff6_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x110_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109e00: ble.w $109ed4
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109ed4_u32
                    } else {
                        0x109e04_u32
                    };
                }
                0x109e04 => {
                    // 00109e04: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109e06: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109e0a: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00109e10: cmpi.w #$9, $58(a0, d0.l)
                    e.step()?;
                    let source = 0x9_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x58_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109e16: ble.w $109ed4
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109ed4_u32
                    } else {
                        0x109e1a_u32
                    };
                }
                0x109e1a => {
                    // 00109e1a: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109e1c: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109e20: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00109e26: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00109e28: tst.b $9dc(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x9dc_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00109e2c: beq.w $109ed4
                    e.step()?;
                    pc = if r.equal()? {
                        0x109ed4_u32
                    } else {
                        0x109e30_u32
                    };
                }
                0x109e30 => {
                    // 00109e30: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109e32: move.w d4, -(a7)
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109e34: bsr.l $13beea
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109e3a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13beea_u32)?;
                    // 00109e3a: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00109e3c: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109e3e: beq.w $109ed4
                    e.step()?;
                    pc = if r.equal()? {
                        0x109ed4_u32
                    } else {
                        0x109e42_u32
                    };
                }
                0x109e42 => {
                    // 00109e42: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109e44: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109e48: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 00109e4e: lea.l $58(a0, d0.l), a0
                    e.step()?;
                    let value = r
                        .address(0)?
                        .wrapping_add(0x58_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    r.put_address(0, value);
                    // 00109e52: subi.w #$a, (a0)
                    e.step()?;
                    let source = 0xa_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 00109e56: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109e58: move.w #$8, -(a7)
                    e.step()?;
                    let value = 0x8_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109e5c: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109e62_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 00109e62: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109e64: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109e66: move.w d3, -(a7)
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109e68: bsr.l $1287c2
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109e6e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1287c2_u32)?;
                    // 00109e6e: move.w #$8, -(a7)
                    e.step()?;
                    let value = 0x8_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109e72: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109e78_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 00109e78: move.w d3, d1
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 00109e7a: mulu.w #$120, d1
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00109e7e: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 00109e84: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00109e86: add.w d0, $110(a0)
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination_address = r.address(0)?.wrapping_add(0x110_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 00109e8a: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109e8c: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109e90: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 00109e96: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00109e98: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109e9a: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109e9e: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 00109ea4: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 00109ea6: move.w $110(a1), d0
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x110_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109eaa: cmp.w $112(a0), d0
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x112_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00109eae: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00109eb0: ble.b $109ed4
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109ed4_u32
                    } else {
                        0x109eb2_u32
                    };
                }
                0x109eb2 => {
                    // 00109eb2: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109eb4: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109eb8: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 00109ebe: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00109ec0: move.w d3, d0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109ec2: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00109ec6: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 00109ecc: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 00109ece: move.w $112(a0), $110(a1)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x112_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(1)?.wrapping_add(0x110_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x109ed4_u32;
                }
                0x109ed4 => {
                    // 00109ed4: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x109ed6_u32;
                }
                0x109ed6 => {
                    // 00109ed6: cmp.w $44257c.l, d3
                    e.step()?;
                    let source_address = 0x44257c_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00109edc: blt.w $109dc4
                    e.step()?;
                    pc = if r.less()? {
                        0x109dc4_u32
                    } else {
                        0x109ee0_u32
                    };
                }
                0x109ee0 => {
                    // 00109ee0: addq.w #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    pc = 0x109ee2_u32;
                }
                0x109ee2 => {
                    // 00109ee2: move.b $44273a.l, d0
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00109ee8: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109eea: cmp.w d0, d4
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(4, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00109eec: ble.w $109ce8
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x109ce8_u32
                    } else {
                        0x109ef0_u32
                    };
                }
                0x109ef0 => {
                    // 00109ef0: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 00109ef2: MacToolbox 0xa974
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa974_u32 as u16))?;
                    // 00109ef4: move.b (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.b(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00109ef6: beq.b $109f04
                    e.step()?;
                    pc = if r.equal()? {
                        0x109f04_u32
                    } else {
                        0x109ef8_u32
                    };
                }
                0x109ef8 => {
                    // 00109ef8: cmpi.w #$87, $442578.l
                    e.step()?;
                    let source = 0x87_u32;
                    let destination_address = 0x442578_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00109f00: blt.w $109cb6
                    e.step()?;
                    pc = if r.less()? {
                        0x109cb6_u32
                    } else {
                        0x109f04_u32
                    };
                }
                0x109f04 => {
                    // 00109f04: movea.l $439478.l, a0
                    e.step()?;
                    let value_address = 0x439478_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109f0a: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109f0c: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109f10: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109f14: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109f16: move.l #$60001, -(a7)
                    e.step()?;
                    let value = 0x60001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109f1c: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109f22_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 00109f22: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x109f26_u32;
                }
                0x109f26 => {
                    // 00109f26: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109f2c: cmpa.l $439494.l, a0
                    e.step()?;
                    let source_address = 0x439494_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00109f32: bne.b $109f8e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109f8e_u32
                    } else {
                        0x109f34_u32
                    };
                }
                0x109f34 => {
                    // 00109f34: movea.l $439494.l, a0
                    e.step()?;
                    let value_address = 0x439494_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109f3a: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109f3c: move.l $8(a0), $44292c.l
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    let destination_address = 0x44292c_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109f44: move.l $c(a0), $442930.l
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    let destination_address = 0x442930_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109f4c: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109f50: bsr.l $153720
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109f56_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153720_u32)?;
                    // 00109f56: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00109f5c: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109f5e: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109f60: move.l #$10000, -(a7)
                    e.step()?;
                    let value = 0x10000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109f66: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00109f6c: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109f6e: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109f70: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00109f76: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109f78: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109f7a: bsr.l $154a46
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109f80_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x154a46_u32)?;
                    // 00109f80: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109f84: bsr.l $1537ee
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109f8a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1537ee_u32)?;
                    // 00109f8a: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x109f8e_u32;
                }
                0x109f8e => {
                    // 00109f8e: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109f94: cmpa.l $439498.l, a0
                    e.step()?;
                    let source_address = 0x439498_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00109f9a: bne.b $109ff2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x109ff2_u32
                    } else {
                        0x109f9c_u32
                    };
                }
                0x109f9c => {
                    // 00109f9c: movea.l $439498.l, a0
                    e.step()?;
                    let value_address = 0x439498_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109fa2: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109fa4: move.l $8(a0), $44292c.l
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    let destination_address = 0x44292c_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109fac: move.l $c(a0), $442930.l
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    let destination_address = 0x442930_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109fb4: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109fb8: bsr.l $153720
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109fbe_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153720_u32)?;
                    // 00109fbe: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00109fc4: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109fc6: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109fc8: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00109fca: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00109fd0: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109fd2: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109fd4: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00109fda: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00109fdc: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109fde: bsr.l $154a46
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109fe4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x154a46_u32)?;
                    // 00109fe4: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00109fe8: bsr.l $1537ee
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x109fee_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1537ee_u32)?;
                    // 00109fee: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x109ff2_u32;
                }
                0x109ff2 => {
                    // 00109ff2: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 00109ff8: cmpa.l $439434.l, a0
                    e.step()?;
                    let source_address = 0x439434_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00109ffe: bne.w $10a0ac
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a0ac_u32
                    } else {
                        0x10a002_u32
                    };
                }
                0x10a002 => {
                    // 0010a002: move.l #$50005, -(a7)
                    e.step()?;
                    let value = 0x50005_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a008: bsr.l $12382a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a00e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12382a_u32)?;
                    // 0010a00e: movea.l $439434.l, a0
                    e.step()?;
                    let value_address = 0x439434_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a014: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a016: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a01a: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a01e: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a020: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a026: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a02c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a02c: move.w #$1771, -(a7)
                    e.step()?;
                    let value = 0x1771_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a030: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a036_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010a036: lea.l $14(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x14_u32);
                    r.put_address(7, value);
                    pc = 0x10a03a_u32;
                }
                0x10a03a => {
                    // 0010a03a: bsr.l $15ec56
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a040_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15ec56_u32)?;
                    // 0010a040: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a044: bsr.l $1205f8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a04a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1205f8_u32)?;
                    // 0010a04a: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a04c: move.l #$fffe, -(a7)
                    e.step()?;
                    let value = 0xfffe_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a052: bsr.l $160a8e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a058_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x160a8e_u32)?;
                    // 0010a058: move.l #$40000, -(a7)
                    e.step()?;
                    let value = 0x40000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a05e: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a064_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 0010a064: move.l #$1ffff, -(a7)
                    e.step()?;
                    let value = 0x1ffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a06a: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a070_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 0010a070: tst.b $4386b2.l
                    e.step()?;
                    let value_address = 0x4386b2_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010a076: lea.l $12(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x12_u32);
                    r.put_address(7, value);
                    // 0010a07a: beq.b $10a082
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a082_u32
                    } else {
                        0x10a07c_u32
                    };
                }
                0x10a07c => {
                    // 0010a07c: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010a07e: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10a082 => {
                    // 0010a082: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 0010a084: MacToolbox 0xa973
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa973_u32 as u16))?;
                    // 0010a086: move.b (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.b(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a088: bne.b $10a03a
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a03a_u32
                    } else {
                        0x10a08a_u32
                    };
                }
                0x10a08a => {
                    // 0010a08a: movea.l $439434.l, a0
                    e.step()?;
                    let value_address = 0x439434_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a090: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a092: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a096: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a09a: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a09c: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a0a2: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a0a8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a0a8: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x10a0ac_u32;
                }
                0x10a0ac => {
                    // 0010a0ac: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a0b2: cmpa.l $439458.l, a0
                    e.step()?;
                    let source_address = 0x439458_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010a0b8: bne.w $10a1f2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a1f2_u32
                    } else {
                        0x10a0bc_u32
                    };
                }
                0x10a0bc => {
                    // 0010a0bc: move.l #$40004, -(a7)
                    e.step()?;
                    let value = 0x40004_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a0c2: bsr.l $12382a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a0c8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12382a_u32)?;
                    // 0010a0c8: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a0ca: bsr.l $1500d8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a0d0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1500d8_u32)?;
                    // 0010a0d0: movea.l $439458.l, a0
                    e.step()?;
                    let value_address = 0x439458_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a0d6: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a0d8: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a0dc: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a0e0: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a0e2: move.l #$50002, -(a7)
                    e.step()?;
                    let value = 0x50002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a0e8: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a0ee_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a0ee: tst.b $44274f.l
                    e.step()?;
                    let value_address = 0x44274f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010a0f4: lea.l $14(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x14_u32);
                    r.put_address(7, value);
                    // 0010a0f8: bne.w $10a196
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a196_u32
                    } else {
                        0x10a0fc_u32
                    };
                }
                0x10a0fc => {
                    // 0010a0fc: tst.w $4394b4.l
                    e.step()?;
                    let value_address = 0x4394b4_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0010a102: bne.b $10a17a
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a17a_u32
                    } else {
                        0x10a104_u32
                    };
                }
                0x10a104 => {
                    // 0010a104: move.w #$83, -(a7)
                    e.step()?;
                    let value = 0x83_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a108: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a10e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010a10e: move.b #$1, $44274f.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x44274f_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a116: clr.b $44273b.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x44273b_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a11c: clr.b $442753.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442753_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a122: clr.b $442758.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442758_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a128: clr.b $442755.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442755_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a12e: clr.b $442756.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442756_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a134: clr.b $442737.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a13a: clr.b $442736.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a140: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a142: bsr.l $13ea82
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a148_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13ea82_u32)?;
                    // 0010a148: move.l #$30000, -(a7)
                    e.step()?;
                    let value = 0x30000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a14e: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a154_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 0010a154: move.l #$20001, -(a7)
                    e.step()?;
                    let value = 0x20001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a15a: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a160_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 0010a160: tst.b $4386b2.l
                    e.step()?;
                    let value_address = 0x4386b2_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010a166: lea.l $c(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xc_u32);
                    r.put_address(7, value);
                    // 0010a16a: beq.b $10a172
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a172_u32
                    } else {
                        0x10a16c_u32
                    };
                }
                0x10a16c => {
                    // 0010a16c: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010a16e: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10a172 => {
                    // 0010a172: bsr.l $16043e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a178_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16043e_u32)?;
                    // 0010a178: bra.b $10a1d0
                    e.step()?;
                    pc = 0x10a1d0_u32;
                }
                0x10a17a => {
                    // 0010a17a: pea.l $1771.w
                    e.step()?;
                    let value = 0x1771_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0010a17e: move.l #$320032, -(a7)
                    e.step()?;
                    let value = 0x320032_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a184: pea.l $403f24.l
                    e.step()?;
                    let value = 0x403f24_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0010a18a: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a190_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 0010a190: lea.l $c(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xc_u32);
                    r.put_address(7, value);
                    // 0010a194: bra.b $10a1d0
                    e.step()?;
                    pc = 0x10a1d0_u32;
                }
                0x10a196 => {
                    // 0010a196: move.w #$8d, -(a7)
                    e.step()?;
                    let value = 0x8d_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a19a: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a1a0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010a1a0: clr.b $44274f.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x44274f_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a1a6: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a1a8: bsr.l $13ea82
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a1ae_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13ea82_u32)?;
                    // 0010a1ae: move.l #$20000, -(a7)
                    e.step()?;
                    let value = 0x20000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a1b4: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a1ba_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 0010a1ba: tst.b $4386b2.l
                    e.step()?;
                    let value_address = 0x4386b2_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010a1c0: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010a1c2: beq.b $10a1ca
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a1ca_u32
                    } else {
                        0x10a1c4_u32
                    };
                }
                0x10a1c4 => {
                    // 0010a1c4: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010a1c6: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10a1ca => {
                    // 0010a1ca: bsr.l $16043e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a1d0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16043e_u32)?;
                    pc = 0x10a1d0_u32;
                }
                0x10a1d0 => {
                    // 0010a1d0: movea.l $439458.l, a0
                    e.step()?;
                    let value_address = 0x439458_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a1d6: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a1d8: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a1dc: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a1e0: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a1e2: move.l #$50001, -(a7)
                    e.step()?;
                    let value = 0x50001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a1e8: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a1ee_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a1ee: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x10a1f2_u32;
                }
                0x10a1f2 => {
                    // 0010a1f2: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a1f8: cmpa.l $43944c.l, a0
                    e.step()?;
                    let source_address = 0x43944c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010a1fe: bne.w $10a338
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a338_u32
                    } else {
                        0x10a202_u32
                    };
                }
                0x10a202 => {
                    // 0010a202: move.l #$60006, -(a7)
                    e.step()?;
                    let value = 0x60006_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a208: bsr.l $12382a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a20e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12382a_u32)?;
                    // 0010a20e: movea.l $43944c.l, a0
                    e.step()?;
                    let value_address = 0x43944c_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a214: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a216: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a21a: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a21e: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a220: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a226: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a22c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a22c: cmpi.w #$86, $442578.l
                    e.step()?;
                    let source = 0x86_u32;
                    let destination_address = 0x442578_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0010a234: lea.l $12(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x12_u32);
                    r.put_address(7, value);
                    // 0010a238: ble.b $10a278
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x10a278_u32
                    } else {
                        0x10a23a_u32
                    };
                }
                0x10a23a => {
                    // 0010a23a: move.w #$36, -(a7)
                    e.step()?;
                    let value = 0x36_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a23e: bsr.l $16474a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a244_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16474a_u32)?;
                    // 0010a244: movea.l $43944c.l, a0
                    e.step()?;
                    let value_address = 0x43944c_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a24a: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a24c: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a250: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a254: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a256: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a25c: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a262_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a262: move.l #$500050, -(a7)
                    e.step()?;
                    let value = 0x500050_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a268: bsr.l $12382a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a26e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12382a_u32)?;
                    // 0010a26e: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010a270: lea.l $14(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x14_u32);
                    r.put_address(7, value);
                    // 0010a274: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10a278 => {
                    // 0010a278: move.w #$1771, -(a7)
                    e.step()?;
                    let value = 0x1771_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a27c: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a282_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010a282: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x10a284_u32;
                }
                0x10a284 => {
                    // 0010a284: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a288: bsr.l $1205f8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a28e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1205f8_u32)?;
                    // 0010a28e: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a292: bsr.l $11177e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a298_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x11177e_u32)?;
                    // 0010a298: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0010a29a: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010a29c: beq.b $10a2ca
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a2ca_u32
                    } else {
                        0x10a29e_u32
                    };
                }
                0x10a29e => {
                    // 0010a29e: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a2a0: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0010a2a4: move.l $442768.l, d0
                    e.step()?;
                    let value_address = 0x442768_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a2aa: add.l $44277c.l, d0
                    e.step()?;
                    let source_address = 0x44277c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010a2b0: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a2b2: move.l $442764.l, d0
                    e.step()?;
                    let value_address = 0x442764_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a2b8: add.l $442778.l, d0
                    e.step()?;
                    let source_address = 0x442778_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010a2be: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a2c0: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a2c6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 0010a2c6: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x10a2ca_u32;
                }
                0x10a2ca => {
                    // 0010a2ca: tst.b $4386b2.l
                    e.step()?;
                    let value_address = 0x4386b2_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010a2d0: beq.b $10a2d8
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a2d8_u32
                    } else {
                        0x10a2d2_u32
                    };
                }
                0x10a2d2 => {
                    // 0010a2d2: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010a2d4: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10a2d8 => {
                    // 0010a2d8: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a2de: bsr.l $15e148
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a2e4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15e148_u32)?;
                    // 0010a2e4: tst.b $4386b2.l
                    e.step()?;
                    let value_address = 0x4386b2_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010a2ea: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010a2ec: beq.b $10a2f4
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a2f4_u32
                    } else {
                        0x10a2ee_u32
                    };
                }
                0x10a2ee => {
                    // 0010a2ee: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010a2f0: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10a2f4 => {
                    // 0010a2f4: bsr.l $15ec56
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a2fa_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15ec56_u32)?;
                    // 0010a2fa: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 0010a2fc: MacToolbox 0xa974
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa974_u32 as u16))?;
                    // 0010a2fe: move.b (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.b(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a300: beq.b $10a30e
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a30e_u32
                    } else {
                        0x10a302_u32
                    };
                }
                0x10a302 => {
                    // 0010a302: cmpi.w #$87, $442578.l
                    e.step()?;
                    let source = 0x87_u32;
                    let destination_address = 0x442578_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0010a30a: blt.w $10a284
                    e.step()?;
                    pc = if r.less()? {
                        0x10a284_u32
                    } else {
                        0x10a30e_u32
                    };
                }
                0x10a30e => {
                    // 0010a30e: move.l $43878c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43878c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a314: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 0010a316: movea.l $43944c.l, a0
                    e.step()?;
                    let value_address = 0x43944c_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a31c: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a31e: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a322: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a326: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a328: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a32e: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a334_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a334: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x10a338_u32;
                }
                0x10a338 => {
                    // 0010a338: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a33e: cmpa.l $439430.l, a0
                    e.step()?;
                    let source_address = 0x439430_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010a344: bne.w $10a43c
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a43c_u32
                    } else {
                        0x10a348_u32
                    };
                }
                0x10a348 => {
                    // 0010a348: move.w #$8d, -(a7)
                    e.step()?;
                    let value = 0x8d_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a34c: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a352_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010a352: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a358: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a35a: muls.w #$32, d0
                    e.step()?;
                    let source = 0x32_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a35e: move.w d0, $4434ba.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a364: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a366: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a36c: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a372: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a378: addi.w #$14a, d0
                    e.step()?;
                    let source = 0x14a_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a37c: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a382: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a384: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a38a: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a390: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a396: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a39c: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a39e: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a3a4: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a3aa_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a3aa: tst.b $442753.l
                    e.step()?;
                    let value_address = 0x442753_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010a3b0: lea.l $10(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x10_u32);
                    r.put_address(7, value);
                    // 0010a3b4: beq.w $10a43c
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a43c_u32
                    } else {
                        0x10a3b8_u32
                    };
                }
                0x10a3b8 => {
                    // 0010a3b8: movea.l $439430.l, a0
                    e.step()?;
                    let value_address = 0x439430_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a3be: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a3c0: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a3c4: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a3c8: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a3ca: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a3d0: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a3d6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a3d6: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a3dc: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a3de: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a3e2: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0010a3e8: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a3ea: lea.l $44721c.l, a1
                    e.step()?;
                    let value = 0x44721c_u32;
                    r.put_address(1, value);
                    // 0010a3f0: move.w #$30f, d0
                    e.step()?;
                    let value = 0x30f_u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x10a3f4_u32;
                }
                0x10a3f4 => {
                    // 0010a3f4: move.l (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = e.l(value_address)?;
                    r.offset_address(0, 4)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.offset_address(1, 4)?;
                    r.move_flags(value, 32);
                    // 0010a3f6: dbra d0, $10a3f4
                    e.step()?;
                    let counter = r.data(0, 16)?;
                    let value = counter.wrapping_sub(1) & 0xffff;
                    r.put_data(0, 16, value);
                    pc = if value != 0xffff {
                        0x10a3f4_u32
                    } else {
                        0x10a3fa_u32
                    };
                }
                0x10a3fa => {
                    // 0010a3fa: move.w (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(0, 2)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.offset_address(1, 2)?;
                    r.move_flags(value, 16);
                    // 0010a3fc: move.b $44273f.l, $44273d.l
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = 0x44273d_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a406: move.b #$ff, $44273e.l
                    e.step()?;
                    let value = 0xff_u32;
                    let destination_address = 0x44273e_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a40e: move.w #$c8, $442bca.l
                    e.step()?;
                    let value = 0xc8_u32;
                    let destination_address = 0x442bca_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a416: tst.w $8(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0010a41a: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    // 0010a41e: bne.b $10a426
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a426_u32
                    } else {
                        0x10a420_u32
                    };
                }
                0x10a420 => {
                    // 0010a420: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a426_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    pc = 0x10a426_u32;
                }
                0x10a426 => {
                    // 0010a426: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a428: bsr.l $152ec2
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a42e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x152ec2_u32)?;
                    // 0010a42e: move.w d0, -$6(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a432: move.w -$6(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a436: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010a438: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10a43c => {
                    // 0010a43c: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a442: cmpa.l $439438.l, a0
                    e.step()?;
                    let source_address = 0x439438_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010a448: bne.w $10a5e0
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a5e0_u32
                    } else {
                        0x10a44c_u32
                    };
                }
                0x10a44c => {
                    // 0010a44c: move.l #$5c005d, -(a7)
                    e.step()?;
                    let value = 0x5c005d_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a452: bsr.l $12382a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a458_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12382a_u32)?;
                    // 0010a458: movea.l $439438.l, a0
                    e.step()?;
                    let value_address = 0x439438_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a45e: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a460: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a464: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a468: move.w #$8d, -(a7)
                    e.step()?;
                    let value = 0x8d_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a46c: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a472: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a478_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a478: move.l $442778.l, d0
                    e.step()?;
                    let value_address = 0x442778_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a47e: add.l $442764.l, d0
                    e.step()?;
                    let source_address = 0x442764_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010a484: move.w d0, -$a(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a488: move.l $44277c.l, d0
                    e.step()?;
                    let value_address = 0x44277c_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a48e: add.l $442768.l, d0
                    e.step()?;
                    let source_address = 0x442768_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010a494: move.w d0, -$c(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a498: moveq #$0, d6
                    e.step()?;
                    r.put_data(6, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010a49a: moveq #$13, d4
                    e.step()?;
                    r.put_data(4, 32, 0x13_u32);
                    r.move_flags(0x13_u32, 32);
                    // 0010a49c: lea.l $12(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x12_u32);
                    r.put_address(7, value);
                    // 0010a4a0: bra.w $10a568
                    e.step()?;
                    pc = 0x10a568_u32;
                }
                0x10a4a4 => {
                    // 0010a4a4: subq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 0010a4a6: move.l -$c(a6), -(a7)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a4aa: movea.w d4, a0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010a4ac: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a4ae: lsl.l #$3, d0
                    e.step()?;
                    let count = 0x3_u32;
                    let source = r.data(0, 32)?;
                    let value = r.shift_left(source, count & 63, 32);
                    r.put_data(0, 32, value);
                    // 0010a4b0: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0010a4b6: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a4b8: pea.l (a0)
                    e.step()?;
                    let value = r.address(0)?.wrapping_add(0x0_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0010a4ba: MacToolbox 0xa8ad
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
                    // 0010a4bc: move.b (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = u32::from(e.b(value_address)?);
                    r.offset_address(7, 2)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a4be: beq.w $10a566
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a566_u32
                    } else {
                        0x10a4c2_u32
                    };
                }
                0x10a4c2 => {
                    // 0010a4c2: movea.w d4, a0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010a4c4: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a4c6: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010a4c8: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0010a4ce: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a4d0: tst.w $a0(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xa0_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0010a4d4: bge.w $10a566
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x10a566_u32
                    } else {
                        0x10a4d8_u32
                    };
                }
                0x10a4d8 => {
                    // 0010a4d8: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010a4da: bra.w $10a55e
                    e.step()?;
                    pc = 0x10a55e_u32;
                }
                0x10a4de => {
                    // 0010a4de: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010a4e0: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a4e2: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010a4e4: moveq #$6, d1
                    e.step()?;
                    r.put_data(1, 32, 0x6_u32);
                    r.move_flags(0x6_u32, 32);
                    // 0010a4e6: mulu.w d4, d1
                    e.step()?;
                    let source = r.data(4, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0010a4e8: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0010a4ee: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a4f0: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a4f2: move.w $190(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x190_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a4f6: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a4fc_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 0010a4fc: move.w d0, -$10(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a500: move.w #$64, -(a7)
                    e.step()?;
                    let value = 0x64_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a504: bsr.l $153ca4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a50a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x153ca4_u32)?;
                    // 0010a50a: cmp.w -$10(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010a50e: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010a510: bgt.b $10a55c
                    e.step()?;
                    pc = if r.greater()? {
                        0x10a55c_u32
                    } else {
                        0x10a512_u32
                    };
                }
                0x10a512 => {
                    // 0010a512: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010a514: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a516: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010a518: moveq #$6, d1
                    e.step()?;
                    r.put_data(1, 32, 0x6_u32);
                    r.move_flags(0x6_u32, 32);
                    // 0010a51a: mulu.w d4, d1
                    e.step()?;
                    let source = r.data(4, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0010a51c: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0010a522: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a524: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a526: move.w $118(a0), d6
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x118_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 0010a52a: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010a52c: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a52e: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010a530: moveq #$6, d1
                    e.step()?;
                    r.put_data(1, 32, 0x6_u32);
                    r.move_flags(0x6_u32, 32);
                    // 0010a532: mulu.w d4, d1
                    e.step()?;
                    let source = r.data(4, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0010a534: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0010a53a: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a53c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a53e: tst.w $190(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x190_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0010a542: ble.b $10a55c
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x10a55c_u32
                    } else {
                        0x10a544_u32
                    };
                }
                0x10a544 => {
                    // 0010a544: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010a546: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a548: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0010a54a: moveq #$6, d1
                    e.step()?;
                    r.put_data(1, 32, 0x6_u32);
                    r.move_flags(0x6_u32, 32);
                    // 0010a54c: mulu.w d4, d1
                    e.step()?;
                    let source = r.data(4, 16)?;
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0010a54e: lea.l $443086.l, a0
                    e.step()?;
                    let value = 0x443086_u32;
                    r.put_address(0, value);
                    // 0010a554: adda.l d1, a0
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a556: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a558: clr.w $190(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x190_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x10a55c_u32;
                }
                0x10a55c => {
                    // 0010a55c: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x10a55e_u32;
                }
                0x10a55e => {
                    // 0010a55e: cmpi.w #$3, d3
                    e.step()?;
                    let source = 0x3_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010a562: blt.w $10a4de
                    e.step()?;
                    pc = if r.less()? {
                        0x10a4de_u32
                    } else {
                        0x10a566_u32
                    };
                }
                0x10a566 => {
                    // 0010a566: subq.w #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    pc = 0x10a568_u32;
                }
                0x10a568 => {
                    // 0010a568: cmpi.w #$ffff, d4
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination = r.data(4, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010a56c: bgt.w $10a4a4
                    e.step()?;
                    pc = if r.greater()? {
                        0x10a4a4_u32
                    } else {
                        0x10a570_u32
                    };
                }
                0x10a570 => {
                    // 0010a570: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a576: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a57c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 0010a57c: movea.l $439438.l, a0
                    e.step()?;
                    let value_address = 0x439438_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a582: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a584: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a588: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a58c: move.w #$8d, -(a7)
                    e.step()?;
                    let value = 0x8d_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a590: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a596: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a59c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a59c: clr.b $442737.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442737_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a5a2: clr.b $442736.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442736_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a5a8: move.b #$1, $43959c.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x43959c_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a5b0: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a5b2: move.w d6, -(a7)
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a5b4: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a5b8: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a5ba: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a5bc: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a5c2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 0010a5c2: move.w d0, -$6(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a5c6: clr.b $43959c.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x43959c_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a5cc: cmpi.w #$ffff, -$6(a6)
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0010a5d2: lea.l $20(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x20_u32);
                    r.put_address(7, value);
                    // 0010a5d6: beq.w $10a9ee
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a9ee_u32
                    } else {
                        0x10a5da_u32
                    };
                }
                0x10a5da => {
                    // 0010a5da: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010a5dc: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10a5e0 => {
                    // 0010a5e0: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a5e6: cmpa.l $439444.l, a0
                    e.step()?;
                    let source_address = 0x439444_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010a5ec: bne.w $10a6ac
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a6ac_u32
                    } else {
                        0x10a5f0_u32
                    };
                }
                0x10a5f0 => {
                    // 0010a5f0: tst.b $44273a.l
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010a5f6: beq.w $10a6ac
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a6ac_u32
                    } else {
                        0x10a5fa_u32
                    };
                }
                0x10a5fa => {
                    // 0010a5fa: move.w #$8d, -(a7)
                    e.step()?;
                    let value = 0x8d_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a5fe: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a604_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010a604: tst.w $8(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0010a608: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010a60a: bne.b $10a612
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a612_u32
                    } else {
                        0x10a60c_u32
                    };
                }
                0x10a60c => {
                    // 0010a60c: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a612_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    pc = 0x10a612_u32;
                }
                0x10a612 => {
                    // 0010a612: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a618: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a61a: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a61e: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0010a624: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a626: lea.l $44721c.l, a1
                    e.step()?;
                    let value = 0x44721c_u32;
                    r.put_address(1, value);
                    // 0010a62c: move.w #$30f, d0
                    e.step()?;
                    let value = 0x30f_u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x10a630_u32;
                }
                0x10a630 => {
                    // 0010a630: move.l (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = e.l(value_address)?;
                    r.offset_address(0, 4)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.offset_address(1, 4)?;
                    r.move_flags(value, 32);
                    // 0010a632: dbra d0, $10a630
                    e.step()?;
                    let counter = r.data(0, 16)?;
                    let value = counter.wrapping_sub(1) & 0xffff;
                    r.put_data(0, 16, value);
                    pc = if value != 0xffff {
                        0x10a630_u32
                    } else {
                        0x10a636_u32
                    };
                }
                0x10a636 => {
                    // 0010a636: move.w (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(0, 2)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.offset_address(1, 2)?;
                    r.move_flags(value, 16);
                    // 0010a638: move.b $44273f.l, $44273d.l
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = 0x44273d_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a642: move.b $44273d.l, d0
                    e.step()?;
                    let value_address = 0x44273d_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a648: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a64a: addq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a64c: move.b d0, $44273e.l
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = 0x44273e_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a652: move.b $44273e.l, d0
                    e.step()?;
                    let value_address = 0x44273e_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a658: cmp.b $44273a.l, d0
                    e.step()?;
                    let source_address = 0x44273a_u32;
                    let source = u32::from(e.b(source_address)?);
                    let destination = r.data(0, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 0010a65e: ble.b $10a666
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x10a666_u32
                    } else {
                        0x10a660_u32
                    };
                }
                0x10a660 => {
                    // 0010a660: clr.b $44273e.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x44273e_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x10a666_u32;
                }
                0x10a666 => {
                    // 0010a666: move.b $44273e.l, d0
                    e.step()?;
                    let value_address = 0x44273e_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a66c: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a66e: mulu.w #$c42, d0
                    e.step()?;
                    let source = 0xc42_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a672: lea.l $448aa0.l, a0
                    e.step()?;
                    let value = 0x448aa0_u32;
                    r.put_address(0, value);
                    // 0010a678: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010a67a: lea.l $447e5e.l, a1
                    e.step()?;
                    let value = 0x447e5e_u32;
                    r.put_address(1, value);
                    // 0010a680: move.w #$30f, d0
                    e.step()?;
                    let value = 0x30f_u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x10a684_u32;
                }
                0x10a684 => {
                    // 0010a684: move.l (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = e.l(value_address)?;
                    r.offset_address(0, 4)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.offset_address(1, 4)?;
                    r.move_flags(value, 32);
                    // 0010a686: dbra d0, $10a684
                    e.step()?;
                    let counter = r.data(0, 16)?;
                    let value = counter.wrapping_sub(1) & 0xffff;
                    r.put_data(0, 16, value);
                    pc = if value != 0xffff {
                        0x10a684_u32
                    } else {
                        0x10a68a_u32
                    };
                }
                0x10a68a => {
                    // 0010a68a: move.w (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(0, 2)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.offset_address(1, 2)?;
                    r.move_flags(value, 16);
                    // 0010a68c: move.w $447e78.l, $442bca.l
                    e.step()?;
                    let value_address = 0x447e78_u32;
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x442bca_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a696: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a698: bsr.l $152ec2
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a69e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x152ec2_u32)?;
                    // 0010a69e: move.w d0, -$6(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a6a2: move.w -$6(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a6a6: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010a6a8: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10a6ac => {
                    // 0010a6ac: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a6b2: cmpa.l $439440.l, a0
                    e.step()?;
                    let source_address = 0x439440_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010a6b8: bne.w $10a8be
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a8be_u32
                    } else {
                        0x10a6bc_u32
                    };
                }
                0x10a6bc => {
                    // 0010a6bc: move.b $44273a.l, d0
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a6c2: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a6c4: move.w d0, -$10(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a6c8: movea.w $4471ee.l, a0
                    e.step()?;
                    let value_address = 0x4471ee_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010a6ce: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a6d0: divs.w #$32, d0
                    e.step()?;
                    let divisor = 0x32_u32;
                    let dividend = r.data(0, 32)?;
                    let value = r.divide_signed(dividend, divisor)?;
                    r.put_data(0, 32, value);
                    // 0010a6d4: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a6d6: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a6dc_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 0010a6dc: cmp.w -$10(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010a6e0: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010a6e2: bgt.w $10a8be
                    e.step()?;
                    pc = if r.greater()? {
                        0x10a8be_u32
                    } else {
                        0x10a6e6_u32
                    };
                }
                0x10a6e6 => {
                    // 0010a6e6: move.l #$100012, -(a7)
                    e.step()?;
                    let value = 0x100012_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a6ec: bsr.l $12382a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a6f2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12382a_u32)?;
                    // 0010a6f2: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a6f8: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a6fa: muls.w #$32, d0
                    e.step()?;
                    let source = 0x32_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a6fe: move.w d0, $4434ba.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a704: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a706: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a70c: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a712: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a718: addi.w #$14a, d0
                    e.step()?;
                    let source = 0x14a_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a71c: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a722: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a724: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a72a: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a730: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a736: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a73c: move.w #$8d, -(a7)
                    e.step()?;
                    let value = 0x8d_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a740: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a746: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a74c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a74c: movea.w $4471ee.l, a0
                    e.step()?;
                    let value_address = 0x4471ee_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010a752: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a754: divs.w #$32, d0
                    e.step()?;
                    let divisor = 0x32_u32;
                    let dividend = r.data(0, 32)?;
                    let value = r.divide_signed(dividend, divisor)?;
                    r.put_data(0, 32, value);
                    // 0010a758: move.b d0, $44273f.l
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = 0x44273f_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a75e: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a764: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a766: muls.w #$32, d0
                    e.step()?;
                    let source = 0x32_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a76a: move.w d0, $4434ba.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a770: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a772: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a778: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a77e: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a784: addi.w #$14a, d0
                    e.step()?;
                    let source = 0x14a_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a788: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a78e: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a790: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a796: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a79c: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a7a2: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a7a8: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a7aa: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a7b0: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a7b6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a7b6: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a7bc: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a7be: cmp.w $4435d0.l, d0
                    e.step()?;
                    let source_address = 0x4435d0_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010a7c4: lea.l $20(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x20_u32);
                    r.put_address(7, value);
                    // 0010a7c8: bne.w $10a868
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a868_u32
                    } else {
                        0x10a7cc_u32
                    };
                }
                0x10a7cc => {
                    // 0010a7cc: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a7d2: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a7d8: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a7da: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a7e0: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a7e6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a7e6: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a7ec_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 0010a7ec: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a7ee: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a7f4: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a7f6: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a7f8: bsr.l $16399c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a7fe_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16399c_u32)?;
                    // 0010a7fe: move.l #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a804: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a80a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 0010a80a: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a810: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a812: muls.w #$32, d0
                    e.step()?;
                    let source = 0x32_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a816: move.w d0, $4434ba.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a81c: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a81e: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a824: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a82a: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a830: addi.w #$14a, d0
                    e.step()?;
                    let source = 0x14a_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a834: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a83a: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a83c: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a842: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a848: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a84e: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a854: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a856: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a85c: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a862_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a862: lea.l $24(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x24_u32);
                    r.put_address(7, value);
                    // 0010a866: bra.b $10a8be
                    e.step()?;
                    pc = 0x10a8be_u32;
                }
                0x10a868 => {
                    // 0010a868: bsr.l $16043e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a86e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16043e_u32)?;
                    // 0010a86e: movea.w $4471ee.l, a0
                    e.step()?;
                    let value_address = 0x4471ee_u32;
                    let value = u32::from(e.w(value_address)?);
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 0010a874: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a876: divs.w #$32, d0
                    e.step()?;
                    let divisor = 0x32_u32;
                    let dividend = r.data(0, 32)?;
                    let value = r.divide_signed(dividend, divisor)?;
                    r.put_data(0, 32, value);
                    // 0010a87a: move.b d0, $44273f.l
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = 0x44273f_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010a880: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a886: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a888: muls.w #$32, d0
                    e.step()?;
                    let source = 0x32_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a88c: move.w d0, $4434ba.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a892: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a894: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a89a: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a8a0: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a8a6: addi.w #$14a, d0
                    e.step()?;
                    let source = 0x14a_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a8aa: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a8b0: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a8b2: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a8b8: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x10a8be_u32;
                }
                0x10a8be => {
                    // 0010a8be: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a8c4: cmpa.l $43945c.l, a0
                    e.step()?;
                    let source_address = 0x43945c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010a8ca: bne.w $10aa3c
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10aa3c_u32
                    } else {
                        0x10a8ce_u32
                    };
                }
                0x10a8ce => {
                    // 0010a8ce: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a8d4: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a8d6: muls.w #$32, d0
                    e.step()?;
                    let source = 0x32_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a8da: move.w d0, $4434ba.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a8e0: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a8e2: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a8e8: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a8ee: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a8f4: addi.w #$14a, d0
                    e.step()?;
                    let source = 0x14a_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a8f8: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a8fe: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a900: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a906: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a90c: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a912: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a918: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a91a: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a920: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a926_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a926: move.w #$8d, -(a7)
                    e.step()?;
                    let value = 0x8d_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a92a: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a930_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010a930: tst.w $8(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0x8_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0010a934: lea.l $10(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x10_u32);
                    r.put_address(7, value);
                    // 0010a938: bne.b $10a940
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10a940_u32
                    } else {
                        0x10a93a_u32
                    };
                }
                0x10a93a => {
                    // 0010a93a: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a940_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    pc = 0x10a940_u32;
                }
                0x10a940 => {
                    // 0010a940: cmpi.w #$ffff, $8(a6)
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = r.address(6)?.wrapping_add(0x8_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0010a946: beq.b $10a96a
                    e.step()?;
                    pc = if r.equal()? {
                        0x10a96a_u32
                    } else {
                        0x10a948_u32
                    };
                }
                0x10a948 => {
                    // 0010a948: movea.l $43945c.l, a0
                    e.step()?;
                    let value_address = 0x43945c_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a94e: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010a950: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a954: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a958: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a95a: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a960: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a966_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a966: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x10a96a_u32;
                }
                0x10a96a => {
                    // 0010a96a: bsr.l $129516
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a970_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x129516_u32)?;
                    // 0010a970: move.w d0, -$6(a6)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a974: move.l $43878c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43878c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a97a: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 0010a97c: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010a982: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a984: muls.w #$32, d0
                    e.step()?;
                    let source = 0x32_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010a988: move.w d0, $4434ba.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a98e: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a990: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a996: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a99c: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010a9a2: addi.w #$14a, d0
                    e.step()?;
                    let source = 0x14a_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a9a6: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a9ac: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010a9ae: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010a9b4: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a9ba: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a9c0: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a9c6: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a9c8: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a9ce: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a9d4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010a9d4: cmpi.w #$ff9d, -$6(a6)
                    e.step()?;
                    let source = 0xff9d_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0010a9da: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    // 0010a9de: bge.b $10aa1a
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x10aa1a_u32
                    } else {
                        0x10a9e0_u32
                    };
                }
                0x10a9e0 => {
                    // 0010a9e0: move.l #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010a9e6: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a9ec_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 0010a9ec: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x10a9ee_u32;
                }
                0x10a9ee => {
                    // 0010a9ee: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a9f0: move.w $44719a.l, -(a7)
                    e.step()?;
                    let value_address = 0x44719a_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010a9f6: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10a9fc_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 0010a9fc: addi.w #$ff9c, d0
                    e.step()?;
                    let source = 0xff9c_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010aa00: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010aa02: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010aa04: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010aa08: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aa0a: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aa0c: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aa12_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 0010aa12: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    // 0010aa16: bra.w $109afe
                    e.step()?;
                    pc = 0x109afe_u32;
                }
                0x10aa1a => {
                    // 0010aa1a: tst.w -$6(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0010aa1e: bge.b $10aa28
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x10aa28_u32
                    } else {
                        0x10aa20_u32
                    };
                }
                0x10aa20 => {
                    // 0010aa20: move.w -$6(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010aa24: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10aa28 => {
                    // 0010aa28: move.l #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aa2e: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aa34_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 0010aa34: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010aa36: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010aa38: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10aa3c => {
                    // 0010aa3c: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010aa42: cmpa.l $439474.l, a0
                    e.step()?;
                    let source_address = 0x439474_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010aa48: bne.w $10ab1e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10ab1e_u32
                    } else {
                        0x10aa4c_u32
                    };
                }
                0x10aa4c => {
                    // 0010aa4c: tst.b $442758.l
                    e.step()?;
                    let value_address = 0x442758_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010aa52: beq.b $10aab4
                    e.step()?;
                    pc = if r.equal()? {
                        0x10aab4_u32
                    } else {
                        0x10aa54_u32
                    };
                }
                0x10aa54 => {
                    // 0010aa54: movea.l $439474.l, a0
                    e.step()?;
                    let value_address = 0x439474_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010aa5a: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010aa5c: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aa60: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aa64: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010aa66: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aa6c: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aa72_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010aa72: move.b #$1, $442757.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x442757_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010aa7a: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aa80_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 0010aa80: move.w $44257a.l, -(a7)
                    e.step()?;
                    let value_address = 0x44257a_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010aa86: bsr.l $15d2f0
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aa8c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15d2f0_u32)?;
                    // 0010aa8c: movea.l $439474.l, a0
                    e.step()?;
                    let value_address = 0x439474_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010aa92: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010aa94: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aa98: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aa9c: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010aa9e: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aaa4: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aaaa_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010aaaa: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010aaac: lea.l $1e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x1e_u32);
                    r.put_address(7, value);
                    // 0010aab0: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10aab4 => {
                    // 0010aab4: movea.l $439474.l, a0
                    e.step()?;
                    let value_address = 0x439474_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010aaba: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010aabc: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aac0: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aac4: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010aac6: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aacc: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aad2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010aad2: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aad8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 0010aad8: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010aadc: move.w $44257a.l, -(a7)
                    e.step()?;
                    let value_address = 0x44257a_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010aae2: bsr.l $105748
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aae8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x105748_u32)?;
                    // 0010aae8: bsr.l $13a510
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aaee_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a510_u32)?;
                    // 0010aaee: move.l $43878c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43878c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aaf4: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 0010aaf6: movea.l $439474.l, a0
                    e.step()?;
                    let value_address = 0x439474_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010aafc: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010aafe: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ab02: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ab06: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ab08: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ab0e: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ab14_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010ab14: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010ab16: lea.l $20(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x20_u32);
                    r.put_address(7, value);
                    // 0010ab1a: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10ab1e => {
                    // 0010ab1e: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010ab24: cmpa.l $43947c.l, a0
                    e.step()?;
                    let source_address = 0x43947c_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010ab2a: bne.b $10ab70
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10ab70_u32
                    } else {
                        0x10ab2c_u32
                    };
                }
                0x10ab2c => {
                    // 0010ab2c: movea.l $43947c.l, a0
                    e.step()?;
                    let value_address = 0x43947c_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010ab32: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010ab34: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ab38: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ab3c: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ab3e: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ab44: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ab4a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010ab4a: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ab50_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 0010ab50: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0010ab54: bsr.l $105748
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ab5a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x105748_u32)?;
                    // 0010ab5a: move.l #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ab60: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ab66_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 0010ab66: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010ab68: lea.l $16(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x16_u32);
                    r.put_address(7, value);
                    // 0010ab6c: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10ab70 => {
                    // 0010ab70: movea.l $439404.l, a0
                    e.step()?;
                    let value_address = 0x439404_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010ab76: cmpa.l $439470.l, a0
                    e.step()?;
                    let source_address = 0x439470_u32;
                    let source = e.l(source_address)?;
                    let destination = r.address(0)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0010ab7c: bne.w $10ae70
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10ae70_u32
                    } else {
                        0x10ab80_u32
                    };
                }
                0x10ab80 => {
                    // 0010ab80: tst.b $437a6a.l
                    e.step()?;
                    let value_address = 0x437a6a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010ab86: beq.b $10abae
                    e.step()?;
                    pc = if r.equal()? {
                        0x10abae_u32
                    } else {
                        0x10ab88_u32
                    };
                }
                0x10ab88 => {
                    // 0010ab88: movea.l $439470.l, a0
                    e.step()?;
                    let value_address = 0x439470_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010ab8e: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010ab90: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ab94: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ab98: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ab9a: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010aba0: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aba6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010aba6: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    // 0010abaa: bra.w $10ac68
                    e.step()?;
                    pc = 0x10ac68_u32;
                }
                0x10abae => {
                    // 0010abae: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010abb4: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010abb6: muls.w #$32, d0
                    e.step()?;
                    let source = 0x32_u32;
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010abba: move.w d0, $4434ba.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434ba_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010abc0: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010abc2: add.w $4434ba.l, d0
                    e.step()?;
                    let source_address = 0x4434ba_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010abc8: move.w d0, $4434be.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434be_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010abce: move.w $40856c.l, d0
                    e.step()?;
                    let value_address = 0x40856c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010abd4: addi.w #$14a, d0
                    e.step()?;
                    let source = 0x14a_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010abd8: move.w d0, $4434bc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434bc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010abde: moveq #$32, d0
                    e.step()?;
                    r.put_data(0, 32, 0x32_u32);
                    r.move_flags(0x32_u32, 32);
                    // 0010abe0: add.w $4434bc.l, d0
                    e.step()?;
                    let source_address = 0x4434bc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010abe6: move.w d0, $4434c0.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x4434c0_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010abec: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010abf2: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010abf8: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010abfa: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ac00: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ac06_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010ac06: move.w #$8d, -(a7)
                    e.step()?;
                    let value = 0x8d_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ac0a: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ac10_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010ac10: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ac12: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010ac18: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010ac1a: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ac1c: bsr.l $13beea
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ac22_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13beea_u32)?;
                    // 0010ac22: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0010ac24: lea.l $14(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x14_u32);
                    r.put_address(7, value);
                    // 0010ac28: bne.b $10ac30
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10ac30_u32
                    } else {
                        0x10ac2a_u32
                    };
                }
                0x10ac2a => {
                    // 0010ac2a: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010ac2c: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10ac30 => {
                    // 0010ac30: move.w #$1, $43877a.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x43877a_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ac38: tst.w $438760.l
                    e.step()?;
                    let value_address = 0x438760_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0010ac3e: bne.b $10ac68
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10ac68_u32
                    } else {
                        0x10ac40_u32
                    };
                }
                0x10ac40 => {
                    // 0010ac40: movea.l $439470.l, a0
                    e.step()?;
                    let value_address = 0x439470_u32;
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010ac46: movea.l (a0), a0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32);
                    let value = e.l(value_address)?;
                    r.put_address(0, value);
                    // 0010ac48: move.l $c(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0xc_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ac4c: move.l $8(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32);
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ac50: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ac52: move.l #$10002, -(a7)
                    e.step()?;
                    let value = 0x10002_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ac58: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ac5e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 0010ac5e: bsr.l $10b89e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ac64_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10b89e_u32)?;
                    // 0010ac64: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x10ac68_u32;
                }
                0x10ac68 => {
                    // 0010ac68: cmpi.w #$ffff, $4435ca.l
                    e.step()?;
                    let source = 0xffff_u32;
                    let destination_address = 0x4435ca_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0010ac70: ble.w $10ae62
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x10ae62_u32
                    } else {
                        0x10ac74_u32
                    };
                }
                0x10ac74 => {
                    // 0010ac74: tst.b -$d(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff3_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010ac78: bne.b $10ac88
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10ac88_u32
                    } else {
                        0x10ac7a_u32
                    };
                }
                0x10ac7a => {
                    // 0010ac7a: move.l #$1ffff, -(a7)
                    e.step()?;
                    let value = 0x1ffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ac80: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ac86_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 0010ac86: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x10ac88_u32;
                }
                0x10ac88 => {
                    // 0010ac88: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ac8a: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010ac90: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010ac92: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ac94: bsr.l $1600fc
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ac9a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1600fc_u32)?;
                    // 0010ac9a: bsr.l $13822a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10aca0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13822a_u32)?;
                    // 0010aca0: moveq #$0, d6
                    e.step()?;
                    r.put_data(6, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010aca2: cmpi.b #$2, $44374f.l
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = 0x44374f_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 0010acaa: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010acac: ble.b $10acb6
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x10acb6_u32
                    } else {
                        0x10acae_u32
                    };
                }
                0x10acae => {
                    // 0010acae: move.b $44273a.l, d6
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(6, 8, value);
                    r.move_flags(value, 8);
                    // 0010acb4: ext.w d6
                    e.step()?;
                    let source = r.data(6, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    pc = 0x10acb6_u32;
                }
                0x10acb6 => {
                    // 0010acb6: tst.b $44374f.l
                    e.step()?;
                    let value_address = 0x44374f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010acbc: bne.b $10acc6
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10acc6_u32
                    } else {
                        0x10acbe_u32
                    };
                }
                0x10acbe => {
                    // 0010acbe: move.w $4435cc.l, d6
                    e.step()?;
                    let value_address = 0x4435cc_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(6, 16, value);
                    r.move_flags(value, 16);
                    // 0010acc4: subq.w #$1, d6
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(6, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(6, 16, value);
                    pc = 0x10acc6_u32;
                }
                0x10acc6 => {
                    // 0010acc6: clr.b -$7(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff9_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010acca: cmpi.b #$5, $44374f.l
                    e.step()?;
                    let source = 0x5_u32;
                    let destination_address = 0x44374f_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 0010acd2: bne.b $10acec
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10acec_u32
                    } else {
                        0x10acd4_u32
                    };
                }
                0x10acd4 => {
                    // 0010acd4: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010acda: extb.l d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010acdc: lea.l $4434e0.l, a0
                    e.step()?;
                    let value = 0x4434e0_u32;
                    r.put_address(0, value);
                    // 0010ace2: move.w #$1, (a0, d0.l * 2)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ace8: bra.w $10adb8
                    e.step()?;
                    pc = 0x10adb8_u32;
                }
                0x10acec => {
                    // 0010acec: move.b $44273a.l, d0
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010acf2: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010acf4: cmp.w d0, d6
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010acf6: bge.b $10ad28
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x10ad28_u32
                    } else {
                        0x10acf8_u32
                    };
                }
                0x10acf8 => {
                    // 0010acf8: pea.l $4471cc.l
                    e.step()?;
                    let value = 0x4471cc_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0010acfe: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ad00: move.l #$3002f, -(a7)
                    e.step()?;
                    let value = 0x3002f_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ad06: bsr.l $15dc34
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ad0c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15dc34_u32)?;
                    // 0010ad0c: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0010ad10: move.w d6, -(a7)
                    e.step()?;
                    let value = r.data(6, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ad12: bsr.l $12512a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ad18_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x12512a_u32)?;
                    // 0010ad18: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0010ad1a: lea.l $12(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x12_u32);
                    r.put_address(7, value);
                    // 0010ad1e: bne.w $10adb8
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10adb8_u32
                    } else {
                        0x10ad22_u32
                    };
                }
                0x10ad22 => {
                    // 0010ad22: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010ad24: bra.w $10ae78
                    e.step()?;
                    pc = 0x10ae78_u32;
                }
                0x10ad28 => {
                    // 0010ad28: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010ad2a: bra.b $10ad3a
                    e.step()?;
                    pc = 0x10ad3a_u32;
                }
                0x10ad2c => {
                    // 0010ad2c: lea.l $4434e0.l, a0
                    e.step()?;
                    let value = 0x4434e0_u32;
                    r.put_address(0, value);
                    // 0010ad32: move.w #$1, (a0, d4.w * 2)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ad38: addq.w #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    pc = 0x10ad3a_u32;
                }
                0x10ad3a => {
                    // 0010ad3a: move.b $44273a.l, d0
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010ad40: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010ad42: cmp.w d0, d4
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(4, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010ad44: ble.b $10ad2c
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x10ad2c_u32
                    } else {
                        0x10ad46_u32
                    };
                }
                0x10ad46 => {
                    // 0010ad46: move.b $44273a.l, d0
                    e.step()?;
                    let value_address = 0x44273a_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 0010ad4c: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010ad4e: cmp.w d0, d6
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let destination = r.data(6, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010ad50: bgt.b $10ad70
                    e.step()?;
                    pc = if r.greater()? {
                        0x10ad70_u32
                    } else {
                        0x10ad52_u32
                    };
                }
                0x10ad52 => {
                    // 0010ad52: cmpi.b #$3, $44374f.l
                    e.step()?;
                    let source = 0x3_u32;
                    let destination_address = 0x44374f_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 0010ad5a: beq.b $10ad70
                    e.step()?;
                    pc = if r.equal()? {
                        0x10ad70_u32
                    } else {
                        0x10ad5c_u32
                    };
                }
                0x10ad5c => {
                    // 0010ad5c: cmpi.b #$7, $44374f.l
                    e.step()?;
                    let source = 0x7_u32;
                    let destination_address = 0x44374f_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 0010ad64: beq.b $10ad70
                    e.step()?;
                    pc = if r.equal()? {
                        0x10ad70_u32
                    } else {
                        0x10ad66_u32
                    };
                }
                0x10ad66 => {
                    // 0010ad66: cmpi.b #$9, $44374f.l
                    e.step()?;
                    let source = 0x9_u32;
                    let destination_address = 0x44374f_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 0010ad6e: bne.b $10adb8
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10adb8_u32
                    } else {
                        0x10ad70_u32
                    };
                }
                0x10ad70 => {
                    // 0010ad70: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010ad72: bra.b $10adb0
                    e.step()?;
                    pc = 0x10adb0_u32;
                }
                0x10ad74 => {
                    // 0010ad74: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010ad76: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010ad7a: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 0010ad80: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010ad82: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010ad84: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010ad88: lea.l $439d3c.l, a1
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(1, value);
                    // 0010ad8e: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0010ad90: moveq #$47, d0
                    e.step()?;
                    r.put_data(0, 32, 0x47_u32);
                    r.move_flags(0x47_u32, 32);
                    pc = 0x10ad92_u32;
                }
                0x10ad92 => {
                    // 0010ad92: move.l (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = e.l(value_address)?;
                    r.offset_address(0, 4)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.offset_address(1, 4)?;
                    r.move_flags(value, 32);
                    // 0010ad94: dbra d0, $10ad92
                    e.step()?;
                    let counter = r.data(0, 16)?;
                    let value = counter.wrapping_sub(1) & 0xffff;
                    r.put_data(0, 16, value);
                    pc = if value != 0xffff {
                        0x10ad92_u32
                    } else {
                        0x10ad98_u32
                    };
                }
                0x10ad98 => {
                    // 0010ad98: moveq #$a, d0
                    e.step()?;
                    r.put_data(0, 32, 0xa_u32);
                    r.move_flags(0xa_u32, 32);
                    // 0010ad9a: add.w d4, d0
                    e.step()?;
                    let source = r.data(4, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010ad9c: lea.l $4434e0.l, a0
                    e.step()?;
                    let value = 0x4434e0_u32;
                    r.put_address(0, value);
                    // 0010ada2: move.w #$1, (a0, d0.w * 2)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ada8: move.b #$1, -$7(a6)
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffff9_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010adae: addq.w #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    pc = 0x10adb0_u32;
                }
                0x10adb0 => {
                    // 0010adb0: cmp.w $44257c.l, d4
                    e.step()?;
                    let source_address = 0x44257c_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(4, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010adb6: blt.b $10ad74
                    e.step()?;
                    pc = if r.less()? {
                        0x10ad74_u32
                    } else {
                        0x10adb8_u32
                    };
                }
                0x10adb8 => {
                    // 0010adb8: move.l $43878c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43878c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010adbe: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 0010adc0: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010adc2: bsr.l $1500d8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10adc8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1500d8_u32)?;
                    // 0010adc8: move.w $443772.l, d0
                    e.step()?;
                    let value_address = 0x443772_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010adce: neg.w d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = r.subtract_flags(0, source, 16, true);
                    r.put_data(0, 16, value);
                    // 0010add0: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010add2: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10add8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010add8: bsr.l $151022
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10adde_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151022_u32)?;
                    // 0010adde: tst.b -$7(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffff9_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 0010ade2: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010ade4: beq.b $10ae18
                    e.step()?;
                    pc = if r.equal()? {
                        0x10ae18_u32
                    } else {
                        0x10ade6_u32
                    };
                }
                0x10ade6 => {
                    // 0010ade6: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010ade8: bra.b $10ae10
                    e.step()?;
                    pc = 0x10ae10_u32;
                }
                0x10adea => {
                    // 0010adea: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010adec: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010adf0: lea.l $439d3c.l, a0
                    e.step()?;
                    let value = 0x439d3c_u32;
                    r.put_address(0, value);
                    // 0010adf6: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 0010adf8: move.w d4, d0
                    e.step()?;
                    let value = r.data(4, 16)?;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0010adfa: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 0010adfe: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 0010ae04: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0010ae06: moveq #$47, d0
                    e.step()?;
                    r.put_data(0, 32, 0x47_u32);
                    r.move_flags(0x47_u32, 32);
                    pc = 0x10ae08_u32;
                }
                0x10ae08 => {
                    // 0010ae08: move.l (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = e.l(value_address)?;
                    r.offset_address(0, 4)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.offset_address(1, 4)?;
                    r.move_flags(value, 32);
                    // 0010ae0a: dbra d0, $10ae08
                    e.step()?;
                    let counter = r.data(0, 16)?;
                    let value = counter.wrapping_sub(1) & 0xffff;
                    r.put_data(0, 16, value);
                    pc = if value != 0xffff {
                        0x10ae08_u32
                    } else {
                        0x10ae0e_u32
                    };
                }
                0x10ae0e => {
                    // 0010ae0e: addq.w #$1, d4
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(4, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(4, 16, value);
                    pc = 0x10ae10_u32;
                }
                0x10ae10 => {
                    // 0010ae10: cmp.w $44257c.l, d4
                    e.step()?;
                    let source_address = 0x44257c_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(4, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0010ae16: blt.b $10adea
                    e.step()?;
                    pc = if r.less()? {
                        0x10adea_u32
                    } else {
                        0x10ae18_u32
                    };
                }
                0x10ae18 => {
                    // 0010ae18: cmpi.b #$b, $44374f.l
                    e.step()?;
                    let source = 0xb_u32;
                    let destination_address = 0x44374f_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 0010ae20: bne.b $10ae30
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x10ae30_u32
                    } else {
                        0x10ae22_u32
                    };
                }
                0x10ae22 => {
                    // 0010ae22: move.w $443774.l, -(a7)
                    e.step()?;
                    let value_address = 0x443774_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ae28: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ae2e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 0010ae2e: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x10ae30_u32;
                }
                0x10ae30 => {
                    // 0010ae30: move.w #$14, -(a7)
                    e.step()?;
                    let value = 0x14_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ae34: bsr.l $1205f8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ae3a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1205f8_u32)?;
                    // 0010ae3a: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ae3c: bsr.l $1500d8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ae42_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1500d8_u32)?;
                    // 0010ae42: clr.b $435662.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x435662_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010ae48: clr.b $437a6a.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x437a6a_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0010ae4e: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010ae50: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0010ae52: move.w d0, $438760.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x438760_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0010ae58: bsr.l $16043e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ae5e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16043e_u32)?;
                    // 0010ae5e: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0010ae60: bra.b $10ae70
                    e.step()?;
                    pc = 0x10ae70_u32;
                }
                0x10ae62 => {
                    // 0010ae62: move.l #$1ffff, -(a7)
                    e.step()?;
                    let value = 0x1ffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0010ae68: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ae6e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 0010ae6e: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x10ae70_u32;
                }
                0x10ae70 => {
                    // 0010ae70: bsr.l $13a5a6
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x10ae76_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a5a6_u32)?;
                    // 0010ae76: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x10ae78_u32;
                }
                0x10ae78 => {
                    // 0010ae78: movem.l (a7)+, d3-d7
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    // 0010ae7c: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 0010ae7e: rts
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
crate::recovered_application_control::entry!(button_choice, 0x10950e);
