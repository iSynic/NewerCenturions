//! Complete provisional gameplay control from canonical CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};

pub fn cast_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x10d4fe_u32;
        loop {
            match match pc {
                0x10d4fe => cast_body_block_0(e, &mut r, frame, pc)?,
                0x10d57a => cast_body_block_1(e, &mut r, frame, pc)?,
                0x10d5a0 => cast_body_block_2(e, &mut r, frame, pc)?,
                0x10d5b2 => cast_body_block_3(e, &mut r, frame, pc)?,
                0x10d5bc => cast_body_block_4(e, &mut r, frame, pc)?,
                0x10d5c6 => cast_body_block_5(e, &mut r, frame, pc)?,
                0x10d5ce => cast_body_block_6(e, &mut r, frame, pc)?,
                0x10d5d6 => cast_body_block_7(e, &mut r, frame, pc)?,
                0x10d5f4 => cast_body_block_8(e, &mut r, frame, pc)?,
                0x10d5fa => cast_body_block_9(e, &mut r, frame, pc)?,
                0x10d604 => cast_body_block_a(e, &mut r, frame, pc)?,
                0x10d60c => cast_body_block_b(e, &mut r, frame, pc)?,
                0x10d614 => cast_body_block_c(e, &mut r, frame, pc)?,
                0x10d628 => cast_body_block_d(e, &mut r, frame, pc)?,
                0x10d632 => cast_body_block_e(e, &mut r, frame, pc)?,
                0x10d654 => cast_body_block_f(e, &mut r, frame, pc)?,
                0x10d65c => cast_body_block_10(e, &mut r, frame, pc)?,
                0x10d68c => cast_body_block_11(e, &mut r, frame, pc)?,
                0x10d6d0 => cast_body_block_12(e, &mut r, frame, pc)?,
                0x10d6f0 => cast_body_block_13(e, &mut r, frame, pc)?,
                0x10d6f6 => cast_body_block_14(e, &mut r, frame, pc)?,
                0x10d718 => cast_body_block_15(e, &mut r, frame, pc)?,
                0x10d71e => cast_body_block_16(e, &mut r, frame, pc)?,
                0x10d738 => cast_body_block_17(e, &mut r, frame, pc)?,
                0x10d870 => cast_body_block_18(e, &mut r, frame, pc)?,
                0x10d886 => cast_body_block_19(e, &mut r, frame, pc)?,
                0x10d88c => cast_body_block_1a(e, &mut r, frame, pc)?,
                0x10d890 => cast_body_block_1b(e, &mut r, frame, pc)?,
                0x10d89c => cast_body_block_1c(e, &mut r, frame, pc)?,
                0x10d8a6 => cast_body_block_1d(e, &mut r, frame, pc)?,
                0x10d8ca => cast_body_block_1e(e, &mut r, frame, pc)?,
                0x10d8e0 => cast_body_block_1f(e, &mut r, frame, pc)?,
                0x10d922 => cast_body_block_20(e, &mut r, frame, pc)?,
                0x10d96a => cast_body_block_21(e, &mut r, frame, pc)?,
                0x10d976 => cast_body_block_22(e, &mut r, frame, pc)?,
                0x10d982 => cast_body_block_23(e, &mut r, frame, pc)?,
                0x10d9aa => cast_body_block_24(e, &mut r, frame, pc)?,
                0x10d9c0 => cast_body_block_25(e, &mut r, frame, pc)?,
                0x10da02 => cast_body_block_26(e, &mut r, frame, pc)?,
                0x10da74 => cast_body_block_27(e, &mut r, frame, pc)?,
                0x10da7e => cast_body_block_28(e, &mut r, frame, pc)?,
                0x10da84 => cast_body_block_29(e, &mut r, frame, pc)?,
                0x10da8e => cast_body_block_2a(e, &mut r, frame, pc)?,
                0x10da96 => cast_body_block_2b(e, &mut r, frame, pc)?,
                0x10dade => cast_body_block_2c(e, &mut r, frame, pc)?,
                0x10db8a => cast_body_block_2d(e, &mut r, frame, pc)?,
                0x10db8c => cast_body_block_2e(e, &mut r, frame, pc)?,
                0x10dc74 => cast_body_block_2f(e, &mut r, frame, pc)?,
                0x10dc96 => cast_body_block_30(e, &mut r, frame, pc)?,
                0x10dcb0 => cast_body_block_31(e, &mut r, frame, pc)?,
                0x10dcb8 => cast_body_block_32(e, &mut r, frame, pc)?,
                0x10dcbe => cast_body_block_33(e, &mut r, frame, pc)?,
                0x10dcc4 => cast_body_block_34(e, &mut r, frame, pc)?,
                0x10dcca => cast_body_block_35(e, &mut r, frame, pc)?,
                0x10dcd0 => cast_body_block_36(e, &mut r, frame, pc)?,
                0x10dcd4 => cast_body_block_37(e, &mut r, frame, pc)?,
                0x10dcfa => cast_body_block_38(e, &mut r, frame, pc)?,
                0x10dd0c => cast_body_block_39(e, &mut r, frame, pc)?,
                0x10dd12 => cast_body_block_3a(e, &mut r, frame, pc)?,
                0x10ddd4 => cast_body_block_3b(e, &mut r, frame, pc)?,
                0x10ddda => cast_body_block_3c(e, &mut r, frame, pc)?,
                0x10ddde => cast_body_block_3d(e, &mut r, frame, pc)?,
                0x10ddf0 => cast_body_block_3e(e, &mut r, frame, pc)?,
                0x10de66 => cast_body_block_3f(e, &mut r, frame, pc)?,
                0x10de6c => cast_body_block_40(e, &mut r, frame, pc)?,
                0x10de70 => cast_body_block_41(e, &mut r, frame, pc)?,
                0x10de84 => cast_body_block_42(e, &mut r, frame, pc)?,
                0x10df1a => cast_body_block_43(e, &mut r, frame, pc)?,
                0x10df22 => cast_body_block_44(e, &mut r, frame, pc)?,
                0x10df26 => cast_body_block_45(e, &mut r, frame, pc)?,
                0x10df50 => cast_body_block_46(e, &mut r, frame, pc)?,
                0x10df76 => cast_body_block_47(e, &mut r, frame, pc)?,
                0x10df7c => cast_body_block_48(e, &mut r, frame, pc)?,
                0x10df80 => cast_body_block_49(e, &mut r, frame, pc)?,
                0x10dfaa => cast_body_block_4a(e, &mut r, frame, pc)?,
                0x10dfd0 => cast_body_block_4b(e, &mut r, frame, pc)?,
                0x10dfd6 => cast_body_block_4c(e, &mut r, frame, pc)?,
                0x10dfda => cast_body_block_4d(e, &mut r, frame, pc)?,
                0x10e004 => cast_body_block_4e(e, &mut r, frame, pc)?,
                0x10e06a => cast_body_block_4f(e, &mut r, frame, pc)?,
                0x10e070 => cast_body_block_50(e, &mut r, frame, pc)?,
                0x10e074 => cast_body_block_51(e, &mut r, frame, pc)?,
                0x10e07a => cast_body_block_52(e, &mut r, frame, pc)?,
                0x10e0d8 => cast_body_block_53(e, &mut r, frame, pc)?,
                0x10e0f0 => cast_body_block_54(e, &mut r, frame, pc)?,
                0x10e0f8 => cast_body_block_55(e, &mut r, frame, pc)?,
                0x10e110 => cast_body_block_56(e, &mut r, frame, pc)?,
                0x10e132 => cast_body_block_57(e, &mut r, frame, pc)?,
                0x10e166 => cast_body_block_58(e, &mut r, frame, pc)?,
                0x10e198 => cast_body_block_59(e, &mut r, frame, pc)?,
                0x10e1a4 => cast_body_block_5a(e, &mut r, frame, pc)?,
                0x10e1c6 => cast_body_block_5b(e, &mut r, frame, pc)?,
                0x10e1da => cast_body_block_5c(e, &mut r, frame, pc)?,
                0x10e1e8 => cast_body_block_5d(e, &mut r, frame, pc)?,
                0x10e208 => cast_body_block_5e(e, &mut r, frame, pc)?,
                0x10e22a => cast_body_block_5f(e, &mut r, frame, pc)?,
                0x10e244 => cast_body_block_60(e, &mut r, frame, pc)?,
                0x10e266 => cast_body_block_61(e, &mut r, frame, pc)?,
                0x10e272 => cast_body_block_62(e, &mut r, frame, pc)?,
                0x10e346 => cast_body_block_63(e, &mut r, frame, pc)?,
                0x10e368 => cast_body_block_64(e, &mut r, frame, pc)?,
                0x10e37c => cast_body_block_65(e, &mut r, frame, pc)?,
                0x10e382 => cast_body_block_66(e, &mut r, frame, pc)?,
                0x10e3b4 => cast_body_block_67(e, &mut r, frame, pc)?,
                0x10e3ca => cast_body_block_68(e, &mut r, frame, pc)?,
                0x10e44c => cast_body_block_69(e, &mut r, frame, pc)?,
                0x10e47e => cast_body_block_6a(e, &mut r, frame, pc)?,
                0x10e4a0 => cast_body_block_6b(e, &mut r, frame, pc)?,
                0x10e4d2 => cast_body_block_6c(e, &mut r, frame, pc)?,
                0x10e4f0 => cast_body_block_6d(e, &mut r, frame, pc)?,
                0x10e522 => cast_body_block_6e(e, &mut r, frame, pc)?,
                0x10e534 => cast_body_block_6f(e, &mut r, frame, pc)?,
                0x10e556 => cast_body_block_70(e, &mut r, frame, pc)?,
                0x10e566 => cast_body_block_71(e, &mut r, frame, pc)?,
                0x10e592 => cast_body_block_72(e, &mut r, frame, pc)?,
                0x10e598 => cast_body_block_73(e, &mut r, frame, pc)?,
                0x10e59e => cast_body_block_74(e, &mut r, frame, pc)?,
                0x10e5a4 => cast_body_block_75(e, &mut r, frame, pc)?,
                0x10e5aa => cast_body_block_76(e, &mut r, frame, pc)?,
                0x10e5b0 => cast_body_block_77(e, &mut r, frame, pc)?,
                0x10e5b6 => cast_body_block_78(e, &mut r, frame, pc)?,
                0x10e5bc => cast_body_block_79(e, &mut r, frame, pc)?,
                0x10e5c8 => cast_body_block_7a(e, &mut r, frame, pc)?,
                0x10e5d6 => cast_body_block_7b(e, &mut r, frame, pc)?,
                0x10e5dc => cast_body_block_7c(e, &mut r, frame, pc)?,
                0x10e5e6 => cast_body_block_7d(e, &mut r, frame, pc)?,
                0x10e5fa => cast_body_block_7e(e, &mut r, frame, pc)?,
                0x10e61c => cast_body_block_7f(e, &mut r, frame, pc)?,
                0x10e626 => cast_body_block_80(e, &mut r, frame, pc)?,
                0x10e62c => cast_body_block_81(e, &mut r, frame, pc)?,
                0x10e642 => cast_body_block_82(e, &mut r, frame, pc)?,
                0x10e646 => cast_body_block_83(e, &mut r, frame, pc)?,
                0x10e652 => cast_body_block_84(e, &mut r, frame, pc)?,
                0x10e658 => cast_body_block_85(e, &mut r, frame, pc)?,
                0x10e65e => cast_body_block_86(e, &mut r, frame, pc)?,
                0x10e66c => cast_body_block_87(e, &mut r, frame, pc)?,
                _ => cast_body_block_88(e, &mut r, frame, pc)?,
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
crate::recovered_application_control::entry!(cast, 0x10d4fe);
mod cast_body_block_0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d4fe: link.w a6, #$ff88
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, -120)?;
        // 0010d502: movem.l d3-d7, -(a7)
        e.step()?;
        r.save_register(e, 7)?;
        r.save_register(e, 6)?;
        r.save_register(e, 5)?;
        r.save_register(e, 4)?;
        r.save_register(e, 3)?;
        // 0010d506: moveq #$1, d4
        e.step()?;
        r.put_data(4, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 0010d508: move.w #$80, d6
        e.step()?;
        let value = 0x80_u32;
        r.put_data(6, 16, value);
        r.move_flags(value, 16);
        // 0010d50c: clr.w -$2e(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffd2_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d510: clr.w -$30(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffd0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d514: move.w #$20, -$2a(a6)
        e.step()?;
        let value = 0x20_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffd6_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d51a: move.w #$20, -$2c(a6)
        e.step()?;
        let value = 0x20_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffd4_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d520: move.w #$14, -(a7)
        e.step()?;
        let value = 0x14_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d524: bsr.l $13154c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10d52a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13154c_u32)?;
        // 0010d52a: move.l $43952c.l, -(a7)
        e.step()?;
        let value_address = 0x43952c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010d530: MacToolbox 0xaa1c
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xaa1c_u32 as u16))?;
        // 0010d532: pea.l -$20(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d536: move.l #$210021, -(a7)
        e.step()?;
        let value = 0x210021_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010d53c: move.l #$11f011f, -(a7)
        e.step()?;
        let value = 0x11f011f_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010d542: MacToolbox 0xa8a7
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8a7_u32 as u16))?;
        // 0010d544: clr.w $4427a6.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4427a6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d54a: move.b #$1, $435882.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x435882_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0010d552: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010d554: move.b $b(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xb_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010d558: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d55a: bsr.l $13a2c4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10d560_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a2c4_u32)?;
        // 0010d560: move.l $43878c.l, -(a7)
        e.step()?;
        let value_address = 0x43878c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010d566: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 0010d568: pea.l $21.w
        e.step()?;
        let value = 0x21_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d56c: MacToolbox 0xa862
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa862_u32 as u16))?;
        // 0010d56e: cmpi.b #$9, $443761.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x443761_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010d576: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010d578: beq.b $10d5a0
        e.step()?;
        pc = if r.equal()? {
            0x10d5a0_u32
        } else {
            0x10d57a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_0_unit::cast_body_block_0;

mod cast_body_block_1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d57a: move.l $4436f6.l, -(a7)
        e.step()?;
        let value_address = 0x4436f6_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010d580: move.l $4436f2.l, -(a7)
        e.step()?;
        let value_address = 0x4436f2_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010d586: move.b $44375f.l, d0
        e.step()?;
        let value_address = 0x44375f_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010d58c: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d58e: lsl.w #$3, d0
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010d590: addi.w #$2ed0, d0
        e.step()?;
        let source = 0x2ed0_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010d594: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d596: bsr.l $14a02c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10d59c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x14a02c_u32)?;
        // 0010d59c: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        pc = 0x10d5a0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_1_unit::cast_body_block_1;

mod cast_body_block_2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d5a0: move.w $439568.l, d6
        e.step()?;
        let value_address = 0x439568_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(6, 16, value);
        r.move_flags(value, 16);
        // 0010d5a6: addi.w #$80, d6
        e.step()?;
        let source = 0x80_u32;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 0010d5aa: tst.w $439574.l
        e.step()?;
        let value_address = 0x439574_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010d5b0: beq.b $10d5d6
        e.step()?;
        pc = if r.equal()? {
            0x10d5d6_u32
        } else {
            0x10d5b2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_2_unit::cast_body_block_2;

mod cast_body_block_3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d5b2: cmpi.b #$6, $44374f.l
        e.step()?;
        let source = 0x6_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010d5ba: beq.b $10d5d6
        e.step()?;
        pc = if r.equal()? {
            0x10d5d6_u32
        } else {
            0x10d5bc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_3_unit::cast_body_block_3;

mod cast_body_block_4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d5bc: cmpi.b #$9, $443761.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x443761_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010d5c4: beq.b $10d5d6
        e.step()?;
        pc = if r.equal()? {
            0x10d5d6_u32
        } else {
            0x10d5c6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_4_unit::cast_body_block_4;

mod cast_body_block_5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d5c6: cmpi.w #$1, $8(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0x8_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010d5cc: ble.b $10d5d6
        e.step()?;
        pc = if r.less_equal()? {
            0x10d5d6_u32
        } else {
            0x10d5ce_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_5_unit::cast_body_block_5;

mod cast_body_block_6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d5ce: moveq #$1e, d6
        e.step()?;
        r.put_data(6, 32, 0x1e_u32);
        r.move_flags(0x1e_u32, 32);
        // 0010d5d0: add.w $439568.l, d6
        e.step()?;
        let source_address = 0x439568_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x10d5d6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_6_unit::cast_body_block_6;

mod cast_body_block_7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d5d6: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010d5d8: move.b $44376c.l, d0
        e.step()?;
        let value_address = 0x44376c_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010d5de: muls.w $4435cc.l, d0
        e.step()?;
        let source_address = 0x4435cc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d5e4: move.w d0, $4470b2.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4470b2_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d5ea: cmpi.b #$9, $44374f.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010d5f2: bne.b $10d5fa
        e.step()?;
        pc = if r.not_equal()? {
            0x10d5fa_u32
        } else {
            0x10d5f4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_7_unit::cast_body_block_7;

mod cast_body_block_8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d5f4: clr.w $4470b2.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4470b2_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10d5fa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_8_unit::cast_body_block_8;

mod cast_body_block_9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d5fa: cmpi.w #$1, $4435cc.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4435cc_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010d602: bge.b $10d60c
        e.step()?;
        pc = if r.greater_equal()? {
            0x10d60c_u32
        } else {
            0x10d604_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_9_unit::cast_body_block_9;
mod cast_body_block_a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d604: move.w #$1, $4435cc.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x4435cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10d60c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_a_unit::cast_body_block_a;

mod cast_body_block_b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d60c: tst.b $44374f.l
        e.step()?;
        let value_address = 0x44374f_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0010d612: bne.b $10d628
        e.step()?;
        pc = if r.not_equal()? {
            0x10d628_u32
        } else {
            0x10d614_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_b_unit::cast_body_block_b;

mod cast_body_block_c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d614: move.w $4470b2.l, d0
        e.step()?;
        let value_address = 0x4470b2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d61a: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d61c: divs.w $4435cc.l, d0
        e.step()?;
        let divisor_address = 0x4435cc_u32;
        let divisor = u32::from(e.w(divisor_address)?);
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 0010d622: move.w d0, $4470b2.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4470b2_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10d628_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_c_unit::cast_body_block_c;

mod cast_body_block_d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d628: clr.w $442bd4.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442bd4_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d62e: bra.w $10e65e
        e.step()?;
        pc = 0x10e65e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_d_unit::cast_body_block_d;

mod cast_body_block_e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d632: move.b #$1, $435661.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x435661_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0010d63a: clr.b -$51(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffaf_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0010d63e: moveq #$0, d7
        e.step()?;
        r.put_data(7, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010d640: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010d646: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 0010d648: pea.l $21.w
        e.step()?;
        let value = 0x21_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d64c: MacToolbox 0xa862
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa862_u32 as u16))?;
        // 0010d64e: pea.l $1e.w
        e.step()?;
        let value = 0x1e_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d652: MacToolbox 0xa863
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa863_u32 as u16))?;
        pc = 0x10d654_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_e_unit::cast_body_block_e;

mod cast_body_block_f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d654: cmpi.b #$9, $b(a6)
        e.step()?;
        let source = 0x9_u32;
        let destination_address = r.address(6)?.wrapping_add(0xb_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010d65a: bcc.b $10d68c
        e.step()?;
        pc = if r.carry_clear()? {
            0x10d68c_u32
        } else {
            0x10d65c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_f_unit::cast_body_block_f;

mod cast_body_block_10_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_10(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d65c: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 0010d662: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d668: move.w $2(a0, d0.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d66c: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010d66e: move.w d0, -$8(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d672: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 0010d678: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d67e: move.w (a0, d0.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d682: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010d684: move.w d0, -$6(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d688: bra.w $10d71e
        e.step()?;
        pc = 0x10d71e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_10_unit::cast_body_block_10;

mod cast_body_block_11_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_11(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d68c: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 0010d692: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d698: move.w $2(a0, d0.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d69c: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010d69e: move.w d0, -$8(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d6a2: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 0010d6a8: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d6ae: move.w (a0, d0.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d6b2: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010d6b4: move.w d0, -$6(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d6b8: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d6be: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d6c2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010d6c8: cmpi.w #$1, $10(a0, d0.l)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010d6ce: beq.b $10d6f0
        e.step()?;
        pc = if r.equal()? {
            0x10d6f0_u32
        } else {
            0x10d6d0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_11_unit::cast_body_block_11;

mod cast_body_block_12_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_12(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d6d0: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d6d6: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d6da: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010d6e0: move.w $10(a0, d0.l), -(a7)
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
        // 0010d6e4: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10d6ea_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0010d6ea: subq.w #$3, d0
        e.step()?;
        let source = 0x3_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010d6ec: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010d6ee: bne.b $10d6f6
        e.step()?;
        pc = if r.not_equal()? {
            0x10d6f6_u32
        } else {
            0x10d6f0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_12_unit::cast_body_block_12;

mod cast_body_block_13_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_13(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d6f0: subi.w #$10, -$8(a6)
        e.step()?;
        let source = 0x10_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x10d6f6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_13_unit::cast_body_block_13;

mod cast_body_block_14_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_14(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d6f6: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d6fc: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d700: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010d706: move.w $10(a0, d0.l), -(a7)
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
        // 0010d70a: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10d710_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0010d710: cmpi.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010d714: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010d716: ble.b $10d71e
        e.step()?;
        pc = if r.less_equal()? {
            0x10d71e_u32
        } else {
            0x10d718_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_14_unit::cast_body_block_14;

mod cast_body_block_15_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_15(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d718: subi.w #$10, -$6(a6)
        e.step()?;
        let source = 0x10_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x10d71e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_15_unit::cast_body_block_15;

mod cast_body_block_16_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_16(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d71e: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010d720: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010d724: move.w d0, -$4(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d728: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010d72a: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010d72e: move.w d0, -$2(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d732: tst.b d7
        e.step()?;
        let value = r.data(7, 8)?;
        r.move_flags(value, 8);
        // 0010d734: beq.w $10d870
        e.step()?;
        pc = if r.equal()? {
            0x10d870_u32
        } else {
            0x10d738_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_16_unit::cast_body_block_16;

mod cast_body_block_17_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_17(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d738: move.w d3, -$78(a6)
        e.step()?;
        let value = r.data(3, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff88_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d73c: pea.l -$78(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff88_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d740: pea.l -$78(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff88_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d744: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d748: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d74a: pea.l -$48(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffb8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d74e: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d752: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d756: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d758: pea.l -$78(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff88_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d75c: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d760: move.w #$4, -(a7)
        e.step()?;
        let value = 0x4_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d764: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d766: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d76a: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d76e: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d772: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d774: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d778: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d77c: move.w #$0, -(a7)
        e.step()?;
        let value = 0x0_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d780: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d782: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d786: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d78a: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d78c: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d790: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d794: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d798: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d79a: move.w -$62(a6), -$8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9e_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d7a0: move.w d3, -$64(a6)
        e.step()?;
        let value = r.data(3, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d7a4: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7a8: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7ac: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d7b0: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d7b2: pea.l -$44(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffbc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7b6: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7ba: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d7be: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d7c0: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7c4: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7c8: move.w #$4, -(a7)
        e.step()?;
        let value = 0x4_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d7cc: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d7ce: pea.l -$6(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7d2: pea.l -$78(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff88_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7d6: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d7da: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d7dc: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7e0: pea.l -$78(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff88_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7e4: move.w #$0, -(a7)
        e.step()?;
        let value = 0x0_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d7e8: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d7ea: pea.l -$78(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff88_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7ee: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d7f2: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d7f4: pea.l -$78(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff88_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7f8: pea.l -$78(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff88_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d7fc: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d800: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d802: move.w -$76(a6), -$6(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff8a_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d808: move.w -$6(a6), -$64(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d80e: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d812: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d816: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d81a: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d81c: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d820: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d824: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d828: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d82a: move.l -$64(a6), -$34(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffcc_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010d830: move.w -$8(a6), -$64(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d836: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d83a: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d83e: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d842: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d844: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d848: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010d84c: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d850: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010d852: move.l -$64(a6), -$38(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffc8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010d858: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010d85a: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010d85e: move.w d0, -$4(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d862: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010d864: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010d868: move.w d0, -$2(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d86c: bra.w $10e3b4
        e.step()?;
        pc = 0x10e3b4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_17_unit::cast_body_block_17;

mod cast_body_block_18_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_18(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d870: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010d872: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d878: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010d87e: move.w $4(a0, d0.l), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x4_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d882: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010d884: beq.b $10d890
        e.step()?;
        pc = if r.equal()? {
            0x10d890_u32
        } else {
            0x10d886_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_18_unit::cast_body_block_18;

mod cast_body_block_19_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_19(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d886: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010d888: beq.w $10d96a
        e.step()?;
        pc = if r.equal()? {
            0x10d96a_u32
        } else {
            0x10d88c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_19_unit::cast_body_block_19;
mod cast_body_block_1a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_1a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d88c: bra.w $10da96
        e.step()?;
        pc = 0x10da96_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_1a_unit::cast_body_block_1a;

mod cast_body_block_1b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_1b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d890: cmpi.b #$9, $443761.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x443761_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010d898: bne.w $10d922
        e.step()?;
        pc = if r.not_equal()? {
            0x10d922_u32
        } else {
            0x10d89c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_1b_unit::cast_body_block_1b;

mod cast_body_block_1c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_1c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d89c: cmpi.b #$2, $44374f.l
        e.step()?;
        let source = 0x2_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010d8a4: bge.b $10d922
        e.step()?;
        pc = if r.greater_equal()? {
            0x10d922_u32
        } else {
            0x10d8a6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_1c_unit::cast_body_block_1c;

mod cast_body_block_1d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_1d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d8a6: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010d8a8: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d8ae: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010d8b4: move.w (a0, d0.l), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d8b8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d8bc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010d8c2: cmpi.w #$1, $50(a0, d0.l)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010d8c8: bge.b $10d922
        e.step()?;
        pc = if r.greater_equal()? {
            0x10d922_u32
        } else {
            0x10d8ca_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_1d_unit::cast_body_block_1d;

mod cast_body_block_1e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_1e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d8ca: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 0010d8d0: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d8d6: cmpi.w #$9, (a0, d0.w * 2)
        e.step()?;
        let source = 0x9_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010d8dc: bge.w $10e642
        e.step()?;
        pc = if r.greater_equal()? {
            0x10e642_u32
        } else {
            0x10d8e0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_1e_unit::cast_body_block_1e;

mod cast_body_block_1f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_1f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d8e0: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d8e6: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d8ea: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010d8f0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010d8f2: move.w $45377e.l, d0
        e.step()?;
        let value_address = 0x45377e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d8f8: add.w d0, $134(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x134_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010d8fc: move.w $4536f6.l, d0
        e.step()?;
        let value_address = 0x4536f6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d902: muls.w $45377e.l, d0
        e.step()?;
        let source_address = 0x45377e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d908: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010d90e: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010d912: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010d918: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010d91a: add.w d0, $9d6(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x9d6_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010d91e: bra.w $10e642
        e.step()?;
        pc = 0x10e642_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_1f_unit::cast_body_block_1f;

mod cast_body_block_20_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_20(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d922: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010d924: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d92a: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010d930: lea.l $44709a.l, a1
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(1, value);
        // 0010d936: move.w (a0, d0.l), d1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010d93a: move.w $2(a1, d1.w * 4), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010d93e: lsl.w #$5, d1
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(1, 16, value);
        // 0010d940: move.w d1, -$10(a6)
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d944: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010d946: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d94c: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010d952: lea.l $44709a.l, a1
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(1, value);
        // 0010d958: move.w (a0, d0.l), d1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010d95c: move.w (a1, d1.w * 4), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010d960: lsl.w #$5, d1
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(1, 16, value);
        // 0010d962: move.w d1, -$e(a6)
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010d966: bra.w $10dade
        e.step()?;
        pc = 0x10dade_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_20_unit::cast_body_block_20;

mod cast_body_block_21_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_21(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d96a: cmpi.b #$9, $443761.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x443761_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010d972: bne.w $10da02
        e.step()?;
        pc = if r.not_equal()? {
            0x10da02_u32
        } else {
            0x10d976_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_21_unit::cast_body_block_21;

mod cast_body_block_22_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_22(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d976: cmpi.b #$2, $44374f.l
        e.step()?;
        let source = 0x2_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010d97e: bge.w $10da02
        e.step()?;
        pc = if r.greater_equal()? {
            0x10da02_u32
        } else {
            0x10d982_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_22_unit::cast_body_block_22;

mod cast_body_block_23_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_23(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d982: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010d984: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d98a: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010d990: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010d992: add.w (a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010d996: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010d99a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010d9a0: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010d9a2: cmpi.w #$1, $110(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x110_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010d9a8: bge.b $10da02
        e.step()?;
        pc = if r.greater_equal()? {
            0x10da02_u32
        } else {
            0x10d9aa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_23_unit::cast_body_block_23;

mod cast_body_block_24_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_24(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d9aa: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 0010d9b0: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d9b6: cmpi.w #$9, (a0, d0.w * 2)
        e.step()?;
        let source = 0x9_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010d9bc: bge.w $10e642
        e.step()?;
        pc = if r.greater_equal()? {
            0x10e642_u32
        } else {
            0x10d9c0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_24_unit::cast_body_block_24;

mod cast_body_block_25_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_25(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010d9c0: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d9c6: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d9ca: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010d9d0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010d9d2: move.w $45377e.l, d0
        e.step()?;
        let value_address = 0x45377e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d9d8: add.w d0, $134(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x134_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010d9dc: move.w $4536f6.l, d0
        e.step()?;
        let value_address = 0x4536f6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010d9e2: muls.w $45377e.l, d0
        e.step()?;
        let source_address = 0x45377e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010d9e8: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010d9ee: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010d9f2: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010d9f8: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010d9fa: add.w d0, $9d6(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x9d6_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010d9fe: bra.w $10e642
        e.step()?;
        pc = 0x10e642_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_25_unit::cast_body_block_25;

mod cast_body_block_26_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_26(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010da02: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010da04: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010da0a: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010da10: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010da12: add.w (a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010da16: move.w d1, $4434b6.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010da1c: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 0010da22: move.w $4434b6.l, d0
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010da28: move.w $2(a0, d0.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010da2c: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010da2e: move.w d0, -$10(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010da32: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 0010da38: move.w $4434b6.l, d0
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010da3e: move.w (a0, d0.w * 4), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010da42: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010da44: move.w d0, -$e(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010da48: move.w $4434b6.l, d0
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010da4e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010da52: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010da58: move.w $10(a0, d0.l), -(a7)
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
        // 0010da5c: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10da62_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0010da62: move.w d0, $4434b6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010da68: cmpi.w #$1, $4434b6.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4434b6_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010da70: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010da72: beq.b $10da7e
        e.step()?;
        pc = if r.equal()? {
            0x10da7e_u32
        } else {
            0x10da74_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_26_unit::cast_body_block_26;

mod cast_body_block_27_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_27(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010da74: cmpi.w #$3, $4434b6.l
        e.step()?;
        let source = 0x3_u32;
        let destination_address = 0x4434b6_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010da7c: bne.b $10da84
        e.step()?;
        pc = if r.not_equal()? {
            0x10da84_u32
        } else {
            0x10da7e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_27_unit::cast_body_block_27;

mod cast_body_block_28_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_28(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010da7e: subi.w #$10, -$10(a6)
        e.step()?;
        let source = 0x10_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x10da84_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_28_unit::cast_body_block_28;

mod cast_body_block_29_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_29(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010da84: cmpi.w #$1, $4434b6.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4434b6_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010da8c: ble.b $10dade
        e.step()?;
        pc = if r.less_equal()? {
            0x10dade_u32
        } else {
            0x10da8e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_29_unit::cast_body_block_29;
mod cast_body_block_2a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_2a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010da8e: subi.w #$10, -$e(a6)
        e.step()?;
        let source = 0x10_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010da94: bra.b $10dade
        e.step()?;
        pc = 0x10dade_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_2a_unit::cast_body_block_2a;

mod cast_body_block_2b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_2b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010da96: move.l $442784.l, d0
        e.step()?;
        let value_address = 0x442784_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010da9c: lsl.l #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 0010da9e: moveq #$6, d1
        e.step()?;
        r.put_data(1, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010daa0: mulu.w $442bd4.l, d1
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010daa6: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010daac: move.w $2(a0, d1.l), d1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010dab0: lsl.w #$5, d1
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(1, 16, value);
        // 0010dab2: ext.l d1
        e.step()?;
        let source = r.data(1, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010dab4: sub.l d0, d1
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.data(1, 32)?;
        let value = r.subtract_flags(destination, source, 32, true);
        r.put_data(1, 32, value);
        // 0010dab6: move.w d1, -$10(a6)
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010daba: move.l $442780.l, d0
        e.step()?;
        let value_address = 0x442780_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010dac0: lsl.l #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 0010dac2: moveq #$6, d1
        e.step()?;
        r.put_data(1, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010dac4: mulu.w $442bd4.l, d1
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010daca: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010dad0: move.w (a0, d1.l), d1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010dad4: lsl.w #$5, d1
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(1, 16, value);
        // 0010dad6: ext.l d1
        e.step()?;
        let source = r.data(1, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010dad8: sub.l d0, d1
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.data(1, 32)?;
        let value = r.subtract_flags(destination, source, 32, true);
        r.put_data(1, 32, value);
        // 0010dada: move.w d1, -$e(a6)
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10dade_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_2b_unit::cast_body_block_2b;

mod cast_body_block_2c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_2c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dade: move.w -$e(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010dae2: asr.w #$4, d0
        e.step()?;
        let count = 0x4_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(0, 16, value);
        // 0010dae4: lsr.w #$8, d0
        e.step()?;
        let count = 0x8_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(0, 16, value);
        // 0010dae6: lsr.w #$3, d0
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(0, 16, value);
        // 0010dae8: add.w -$e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010daec: asr.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(0, 16, value);
        // 0010daee: move.w d0, -$54(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffac_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010daf2: move.w -$10(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010daf6: asr.w #$4, d0
        e.step()?;
        let count = 0x4_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(0, 16, value);
        // 0010daf8: lsr.w #$8, d0
        e.step()?;
        let count = 0x8_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(0, 16, value);
        // 0010dafa: lsr.w #$3, d0
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(0, 16, value);
        // 0010dafc: add.w -$10(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010db00: asr.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(0, 16, value);
        // 0010db02: move.w d0, -$56(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffaa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db06: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010db08: add.w -$10(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010db0c: move.w d0, -$c(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db10: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010db12: add.w -$e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010db16: move.w d0, -$a(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db1a: move.w $443772.l, -(a7)
        e.step()?;
        let value_address = 0x443772_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db20: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10db26_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 0010db26: move.w -$e(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010db2a: sub.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010db2e: move.w d0, -$64(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db32: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010db36: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010db3a: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db3e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010db40: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010db44: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010db48: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db4c: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010db4e: move.l -$64(a6), -$4c(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffb4_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010db54: move.w -$10(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010db58: sub.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010db5c: move.w d0, -$64(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db60: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010db64: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010db68: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db6c: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010db6e: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010db72: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010db76: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db7a: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010db7c: move.l -$64(a6), -$50(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffb0_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010db82: cmpi.w #$1, d6
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(6, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010db86: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010db88: bge.b $10db8c
        e.step()?;
        pc = if r.greater_equal()? {
            0x10db8c_u32
        } else {
            0x10db8a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_2c_unit::cast_body_block_2c;

mod cast_body_block_2d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_2d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010db8a: moveq #$1, d6
        e.step()?;
        r.put_data(6, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        pc = 0x10db8c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_2d_unit::cast_body_block_2d;

mod cast_body_block_2e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_2e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010db8c: move.w d6, -$64(a6)
        e.step()?;
        let value = r.data(6, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db90: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010db94: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010db98: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010db9c: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010db9e: pea.l -$4c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffb4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dba2: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dba6: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dbaa: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dbac: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dbb0: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dbb4: move.w #$6, -(a7)
        e.step()?;
        let value = 0x6_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dbb8: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dbba: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dbbe: pea.l -$5a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dbc2: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dbc6: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dbc8: move.l -$5a(a6), -$44(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffa6_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffbc_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dbce: move.w d6, -$64(a6)
        e.step()?;
        let value = r.data(6, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dbd2: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dbd6: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dbda: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dbde: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dbe0: pea.l -$50(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffb0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dbe4: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dbe8: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dbec: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dbee: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dbf2: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dbf6: move.w #$6, -(a7)
        e.step()?;
        let value = 0x6_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dbfa: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dbfc: pea.l -$6e(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff92_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc00: pea.l -$5a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc04: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dc08: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dc0a: move.l -$5a(a6), -$48(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffa6_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffb8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dc10: move.w -$6(a6), -$64(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dc16: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc1a: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc1e: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dc22: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dc24: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc28: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc2c: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dc30: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dc32: move.l -$64(a6), -$34(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffcc_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dc38: move.w -$8(a6), -$64(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dc3e: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc42: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc46: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dc4a: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dc4c: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc50: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc54: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dc58: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dc5a: move.l -$64(a6), -$38(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9c_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffc8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dc60: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0010dc62: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc66: pea.l -$20(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc6a: pea.l -$28(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc6e: MacToolbox 0xa8aa
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8aa_u32 as u16))?;
        // 0010dc70: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010dc72: beq.b $10dc96
        e.step()?;
        pc = if r.equal()? {
            0x10dc96_u32
        } else {
            0x10dc74_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_2e_unit::cast_body_block_2e;

mod cast_body_block_2f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_2f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dc74: movea.l $3fe302.l, a0
        e.step()?;
        let value_address = 0x3fe302_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010dc7a: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc7e: movea.l $4386c0.l, a0
        e.step()?;
        let value_address = 0x4386c0_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010dc84: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc88: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc8c: pea.l -$30(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dc90: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dc92: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dc94: MacToolbox 0xa8ec
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
        pc = 0x10dc96_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_2f_unit::cast_body_block_2f;

mod cast_body_block_30_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_30(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dc96: move.l -$8(a6), -$18(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dc9c: move.l -$4(a6), -$14(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dca2: move.b $44375f.l, d0
        e.step()?;
        let value_address = 0x44375f_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010dca8: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010dcaa: subi.w #$d, d0
        e.step()?;
        let source = 0xd_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010dcae: beq.b $10dcd4
        e.step()?;
        pc = if r.equal()? {
            0x10dcd4_u32
        } else {
            0x10dcb0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_30_unit::cast_body_block_30;

mod cast_body_block_31_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_31(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dcb0: subi.w #$a, d0
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010dcb4: beq.w $10ddde
        e.step()?;
        pc = if r.equal()? {
            0x10ddde_u32
        } else {
            0x10dcb8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_31_unit::cast_body_block_31;

mod cast_body_block_32_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_32(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dcb8: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010dcba: beq.w $10de70
        e.step()?;
        pc = if r.equal()? {
            0x10de70_u32
        } else {
            0x10dcbe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_32_unit::cast_body_block_32;

mod cast_body_block_33_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_33(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dcbe: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010dcc0: beq.w $10df26
        e.step()?;
        pc = if r.equal()? {
            0x10df26_u32
        } else {
            0x10dcc4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_33_unit::cast_body_block_33;

mod cast_body_block_34_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_34(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dcc4: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010dcc6: beq.w $10df80
        e.step()?;
        pc = if r.equal()? {
            0x10df80_u32
        } else {
            0x10dcca_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_34_unit::cast_body_block_34;

mod cast_body_block_35_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_35(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dcca: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010dccc: beq.w $10dfda
        e.step()?;
        pc = if r.equal()? {
            0x10dfda_u32
        } else {
            0x10dcd0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_35_unit::cast_body_block_35;

mod cast_body_block_36_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_36(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dcd0: bra.w $10e074
        e.step()?;
        pc = 0x10e074_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_36_unit::cast_body_block_36;

mod cast_body_block_37_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_37(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dcd4: move.l -$10(a6), -$18(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dcda: move.l -$c(a6), -$14(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dce0: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dce4: move.l #$ffe0ffe0, -(a7)
        e.step()?;
        let value = 0xffe0ffe0_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dcea: MacToolbox 0xa8a9
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
        // 0010dcec: move.w #$ffff, $4434b6.l
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dcf4: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010dcf6: bra.w $10ddd4
        e.step()?;
        pc = 0x10ddd4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_37_unit::cast_body_block_37;

mod cast_body_block_38_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_38(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dcfa: move.w $4434b6.l, d0
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010dd00: addq.w #$1, $4434b6.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4434b6_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010dd06: cmpi.w #$6, d0
        e.step()?;
        let source = 0x6_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010dd0a: ble.b $10dd12
        e.step()?;
        pc = if r.less_equal()? {
            0x10dd12_u32
        } else {
            0x10dd0c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_38_unit::cast_body_block_38;

mod cast_body_block_39_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_39(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dd0c: clr.w $4434b6.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10dd12_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_39_unit::cast_body_block_39;
mod cast_body_block_3a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_3a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dd12: pea.l -$38(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd16: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd1a: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd1e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dd20: pea.l -$48(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffb8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd24: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd28: move.w #$1000, -(a7)
        e.step()?;
        let value = 0x1000_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd2c: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dd2e: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd32: pea.l -$38(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd36: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd3a: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dd3c: pea.l -$34(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffcc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd40: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd44: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd48: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dd4a: pea.l -$44(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffbc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd4e: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd52: move.w #$1000, -(a7)
        e.step()?;
        let value = 0x1000_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd56: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dd58: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd5c: pea.l -$34(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffcc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd60: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd64: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dd66: pea.l -$38(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd6a: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd6e: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd72: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dd74: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd78: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd7c: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dd7e: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd82: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd86: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd8a: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dd8c: move.w -$62(a6), -$8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9e_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd92: pea.l -$34(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffcc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd96: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dd9a: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dd9e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010dda0: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dda4: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dda8: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010ddaa: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010ddae: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010ddb2: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010ddb6: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010ddb8: move.w -$62(a6), -$6(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9e_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010ddbe: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010ddc0: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010ddc4: move.w d0, -$2(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010ddc8: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010ddca: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010ddce: move.w d0, -$4(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010ddd2: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x10ddd4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_3a_unit::cast_body_block_3a;

mod cast_body_block_3b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_3b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010ddd4: cmp.w d6, d3
        e.step()?;
        let source = r.data(6, 16)?;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010ddd6: blt.w $10dcfa
        e.step()?;
        pc = if r.less()? {
            0x10dcfa_u32
        } else {
            0x10ddda_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_3b_unit::cast_body_block_3b;

mod cast_body_block_3c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_3c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010ddda: bra.w $10e5dc
        e.step()?;
        pc = 0x10e5dc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_3c_unit::cast_body_block_3c;

mod cast_body_block_3d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_3d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010ddde: move.w #$2, -(a7)
        e.step()?;
        let value = 0x2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dde2: MacToolbox 0xa89c
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa89c_u32 as u16))?;
        // 0010dde4: move.w #$ffff, $4434b6.l
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010ddec: moveq #$fc, d4
        e.step()?;
        r.put_data(4, 32, 0xfffffffc_u32);
        r.move_flags(0xfffffffc_u32, 32);
        // 0010ddee: bra.b $10de66
        e.step()?;
        pc = 0x10de66_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_3d_unit::cast_body_block_3d;

mod cast_body_block_3e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_3e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010ddf0: addq.w #$1, $4434b6.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4434b6_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010ddf6: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010ddf8: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10ddfe_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0010ddfe: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010de00: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de02: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de04: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10de0a_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0010de0a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010de0c: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de0e: MacToolbox 0xa89b
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa89b_u32 as u16))?;
        // 0010de10: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010de12: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010de16: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de18: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010de1a: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010de1e: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de20: MacToolbox 0xa893
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa893_u32 as u16))?;
        // 0010de22: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010de24: add.w -$e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010de28: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de2a: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010de2c: add.w -$10(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010de30: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de32: MacToolbox 0xa891
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa891_u32 as u16))?;
        // 0010de34: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010de36: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010de3a: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de3c: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010de3e: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010de42: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de44: MacToolbox 0xa893
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa893_u32 as u16))?;
        // 0010de46: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de4a: bsr.l $1205f8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10de50_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1205f8_u32)?;
        // 0010de50: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010de52: add.w -$e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010de56: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de58: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010de5a: add.w -$10(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010de5e: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de60: MacToolbox 0xa891
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa891_u32 as u16))?;
        // 0010de62: addq.w #$1, d4
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(4, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(4, 16, value);
        // 0010de64: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10de66_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_3e_unit::cast_body_block_3e;

mod cast_body_block_3f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_3f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010de66: cmpi.w #$4, d4
        e.step()?;
        let source = 0x4_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010de6a: blt.b $10ddf0
        e.step()?;
        pc = if r.less()? {
            0x10ddf0_u32
        } else {
            0x10de6c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_3f_unit::cast_body_block_3f;

mod cast_body_block_40_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_40(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010de6c: bra.w $10e5dc
        e.step()?;
        pc = 0x10e5dc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_40_unit::cast_body_block_40;

mod cast_body_block_41_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_41(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010de70: move.w #$2, -(a7)
        e.step()?;
        let value = 0x2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de74: MacToolbox 0xa89c
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa89c_u32 as u16))?;
        // 0010de76: move.w #$ffff, $4434b6.l
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de7e: moveq #$fc, d4
        e.step()?;
        r.put_data(4, 32, 0xfffffffc_u32);
        r.move_flags(0xfffffffc_u32, 32);
        // 0010de80: bra.w $10df1a
        e.step()?;
        pc = 0x10df1a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_41_unit::cast_body_block_41;

mod cast_body_block_42_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_42(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010de84: addq.w #$1, $4434b6.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4434b6_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010de8a: move.w #$a, -(a7)
        e.step()?;
        let value = 0xa_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de8e: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10de94_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 0010de94: subq.w #$5, d0
        e.step()?;
        let source = 0x5_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010de96: add.w d0, -$e(a6)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010de9a: move.w #$a, -(a7)
        e.step()?;
        let value = 0xa_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010de9e: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10dea4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 0010dea4: subq.w #$5, d0
        e.step()?;
        let source = 0x5_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010dea6: add.w d0, -$10(a6)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010deaa: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010deac: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10deb2_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0010deb2: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010deb4: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010deb6: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010deb8: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10debe_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0010debe: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010dec0: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dec2: MacToolbox 0xa89b
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa89b_u32 as u16))?;
        // 0010dec4: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010dec6: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010deca: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010decc: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010dece: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010ded2: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010ded4: MacToolbox 0xa893
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa893_u32 as u16))?;
        // 0010ded6: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010ded8: add.w -$e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010dedc: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dede: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010dee0: add.w -$10(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010dee4: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dee6: MacToolbox 0xa891
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa891_u32 as u16))?;
        // 0010dee8: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010deea: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010deee: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010def0: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010def2: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010def6: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010def8: MacToolbox 0xa893
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa893_u32 as u16))?;
        // 0010defa: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010defe: bsr.l $1205f8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10df04_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1205f8_u32)?;
        // 0010df04: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010df06: add.w -$e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010df0a: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010df0c: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010df0e: add.w -$10(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010df12: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010df14: MacToolbox 0xa891
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa891_u32 as u16))?;
        // 0010df16: addq.w #$1, d4
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(4, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(4, 16, value);
        // 0010df18: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10df1a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_42_unit::cast_body_block_42;

mod cast_body_block_43_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_43(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010df1a: cmpi.w #$4, d4
        e.step()?;
        let source = 0x4_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010df1e: blt.w $10de84
        e.step()?;
        pc = if r.less()? {
            0x10de84_u32
        } else {
            0x10df22_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_43_unit::cast_body_block_43;

mod cast_body_block_44_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_44(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010df22: bra.w $10e5dc
        e.step()?;
        pc = 0x10e5dc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_44_unit::cast_body_block_44;

mod cast_body_block_45_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_45(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010df26: move.w #$2, -(a7)
        e.step()?;
        let value = 0x2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010df2a: MacToolbox 0xa89c
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa89c_u32 as u16))?;
        // 0010df2c: move.l #$40004, -(a7)
        e.step()?;
        let value = 0x40004_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010df32: MacToolbox 0xa89b
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa89b_u32 as u16))?;
        // 0010df34: move.l -$10(a6), -$18(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010df3a: move.l -$c(a6), -$14(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010df40: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010df44: move.l #$c000c, -(a7)
        e.step()?;
        let value = 0xc000c_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010df4a: MacToolbox 0xa8a9
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
        // 0010df4c: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010df4e: bra.b $10df76
        e.step()?;
        pc = 0x10df76_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_45_unit::cast_body_block_45;

mod cast_body_block_46_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_46(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010df50: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010df54: MacToolbox 0xa8b7
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8b7_u32 as u16))?;
        // 0010df56: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010df5a: bsr.l $1205f8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10df60_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1205f8_u32)?;
        // 0010df60: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010df64: MacToolbox 0xa8b7
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8b7_u32 as u16))?;
        // 0010df66: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010df6a: move.l #$fffcfffc, -(a7)
        e.step()?;
        let value = 0xfffcfffc_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010df70: MacToolbox 0xa8a9
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
        // 0010df72: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        // 0010df74: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10df76_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_46_unit::cast_body_block_46;

mod cast_body_block_47_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_47(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010df76: cmpi.w #$8, d3
        e.step()?;
        let source = 0x8_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010df7a: blt.b $10df50
        e.step()?;
        pc = if r.less()? {
            0x10df50_u32
        } else {
            0x10df7c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_47_unit::cast_body_block_47;

mod cast_body_block_48_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_48(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010df7c: bra.w $10e5dc
        e.step()?;
        pc = 0x10e5dc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_48_unit::cast_body_block_48;

mod cast_body_block_49_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_49(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010df80: move.w #$2, -(a7)
        e.step()?;
        let value = 0x2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010df84: MacToolbox 0xa89c
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa89c_u32 as u16))?;
        // 0010df86: move.l #$40004, -(a7)
        e.step()?;
        let value = 0x40004_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010df8c: MacToolbox 0xa89b
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa89b_u32 as u16))?;
        // 0010df8e: move.l -$10(a6), -$18(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010df94: move.l -$c(a6), -$14(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010df9a: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010df9e: move.l #$ffe0ffe0, -(a7)
        e.step()?;
        let value = 0xffe0ffe0_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dfa4: MacToolbox 0xa8a9
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
        // 0010dfa6: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010dfa8: bra.b $10dfd0
        e.step()?;
        pc = 0x10dfd0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_49_unit::cast_body_block_49;
mod cast_body_block_4a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_4a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dfaa: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dfae: MacToolbox 0xa8b7
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8b7_u32 as u16))?;
        // 0010dfb0: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dfb4: bsr.l $1205f8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10dfba_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1205f8_u32)?;
        // 0010dfba: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dfbe: MacToolbox 0xa8b7
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8b7_u32 as u16))?;
        // 0010dfc0: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dfc4: move.l #$40004, -(a7)
        e.step()?;
        let value = 0x40004_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dfca: MacToolbox 0xa8a9
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
        // 0010dfcc: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        // 0010dfce: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10dfd0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_4a_unit::cast_body_block_4a;

mod cast_body_block_4b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_4b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dfd0: cmpi.w #$8, d3
        e.step()?;
        let source = 0x8_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010dfd4: blt.b $10dfaa
        e.step()?;
        pc = if r.less()? {
            0x10dfaa_u32
        } else {
            0x10dfd6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_4b_unit::cast_body_block_4b;

mod cast_body_block_4c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_4c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dfd6: bra.w $10e5dc
        e.step()?;
        pc = 0x10e5dc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_4c_unit::cast_body_block_4c;

mod cast_body_block_4d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_4d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010dfda: move.w #$2, -(a7)
        e.step()?;
        let value = 0x2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010dfde: MacToolbox 0xa89c
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa89c_u32 as u16))?;
        // 0010dfe0: move.l #$40004, -(a7)
        e.step()?;
        let value = 0x40004_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dfe6: MacToolbox 0xa89b
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa89b_u32 as u16))?;
        // 0010dfe8: move.l -$10(a6), -$18(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dfee: move.l -$c(a6), -$14(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dff4: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010dff8: move.l #$ffe0ffe0, -(a7)
        e.step()?;
        let value = 0xffe0ffe0_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010dffe: MacToolbox 0xa8a9
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
        // 0010e000: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010e002: bra.b $10e06a
        e.step()?;
        pc = 0x10e06a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_4d_unit::cast_body_block_4d;

mod cast_body_block_4e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_4e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e004: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e008: MacToolbox 0xa8b7
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8b7_u32 as u16))?;
        // 0010e00a: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e00e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010e010: lsl.w #$3, d0
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010e012: moveq #$20, d1
        e.step()?;
        r.put_data(1, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010e014: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010e016: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e018: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010e01a: lsl.w #$3, d0
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010e01c: moveq #$20, d1
        e.step()?;
        r.put_data(1, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010e01e: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010e020: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e022: MacToolbox 0xa8a9
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
        // 0010e024: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e028: MacToolbox 0xa8b7
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8b7_u32 as u16))?;
        // 0010e02a: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e02e: bsr.l $1205f8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e034_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1205f8_u32)?;
        // 0010e034: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e038: MacToolbox 0xa8b7
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8b7_u32 as u16))?;
        // 0010e03a: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e03e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010e040: lsl.w #$3, d0
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010e042: addi.w #$ffe0, d0
        e.step()?;
        let source = 0xffe0_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010e046: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e048: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010e04a: lsl.w #$3, d0
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010e04c: addi.w #$ffe0, d0
        e.step()?;
        let source = 0xffe0_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010e050: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e052: MacToolbox 0xa8a9
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
        // 0010e054: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e058: MacToolbox 0xa8b7
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8b7_u32 as u16))?;
        // 0010e05a: pea.l -$10(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e05e: move.l #$40004, -(a7)
        e.step()?;
        let value = 0x40004_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e064: MacToolbox 0xa8a9
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8a9_u32 as u16))?;
        // 0010e066: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        // 0010e068: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10e06a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_4e_unit::cast_body_block_4e;

mod cast_body_block_4f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_4f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e06a: cmpi.w #$8, d3
        e.step()?;
        let source = 0x8_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e06e: blt.b $10e004
        e.step()?;
        pc = if r.less()? {
            0x10e004_u32
        } else {
            0x10e070_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_4f_unit::cast_body_block_4f;

mod cast_body_block_50_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_50(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e070: bra.w $10e5dc
        e.step()?;
        pc = 0x10e5dc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_50_unit::cast_body_block_50;

mod cast_body_block_51_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_51(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e074: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010e076: bra.w $10e5d6
        e.step()?;
        pc = 0x10e5d6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_51_unit::cast_body_block_51;

mod cast_body_block_52_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_52(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e07a: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010e07c: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010e080: move.w d0, d1
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010e082: asr.w #$4, d1
        e.step()?;
        let count = 0x4_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(1, 16, value);
        // 0010e084: lsr.w #$8, d1
        e.step()?;
        let count = 0x8_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(1, 16, value);
        // 0010e086: lsr.w #$3, d1
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(1, 16, value);
        // 0010e088: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010e08a: asr.w #$5, d1
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(1, 16, value);
        // 0010e08c: ext.l d1
        e.step()?;
        let source = r.data(1, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010e08e: add.l $442780.l, d1
        e.step()?;
        let source_address = 0x442780_u32;
        let source = e.l(source_address)?;
        let destination = r.data(1, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(1, 32, value);
        // 0010e094: muls.l #$b4, d1
        e.step()?;
        let source = 0xb4_u32;
        let destination = r.data(1, 32)?;
        let product = i64::from(source as i32) * i64::from(destination as i32);
        let value = product as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        r.put_flag(2, !(-2147483648..=2147483647).contains(&product));
        // 0010e09c: lea.l $4085b8.l, a0
        e.step()?;
        let value = 0x4085b8_u32;
        r.put_address(0, value);
        // 0010e0a2: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010e0a4: moveq #$10, d0
        e.step()?;
        r.put_data(0, 32, 0x10_u32);
        r.move_flags(0x10_u32, 32);
        // 0010e0a6: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010e0aa: move.w d0, d1
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010e0ac: asr.w #$4, d1
        e.step()?;
        let count = 0x4_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(1, 16, value);
        // 0010e0ae: lsr.w #$8, d1
        e.step()?;
        let count = 0x8_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(1, 16, value);
        // 0010e0b0: lsr.w #$3, d1
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(1, 16, value);
        // 0010e0b2: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010e0b4: asr.w #$5, d1
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(1, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(1, 16, value);
        // 0010e0b6: ext.l d1
        e.step()?;
        let source = r.data(1, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010e0b8: add.l $442784.l, d1
        e.step()?;
        let source_address = 0x442784_u32;
        let source = e.l(source_address)?;
        let destination = r.data(1, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(1, 32, value);
        // 0010e0be: move.w (a0, d1.l * 2), -(a7)
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
        // 0010e0c2: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e0c8_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0010e0c8: move.w d0, d5
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(5, 16, value);
        r.move_flags(value, 16);
        // 0010e0ca: cmpi.b #$6, $44374f.l
        e.step()?;
        let source = 0x6_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010e0d2: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010e0d4: bne.w $10e198
        e.step()?;
        pc = if r.not_equal()? {
            0x10e198_u32
        } else {
            0x10e0d8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_52_unit::cast_body_block_52;

mod cast_body_block_53_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_53(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e0d8: move.b $443751.l, d0
        e.step()?;
        let value_address = 0x443751_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010e0de: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010e0e0: move.b $443750.l, d1
        e.step()?;
        let value_address = 0x443750_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 0010e0e6: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010e0e8: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010e0ea: tst.w d1
        e.step()?;
        let value = r.data(1, 16)?;
        r.move_flags(value, 16);
        // 0010e0ec: ble.w $10e198
        e.step()?;
        pc = if r.less_equal()? {
            0x10e198_u32
        } else {
            0x10e0f0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_53_unit::cast_body_block_53;

mod cast_body_block_54_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_54(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e0f0: cmpi.w #$3e8, d5
        e.step()?;
        let source = 0x3e8_u32;
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e0f4: ble.w $10e198
        e.step()?;
        pc = if r.less_equal()? {
            0x10e198_u32
        } else {
            0x10e0f8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_54_unit::cast_body_block_54;

mod cast_body_block_55_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_55(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e0f8: move.w d5, d0
        e.step()?;
        let value = r.data(5, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010e0fa: addi.w #$fc18, d0
        e.step()?;
        let source = 0xfc18_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010e0fe: mulu.w #$28, d0
        e.step()?;
        let source = 0x28_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010e102: lea.l $43167c.l, a0
        e.step()?;
        let value = 0x43167c_u32;
        r.put_address(0, value);
        // 0010e108: tst.w $c(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010e10c: beq.w $10e198
        e.step()?;
        pc = if r.equal()? {
            0x10e198_u32
        } else {
            0x10e110_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_55_unit::cast_body_block_55;

mod cast_body_block_56_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_56(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e110: move.w #$d89e, -(a7)
        e.step()?;
        let value = 0xd89e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e114: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e11a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 0010e11a: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010e11c: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010e122: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010e128: cmpi.w #$3, $2(a0, d0.l)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010e12e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010e130: bge.b $10e166
        e.step()?;
        pc = if r.greater_equal()? {
            0x10e166_u32
        } else {
            0x10e132_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_56_unit::cast_body_block_56;

mod cast_body_block_57_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_57(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e132: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010e134: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010e13a: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010e140: move.w (a0, d0.l), -(a7)
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
        // 0010e144: bsr.l $105572
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e14a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x105572_u32)?;
        // 0010e14a: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e14c: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010e14e: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010e154: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010e15a: move.w (a0, d0.l), -(a7)
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
        // 0010e15e: bsr.l $120c14
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e164_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x120c14_u32)?;
        // 0010e164: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10e166_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_57_unit::cast_body_block_57;

mod cast_body_block_58_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_58(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e166: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010e168: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010e16e: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010e174: move.w #$ffff, $2(a0, d0.l)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e17a: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 0010e17c: mulu.w $442bd4.l, d0
        e.step()?;
        let source_address = 0x442bd4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010e182: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 0010e188: move.w #$ffff, (a0, d0.l)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e18e: move.w #$3e8, d3
        e.step()?;
        let value = 0x3e8_u32;
        r.put_data(3, 16, value);
        r.move_flags(value, 16);
        // 0010e192: move.b #$1, -$51(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffaf_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x10e198_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_58_unit::cast_body_block_58;

mod cast_body_block_59_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_59(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e198: cmpi.b #$6, $44374f.l
        e.step()?;
        let source = 0x6_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010e1a0: bne.w $10e272
        e.step()?;
        pc = if r.not_equal()? {
            0x10e272_u32
        } else {
            0x10e1a4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_59_unit::cast_body_block_59;
mod cast_body_block_5a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_5a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e1a4: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 0010e1aa: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010e1b0: lea.l $4434e0.l, a1
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(1, value);
        // 0010e1b6: move.w (a0, d0.w * 2), d1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0010e1ba: clr.w (a1, d1.w * 2)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e1be: cmpi.w #$6e, d5
        e.step()?;
        let source = 0x6e_u32;
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e1c2: bgt.w $10e272
        e.step()?;
        pc = if r.greater()? {
            0x10e272_u32
        } else {
            0x10e1c6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_5a_unit::cast_body_block_5a;

mod cast_body_block_5b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_5b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e1c6: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 0010e1cc: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010e1d2: cmp.w (a0, d0.w * 2), d5
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e1d6: beq.w $10e272
        e.step()?;
        pc = if r.equal()? {
            0x10e272_u32
        } else {
            0x10e1da_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_5b_unit::cast_body_block_5b;

mod cast_body_block_5c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_5c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e1da: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 0010e1e0: tst.w (a0, d5.w * 2)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(5, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010e1e4: bne.w $10e272
        e.step()?;
        pc = if r.not_equal()? {
            0x10e272_u32
        } else {
            0x10e1e8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_5c_unit::cast_body_block_5c;

mod cast_body_block_5d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_5d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e1e8: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 0010e1ee: move.w #$1, (a0, d5.w * 2)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(5, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e1f4: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0010e1f6: pea.l -$18(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e1fa: pea.l -$20(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e1fe: pea.l -$28(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e202: MacToolbox 0xa8aa
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8aa_u32 as u16))?;
        // 0010e204: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010e206: beq.b $10e22a
        e.step()?;
        pc = if r.equal()? {
            0x10e22a_u32
        } else {
            0x10e208_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_5d_unit::cast_body_block_5d;

mod cast_body_block_5e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_5e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e208: movea.l $4386c0.l, a0
        e.step()?;
        let value_address = 0x4386c0_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e20e: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e212: movea.l $43879c.l, a0
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e218: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e21c: pea.l -$30(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e220: pea.l -$18(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e224: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e226: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e228: MacToolbox 0xa8ec
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
        pc = 0x10e22a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_5e_unit::cast_body_block_5e;

mod cast_body_block_5f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_5f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e22a: bsr.l $151022
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e230_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x151022_u32)?;
        // 0010e230: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0010e232: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e236: pea.l -$20(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e23a: pea.l -$28(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e23e: MacToolbox 0xa8aa
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8aa_u32 as u16))?;
        // 0010e240: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010e242: beq.b $10e266
        e.step()?;
        pc = if r.equal()? {
            0x10e266_u32
        } else {
            0x10e244_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_5f_unit::cast_body_block_5f;

mod cast_body_block_60_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_60(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e244: movea.l $43879c.l, a0
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e24a: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e24e: movea.l $4386c0.l, a0
        e.step()?;
        let value_address = 0x4386c0_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e254: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e258: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e25c: pea.l -$30(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e260: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e262: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e264: MacToolbox 0xa8ec
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
        pc = 0x10e266_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_60_unit::cast_body_block_60;

mod cast_body_block_61_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_61(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e266: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 0010e26c: move.w #$ffff, (a0, d5.w * 2)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(5, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10e272_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_61_unit::cast_body_block_61;

mod cast_body_block_62_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_62(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e272: pea.l -$38(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e276: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e27a: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e27e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e280: pea.l -$48(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffb8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e284: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e288: move.w #$1000, -(a7)
        e.step()?;
        let value = 0x1000_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e28c: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e28e: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e292: pea.l -$38(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e296: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e29a: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e29c: pea.l -$34(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffcc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2a0: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2a4: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e2a8: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e2aa: pea.l -$44(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffbc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2ae: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2b2: move.w #$1000, -(a7)
        e.step()?;
        let value = 0x1000_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e2b6: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e2b8: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2bc: pea.l -$34(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffcc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2c0: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e2c4: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e2c6: pea.l -$38(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2ca: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2ce: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e2d2: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e2d4: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2d8: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e2dc: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e2de: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2e2: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2e6: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e2ea: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e2ec: move.w -$62(a6), -$8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9e_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e2f2: pea.l -$34(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffcc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2f6: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e2fa: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e2fe: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e300: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e304: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e308: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e30a: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e30e: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e312: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e316: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e318: move.w -$62(a6), -$6(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff9e_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e31e: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010e320: add.w -$6(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010e324: move.w d0, -$2(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e328: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010e32a: add.w -$8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010e32e: move.w d0, -$4(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e332: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0010e334: pea.l -$18(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e338: pea.l -$20(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e33c: pea.l -$28(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e340: MacToolbox 0xa8aa
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8aa_u32 as u16))?;
        // 0010e342: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010e344: beq.b $10e368
        e.step()?;
        pc = if r.equal()? {
            0x10e368_u32
        } else {
            0x10e346_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_62_unit::cast_body_block_62;

mod cast_body_block_63_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_63(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e346: movea.l $4386c0.l, a0
        e.step()?;
        let value_address = 0x4386c0_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e34c: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e350: movea.l $43879c.l, a0
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e356: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e35a: pea.l -$30(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e35e: pea.l -$18(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e362: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e364: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e366: MacToolbox 0xa8ec
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
        pc = 0x10e368_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_63_unit::cast_body_block_63;

mod cast_body_block_64_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_64(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e368: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0010e36a: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e36e: pea.l -$20(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e372: pea.l -$28(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e376: MacToolbox 0xa8aa
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8aa_u32 as u16))?;
        // 0010e378: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010e37a: bne.b $10e3b4
        e.step()?;
        pc = if r.not_equal()? {
            0x10e3b4_u32
        } else {
            0x10e37c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_64_unit::cast_body_block_64;

mod cast_body_block_65_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_65(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e37c: tst.b d7
        e.step()?;
        let value = r.data(7, 8)?;
        r.move_flags(value, 8);
        // 0010e37e: bne.w $10e5c8
        e.step()?;
        pc = if r.not_equal()? {
            0x10e5c8_u32
        } else {
            0x10e382_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_65_unit::cast_body_block_65;

mod cast_body_block_66_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_66(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e382: move.w -$56(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffaa_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e386: move.w -$54(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffac_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e38a: bsr.l $10e6b6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e390_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10e6b6_u32)?;
        // 0010e390: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e396: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 0010e398: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e39a: bsr.l $15a1f6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e3a0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15a1f6_u32)?;
        // 0010e3a0: moveq #$1, d7
        e.step()?;
        r.put_data(7, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 0010e3a2: move.l -$8(a6), -$18(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e3a8: move.l -$4(a6), -$14(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e3ae: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010e3b0: bra.w $10d654
        e.step()?;
        pc = 0x10d654_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_66_unit::cast_body_block_66;

mod cast_body_block_67_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_67(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e3b4: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0010e3b6: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e3ba: pea.l -$20(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e3be: pea.l -$28(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e3c2: MacToolbox 0xa8aa
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8aa_u32 as u16))?;
        // 0010e3c4: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010e3c6: beq.w $10e5c8
        e.step()?;
        pc = if r.equal()? {
            0x10e5c8_u32
        } else {
            0x10e3ca_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_67_unit::cast_body_block_67;

mod cast_body_block_68_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_68(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e3ca: movea.l $43879c.l, a0
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e3d0: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e3d4: movea.l $4386c0.l, a0
        e.step()?;
        let value_address = 0x4386c0_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e3da: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e3de: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e3e2: pea.l -$30(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e3e6: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e3e8: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e3ea: MacToolbox 0xa8ec
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
        // 0010e3ec: move.w #$40, $4434ba.l
        e.step()?;
        let value = 0x40_u32;
        let destination_address = 0x4434ba_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e3f4: move.b $44375f.l, d0
        e.step()?;
        let value_address = 0x44375f_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010e3fa: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010e3fc: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0010e3fe: addi.w #$ffe0, d0
        e.step()?;
        let source = 0xffe0_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010e402: move.w d0, $4434bc.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434bc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e408: moveq #$20, d0
        e.step()?;
        r.put_data(0, 32, 0x20_u32);
        r.move_flags(0x20_u32, 32);
        // 0010e40a: add.w $4434bc.l, d0
        e.step()?;
        let source_address = 0x4434bc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010e410: move.w d0, $4434c0.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434c0_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e416: move.w #$60, $4434be.l
        e.step()?;
        let value = 0x60_u32;
        let destination_address = 0x4434be_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e41e: move.l -$4c(a6), -$3c(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffb4_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffc4_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e424: move.l -$50(a6), -$40(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffb0_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e42a: clr.l -$5a(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e42e: pea.l -$3c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e432: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e436: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e43a: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e43c: pea.l -$5a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e440: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e444: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e448: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e44a: bge.b $10e47e
        e.step()?;
        pc = if r.greater_equal()? {
            0x10e47e_u32
        } else {
            0x10e44c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_68_unit::cast_body_block_68;

mod cast_body_block_69_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_69(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e44c: move.l #$bf800000, -$5a(a6)
        e.step()?;
        let value = 0xbf800000_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e454: pea.l -$3c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e458: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e45c: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e460: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e462: pea.l -$5a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e466: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e46a: move.w #$1004, -(a7)
        e.step()?;
        let value = 0x1004_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e46e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e470: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e474: pea.l -$3c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e478: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e47c: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        pc = 0x10e47e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_69_unit::cast_body_block_69;
mod cast_body_block_6a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_6a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e47e: clr.l -$5a(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e482: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e486: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e48a: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e48e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e490: pea.l -$5a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e494: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e498: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e49c: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e49e: bge.b $10e4d2
        e.step()?;
        pc = if r.greater_equal()? {
            0x10e4d2_u32
        } else {
            0x10e4a0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_6a_unit::cast_body_block_6a;

mod cast_body_block_6b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_6b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e4a0: move.l #$bf800000, -$5a(a6)
        e.step()?;
        let value = 0xbf800000_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e4a8: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e4ac: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e4b0: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e4b4: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e4b6: pea.l -$5a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e4ba: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e4be: move.w #$1004, -(a7)
        e.step()?;
        let value = 0x1004_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e4c2: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e4c4: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e4c8: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e4cc: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e4d0: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        pc = 0x10e4d2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_6b_unit::cast_body_block_6b;

mod cast_body_block_6c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_6c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e4d2: pea.l -$3c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e4d6: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e4da: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e4de: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e4e0: pea.l -$40(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffc0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e4e4: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e4e8: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e4ec: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e4ee: ble.b $10e534
        e.step()?;
        pc = if r.less_equal()? {
            0x10e534_u32
        } else {
            0x10e4f0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_6c_unit::cast_body_block_6c;

mod cast_body_block_6d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_6d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e4f0: addi.w #$40, $4434ba.l
        e.step()?;
        let source = 0x40_u32;
        let destination_address = 0x4434ba_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010e4f8: addi.w #$40, $4434be.l
        e.step()?;
        let source = 0x40_u32;
        let destination_address = 0x4434be_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010e500: clr.l -$5a(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e504: pea.l -$44(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffbc_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e508: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e50c: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e510: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e512: pea.l -$5a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e516: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e51a: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e51e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e520: bge.b $10e566
        e.step()?;
        pc = if r.greater_equal()? {
            0x10e566_u32
        } else {
            0x10e522_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_6d_unit::cast_body_block_6d;

mod cast_body_block_6e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_6e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e522: addi.w #$20, $4434ba.l
        e.step()?;
        let source = 0x20_u32;
        let destination_address = 0x4434ba_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010e52a: addi.w #$20, $4434be.l
        e.step()?;
        let source = 0x20_u32;
        let destination_address = 0x4434be_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010e532: bra.b $10e566
        e.step()?;
        pc = 0x10e566_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_6e_unit::cast_body_block_6e;

mod cast_body_block_6f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_6f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e534: clr.l -$5a(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e538: pea.l -$48(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffb8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e53c: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e540: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e544: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e546: pea.l -$5a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffa6_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e54a: pea.l -$64(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffff9c_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e54e: move.w #$1008, -(a7)
        e.step()?;
        let value = 0x1008_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e552: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010e554: ble.b $10e566
        e.step()?;
        pc = if r.less_equal()? {
            0x10e566_u32
        } else {
            0x10e556_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_6f_unit::cast_body_block_6f;

mod cast_body_block_70_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_70(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e556: addi.w #$20, $4434ba.l
        e.step()?;
        let source = 0x20_u32;
        let destination_address = 0x4434ba_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010e55e: addi.w #$20, $4434be.l
        e.step()?;
        let source = 0x20_u32;
        let destination_address = 0x4434be_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x10e566_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_70_unit::cast_body_block_70;

mod cast_body_block_71_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_71(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e566: movea.l $4386c8.l, a0
        e.step()?;
        let value_address = 0x4386c8_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e56c: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e570: movea.l $43879c.l, a0
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e576: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e57a: pea.l $4434ba.l
        e.step()?;
        let value = 0x4434ba_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e580: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e584: move.w #$24, -(a7)
        e.step()?;
        let value = 0x24_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e588: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e58a: MacToolbox 0xa8ec
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
        // 0010e58c: cmpi.w #$f, d3
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e590: beq.b $10e5bc
        e.step()?;
        pc = if r.equal()? {
            0x10e5bc_u32
        } else {
            0x10e592_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_71_unit::cast_body_block_71;

mod cast_body_block_72_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_72(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e592: cmpi.w #$1e, d3
        e.step()?;
        let source = 0x1e_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e596: beq.b $10e5bc
        e.step()?;
        pc = if r.equal()? {
            0x10e5bc_u32
        } else {
            0x10e598_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_72_unit::cast_body_block_72;

mod cast_body_block_73_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_73(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e598: cmpi.w #$2d, d3
        e.step()?;
        let source = 0x2d_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e59c: beq.b $10e5bc
        e.step()?;
        pc = if r.equal()? {
            0x10e5bc_u32
        } else {
            0x10e59e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_73_unit::cast_body_block_73;

mod cast_body_block_74_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_74(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e59e: cmpi.w #$3c, d3
        e.step()?;
        let source = 0x3c_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e5a2: beq.b $10e5bc
        e.step()?;
        pc = if r.equal()? {
            0x10e5bc_u32
        } else {
            0x10e5a4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_74_unit::cast_body_block_74;

mod cast_body_block_75_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_75(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e5a4: cmpi.w #$4b, d3
        e.step()?;
        let source = 0x4b_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e5a8: beq.b $10e5bc
        e.step()?;
        pc = if r.equal()? {
            0x10e5bc_u32
        } else {
            0x10e5aa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_75_unit::cast_body_block_75;

mod cast_body_block_76_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_76(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e5aa: cmpi.w #$5a, d3
        e.step()?;
        let source = 0x5a_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e5ae: beq.b $10e5bc
        e.step()?;
        pc = if r.equal()? {
            0x10e5bc_u32
        } else {
            0x10e5b0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_76_unit::cast_body_block_76;

mod cast_body_block_77_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_77(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e5b0: cmpi.w #$69, d3
        e.step()?;
        let source = 0x69_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e5b4: beq.b $10e5bc
        e.step()?;
        pc = if r.equal()? {
            0x10e5bc_u32
        } else {
            0x10e5b6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_77_unit::cast_body_block_77;

mod cast_body_block_78_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_78(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e5b6: cmpi.w #$78, d3
        e.step()?;
        let source = 0x78_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e5ba: bne.b $10e5c8
        e.step()?;
        pc = if r.not_equal()? {
            0x10e5c8_u32
        } else {
            0x10e5bc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_78_unit::cast_body_block_78;

mod cast_body_block_79_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_79(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e5bc: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e5c0: bsr.l $1205f8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e5c6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1205f8_u32)?;
        // 0010e5c6: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10e5c8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_79_unit::cast_body_block_79;
mod cast_body_block_7a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_7a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e5c8: move.l -$8(a6), -$18(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e5ce: move.l -$4(a6), -$14(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e5d4: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x10e5d6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_7a_unit::cast_body_block_7a;

mod cast_body_block_7b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_7b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e5d6: cmp.w d6, d3
        e.step()?;
        let source = r.data(6, 16)?;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e5d8: blt.w $10e07a
        e.step()?;
        pc = if r.less()? {
            0x10e07a_u32
        } else {
            0x10e5dc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_7b_unit::cast_body_block_7b;

mod cast_body_block_7c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_7c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e5dc: cmpi.b #$19, $44375f.l
        e.step()?;
        let source = 0x19_u32;
        let destination_address = 0x44375f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010e5e4: bge.b $10e61c
        e.step()?;
        pc = if r.greater_equal()? {
            0x10e61c_u32
        } else {
            0x10e5e6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_7c_unit::cast_body_block_7c;

mod cast_body_block_7d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_7d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e5e6: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0010e5e8: pea.l -$18(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e5ec: pea.l -$20(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e5f0: pea.l -$28(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e5f4: MacToolbox 0xa8aa
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8aa_u32 as u16))?;
        // 0010e5f6: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010e5f8: beq.b $10e61c
        e.step()?;
        pc = if r.equal()? {
            0x10e61c_u32
        } else {
            0x10e5fa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_7d_unit::cast_body_block_7d;

mod cast_body_block_7e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_7e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e5fa: movea.l $4386c0.l, a0
        e.step()?;
        let value_address = 0x4386c0_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e600: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e604: movea.l $43879c.l, a0
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.put_address(0, value);
        // 0010e60a: pea.l $2(a0)
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x2_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e60e: pea.l -$30(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffd0_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e612: pea.l -$18(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010e616: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e618: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e61a: MacToolbox 0xa8ec
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ec_u32 as u16))?;
        pc = 0x10e61c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_7e_unit::cast_body_block_7e;

mod cast_body_block_7f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_7f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e61c: cmpi.b #$6, $44374f.l
        e.step()?;
        let source = 0x6_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010e624: beq.b $10e642
        e.step()?;
        pc = if r.equal()? {
            0x10e642_u32
        } else {
            0x10e626_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_7f_unit::cast_body_block_7f;

mod cast_body_block_80_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_80(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e626: tst.b -$51(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffaf_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0010e62a: bne.b $10e642
        e.step()?;
        pc = if r.not_equal()? {
            0x10e642_u32
        } else {
            0x10e62c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_80_unit::cast_body_block_80;

mod cast_body_block_81_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_81(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e62c: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e62e: move.l #$ffff0000, -(a7)
        e.step()?;
        let value = 0xffff0000_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e634: move.w $442bd4.l, -(a7)
        e.step()?;
        let value_address = 0x442bd4_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e63a: bsr.l $15cb0e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e640_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15cb0e_u32)?;
        // 0010e640: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10e642_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_81_unit::cast_body_block_81;

mod cast_body_block_82_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_82(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e642: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010e644: bra.b $10e652
        e.step()?;
        pc = 0x10e652_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_82_unit::cast_body_block_82;

mod cast_body_block_83_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_83(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e646: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 0010e64c: clr.w (a0, d3.w * 2)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e650: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x10e652_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_83_unit::cast_body_block_83;

mod cast_body_block_84_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_84(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e652: cmpi.w #$6e, d3
        e.step()?;
        let source = 0x6e_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e656: blt.b $10e646
        e.step()?;
        pc = if r.less()? {
            0x10e646_u32
        } else {
            0x10e658_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_84_unit::cast_body_block_84;

mod cast_body_block_85_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_85(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e658: addq.w #$1, $442bd4.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x442bd4_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x10e65e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_85_unit::cast_body_block_85;

mod cast_body_block_86_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_86(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e65e: move.w $442bd4.l, d0
        e.step()?;
        let value_address = 0x442bd4_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010e664: cmp.w $8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0x8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010e668: blt.w $10d632
        e.step()?;
        pc = if r.less()? {
            0x10d632_u32
        } else {
            0x10e66c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_86_unit::cast_body_block_86;

mod cast_body_block_87_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_87(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010e66c: clr.b $435661.l
        e.step()?;
        let value = 0;
        let destination_address = 0x435661_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0010e672: clr.b $435882.l
        e.step()?;
        let value = 0;
        let destination_address = 0x435882_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0010e678: bsr.l $13822a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e67e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13822a_u32)?;
        // 0010e67e: move.w $408572.l, -(a7)
        e.step()?;
        let value_address = 0x408572_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e684: move.w $408570.l, -(a7)
        e.step()?;
        let value_address = 0x408570_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e68a: bsr.l $10e6b6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e690_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10e6b6_u32)?;
        // 0010e690: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010e696: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 0010e698: move.w #$15, -(a7)
        e.step()?;
        let value = 0x15_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010e69c: bsr.l $13154c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10e6a2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13154c_u32)?;
        // 0010e6a2: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010e6a4: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010e6a6: movem.l (a7)+, d3-d7
        e.step()?;
        r.restore_register(e, 3)?;
        r.restore_register(e, 4)?;
        r.restore_register(e, 5)?;
        r.restore_register(e, 6)?;
        r.restore_register(e, 7)?;
        // 0010e6aa: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 0010e6ac: rts
        e.step()?;
        return finish(e, r).map(|_| std::ops::ControlFlow::Break(()));

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use cast_body_block_87_unit::cast_body_block_87;

mod cast_body_block_88_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn cast_body_block_88(
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
use cast_body_block_88_unit::cast_body_block_88;
