//! Complete provisional gameplay control from canonical CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{ApplicationControlServices, Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};

pub(crate) fn encounter_body(
    e: &mut Execution<'_, impl ApplicationControlServices>,
    frame: u32,
) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x12185a_u32;
        loop {
            match pc {
                0x12185a => {
                    // 0012185a: link.w a6, #$fef0
                    e.step()?;
                    r.save_register(e, 14)?;
                    r.put_address(6, r.address(7)?);
                    r.offset_address(7, -272)?;
                    // 0012185e: movem.l d3-d7/a2-a3, -(a7)
                    e.step()?;
                    r.save_register(e, 11)?;
                    r.save_register(e, 10)?;
                    r.save_register(e, 7)?;
                    r.save_register(e, 6)?;
                    r.save_register(e, 5)?;
                    r.save_register(e, 4)?;
                    r.save_register(e, 3)?;
                    // 00121862: suba.l a3, a3
                    e.step()?;
                    r.put_address(3, 0);
                    // 00121864: moveq #$1, d6
                    e.step()?;
                    r.put_data(6, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 00121866: clr.b -$110(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffef0_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0012186a: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0012186c: move.l $43952c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43952c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121872: MacToolbox 0xaa1c
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa1c_u32 as u16))?;
                    // 00121874: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12187a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 0012187a: subq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_sub(source);
                    r.put_address(7, value);
                    // 0012187c: move.w #$99, -(a7)
                    e.step()?;
                    let value = 0x99_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121880: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121882: pea.l $ffff.w
                    e.step()?;
                    let value = 0xffffffff_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121886: MacToolbox 0xa97c
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa97c_u32 as u16))?;
                    // 00121888: movea.l (a7)+, a0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_address(0, value);
                    // 0012188a: movea.l a0, a2
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_address(2, value);
                    // 0012188c: tst.b $44342d.l
                    e.step()?;
                    let value_address = 0x44342d_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00121892: bne.b $1218a2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1218a2_u32
                    } else {
                        0x121894_u32
                    };
                }
                0x121894 => {
                    // 00121894: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121896: move.l #$3200c8, -(a7)
                    e.step()?;
                    let value = 0x3200c8_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012189c: clr.b -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 0012189e: MacToolbox 0xa91d
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa91d_u32 as u16))?;
                    // 001218a0: bra.b $1218ae
                    e.step()?;
                    pc = 0x1218ae_u32;
                }
                0x1218a2 => {
                    // 001218a2: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001218a4: move.l #$3200fa, -(a7)
                    e.step()?;
                    let value = 0x3200fa_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001218aa: clr.b -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 001218ac: MacToolbox 0xa91d
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa91d_u32 as u16))?;
                    pc = 0x1218ae_u32;
                }
                0x1218ae => {
                    // 001218ae: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001218b0: moveq #$b, d0
                    e.step()?;
                    r.put_data(0, 32, 0xb_u32);
                    r.move_flags(0xb_u32, 32);
                    // 001218b2: add.w $4395a4.l, d0
                    e.step()?;
                    let source_address = 0x4395a4_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001218b8: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001218ba: move.w $4395a2.l, d0
                    e.step()?;
                    let value_address = 0x4395a2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001218c0: addi.w #$fb, d0
                    e.step()?;
                    let source = 0xfb_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001218c4: add.w $40856e.l, d0
                    e.step()?;
                    let source_address = 0x40856e_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001218ca: move.w $439582.l, d1
                    e.step()?;
                    let value_address = 0x439582_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001218d0: lsl.w #$5, d1
                    e.step()?;
                    let count = 0x5_u32;
                    let source = r.data(1, 16)?;
                    let value = r.shift_left(source, count & 63, 16);
                    r.put_data(1, 16, value);
                    // 001218d2: sub.w d1, d0
                    e.step()?;
                    let source = r.data(1, 16)?;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001218d4: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001218d6: clr.b -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 001218d8: MacToolbox 0xa91b
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa91b_u32 as u16))?;
                    // 001218da: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001218dc: MacToolbox 0xa915
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa915_u32 as u16))?;
                    // 001218de: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001218e0: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 001218e2: move.l $4358ce.l, -(a7)
                    e.step()?;
                    let value_address = 0x4358ce_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001218e8: MacToolbox 0xaa0b
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa0b_u32 as u16))?;
                    // 001218ea: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001218ec: MacToolbox 0xa922
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa922_u32 as u16))?;
                    // 001218ee: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001218f0: MacToolbox 0xa923
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa923_u32 as u16))?;
                    // 001218f2: move.l #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001218f8: move.l (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001218fa: MacToolbox 0xa032
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa032_u32 as u16))?;
                    pc = 0x1218fc_u32;
                }
                0x1218fc => {
                    // 001218fc: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001218fe: MacToolbox 0xa91f
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa91f_u32 as u16))?;
                    // 00121900: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121902: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 00121904: move.w $402f44.l, -(a7)
                    e.step()?;
                    let value_address = 0x402f44_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0012190a: MacToolbox 0xa887
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa887_u32 as u16))?;
                    // 0012190c: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012190e: MacToolbox 0xa981
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa981_u32 as u16))?;
                    // 00121910: move.l $43952c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43952c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121916: MacToolbox 0xaa1c
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa1c_u32 as u16))?;
                    // 00121918: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012191a: move.w #$6, -(a7)
                    e.step()?;
                    let value = 0x6_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0012191e: pea.l $4434b0.l
                    e.step()?;
                    let value = 0x4434b0_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121924: pea.l $4434dc.l
                    e.step()?;
                    let value = 0x4434dc_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012192a: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121930: MacToolbox 0xa98d
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa98d_u32 as u16))?;
                    // 00121932: tst.b $4433f7.l
                    e.step()?;
                    let value_address = 0x4433f7_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00121938: bne.b $121960
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121960_u32
                    } else {
                        0x12193a_u32
                    };
                }
                0x12193a => {
                    // 0012193a: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121940: move.l #$90009, -(a7)
                    e.step()?;
                    let value = 0x90009_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121946: MacToolbox 0xa8a9
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
                    // 00121948: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012194e: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121954: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121956: bsr.l $14a02c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12195c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14a02c_u32)?;
                    // 0012195c: lea.l $a(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xa_u32);
                    r.put_address(7, value);
                    pc = 0x121960_u32;
                }
                0x121960 => {
                    // 00121960: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121962: move.w #$4, -(a7)
                    e.step()?;
                    let value = 0x4_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121966: pea.l $4434b0.l
                    e.step()?;
                    let value = 0x4434b0_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012196c: pea.l $4434dc.l
                    e.step()?;
                    let value = 0x4434dc_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121972: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121978: MacToolbox 0xa98d
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa98d_u32 as u16))?;
                    // 0012197a: tst.b $4433f6.l
                    e.step()?;
                    let value_address = 0x4433f6_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00121980: bne.b $1219a8
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1219a8_u32
                    } else {
                        0x121982_u32
                    };
                }
                0x121982 => {
                    // 00121982: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121988: move.l #$90009, -(a7)
                    e.step()?;
                    let value = 0x90009_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012198e: MacToolbox 0xa8a9
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
                    // 00121990: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121996: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012199c: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0012199e: bsr.l $14a02c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1219a4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14a02c_u32)?;
                    // 001219a4: lea.l $a(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xa_u32);
                    r.put_address(7, value);
                    pc = 0x1219a8_u32;
                }
                0x1219a8 => {
                    // 001219a8: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001219aa: move.w #$2, -(a7)
                    e.step()?;
                    let value = 0x2_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001219ae: pea.l $4434b0.l
                    e.step()?;
                    let value = 0x4434b0_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001219b4: pea.l $4434dc.l
                    e.step()?;
                    let value = 0x4434dc_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001219ba: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001219c0: MacToolbox 0xa98d
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa98d_u32 as u16))?;
                    // 001219c2: tst.w $443400.l
                    e.step()?;
                    let value_address = 0x443400_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001219c8: bne.b $1219f0
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1219f0_u32
                    } else {
                        0x1219ca_u32
                    };
                }
                0x1219ca => {
                    // 001219ca: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001219d0: move.l #$90009, -(a7)
                    e.step()?;
                    let value = 0x90009_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001219d6: MacToolbox 0xa8a9
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
                    // 001219d8: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001219de: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001219e4: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001219e6: bsr.l $14a02c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1219ec_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14a02c_u32)?;
                    // 001219ec: lea.l $a(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xa_u32);
                    r.put_address(7, value);
                    pc = 0x1219f0_u32;
                }
                0x1219f0 => {
                    // 001219f0: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001219f2: move.w #$5, -(a7)
                    e.step()?;
                    let value = 0x5_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001219f6: pea.l $4434b0.l
                    e.step()?;
                    let value = 0x4434b0_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001219fc: pea.l $4434dc.l
                    e.step()?;
                    let value = 0x4434dc_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121a02: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121a08: MacToolbox 0xa98d
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa98d_u32 as u16))?;
                    // 00121a0a: tst.w $44341e.l
                    e.step()?;
                    let value_address = 0x44341e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00121a10: bne.b $121a38
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121a38_u32
                    } else {
                        0x121a12_u32
                    };
                }
                0x121a12 => {
                    // 00121a12: pea.l $4434ba.l
                    e.step()?;
                    let value = 0x4434ba_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121a18: move.l #$90009, -(a7)
                    e.step()?;
                    let value = 0x90009_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121a1e: MacToolbox 0xa8a9
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
                    // 00121a20: move.l $4434be.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434be_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121a26: move.l $4434ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121a2c: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121a2e: bsr.l $14a02c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121a34_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14a02c_u32)?;
                    // 00121a34: lea.l $a(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xa_u32);
                    r.put_address(7, value);
                    pc = 0x121a38_u32;
                }
                0x121a38 => {
                    // 00121a38: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121a3a: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121a3c: move.w $443430.l, -(a7)
                    e.step()?;
                    let value_address = 0x443430_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121a42: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121a48_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 00121a48: neg.w d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = r.subtract_flags(0, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00121a4a: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121a4c: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121a4e: move.w #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121a52: bsr.l $15dc34
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121a58_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15dc34_u32)?;
                    // 00121a58: move.w #$4e25, -(a7)
                    e.step()?;
                    let value = 0x4e25_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121a5c: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121a62_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 00121a62: move.w #$f, -(a7)
                    e.step()?;
                    let value = 0xf_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121a66: bsr.l $1205f8
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121a6c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1205f8_u32)?;
                    // 00121a6c: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121a6e: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 00121a70: pea.l $45.w
                    e.step()?;
                    let value = 0x45_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121a74: MacToolbox 0xa862
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa862_u32 as u16))?;
                    // 00121a76: move.w #$2, $447198.l
                    e.step()?;
                    let value = 0x2_u32;
                    let destination_address = 0x447198_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121a7e: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121a80: MacToolbox 0xa91f
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa91f_u32 as u16))?;
                    // 00121a82: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121a84: MacToolbox 0xa873
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa873_u32 as u16))?;
                    // 00121a86: move.l #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121a8c: move.l (a7)+, d0
                    e.step()?;
                    let value_address = r.address(7)?;
                    let value = e.l(value_address)?;
                    r.offset_address(7, 4)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00121a8e: MacToolbox 0xa032
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa032_u32 as u16))?;
                    // 00121a90: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121a92: pea.l $4434b2.l
                    e.step()?;
                    let value = 0x4434b2_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121a98: MacToolbox 0xa991
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa991_u32 as u16))?;
                    // 00121a9a: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121a9c: move.w $4434b2.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434b2_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121aa2: pea.l $4434b0.l
                    e.step()?;
                    let value = 0x4434b0_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121aa8: pea.l $4434dc.l
                    e.step()?;
                    let value = 0x4434dc_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121aae: pea.l $44292c.l
                    e.step()?;
                    let value = 0x44292c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121ab4: MacToolbox 0xa98d
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa98d_u32 as u16))?;
                    // 00121ab6: move.l $442930.l, -(a7)
                    e.step()?;
                    let value_address = 0x442930_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121abc: move.l $44292c.l, -(a7)
                    e.step()?;
                    let value_address = 0x44292c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121ac2: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121ac4: move.l #$10001, -(a7)
                    e.step()?;
                    let value = 0x10001_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121aca: bsr.l $14ce2a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121ad0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x14ce2a_u32)?;
                    // 00121ad0: cmpi.w #$6, $4434b2.l
                    e.step()?;
                    let source = 0x6_u32;
                    let destination_address = 0x4434b2_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00121ad8: lea.l $1c(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x1c_u32);
                    r.put_address(7, value);
                    // 00121adc: bne.w $122894
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122894_u32
                    } else {
                        0x121ae0_u32
                    };
                }
                0x121ae0 => {
                    // 00121ae0: tst.b $4433f7.l
                    e.step()?;
                    let value_address = 0x4433f7_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00121ae6: beq.w $1218fc
                    e.step()?;
                    pc = if r.equal()? {
                        0x1218fc_u32
                    } else {
                        0x121aea_u32
                    };
                }
                0x121aea => {
                    // 00121aea: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121af0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00121af0: move.w #$44d, -(a7)
                    e.step()?;
                    let value = 0x44d_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121af4: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121afa_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 00121afa: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121b00: jsr $400088.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121b06_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400088_u32))?;
                    // 00121b06: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00121b08: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121b0a: bne.b $121b16
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121b16_u32
                    } else {
                        0x121b0c_u32
                    };
                }
                0x121b0c => {
                    // 00121b0c: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121b12_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 00121b12: bra.w $1218fc
                    e.step()?;
                    pc = 0x1218fc_u32;
                }
                0x121b16 => {
                    // 00121b16: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121b1c: jsr $400180.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121b22_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400180_u32))?;
                    // 00121b22: pea.l $4392f4.l
                    e.step()?;
                    let value = 0x4392f4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121b28: jsr $400180.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121b2e_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400180_u32))?;
                    // 00121b2e: pea.l $4392f4.l
                    e.step()?;
                    let value = 0x4392f4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121b34: pea.l -$100(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffff00_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121b38: jsr $400090.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121b3e_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400090_u32))?;
                    // 00121b3e: pea.l $4392f4.l
                    e.step()?;
                    let value = 0x4392f4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121b44: jsr $400188.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121b4a_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400188_u32))?;
                    // 00121b4a: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00121b4c: lea.l $14(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x14_u32);
                    r.put_address(7, value);
                    // 00121b50: bra.b $121ba2
                    e.step()?;
                    pc = 0x121ba2_u32;
                }
                0x121b52 => {
                    // 00121b52: lea.l -$100(a6), a0
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffff00_u32);
                    r.put_address(0, value);
                    // 00121b56: cmpi.b #$20, (a0, d3.w)
                    e.step()?;
                    let source = 0x20_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00121b5c: beq.b $121ba8
                    e.step()?;
                    pc = if r.equal()? {
                        0x121ba8_u32
                    } else {
                        0x121b5e_u32
                    };
                }
                0x121b5e => {
                    // 00121b5e: lea.l -$100(a6), a0
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffff00_u32);
                    r.put_address(0, value);
                    // 00121b62: tst.b (a0, d3.w)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00121b66: beq.b $121ba8
                    e.step()?;
                    pc = if r.equal()? {
                        0x121ba8_u32
                    } else {
                        0x121b68_u32
                    };
                }
                0x121b68 => {
                    // 00121b68: lea.l $438aa4.l, a0
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.put_address(0, value);
                    // 00121b6e: move.b (a0, d3.w), d0
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00121b72: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00121b74: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121b76: jsr $400040.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121b7c_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400040_u32))?;
                    // 00121b7c: lea.l $438aa4.l, a0
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.put_address(0, value);
                    // 00121b82: move.b d0, (a0, d3.w)
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00121b86: lea.l $438aa4.l, a0
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.put_address(0, value);
                    // 00121b8c: lea.l -$100(a6), a1
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xffffff00_u32);
                    r.put_address(1, value);
                    // 00121b90: move.b (a1, d3.w), d0
                    e.step()?;
                    let value_address = r.address(1)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00121b94: cmp.b (a0, d3.w), d0
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let source = u32::from(e.b(source_address)?);
                    let destination = r.data(0, 8)?;
                    r.subtract_flags(destination, source, 8, false);
                    // 00121b98: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121b9a: beq.b $121ba0
                    e.step()?;
                    pc = if r.equal()? {
                        0x121ba0_u32
                    } else {
                        0x121b9c_u32
                    };
                }
                0x121b9c => {
                    // 00121b9c: moveq #$0, d6
                    e.step()?;
                    r.put_data(6, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00121b9e: bra.b $121ba8
                    e.step()?;
                    pc = 0x121ba8_u32;
                }
                0x121ba0 => {
                    // 00121ba0: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x121ba2_u32;
                }
                0x121ba2 => {
                    // 00121ba2: cmpi.w #$28, d3
                    e.step()?;
                    let source = 0x28_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00121ba6: blt.b $121b52
                    e.step()?;
                    pc = if r.less()? {
                        0x121b52_u32
                    } else {
                        0x121ba8_u32
                    };
                }
                0x121ba8 => {
                    // 00121ba8: pea.l $403a82.l
                    e.step()?;
                    let value = 0x403a82_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121bae: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121bb4: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121bba_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 00121bba: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00121bbc: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121bbe: bne.w $121c56
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121c56_u32
                    } else {
                        0x121bc2_u32
                    };
                }
                0x121bc2 => {
                    // 00121bc2: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121bc8: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121bce_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 00121bce: clr.l $442774.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442774_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121bd4: clr.l $442768.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442768_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121bda: clr.l $442764.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442764_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121be0: move.w $435500.l, -(a7)
                    e.step()?;
                    let value_address = 0x435500_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121be6: bsr.l $133f3e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121bec_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133f3e_u32)?;
                    // 00121bec: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121bf2: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121bf8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 00121bf8: move.l $442778.l, $442764.l
                    e.step()?;
                    let value_address = 0x442778_u32;
                    let value = e.l(value_address)?;
                    let destination_address = 0x442764_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121c02: move.b $44277b.l, $479eaf.l
                    e.step()?;
                    let value_address = 0x44277b_u32;
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = 0x479eaf_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00121c0c: move.l $44277c.l, $442768.l
                    e.step()?;
                    let value_address = 0x44277c_u32;
                    let value = e.l(value_address)?;
                    let destination_address = 0x442768_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121c16: move.b $44277f.l, $479eb0.l
                    e.step()?;
                    let value_address = 0x44277f_u32;
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = 0x479eb0_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00121c20: move.b $442777.l, $479eae.l
                    e.step()?;
                    let value_address = 0x442777_u32;
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = 0x479eae_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00121c2a: clr.l $442760.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442760_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121c30: clr.l $44275c.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x44275c_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121c36: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121c3c: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121c42_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 00121c42: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121c48_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 00121c48: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121c4e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 00121c4e: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    // 00121c52: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x121c56 => {
                    // 00121c56: pea.l $403a7c.l
                    e.step()?;
                    let value = 0x403a7c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121c5c: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121c62: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121c68_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 00121c68: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00121c6a: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121c6c: bne.b $121cce
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121cce_u32
                    } else {
                        0x121c6e_u32
                    };
                }
                0x121c6e => {
                    // 00121c6e: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121c74: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121c7a: pea.l $403a6c.l
                    e.step()?;
                    let value = 0x403a6c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121c80: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121c86_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00121c86: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121c8c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00121c8c: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121c92: pea.l $4427b6.l
                    e.step()?;
                    let value = 0x4427b6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121c98: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121c9e_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00121c9e: cmpi.l #$29a, $4427b6.l
                    e.step()?;
                    let source = 0x29a_u32;
                    let destination_address = 0x4427b6_u32;
                    let destination = e.l(destination_address)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00121ca8: lea.l $c(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xc_u32);
                    r.put_address(7, value);
                    // 00121cac: bne.b $121cb8
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121cb8_u32
                    } else {
                        0x121cae_u32
                    };
                }
                0x121cae => {
                    // 00121cae: move.b #$1, $442748.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x442748_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00121cb6: bra.b $121cbe
                    e.step()?;
                    pc = 0x121cbe_u32;
                }
                0x121cb8 => {
                    // 00121cb8: clr.b $442748.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442748_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x121cbe_u32;
                }
                0x121cbe => {
                    // 00121cbe: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121cc4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 00121cc4: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121cca_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 00121cca: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x121cce => {
                    // 00121cce: pea.l $403a64.l
                    e.step()?;
                    let value = 0x403a64_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121cd4: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121cda: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121ce0_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 00121ce0: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00121ce2: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121ce4: bne.w $121e46
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121e46_u32
                    } else {
                        0x121ce8_u32
                    };
                }
                0x121ce8 => {
                    // 00121ce8: bsr.l $13a6d4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121cee_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a6d4_u32)?;
                    // 00121cee: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00121cf0: beq.w $121e46
                    e.step()?;
                    pc = if r.equal()? {
                        0x121e46_u32
                    } else {
                        0x121cf4_u32
                    };
                }
                0x121cf4 => {
                    // 00121cf4: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121cfa: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121d00: pea.l $403a4e.l
                    e.step()?;
                    let value = 0x403a4e_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121d06: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121d0c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00121d0c: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121d12_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00121d12: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121d18: pea.l $4427b6.l
                    e.step()?;
                    let value = 0x4427b6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121d1e: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121d24_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00121d24: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121d2a: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121d30: pea.l $403a4c.l
                    e.step()?;
                    let value = 0x403a4c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121d36: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121d3c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00121d3c: move.w $4427b8.l, $4434b6.l
                    e.step()?;
                    let value_address = 0x4427b8_u32;
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121d46: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121d4c: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121d52_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 00121d52: move.w $4434b6.l, $435500.l
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x435500_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121d5c: move.w $4434b6.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121d62: bsr.l $133f3e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121d68_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133f3e_u32)?;
                    // 00121d68: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121d6e: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121d74_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 00121d74: move.l $442778.l, $442764.l
                    e.step()?;
                    let value_address = 0x442778_u32;
                    let value = e.l(value_address)?;
                    let destination_address = 0x442764_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121d7e: move.b $44277b.l, $479eaf.l
                    e.step()?;
                    let value_address = 0x44277b_u32;
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = 0x479eaf_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00121d88: move.l $44277c.l, $442768.l
                    e.step()?;
                    let value_address = 0x44277c_u32;
                    let value = e.l(value_address)?;
                    let destination_address = 0x442768_u32;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121d92: move.b $44277f.l, $479eb0.l
                    e.step()?;
                    let value_address = 0x44277f_u32;
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = 0x479eb0_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00121d9c: move.b $442777.l, $479eae.l
                    e.step()?;
                    let value_address = 0x442777_u32;
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = 0x479eae_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00121da6: clr.w $4434b6.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121dac: lea.l $22(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x22_u32);
                    r.put_address(7, value);
                    // 00121db0: bra.b $121dce
                    e.step()?;
                    pc = 0x121dce_u32;
                }
                0x121db2 => {
                    // 00121db2: move.w $4434b6.l, d0
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00121db8: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00121dbc: lea.l $440edc.l, a0
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(0, value);
                    // 00121dc2: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00121dc4: clr.b $103(a0)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x103_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00121dc8: addq.w #$1, $4434b6.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4434b6_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    pc = 0x121dce_u32;
                }
                0x121dce => {
                    // 00121dce: cmpi.w #$14, $4434b6.l
                    e.step()?;
                    let source = 0x14_u32;
                    let destination_address = 0x4434b6_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00121dd6: blt.b $121db2
                    e.step()?;
                    pc = if r.less()? {
                        0x121db2_u32
                    } else {
                        0x121dd8_u32
                    };
                }
                0x121dd8 => {
                    // 00121dd8: bsr.l $1046e0
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121dde_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1046e0_u32)?;
                    // 00121dde: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121de4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 00121de4: move.l #$870000, -(a7)
                    e.step()?;
                    let value = 0x870000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121dea: move.w $435500.l, d0
                    e.step()?;
                    let value_address = 0x435500_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00121df0: addi.w #$3e8, d0
                    e.step()?;
                    let source = 0x3e8_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00121df4: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121df6: bsr.l $13ebde
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121dfc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13ebde_u32)?;
                    // 00121dfc: move.w $435500.l, d0
                    e.step()?;
                    let value_address = 0x435500_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00121e02: mulu.w #$1d0, d0
                    e.step()?;
                    let source = 0x1d0_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00121e06: lea.l $41d1c4.l, a0
                    e.step()?;
                    let value = 0x41d1c4_u32;
                    r.put_address(0, value);
                    // 00121e0c: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00121e0e: tst.w $106(a0)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x106_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00121e12: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121e14: beq.b $121e40
                    e.step()?;
                    pc = if r.equal()? {
                        0x121e40_u32
                    } else {
                        0x121e16_u32
                    };
                }
                0x121e16 => {
                    // 00121e16: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121e18: move.w $435500.l, d0
                    e.step()?;
                    let value_address = 0x435500_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00121e1e: mulu.w #$1d0, d0
                    e.step()?;
                    let source = 0x1d0_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00121e22: lea.l $41d1c4.l, a0
                    e.step()?;
                    let value = 0x41d1c4_u32;
                    r.put_address(0, value);
                    // 00121e28: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00121e2a: move.w $106(a0), -(a7)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x106_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121e2e: move.w #$1, -(a7)
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121e32: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121e34: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121e36: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121e3c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 00121e3c: lea.l $e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xe_u32);
                    r.put_address(7, value);
                    pc = 0x121e40_u32;
                }
                0x121e40 => {
                    // 00121e40: moveq #$0, d0
                    e.step()?;
                    r.put_data(0, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00121e42: bra.w $122b02
                    e.step()?;
                    pc = 0x122b02_u32;
                }
                0x121e46 => {
                    // 00121e46: pea.l $403a7c.l
                    e.step()?;
                    let value = 0x403a7c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121e4c: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121e52: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121e58_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 00121e58: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00121e5a: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121e5c: bne.b $121ebe
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121ebe_u32
                    } else {
                        0x121e5e_u32
                    };
                }
                0x121e5e => {
                    // 00121e5e: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121e64: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121e6a: pea.l $403a6c.l
                    e.step()?;
                    let value = 0x403a6c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121e70: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121e76_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00121e76: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121e7c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00121e7c: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121e82: pea.l $4427b6.l
                    e.step()?;
                    let value = 0x4427b6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121e88: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121e8e_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00121e8e: cmpi.l #$29a, $4427b6.l
                    e.step()?;
                    let source = 0x29a_u32;
                    let destination_address = 0x4427b6_u32;
                    let destination = e.l(destination_address)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 00121e98: lea.l $c(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xc_u32);
                    r.put_address(7, value);
                    // 00121e9c: bne.b $121ea8
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121ea8_u32
                    } else {
                        0x121e9e_u32
                    };
                }
                0x121e9e => {
                    // 00121e9e: move.b #$1, $442748.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x442748_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00121ea6: bra.b $121eae
                    e.step()?;
                    pc = 0x121eae_u32;
                }
                0x121ea8 => {
                    // 00121ea8: clr.b $442748.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x442748_u32;
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    pc = 0x121eae_u32;
                }
                0x121eae => {
                    // 00121eae: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121eb4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 00121eb4: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121eba_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 00121eba: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x121ebe => {
                    // 00121ebe: pea.l $403a42.l
                    e.step()?;
                    let value = 0x403a42_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121ec4: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121eca: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121ed0_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 00121ed0: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00121ed2: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121ed4: bne.w $121f90
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121f90_u32
                    } else {
                        0x121ed8_u32
                    };
                }
                0x121ed8 => {
                    // 00121ed8: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121ede: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121ee4: pea.l $403a32.l
                    e.step()?;
                    let value = 0x403a32_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121eea: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121ef0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00121ef0: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121ef6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00121ef6: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00121ef8: lea.l $c(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0xc_u32);
                    r.put_address(7, value);
                    // 00121efc: bra.b $121f14
                    e.step()?;
                    pc = 0x121f14_u32;
                }
                0x121efe => {
                    // 00121efe: lea.l $44331c.l, a0
                    e.step()?;
                    let value = 0x44331c_u32;
                    r.put_address(0, value);
                    // 00121f04: clr.w $28(a0, d3.w * 2)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x28_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121f08: lea.l $44331c.l, a0
                    e.step()?;
                    let value = 0x44331c_u32;
                    r.put_address(0, value);
                    // 00121f0e: clr.w (a0, d3.w * 2)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121f12: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x121f14_u32;
                }
                0x121f14 => {
                    // 00121f14: cmpi.w #$14, d3
                    e.step()?;
                    let source = 0x14_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00121f18: blt.b $121efe
                    e.step()?;
                    pc = if r.less()? {
                        0x121efe_u32
                    } else {
                        0x121f1a_u32
                    };
                }
                0x121f1a => {
                    // 00121f1a: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121f20: pea.l $4427b6.l
                    e.step()?;
                    let value = 0x4427b6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121f26: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121f2c_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00121f2c: move.w $4427b8.l, $44331c.l
                    e.step()?;
                    let value_address = 0x4427b8_u32;
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x44331c_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121f36: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121f3c: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121f42: pea.l $403a4c.l
                    e.step()?;
                    let value = 0x403a4c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121f48: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121f4e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00121f4e: move.l #$10000, -(a7)
                    e.step()?;
                    let value = 0x10000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121f54: bsr.l $105748
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121f5a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x105748_u32)?;
                    // 00121f5a: move.l #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121f60: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121f66_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 00121f66: move.l $4427ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4427ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121f6c: MacToolbox 0xa93c
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa93c_u32 as u16))?;
                    // 00121f6e: move.l $43871a.l, -(a7)
                    e.step()?;
                    let value_address = 0x43871a_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121f74: move.w #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121f78: MacToolbox 0xa935
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa935_u32 as u16))?;
                    // 00121f7a: move.l $438716.l, -(a7)
                    e.step()?;
                    let value_address = 0x438716_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121f80: move.w #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121f84: MacToolbox 0xa935
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa935_u32 as u16))?;
                    // 00121f86: MacToolbox 0xa937
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa937_u32 as u16))?;
                    // 00121f88: lea.l $14(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x14_u32);
                    r.put_address(7, value);
                    // 00121f8c: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x121f90 => {
                    // 00121f90: pea.l $403a2c.l
                    e.step()?;
                    let value = 0x403a2c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121f96: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121f9c: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121fa2_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 00121fa2: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00121fa4: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121fa6: bne.b $121fe2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x121fe2_u32
                    } else {
                        0x121fa8_u32
                    };
                }
                0x121fa8 => {
                    // 00121fa8: clr.w $4394b4.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x4394b4_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121fae: move.l #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121fb4: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121fba_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 00121fba: move.l $4427ba.l, -(a7)
                    e.step()?;
                    let value_address = 0x4427ba_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121fc0: MacToolbox 0xa93c
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa93c_u32 as u16))?;
                    // 00121fc2: move.l $43871a.l, -(a7)
                    e.step()?;
                    let value_address = 0x43871a_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121fc8: move.w #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121fcc: MacToolbox 0xa935
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa935_u32 as u16))?;
                    // 00121fce: move.l $438716.l, -(a7)
                    e.step()?;
                    let value_address = 0x438716_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00121fd4: move.w #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00121fd8: MacToolbox 0xa935
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa935_u32 as u16))?;
                    // 00121fda: MacToolbox 0xa937
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa937_u32 as u16))?;
                    // 00121fdc: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121fde: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x121fe2 => {
                    // 00121fe2: pea.l $403a20.l
                    e.step()?;
                    let value = 0x403a20_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121fe8: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00121fee: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x121ff4_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 00121ff4: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00121ff6: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00121ff8: bne.w $122128
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122128_u32
                    } else {
                        0x121ffc_u32
                    };
                }
                0x121ffc => {
                    // 00121ffc: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122002: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122008_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 00122008: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012200e: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122014: pea.l $403a06.l
                    e.step()?;
                    let value = 0x403a06_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012201a: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122020_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00122020: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122026_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00122026: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012202c: pea.l $4427b6.l
                    e.step()?;
                    let value = 0x4427b6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122032: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122038_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00122038: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12203e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 0012203e: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122044: pea.l -$104(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffefc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122048: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12204e_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 0012204e: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122054_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00122054: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012205a: pea.l -$108(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffef8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012205e: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122064_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00122064: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12206a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 0012206a: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122070: pea.l -$10c(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffef4_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122074: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12207a_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 0012207a: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122080: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122086: pea.l $403a4c.l
                    e.step()?;
                    let value = 0x403a4c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012208c: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122092_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00122092: move.l $4427b6.l, -(a7)
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122098: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12209e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 0012209e: move.w -$102(a6), $4434b6.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffefe_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001220a6: move.w -$106(a6), d7
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffefa_u32);
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(7, 16, value);
                    r.move_flags(value, 16);
                    // 001220aa: move.w -$10a(a6), -$10e(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffef6_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(6)?.wrapping_add(0xfffffef2_u32);
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001220b0: move.w $4434b6.l, d0
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001220b6: mulu.w #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001220ba: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 001220c0: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001220c2: cmpi.w #$3e7, (a0, d7.w * 2)
                    e.step()?;
                    let source = 0x3e7_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(7, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 001220c8: lea.l $20(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x20_u32);
                    r.put_address(7, value);
                    // 001220cc: bge.b $1220fe
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x1220fe_u32
                    } else {
                        0x1220ce_u32
                    };
                }
                0x1220ce => {
                    // 001220ce: move.w $4434b6.l, d0
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001220d4: mulu.w #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001220d8: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 001220de: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001220e0: tst.w (a0, d7.w * 2)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(7, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001220e4: ble.b $1220fe
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1220fe_u32
                    } else {
                        0x1220e6_u32
                    };
                }
                0x1220e6 => {
                    // 001220e6: move.w $4434b6.l, d0
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001220ec: mulu.w #$b4, d0
                    e.step()?;
                    let source = 0xb4_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001220f0: lea.l $4085b8.l, a0
                    e.step()?;
                    let value = 0x4085b8_u32;
                    r.put_address(0, value);
                    // 001220f6: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001220f8: move.w -$10e(a6), (a0, d7.w * 2)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffef2_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(7, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    pc = 0x1220fe_u32;
                }
                0x1220fe => {
                    // 001220fe: move.l $4427b6.l, -(a7)
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122104: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12210a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 0012210a: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122110: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122116_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 00122116: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12211c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 0012211c: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122122_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 00122122: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00122124: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122128 => {
                    // 00122128: pea.l $4039fa.l
                    e.step()?;
                    let value = 0x4039fa_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012212e: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122134: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12213a_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 0012213a: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0012213c: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0012213e: bne.w $122306
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122306_u32
                    } else {
                        0x122142_u32
                    };
                }
                0x122142 => {
                    // 00122142: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122148: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12214e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 0012214e: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122154: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012215a: pea.l $4039de.l
                    e.step()?;
                    let value = 0x4039de_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122160: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122166_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00122166: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12216c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 0012216c: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122172: pea.l $4427b6.l
                    e.step()?;
                    let value = 0x4427b6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122178: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12217e_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 0012217e: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122184_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00122184: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012218a: pea.l -$104(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffefc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012218e: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122194_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00122194: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012219a: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001221a0: pea.l $403a4c.l
                    e.step()?;
                    let value = 0x403a4c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001221a6: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1221ac_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 001221ac: move.l $4427b6.l, -(a7)
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001221b2: jsr $400038.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1221b8_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400038_u32))?;
                    // 001221b8: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001221ba: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001221bc: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001221be: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1221c4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 001221c4: move.w -$102(a6), $4434b6.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffefe_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001221cc: tst.l $4427b6.l
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.move_flags(value, 32);
                    // 001221d2: lea.l $1e(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x1e_u32);
                    r.put_address(7, value);
                    // 001221d6: bge.b $1221e8
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x1221e8_u32
                    } else {
                        0x1221d8_u32
                    };
                }
                0x1221d8 => {
                    // 001221d8: pea.l $4039d4.l
                    e.step()?;
                    let value = 0x4039d4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001221de: bsr.l $136cfa
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1221e4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x136cfa_u32)?;
                    // 001221e4: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001221e6: bra.b $1221f6
                    e.step()?;
                    pc = 0x1221f6_u32;
                }
                0x1221e8 => {
                    // 001221e8: pea.l $4039cc.l
                    e.step()?;
                    let value = 0x4039cc_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001221ee: bsr.l $136cfa
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1221f4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x136cfa_u32)?;
                    // 001221f4: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x1221f6_u32;
                }
                0x1221f6 => {
                    // 001221f6: pea.l $403a88.l
                    e.step()?;
                    let value = 0x403a88_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001221fc: pea.l $4389a4.l
                    e.step()?;
                    let value = 0x4389a4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122202: jsr $400070.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122208_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400070_u32))?;
                    // 00122208: movea.l a0, a3
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_address(3, value);
                    // 0012220a: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0012220c: moveq #$2a, d0
                    e.step()?;
                    r.put_data(0, 32, 0x2a_u32);
                    r.move_flags(0x2a_u32, 32);
                    // 0012220e: mulu.w $4434b6.l, d0
                    e.step()?;
                    let source_address = 0x4434b6_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00122214: move.l $4427b6.l, d1
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0012221a: subq.l #$1, d1
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(1, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 0012221c: mulu.l #$1100c, d1
                    e.step()?;
                    let source = 0x1100c_u32;
                    let destination = r.data(1, 32)?;
                    let product = u64::from(source) * u64::from(destination);
                    let value = product as u32;
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, product > u64::from(u32::MAX));
                    // 00122224: add.l d0, d1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(1, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(1, 32, value);
                    // 00122226: move.l d1, -(a7)
                    e.step()?;
                    let value = r.data(1, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122228: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012222a: jsr $400078.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122230_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400078_u32))?;
                    // 00122230: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122232: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122236: pea.l $2a.w
                    e.step()?;
                    let value = 0x2a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012223a: pea.l $479eaa.l
                    e.step()?;
                    let value = 0x479eaa_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122240: jsr $400048.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122246_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400048_u32))?;
                    // 00122246: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122248: jsr $400068.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12224e_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400068_u32))?;
                    // 0012224e: clr.w $4434ac.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x4434ac_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122254: lea.l $26(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x26_u32);
                    r.put_address(7, value);
                    // 00122258: bra.b $1222b0
                    e.step()?;
                    pc = 0x1222b0_u32;
                }
                0x12225a => {
                    // 0012225a: lea.l $479eaa.l, a0
                    e.step()?;
                    let value = 0x479eaa_u32;
                    r.put_address(0, value);
                    // 00122260: move.w $4434ac.l, d0
                    e.step()?;
                    let value_address = 0x4434ac_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122266: moveq #$2a, d1
                    e.step()?;
                    r.put_data(1, 32, 0x2a_u32);
                    r.move_flags(0x2a_u32, 32);
                    // 00122268: mulu.w $4434b6.l, d1
                    e.step()?;
                    let source_address = 0x4434b6_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 0012226e: lea.l $478e42.l, a1
                    e.step()?;
                    let value = 0x478e42_u32;
                    r.put_address(1, value);
                    // 00122274: adda.l d1, a1
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 00122276: move.w $4434ac.l, d1
                    e.step()?;
                    let value_address = 0x4434ac_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 0012227c: move.w $8(a0, d0.w * 2), $8(a1, d1.w)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x8_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(1)?.wrapping_add(0x8_u32).wrapping_add(
                        (i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122282: lea.l $479eaa.l, a0
                    e.step()?;
                    let value = 0x479eaa_u32;
                    r.put_address(0, value);
                    // 00122288: move.w $4434ac.l, d0
                    e.step()?;
                    let value_address = 0x4434ac_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 0012228e: moveq #$2a, d1
                    e.step()?;
                    r.put_data(1, 32, 0x2a_u32);
                    r.move_flags(0x2a_u32, 32);
                    // 00122290: mulu.w $4434b6.l, d1
                    e.step()?;
                    let source_address = 0x4434b6_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(1, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(1, 32, value);
                    r.move_flags(value, 32);
                    // 00122296: lea.l $478e42.l, a1
                    e.step()?;
                    let value = 0x478e42_u32;
                    r.put_address(1, value);
                    // 0012229c: adda.l d1, a1
                    e.step()?;
                    let source = r.data(1, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0012229e: move.w $4434ac.l, d1
                    e.step()?;
                    let value_address = 0x4434ac_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(1, 16, value);
                    r.move_flags(value, 16);
                    // 001222a4: move.w $18(a0, d0.w * 2), $18(a1, d1.w)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x18_u32).wrapping_add(
                        (i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = r.address(1)?.wrapping_add(0x18_u32).wrapping_add(
                        (i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001222aa: addq.w #$1, $4434ac.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4434ac_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    pc = 0x1222b0_u32;
                }
                0x1222b0 => {
                    // 001222b0: cmpi.w #$8, $4434ac.l
                    e.step()?;
                    let source = 0x8_u32;
                    let destination_address = 0x4434ac_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 001222b8: blt.b $12225a
                    e.step()?;
                    pc = if r.less()? {
                        0x12225a_u32
                    } else {
                        0x1222ba_u32
                    };
                }
                0x1222ba => {
                    // 001222ba: tst.w $4434b6.l
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001222c0: ble.b $1222d6
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1222d6_u32
                    } else {
                        0x1222c2_u32
                    };
                }
                0x1222c2 => {
                    // 001222c2: move.l $4427b6.l, d0
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001222c8: subq.l #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001222ca: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001222cc: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1222d2_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 001222d2: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001222d4: bra.b $1222e8
                    e.step()?;
                    pc = 0x1222e8_u32;
                }
                0x1222d6 => {
                    // 001222d6: move.l $4427b6.l, d0
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001222dc: subq.l #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001222de: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001222e0: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1222e6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 001222e6: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x1222e8_u32;
                }
                0x1222e8 => {
                    // 001222e8: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001222ee: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1222f4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 001222f4: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1222fa_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 001222fa: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122300_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 00122300: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00122302: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122306 => {
                    // 00122306: pea.l $4039be.l
                    e.step()?;
                    let value = 0x4039be_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012230c: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122312: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122318_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 00122318: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0012231a: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0012231c: bne.w $122432
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122432_u32
                    } else {
                        0x122320_u32
                    };
                }
                0x122320 => {
                    // 00122320: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122326: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12232c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 0012232c: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122332: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122338: pea.l $403992.l
                    e.step()?;
                    let value = 0x403992_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012233e: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122344_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00122344: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12234a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 0012234a: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122350: pea.l $4427b6.l
                    e.step()?;
                    let value = 0x4427b6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122356: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12235c_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 0012235c: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122362_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00122362: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122368: pea.l -$104(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffefc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012236c: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122372_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00122372: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122378_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00122378: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012237e: pea.l -$108(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffef8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122382: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122388_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00122388: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012238e: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122394: pea.l $403a4c.l
                    e.step()?;
                    let value = 0x403a4c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012239a: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1223a0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 001223a0: tst.l $4427b6.l
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.move_flags(value, 32);
                    // 001223a6: lea.l $1c(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x1c_u32);
                    r.put_address(7, value);
                    // 001223aa: ble.b $1223c0
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x1223c0_u32
                    } else {
                        0x1223ac_u32
                    };
                }
                0x1223ac => {
                    // 001223ac: move.l $4427b6.l, d0
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001223b2: subq.l #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001223b4: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001223b6: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1223bc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 001223bc: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001223be: bra.b $1223d2
                    e.step()?;
                    pc = 0x1223d2_u32;
                }
                0x1223c0 => {
                    // 001223c0: move.l $4427b6.l, d0
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001223c6: subq.l #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001223c8: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001223ca: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1223d0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 001223d0: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x1223d2_u32;
                }
                0x1223d2 => {
                    // 001223d2: moveq #$2a, d0
                    e.step()?;
                    r.put_data(0, 32, 0x2a_u32);
                    r.move_flags(0x2a_u32, 32);
                    // 001223d4: muls.l -$104(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffefc_u32);
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let product = i64::from(source as i32) * i64::from(destination as i32);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
                    // 001223da: lea.l $478e42.l, a0
                    e.step()?;
                    let value = 0x478e42_u32;
                    r.put_address(0, value);
                    // 001223e0: move.b -$105(a6), $7(a0, d0.l)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffefb_u32);
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x7_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 001223e6: tst.w $4434b6.l
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001223ec: ble.b $122402
                    e.step()?;
                    pc = if r.less_equal()? {
                        0x122402_u32
                    } else {
                        0x1223ee_u32
                    };
                }
                0x1223ee => {
                    // 001223ee: move.l $4427b6.l, d0
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001223f4: subq.l #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001223f6: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001223f8: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1223fe_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 001223fe: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00122400: bra.b $122414
                    e.step()?;
                    pc = 0x122414_u32;
                }
                0x122402 => {
                    // 00122402: move.l $4427b6.l, d0
                    e.step()?;
                    let value_address = 0x4427b6_u32;
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00122408: subq.l #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 32)?;
                    let value = r.subtract_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 0012240a: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012240c: bsr.l $151e24
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122412_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151e24_u32)?;
                    // 00122412: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x122414_u32;
                }
                0x122414 => {
                    // 00122414: move.l $442774.l, -(a7)
                    e.step()?;
                    let value_address = 0x442774_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012241a: bsr.l $13107a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122420_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13107a_u32)?;
                    // 00122420: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122426_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 00122426: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12242c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 0012242c: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0012242e: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122432 => {
                    // 00122432: pea.l $403984.l
                    e.step()?;
                    let value = 0x403984_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122438: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012243e: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122444_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 00122444: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00122446: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00122448: bne.w $12258a
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x12258a_u32
                    } else {
                        0x12244c_u32
                    };
                }
                0x12244c => {
                    // 0012244c: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122452: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122458: pea.l $403974.l
                    e.step()?;
                    let value = 0x403974_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012245e: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122464_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00122464: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12246a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 0012246a: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122470: pea.l $4427b6.l
                    e.step()?;
                    let value = 0x4427b6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122476: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12247c_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 0012247c: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122482: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122488: pea.l $403a4c.l
                    e.step()?;
                    let value = 0x403a4c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012248e: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122494_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00122494: pea.l $40396e.l
                    e.step()?;
                    let value = 0x40396e_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012249a: bsr.l $136d86
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1224a0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x136d86_u32)?;
                    // 001224a0: pea.l $40396a.l
                    e.step()?;
                    let value = 0x40396a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001224a6: pea.l $4389a4.l
                    e.step()?;
                    let value = 0x4389a4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001224ac: jsr $400070.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1224b2_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400070_u32))?;
                    // 001224b2: movea.l a0, a3
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_address(3, value);
                    // 001224b4: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001224b6: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 001224b8: mulu.l $4427b6.l, d0
                    e.step()?;
                    let source_address = 0x4427b6_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let product = u64::from(source) * u64::from(destination);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, product > u64::from(u32::MAX));
                    // 001224c0: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001224c2: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001224c4: jsr $400078.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1224ca_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400078_u32))?;
                    // 001224ca: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001224cc: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001224d0: pea.l $28.w
                    e.step()?;
                    let value = 0x28_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001224d4: pea.l $439d14.l
                    e.step()?;
                    let value = 0x439d14_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001224da: jsr $400048.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1224e0_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400048_u32))?;
                    // 001224e0: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001224e2: jsr $400068.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1224e8_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400068_u32))?;
                    // 001224e8: move.w $4427d4.l, d0
                    e.step()?;
                    let value_address = 0x4427d4_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001224ee: addq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001224f0: move.w d0, $439d14.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x439d14_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001224f6: move.w #$ffff, $439d22.l
                    e.step()?;
                    let value = 0xffff_u32;
                    let destination_address = 0x439d22_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001224fe: move.w #$ffff, $439d20.l
                    e.step()?;
                    let value = 0xffff_u32;
                    let destination_address = 0x439d20_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122506: move.w #$ffff, $439d26.l
                    e.step()?;
                    let value = 0xffff_u32;
                    let destination_address = 0x439d26_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0012250e: move.w #$ffff, $439d24.l
                    e.step()?;
                    let value = 0xffff_u32;
                    let destination_address = 0x439d24_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122516: move.w #$ffff, $439d1e.l
                    e.step()?;
                    let value = 0xffff_u32;
                    let destination_address = 0x439d1e_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0012251e: move.w #$ffff, $439d1c.l
                    e.step()?;
                    let value = 0xffff_u32;
                    let destination_address = 0x439d1c_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122526: move.w #$64, $439d18.l
                    e.step()?;
                    let value = 0x64_u32;
                    let destination_address = 0x439d18_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0012252e: pea.l $40396a.l
                    e.step()?;
                    let value = 0x40396a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122534: pea.l $4389a4.l
                    e.step()?;
                    let value = 0x4389a4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012253a: jsr $400070.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122540_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400070_u32))?;
                    // 00122540: movea.l a0, a3
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_address(3, value);
                    // 00122542: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122544: moveq #$28, d0
                    e.step()?;
                    r.put_data(0, 32, 0x28_u32);
                    r.move_flags(0x28_u32, 32);
                    // 00122546: mulu.l $4427b6.l, d0
                    e.step()?;
                    let source_address = 0x4427b6_u32;
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    let product = u64::from(source) * u64::from(destination);
                    let value = product as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    r.put_flag(2, product > u64::from(u32::MAX));
                    // 0012254e: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122550: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122552: jsr $400078.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122558_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400078_u32))?;
                    // 00122558: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012255a: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012255e: pea.l $28.w
                    e.step()?;
                    let value = 0x28_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122562: pea.l $439d14.l
                    e.step()?;
                    let value = 0x439d14_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122568: jsr $400050.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12256e_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400050_u32))?;
                    // 0012256e: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122570: jsr $400068.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122576_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400068_u32))?;
                    // 00122576: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12257c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 0012257c: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122582_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 00122582: lea.l $68(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x68_u32);
                    r.put_address(7, value);
                    // 00122586: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x12258a => {
                    // 0012258a: pea.l $40395a.l
                    e.step()?;
                    let value = 0x40395a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122590: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122596: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12259c_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 0012259c: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0012259e: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001225a0: bne.b $12261e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x12261e_u32
                    } else {
                        0x1225a2_u32
                    };
                }
                0x1225a2 => {
                    // 001225a2: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001225a8: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001225ae: pea.l $403948.l
                    e.step()?;
                    let value = 0x403948_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001225b4: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1225ba_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 001225ba: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1225c0_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 001225c0: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001225c6: pea.l $4427b6.l
                    e.step()?;
                    let value = 0x4427b6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001225cc: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1225d2_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 001225d2: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001225d8: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001225de: pea.l $403a4c.l
                    e.step()?;
                    let value = 0x403a4c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001225e4: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1225ea_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 001225ea: move.w $4427b8.l, $4434b6.l
                    e.step()?;
                    let value_address = 0x4427b8_u32;
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001225f4: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001225f6: move.w $4434b6.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001225fc: move.w #$2, -(a7)
                    e.step()?;
                    let value = 0x2_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122600: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122602: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122604: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12260a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 0012260a: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122610_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 00122610: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122616_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 00122616: lea.l $26(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x26_u32);
                    r.put_address(7, value);
                    // 0012261a: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x12261e => {
                    // 0012261e: pea.l $403938.l
                    e.step()?;
                    let value = 0x403938_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122624: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012262a: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122630_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 00122630: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 00122632: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00122634: bne.b $1226b2
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1226b2_u32
                    } else {
                        0x122636_u32
                    };
                }
                0x122636 => {
                    // 00122636: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012263c: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122642: pea.l $403924.l
                    e.step()?;
                    let value = 0x403924_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122648: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12264e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 0012264e: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122654_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00122654: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012265a: pea.l $4427b6.l
                    e.step()?;
                    let value = 0x4427b6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122660: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122666_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00122666: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012266c: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122672: pea.l $403a4c.l
                    e.step()?;
                    let value = 0x403a4c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122678: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12267e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 0012267e: move.w $4427b8.l, $4434b6.l
                    e.step()?;
                    let value_address = 0x4427b8_u32;
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122688: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0012268a: move.w $4434b6.l, -(a7)
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122690: move.w #$3, -(a7)
                    e.step()?;
                    let value = 0x3_u32;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122694: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122696: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122698: bsr.l $1400e4
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12269e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1400e4_u32)?;
                    // 0012269e: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1226a4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 001226a4: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1226aa_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 001226aa: lea.l $26(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x26_u32);
                    r.put_address(7, value);
                    // 001226ae: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x1226b2 => {
                    // 001226b2: pea.l $40391a.l
                    e.step()?;
                    let value = 0x40391a_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001226b8: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001226be: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1226c4_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 001226c4: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 001226c6: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001226c8: bne.w $122758
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122758_u32
                    } else {
                        0x1226cc_u32
                    };
                }
                0x1226cc => {
                    // 001226cc: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001226d2: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001226d8: pea.l $4038fa.l
                    e.step()?;
                    let value = 0x4038fa_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001226de: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1226e4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 001226e4: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1226ea_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 001226ea: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001226f0: pea.l -$104(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffefc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001226f4: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1226fa_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 001226fa: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122700_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00122700: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122706: pea.l -$108(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffef8_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012270a: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122710_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 00122710: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122716: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012271c: pea.l $403a4c.l
                    e.step()?;
                    let value = 0x403a4c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122722: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122728_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 00122728: moveq #$7f, d0
                    e.step()?;
                    r.put_data(0, 32, 0x7f_u32);
                    r.move_flags(0x7f_u32, 32);
                    // 0012272a: cmp.l -$104(a6), d0
                    e.step()?;
                    let source_address = r.address(6)?.wrapping_add(0xfffffefc_u32);
                    let source = e.l(source_address)?;
                    let destination = r.data(0, 32)?;
                    r.subtract_flags(destination, source, 32, false);
                    // 0012272e: lea.l $18(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x18_u32);
                    r.put_address(7, value);
                    // 00122732: bne.b $122738
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122738_u32
                    } else {
                        0x122734_u32
                    };
                }
                0x122734 => {
                    // 00122734: clr.l -$108(a6)
                    e.step()?;
                    let value = 0;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffef8_u32);
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    pc = 0x122738_u32;
                }
                0x122738 => {
                    // 00122738: lea.l $44258c.l, a0
                    e.step()?;
                    let value = 0x44258c_u32;
                    r.put_address(0, value);
                    // 0012273e: move.l -$104(a6), d0
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffefc_u32);
                    let value = e.l(value_address)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00122742: move.b -$105(a6), (a0, d0.l)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffefb_u32);
                    let value = u32::from(e.b(value_address)?);
                    let destination_address = r
                        .address(0)?
                        .wrapping_add(0x0_u32)
                        .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00122748: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12274e_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 0012274e: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122754_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 00122754: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122758 => {
                    // 00122758: pea.l $4038ec.l
                    e.step()?;
                    let value = 0x4038ec_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012275e: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122764: jsr $4000a8.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12276a_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x4000a8_u32))?;
                    // 0012276a: tst.w d0
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.move_flags(value, 16);
                    // 0012276c: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0012276e: bne.w $122880
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122880_u32
                    } else {
                        0x122772_u32
                    };
                }
                0x122772 => {
                    // 00122772: move.l #$ffff1770, -(a7)
                    e.step()?;
                    let value = 0xffff1770_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122778: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012277e: pea.l $4038de.l
                    e.step()?;
                    let value = 0x4038de_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122784: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12278a_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 0012278a: bsr.l $127132
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122790_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x127132_u32)?;
                    // 00122790: pea.l $438aa4.l
                    e.step()?;
                    let value = 0x438aa4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122796: pea.l -$104(a6)
                    e.step()?;
                    let value = r.address(6)?.wrapping_add(0xfffffefc_u32);
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 0012279a: jsr $400160.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1227a0_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400160_u32))?;
                    // 001227a0: move.l #$ffff0000, -(a7)
                    e.step()?;
                    let value = 0xffff0000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001227a6: move.l #$140028, -(a7)
                    e.step()?;
                    let value = 0x140028_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001227ac: pea.l $403a4c.l
                    e.step()?;
                    let value = 0x403a4c_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001227b2: bsr.l $123d1c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1227b8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x123d1c_u32)?;
                    // 001227b8: move.w -$102(a6), $4434b6.l
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffefe_u32);
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001227c0: pea.l $4038d6.l
                    e.step()?;
                    let value = 0x4038d6_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001227c6: bsr.l $136cfa
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1227cc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x136cfa_u32)?;
                    // 001227cc: pea.l $403a88.l
                    e.step()?;
                    let value = 0x403a88_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001227d2: pea.l $4389a4.l
                    e.step()?;
                    let value = 0x4389a4_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001227d8: jsr $400070.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1227de_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400070_u32))?;
                    // 001227de: movea.l a0, a3
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_address(3, value);
                    // 001227e0: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001227e2: move.w $4434b6.l, d0
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001227e8: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001227ec: move.l d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 32)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001227ee: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001227f0: jsr $400078.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1227f6_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400078_u32))?;
                    // 001227f6: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001227f8: pea.l $1.w
                    e.step()?;
                    let value = 0x1_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 001227fc: pea.l $120.w
                    e.step()?;
                    let value = 0x120_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122800: pea.l $440dbc.l
                    e.step()?;
                    let value = 0x440dbc_u32;
                    r.offset_address(7, -4)?;
                    e.long(r.address(7)?, value)?;
                    // 00122806: jsr $400048.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12280c_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400048_u32))?;
                    // 0012280c: move.l a3, -(a7)
                    e.step()?;
                    let value = r.address(3)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 0012280e: jsr $400068.l
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122814_u32)?;
                    imported(e, &mut r, NativeCall::Library(0x400068_u32))?;
                    // 00122814: addq.w #$1, $44257c.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x44257c_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    // 0012281a: moveq #$6, d0
                    e.step()?;
                    r.put_data(0, 32, 0x6_u32);
                    r.move_flags(0x6_u32, 32);
                    // 0012281c: muls.w $440dbc.l, d0
                    e.step()?;
                    let source_address = 0x440dbc_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = i32::from(source as u16 as i16)
                        .wrapping_mul(i32::from(destination as u16 as i16))
                        as u32;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00122822: add.w $440dbe.l, d0
                    e.step()?;
                    let source_address = 0x440dbe_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122828: move.w d0, $440ece.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x440ece_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0012282e: move.w d0, $440ecc.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x440ecc_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122834: move.w $440eca.l, $440e0e.l
                    e.step()?;
                    let value_address = 0x440eca_u32;
                    let value = u32::from(e.w(value_address)?);
                    let destination_address = 0x440e0e_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 0012283e: clr.w $440dca.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x440dca_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122844: lea.l $440dbc.l, a0
                    e.step()?;
                    let value = 0x440dbc_u32;
                    r.put_address(0, value);
                    // 0012284a: move.w $44257c.l, d0
                    e.step()?;
                    let value_address = 0x44257c_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122850: subq.w #$1, d0
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(0, 16)?;
                    let value = r.subtract_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 00122852: mulu.w #$120, d0
                    e.step()?;
                    let source = 0x120_u32;
                    let destination = r.data(0, 16)?;
                    let value = source.wrapping_mul(destination);
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 00122856: lea.l $440edc.l, a1
                    e.step()?;
                    let value = 0x440edc_u32;
                    r.put_address(1, value);
                    // 0012285c: adda.l d0, a1
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(1)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(1, value);
                    // 0012285e: moveq #$47, d0
                    e.step()?;
                    r.put_data(0, 32, 0x47_u32);
                    r.move_flags(0x47_u32, 32);
                    pc = 0x122860_u32;
                }
                0x122860 => {
                    // 00122860: move.l (a0)+, (a1)+
                    e.step()?;
                    let value_address = r.address(0)?;
                    let value = e.l(value_address)?;
                    r.offset_address(0, 4)?;
                    let destination_address = r.address(1)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.offset_address(1, 4)?;
                    r.move_flags(value, 32);
                    // 00122862: dbra d0, $122860
                    e.step()?;
                    let counter = r.data(0, 16)?;
                    let value = counter.wrapping_sub(1) & 0xffff;
                    r.put_data(0, 16, value);
                    pc = if value != 0xffff {
                        0x122860_u32
                    } else {
                        0x122866_u32
                    };
                }
                0x122866 => {
                    // 00122866: bsr.l $1046e0
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x12286c_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1046e0_u32)?;
                    // 0012286c: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122872_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 00122872: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122878_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 00122878: lea.l $42(a7), a7
                    e.step()?;
                    let value = r.address(7)?.wrapping_add(0x42_u32);
                    r.put_address(7, value);
                    // 0012287c: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122880 => {
                    // 00122880: tst.b d6
                    e.step()?;
                    let value = r.data(6, 8)?;
                    r.move_flags(value, 8);
                    // 00122882: beq.b $12288e
                    e.step()?;
                    pc = if r.equal()? {
                        0x12288e_u32
                    } else {
                        0x122884_u32
                    };
                }
                0x122884 => {
                    // 00122884: move.b $4433f7.l, d4
                    e.step()?;
                    let value_address = 0x4433f7_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(4, 8, value);
                    r.move_flags(value, 8);
                    // 0012288a: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x12288e => {
                    // 0012288e: moveq #$4, d4
                    e.step()?;
                    r.put_data(4, 32, 0x4_u32);
                    r.move_flags(0x4_u32, 32);
                    // 00122890: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122894 => {
                    // 00122894: cmpi.w #$2, $4434b2.l
                    e.step()?;
                    let source = 0x2_u32;
                    let destination_address = 0x4434b2_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0012289c: bne.w $12298a
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x12298a_u32
                    } else {
                        0x1228a0_u32
                    };
                }
                0x1228a0 => {
                    // 001228a0: tst.w $443400.l
                    e.step()?;
                    let value_address = 0x443400_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001228a6: beq.w $1218fc
                    e.step()?;
                    pc = if r.equal()? {
                        0x1218fc_u32
                    } else {
                        0x1228aa_u32
                    };
                }
                0x1228aa => {
                    // 001228aa: move.w #$1, $43877a.l
                    e.step()?;
                    let value = 0x1_u32;
                    let destination_address = 0x43877a_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001228b2: bsr.l $10b89e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1228b8_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10b89e_u32)?;
                    // 001228b8: move.w d0, d5
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.put_data(5, 16, value);
                    r.move_flags(value, 16);
                    // 001228ba: move.l #$1ffff, -(a7)
                    e.step()?;
                    let value = 0x1ffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001228c0: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1228c6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 001228c6: clr.w $438760.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x438760_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001228cc: cmpi.w #$44d, d5
                    e.step()?;
                    let source = 0x44d_u32;
                    let destination = r.data(5, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 001228d0: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001228d2: blt.w $1218fc
                    e.step()?;
                    pc = if r.less()? {
                        0x1218fc_u32
                    } else {
                        0x1228d6_u32
                    };
                }
                0x1228d6 => {
                    // 001228d6: bsr.l $16043e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1228dc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x16043e_u32)?;
                    // 001228dc: clr.w -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001228de: move.b $44273f.l, d0
                    e.step()?;
                    let value_address = 0x44273f_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 001228e4: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 001228e6: move.w d0, -(a7)
                    e.step()?;
                    let value = r.data(0, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001228e8: bsr.l $1600fc
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1228ee_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x1600fc_u32)?;
                    // 001228ee: bsr.l $13822a
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1228f4_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13822a_u32)?;
                    // 001228f4: move.w $443774.l, -(a7)
                    e.step()?;
                    let value_address = 0x443774_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001228fa: bsr.l $133a18
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122900_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x133a18_u32)?;
                    // 00122900: cmpi.b #$7, $44374f.l
                    e.step()?;
                    let source = 0x7_u32;
                    let destination_address = 0x44374f_u32;
                    let destination = u32::from(e.b(destination_address)?);
                    r.subtract_flags(destination, source, 8, false);
                    // 00122908: addq.w #$6, a7
                    e.step()?;
                    let source = 0x6_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0012290a: bne.b $122918
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122918_u32
                    } else {
                        0x12290c_u32
                    };
                }
                0x12290c => {
                    // 0012290c: bsr.l $151022
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122912_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x151022_u32)?;
                    // 00122912: bsr.l $15ec56
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122918_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x15ec56_u32)?;
                    pc = 0x122918_u32;
                }
                0x122918 => {
                    // 00122918: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 0012291a: bra.b $12297e
                    e.step()?;
                    pc = 0x12297e_u32;
                }
                0x12291c => {
                    // 0012291c: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 00122922: cmp.w $6a(a0, d3.w * 2), d5
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x6a_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(5, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00122926: bne.b $122936
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122936_u32
                    } else {
                        0x122928_u32
                    };
                }
                0x122928 => {
                    // 00122928: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 0012292e: move.b $7e(a0, d3.w), d4
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x7e_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(4, 8, value);
                    r.move_flags(value, 8);
                    // 00122932: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122936 => {
                    // 00122936: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 0012293c: tst.w $6a(a0, d3.w * 2)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x6a_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00122940: beq.b $12297c
                    e.step()?;
                    pc = if r.equal()? {
                        0x12297c_u32
                    } else {
                        0x122942_u32
                    };
                }
                0x122942 => {
                    // 00122942: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 00122948: cmpi.w #$7, $6a(a0, d3.w * 2)
                    e.step()?;
                    let source = 0x7_u32;
                    let destination_address = r.address(0)?.wrapping_add(0x6a_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 0012294e: bge.b $12297c
                    e.step()?;
                    pc = if r.greater_equal()? {
                        0x12297c_u32
                    } else {
                        0x122950_u32
                    };
                }
                0x122950 => {
                    // 00122950: move.w d5, -(a7)
                    e.step()?;
                    let value = r.data(5, 16)?;
                    r.offset_address(7, -2)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122952: bsr.l $13145c
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122958_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13145c_u32)?;
                    // 00122958: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 0012295e: move.b $443761.l, d0
                    e.step()?;
                    let value_address = 0x443761_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00122964: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122966: cmp.w $6a(a0, d3.w * 2), d0
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x6a_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 0012296a: addq.w #$2, a7
                    e.step()?;
                    let source = 0x2_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 0012296c: bne.b $12297c
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x12297c_u32
                    } else {
                        0x12296e_u32
                    };
                }
                0x12296e => {
                    // 0012296e: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 00122974: move.b $7e(a0, d3.w), d4
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x7e_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(4, 8, value);
                    r.move_flags(value, 8);
                    // 00122978: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x12297c => {
                    // 0012297c: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x12297e_u32;
                }
                0x12297e => {
                    // 0012297e: cmpi.w #$a, d3
                    e.step()?;
                    let source = 0xa_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00122982: blt.b $12291c
                    e.step()?;
                    pc = if r.less()? {
                        0x12291c_u32
                    } else {
                        0x122984_u32
                    };
                }
                0x122984 => {
                    // 00122984: moveq #$4, d4
                    e.step()?;
                    r.put_data(4, 32, 0x4_u32);
                    r.move_flags(0x4_u32, 32);
                    // 00122986: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x12298a => {
                    // 0012298a: cmpi.w #$5, $4434b2.l
                    e.step()?;
                    let source = 0x5_u32;
                    let destination_address = 0x4434b2_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00122992: bne.w $122a26
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122a26_u32
                    } else {
                        0x122996_u32
                    };
                }
                0x122996 => {
                    // 00122996: tst.w $44341e.l
                    e.step()?;
                    let value_address = 0x44341e_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 0012299c: beq.w $1218fc
                    e.step()?;
                    pc = if r.equal()? {
                        0x1218fc_u32
                    } else {
                        0x1229a0_u32
                    };
                }
                0x1229a0 => {
                    // 001229a0: bsr.l $13a498
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1229a6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13a498_u32)?;
                    // 001229a6: bsr.l $129516
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1229ac_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x129516_u32)?;
                    // 001229ac: move.w d0, $44719a.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x44719a_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001229b2: move.l #$ffff, -(a7)
                    e.step()?;
                    let value = 0xffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 001229b8: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x1229be_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 001229be: tst.w $44719a.l
                    e.step()?;
                    let value_address = 0x44719a_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 001229c4: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 001229c6: beq.w $1218fc
                    e.step()?;
                    pc = if r.equal()? {
                        0x1218fc_u32
                    } else {
                        0x1229ca_u32
                    };
                }
                0x1229ca => {
                    // 001229ca: cmpi.w #$17, $453656.l
                    e.step()?;
                    let source = 0x17_u32;
                    let destination_address = 0x453656_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 001229d2: bne.b $1229ea
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x1229ea_u32
                    } else {
                        0x1229d4_u32
                    };
                }
                0x1229d4 => {
                    // 001229d4: moveq #$64, d0
                    e.step()?;
                    r.put_data(0, 32, 0x64_u32);
                    r.move_flags(0x64_u32, 32);
                    // 001229d6: add.w $45366e.l, d0
                    e.step()?;
                    let source_address = 0x45366e_u32;
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001229dc: neg.w d0
                    e.step()?;
                    let source = r.data(0, 16)?;
                    let value = r.subtract_flags(0, source, 16, true);
                    r.put_data(0, 16, value);
                    // 001229de: move.w d0, $44719a.l
                    e.step()?;
                    let value = r.data(0, 16)?;
                    let destination_address = 0x44719a_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 001229e4: moveq #$ff, d4
                    e.step()?;
                    r.put_data(4, 32, 0xffffffff_u32);
                    r.move_flags(0xffffffff_u32, 32);
                    // 001229e6: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x1229ea => {
                    // 001229ea: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 001229ec: bra.b $122a1a
                    e.step()?;
                    pc = 0x122a1a_u32;
                }
                0x1229ee => {
                    // 001229ee: movea.w d3, a0
                    e.step()?;
                    let value = r.data(3, 16)?;
                    let value = i32::from(value as u16 as i16) as u32;
                    r.put_address(0, value);
                    // 001229f0: move.l a0, d0
                    e.step()?;
                    let value = r.address(0)?;
                    r.put_data(0, 32, value);
                    r.move_flags(value, 32);
                    // 001229f2: add.l d0, d0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.data(0, 32)?;
                    let value = r.add_flags(destination, source, 32, true);
                    r.put_data(0, 32, value);
                    // 001229f4: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 001229fa: adda.l d0, a0
                    e.step()?;
                    let source = r.data(0, 32)?;
                    let destination = r.address(0)?;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 001229fc: move.w $44719a.l, d0
                    e.step()?;
                    let value_address = 0x44719a_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122a02: cmp.w $88(a0), d0
                    e.step()?;
                    let source_address = r.address(0)?.wrapping_add(0x88_u32);
                    let source = u32::from(e.w(source_address)?);
                    let destination = r.data(0, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00122a06: bne.b $122a18
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122a18_u32
                    } else {
                        0x122a08_u32
                    };
                }
                0x122a08 => {
                    // 00122a08: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 00122a0e: adda.w d3, a0
                    e.step()?;
                    let source = r.data(3, 16)?;
                    let destination = r.address(0)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(0, value);
                    // 00122a10: move.b $92(a0), d4
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x92_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(4, 8, value);
                    r.move_flags(value, 8);
                    // 00122a14: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122a18 => {
                    // 00122a18: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x122a1a_u32;
                }
                0x122a1a => {
                    // 00122a1a: cmpi.w #$5, d3
                    e.step()?;
                    let source = 0x5_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00122a1e: blt.b $1229ee
                    e.step()?;
                    pc = if r.less()? {
                        0x1229ee_u32
                    } else {
                        0x122a20_u32
                    };
                }
                0x122a20 => {
                    // 00122a20: moveq #$4, d4
                    e.step()?;
                    r.put_data(4, 32, 0x4_u32);
                    r.move_flags(0x4_u32, 32);
                    // 00122a22: bra.w $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122a26 => {
                    // 00122a26: cmpi.w #$4, $4434b2.l
                    e.step()?;
                    let source = 0x4_u32;
                    let destination_address = 0x4434b2_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    r.subtract_flags(destination, source, 16, false);
                    // 00122a2e: bne.w $122adc
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122adc_u32
                    } else {
                        0x122a32_u32
                    };
                }
                0x122a32 => {
                    // 00122a32: tst.b $4433f6.l
                    e.step()?;
                    let value_address = 0x4433f6_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00122a38: beq.w $1218fc
                    e.step()?;
                    pc = if r.equal()? {
                        0x1218fc_u32
                    } else {
                        0x122a3c_u32
                    };
                }
                0x122a3c => {
                    // 00122a3c: bsr.l $100642
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122a42_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x100642_u32)?;
                    // 00122a42: move.b d0, -$10f(a6)
                    e.step()?;
                    let value = r.data(0, 8)?;
                    let destination_address = r.address(6)?.wrapping_add(0xfffffef1_u32);
                    r.write_memory(e, destination_address, 8, value)?;
                    r.move_flags(value, 8);
                    // 00122a46: move.l #$1ffff, -(a7)
                    e.step()?;
                    let value = 0x1ffff_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122a4c: bsr.l $13bb20
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122a52_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x13bb20_u32)?;
                    // 00122a52: tst.b -$10f(a6)
                    e.step()?;
                    let value_address = r.address(6)?.wrapping_add(0xfffffef1_u32);
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00122a56: addq.w #$4, a7
                    e.step()?;
                    let source = 0x4_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    // 00122a58: beq.w $1218fc
                    e.step()?;
                    pc = if r.equal()? {
                        0x1218fc_u32
                    } else {
                        0x122a5c_u32
                    };
                }
                0x122a5c => {
                    // 00122a5c: clr.w $4434b6.l
                    e.step()?;
                    let value = 0;
                    let destination_address = 0x4434b6_u32;
                    r.write_memory(e, destination_address, 16, value)?;
                    r.move_flags(value, 16);
                    // 00122a62: moveq #$1, d4
                    e.step()?;
                    r.put_data(4, 32, 0x1_u32);
                    r.move_flags(0x1_u32, 32);
                    // 00122a64: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00122a66: bra.b $122a7c
                    e.step()?;
                    pc = 0x122a7c_u32;
                }
                0x122a68 => {
                    // 00122a68: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 00122a6e: tst.b $62(a0, d3.w)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x62_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00122a72: beq.b $122a7a
                    e.step()?;
                    pc = if r.equal()? {
                        0x122a7a_u32
                    } else {
                        0x122a74_u32
                    };
                }
                0x122a74 => {
                    // 00122a74: addq.w #$1, $4434b6.l
                    e.step()?;
                    let source = 0x1_u32;
                    let destination_address = 0x4434b6_u32;
                    let destination = u32::from(e.w(destination_address)?);
                    let value = r.add_flags(destination, source, 16, true);
                    r.write_memory(e, destination_address, 16, value)?;
                    pc = 0x122a7a_u32;
                }
                0x122a7a => {
                    // 00122a7a: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x122a7c_u32;
                }
                0x122a7c => {
                    // 00122a7c: cmpi.w #$8, d3
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00122a80: blt.b $122a68
                    e.step()?;
                    pc = if r.less()? {
                        0x122a68_u32
                    } else {
                        0x122a82_u32
                    };
                }
                0x122a82 => {
                    // 00122a82: tst.w $4434b6.l
                    e.step()?;
                    let value_address = 0x4434b6_u32;
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00122a88: bne.b $122a8e
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122a8e_u32
                    } else {
                        0x122a8a_u32
                    };
                }
                0x122a8a => {
                    // 00122a8a: moveq #$4, d4
                    e.step()?;
                    r.put_data(4, 32, 0x4_u32);
                    r.move_flags(0x4_u32, 32);
                    // 00122a8c: bra.b $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122a8e => {
                    // 00122a8e: moveq #$0, d3
                    e.step()?;
                    r.put_data(3, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    // 00122a90: bra.b $122ac8
                    e.step()?;
                    pc = 0x122ac8_u32;
                }
                0x122a92 => {
                    // 00122a92: lea.l $438768.l, a0
                    e.step()?;
                    let value = 0x438768_u32;
                    r.put_address(0, value);
                    // 00122a98: tst.w (a0, d3.w * 2)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00122a9c: beq.b $122aac
                    e.step()?;
                    pc = if r.equal()? {
                        0x122aac_u32
                    } else {
                        0x122a9e_u32
                    };
                }
                0x122a9e => {
                    // 00122a9e: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 00122aa4: tst.b $62(a0, d3.w)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x62_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00122aa8: bne.b $122aac
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122aac_u32
                    } else {
                        0x122aaa_u32
                    };
                }
                0x122aaa => {
                    // 00122aaa: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x122aac_u32;
                }
                0x122aac => {
                    // 00122aac: lea.l $438768.l, a0
                    e.step()?;
                    let value = 0x438768_u32;
                    r.put_address(0, value);
                    // 00122ab2: tst.w (a0, d3.w * 2)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x0_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2),
                    );
                    let value = u32::from(e.w(value_address)?);
                    r.move_flags(value, 16);
                    // 00122ab6: bne.b $122ac6
                    e.step()?;
                    pc = if r.not_equal()? {
                        0x122ac6_u32
                    } else {
                        0x122ab8_u32
                    };
                }
                0x122ab8 => {
                    // 00122ab8: lea.l $443396.l, a0
                    e.step()?;
                    let value = 0x443396_u32;
                    r.put_address(0, value);
                    // 00122abe: tst.b $62(a0, d3.w)
                    e.step()?;
                    let value_address = r.address(0)?.wrapping_add(0x62_u32).wrapping_add(
                        (i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1),
                    );
                    let value = u32::from(e.b(value_address)?);
                    r.move_flags(value, 8);
                    // 00122ac2: beq.b $122ac6
                    e.step()?;
                    pc = if r.equal()? {
                        0x122ac6_u32
                    } else {
                        0x122ac4_u32
                    };
                }
                0x122ac4 => {
                    // 00122ac4: moveq #$0, d4
                    e.step()?;
                    r.put_data(4, 32, 0x0_u32);
                    r.move_flags(0x0_u32, 32);
                    pc = 0x122ac6_u32;
                }
                0x122ac6 => {
                    // 00122ac6: addq.w #$1, d3
                    e.step()?;
                    let source = 0x1_u32;
                    let destination = r.data(3, 16)?;
                    let value = r.add_flags(destination, source, 16, true);
                    r.put_data(3, 16, value);
                    pc = 0x122ac8_u32;
                }
                0x122ac8 => {
                    // 00122ac8: cmpi.w #$8, d3
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.data(3, 16)?;
                    r.subtract_flags(destination, source, 16, false);
                    // 00122acc: blt.b $122a92
                    e.step()?;
                    pc = if r.less()? {
                        0x122a92_u32
                    } else {
                        0x122ace_u32
                    };
                }
                0x122ace => {
                    // 00122ace: tst.b d4
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.move_flags(value, 8);
                    // 00122ad0: beq.b $122ada
                    e.step()?;
                    pc = if r.equal()? {
                        0x122ada_u32
                    } else {
                        0x122ad2_u32
                    };
                }
                0x122ad2 => {
                    // 00122ad2: move.b $4433f6.l, d4
                    e.step()?;
                    let value_address = 0x4433f6_u32;
                    let value = u32::from(e.b(value_address)?);
                    r.put_data(4, 8, value);
                    r.move_flags(value, 8);
                    // 00122ad8: bra.b $122adc
                    e.step()?;
                    pc = 0x122adc_u32;
                }
                0x122ada => {
                    // 00122ada: moveq #$4, d4
                    e.step()?;
                    r.put_data(4, 32, 0x4_u32);
                    r.move_flags(0x4_u32, 32);
                    pc = 0x122adc_u32;
                }
                0x122adc => {
                    // 00122adc: move.l $43952c.l, -(a7)
                    e.step()?;
                    let value_address = 0x43952c_u32;
                    let value = e.l(value_address)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122ae2: MacToolbox 0xaa1c
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xaa1c_u32 as u16))?;
                    // 00122ae4: move.l a2, -(a7)
                    e.step()?;
                    let value = r.address(2)?;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122ae6: MacToolbox 0xa983
                    e.step()?;
                    imported(e, &mut r, NativeCall::Toolbox(0xa983_u32 as u16))?;
                    // 00122ae8: clr.l -(a7)
                    e.step()?;
                    let value = 0;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122aea: move.l #$10000, -(a7)
                    e.step()?;
                    let value = 0x10000_u32;
                    r.offset_address(7, -4)?;
                    let destination_address = r.address(7)?;
                    r.write_memory(e, destination_address, 32, value)?;
                    r.move_flags(value, 32);
                    // 00122af0: bsr.l $160a8e
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122af6_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x160a8e_u32)?;
                    // 00122af6: bsr.l $10f186
                    e.step()?;
                    e.long(r.address(7)?.wrapping_sub(4), 0x122afc_u32)?;
                    e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
                    child(e, &mut r, 0x10f186_u32)?;
                    // 00122afc: move.b d4, d0
                    e.step()?;
                    let value = r.data(4, 8)?;
                    r.put_data(0, 8, value);
                    r.move_flags(value, 8);
                    // 00122afe: ext.w d0
                    e.step()?;
                    let source = r.data(0, 8)?;
                    let value = i32::from(source as u8 as i8) as u32;
                    r.put_data(0, 16, value);
                    r.move_flags(value, 16);
                    // 00122b00: addq.w #$8, a7
                    e.step()?;
                    let source = 0x8_u32;
                    let destination = r.address(7)?;
                    let source = i32::from(source as u16 as i16) as u32;
                    let value = destination.wrapping_add(source);
                    r.put_address(7, value);
                    pc = 0x122b02_u32;
                }
                0x122b02 => {
                    // 00122b02: movem.l (a7)+, d3-d7/a2-a3
                    e.step()?;
                    r.restore_register(e, 3)?;
                    r.restore_register(e, 4)?;
                    r.restore_register(e, 5)?;
                    r.restore_register(e, 6)?;
                    r.restore_register(e, 7)?;
                    r.restore_register(e, 10)?;
                    r.restore_register(e, 11)?;
                    // 00122b06: unlk a6
                    e.step()?;
                    r.put_address(7, r.address(6)?);
                    r.restore_register(e, 14)?;
                    // 00122b08: rts
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
crate::recovered_application_control::entry!(encounter, 0x12185a);
