//! Complete provisional gameplay control from canonical CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};

pub fn attack_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x100af0_u32;
        loop {
            match match pc {
                0x100af0 => attack_body_block_0(e, &mut r, frame, pc)?,
                0x100b2a => attack_body_block_1(e, &mut r, frame, pc)?,
                0x100b3c => attack_body_block_2(e, &mut r, frame, pc)?,
                0x100b64 => attack_body_block_3(e, &mut r, frame, pc)?,
                0x100b6a => attack_body_block_4(e, &mut r, frame, pc)?,
                0x100b72 => attack_body_block_5(e, &mut r, frame, pc)?,
                0x100b88 => attack_body_block_6(e, &mut r, frame, pc)?,
                0x100b8e => attack_body_block_7(e, &mut r, frame, pc)?,
                0x100b98 => attack_body_block_8(e, &mut r, frame, pc)?,
                0x100bba => attack_body_block_9(e, &mut r, frame, pc)?,
                0x100bbe => attack_body_block_a(e, &mut r, frame, pc)?,
                0x100bc2 => attack_body_block_b(e, &mut r, frame, pc)?,
                0x100bd4 => attack_body_block_c(e, &mut r, frame, pc)?,
                0x100c64 => attack_body_block_d(e, &mut r, frame, pc)?,
                0x100c88 => attack_body_block_e(e, &mut r, frame, pc)?,
                0x100d3e => attack_body_block_f(e, &mut r, frame, pc)?,
                0x100d42 => attack_body_block_10(e, &mut r, frame, pc)?,
                0x100d9e => attack_body_block_11(e, &mut r, frame, pc)?,
                0x100dcc => attack_body_block_12(e, &mut r, frame, pc)?,
                0x100dea => attack_body_block_13(e, &mut r, frame, pc)?,
                0x100e3e => attack_body_block_14(e, &mut r, frame, pc)?,
                0x100e62 => attack_body_block_15(e, &mut r, frame, pc)?,
                0x100e7c => attack_body_block_16(e, &mut r, frame, pc)?,
                0x100e9e => attack_body_block_17(e, &mut r, frame, pc)?,
                0x100f2c => attack_body_block_18(e, &mut r, frame, pc)?,
                0x101056 => attack_body_block_19(e, &mut r, frame, pc)?,
                0x101084 => attack_body_block_1a(e, &mut r, frame, pc)?,
                0x1010a2 => attack_body_block_1b(e, &mut r, frame, pc)?,
                0x1010c0 => attack_body_block_1c(e, &mut r, frame, pc)?,
                0x1010e2 => attack_body_block_1d(e, &mut r, frame, pc)?,
                0x101138 => attack_body_block_1e(e, &mut r, frame, pc)?,
                0x10113e => attack_body_block_1f(e, &mut r, frame, pc)?,
                0x10114a => attack_body_block_20(e, &mut r, frame, pc)?,
                0x10115e => attack_body_block_21(e, &mut r, frame, pc)?,
                0x101170 => attack_body_block_22(e, &mut r, frame, pc)?,
                0x101178 => attack_body_block_23(e, &mut r, frame, pc)?,
                0x10117a => attack_body_block_24(e, &mut r, frame, pc)?,
                0x101186 => attack_body_block_25(e, &mut r, frame, pc)?,
                0x10118e => attack_body_block_26(e, &mut r, frame, pc)?,
                0x10119e => attack_body_block_27(e, &mut r, frame, pc)?,
                0x1011ba => attack_body_block_28(e, &mut r, frame, pc)?,
                0x1011d6 => attack_body_block_29(e, &mut r, frame, pc)?,
                0x1011f0 => attack_body_block_2a(e, &mut r, frame, pc)?,
                0x1011f6 => attack_body_block_2b(e, &mut r, frame, pc)?,
                0x101206 => attack_body_block_2c(e, &mut r, frame, pc)?,
                0x101214 => attack_body_block_2d(e, &mut r, frame, pc)?,
                0x10122a => attack_body_block_2e(e, &mut r, frame, pc)?,
                0x101230 => attack_body_block_2f(e, &mut r, frame, pc)?,
                0x101242 => attack_body_block_30(e, &mut r, frame, pc)?,
                0x101248 => attack_body_block_31(e, &mut r, frame, pc)?,
                0x10124e => attack_body_block_32(e, &mut r, frame, pc)?,
                0x101266 => attack_body_block_33(e, &mut r, frame, pc)?,
                0x101298 => attack_body_block_34(e, &mut r, frame, pc)?,
                0x10129e => attack_body_block_35(e, &mut r, frame, pc)?,
                0x1012a8 => attack_body_block_36(e, &mut r, frame, pc)?,
                0x1012b2 => attack_body_block_37(e, &mut r, frame, pc)?,
                0x1012c6 => attack_body_block_38(e, &mut r, frame, pc)?,
                0x1012c8 => attack_body_block_39(e, &mut r, frame, pc)?,
                0x1012d2 => attack_body_block_3a(e, &mut r, frame, pc)?,
                0x1012e6 => attack_body_block_3b(e, &mut r, frame, pc)?,
                0x1012fe => attack_body_block_3c(e, &mut r, frame, pc)?,
                0x10130a => attack_body_block_3d(e, &mut r, frame, pc)?,
                0x10131c => attack_body_block_3e(e, &mut r, frame, pc)?,
                0x10132a => attack_body_block_3f(e, &mut r, frame, pc)?,
                0x101330 => attack_body_block_40(e, &mut r, frame, pc)?,
                0x101334 => attack_body_block_41(e, &mut r, frame, pc)?,
                0x10133a => attack_body_block_42(e, &mut r, frame, pc)?,
                0x10133e => attack_body_block_43(e, &mut r, frame, pc)?,
                0x101344 => attack_body_block_44(e, &mut r, frame, pc)?,
                0x101348 => attack_body_block_45(e, &mut r, frame, pc)?,
                0x10134e => attack_body_block_46(e, &mut r, frame, pc)?,
                0x10135c => attack_body_block_47(e, &mut r, frame, pc)?,
                0x101362 => attack_body_block_48(e, &mut r, frame, pc)?,
                0x101366 => attack_body_block_49(e, &mut r, frame, pc)?,
                0x101372 => attack_body_block_4a(e, &mut r, frame, pc)?,
                0x10139e => attack_body_block_4b(e, &mut r, frame, pc)?,
                0x1013a0 => attack_body_block_4c(e, &mut r, frame, pc)?,
                0x1013a6 => attack_body_block_4d(e, &mut r, frame, pc)?,
                0x1013ae => attack_body_block_4e(e, &mut r, frame, pc)?,
                0x1013cc => attack_body_block_4f(e, &mut r, frame, pc)?,
                0x1013de => attack_body_block_50(e, &mut r, frame, pc)?,
                0x1013e4 => attack_body_block_51(e, &mut r, frame, pc)?,
                0x1013e8 => attack_body_block_52(e, &mut r, frame, pc)?,
                0x1013ee => attack_body_block_53(e, &mut r, frame, pc)?,
                0x1013f2 => attack_body_block_54(e, &mut r, frame, pc)?,
                0x1013f8 => attack_body_block_55(e, &mut r, frame, pc)?,
                0x1013fc => attack_body_block_56(e, &mut r, frame, pc)?,
                0x101402 => attack_body_block_57(e, &mut r, frame, pc)?,
                0x101414 => attack_body_block_58(e, &mut r, frame, pc)?,
                0x10141a => attack_body_block_59(e, &mut r, frame, pc)?,
                0x10142c => attack_body_block_5a(e, &mut r, frame, pc)?,
                0x101462 => attack_body_block_5b(e, &mut r, frame, pc)?,
                0x10146a => attack_body_block_5c(e, &mut r, frame, pc)?,
                0x101470 => attack_body_block_5d(e, &mut r, frame, pc)?,
                0x101478 => attack_body_block_5e(e, &mut r, frame, pc)?,
                0x1014ca => attack_body_block_5f(e, &mut r, frame, pc)?,
                0x1014e4 => attack_body_block_60(e, &mut r, frame, pc)?,
                0x1014f6 => attack_body_block_61(e, &mut r, frame, pc)?,
                0x1014fa => attack_body_block_62(e, &mut r, frame, pc)?,
                0x10150e => attack_body_block_63(e, &mut r, frame, pc)?,
                0x101512 => attack_body_block_64(e, &mut r, frame, pc)?,
                0x101526 => attack_body_block_65(e, &mut r, frame, pc)?,
                0x10152a => attack_body_block_66(e, &mut r, frame, pc)?,
                0x10153e => attack_body_block_67(e, &mut r, frame, pc)?,
                0x10155a => attack_body_block_68(e, &mut r, frame, pc)?,
                0x10156e => attack_body_block_69(e, &mut r, frame, pc)?,
                0x10158a => attack_body_block_6a(e, &mut r, frame, pc)?,
                0x1015c0 => attack_body_block_6b(e, &mut r, frame, pc)?,
                0x1015d2 => attack_body_block_6c(e, &mut r, frame, pc)?,
                0x1015d4 => attack_body_block_6d(e, &mut r, frame, pc)?,
                0x1015da => attack_body_block_6e(e, &mut r, frame, pc)?,
                0x1015e2 => attack_body_block_6f(e, &mut r, frame, pc)?,
                0x1015e6 => attack_body_block_70(e, &mut r, frame, pc)?,
                0x1015f0 => attack_body_block_71(e, &mut r, frame, pc)?,
                0x101604 => attack_body_block_72(e, &mut r, frame, pc)?,
                0x101622 => attack_body_block_73(e, &mut r, frame, pc)?,
                0x101628 => attack_body_block_74(e, &mut r, frame, pc)?,
                0x101630 => attack_body_block_75(e, &mut r, frame, pc)?,
                0x101634 => attack_body_block_76(e, &mut r, frame, pc)?,
                0x10163e => attack_body_block_77(e, &mut r, frame, pc)?,
                0x101652 => attack_body_block_78(e, &mut r, frame, pc)?,
                0x101670 => attack_body_block_79(e, &mut r, frame, pc)?,
                0x101676 => attack_body_block_7a(e, &mut r, frame, pc)?,
                0x10167c => attack_body_block_7b(e, &mut r, frame, pc)?,
                0x101684 => attack_body_block_7c(e, &mut r, frame, pc)?,
                0x10168c => attack_body_block_7d(e, &mut r, frame, pc)?,
                0x101690 => attack_body_block_7e(e, &mut r, frame, pc)?,
                0x1016a0 => attack_body_block_7f(e, &mut r, frame, pc)?,
                0x1016be => attack_body_block_80(e, &mut r, frame, pc)?,
                0x1016c4 => attack_body_block_81(e, &mut r, frame, pc)?,
                0x1016ca => attack_body_block_82(e, &mut r, frame, pc)?,
                0x1016d0 => attack_body_block_83(e, &mut r, frame, pc)?,
                0x1016e2 => attack_body_block_84(e, &mut r, frame, pc)?,
                0x1016e6 => attack_body_block_85(e, &mut r, frame, pc)?,
                0x1016ec => attack_body_block_86(e, &mut r, frame, pc)?,
                0x1016f2 => attack_body_block_87(e, &mut r, frame, pc)?,
                0x101704 => attack_body_block_88(e, &mut r, frame, pc)?,
                0x101718 => attack_body_block_89(e, &mut r, frame, pc)?,
                0x10171a => attack_body_block_8a(e, &mut r, frame, pc)?,
                0x101742 => attack_body_block_8b(e, &mut r, frame, pc)?,
                0x101838 => attack_body_block_8c(e, &mut r, frame, pc)?,
                0x101898 => attack_body_block_8d(e, &mut r, frame, pc)?,
                0x1018a0 => attack_body_block_8e(e, &mut r, frame, pc)?,
                0x101900 => attack_body_block_8f(e, &mut r, frame, pc)?,
                0x101908 => attack_body_block_90(e, &mut r, frame, pc)?,
                0x101968 => attack_body_block_91(e, &mut r, frame, pc)?,
                0x101970 => attack_body_block_92(e, &mut r, frame, pc)?,
                0x1019d0 => attack_body_block_93(e, &mut r, frame, pc)?,
                0x1019d8 => attack_body_block_94(e, &mut r, frame, pc)?,
                0x101a38 => attack_body_block_95(e, &mut r, frame, pc)?,
                0x101a40 => attack_body_block_96(e, &mut r, frame, pc)?,
                0x101aa0 => attack_body_block_97(e, &mut r, frame, pc)?,
                0x101aa8 => attack_body_block_98(e, &mut r, frame, pc)?,
                0x101b0a => attack_body_block_99(e, &mut r, frame, pc)?,
                0x101b20 => attack_body_block_9a(e, &mut r, frame, pc)?,
                0x101b46 => attack_body_block_9b(e, &mut r, frame, pc)?,
                0x101b4c => attack_body_block_9c(e, &mut r, frame, pc)?,
                0x101b62 => attack_body_block_9d(e, &mut r, frame, pc)?,
                0x101b80 => attack_body_block_9e(e, &mut r, frame, pc)?,
                0x101b8e => attack_body_block_9f(e, &mut r, frame, pc)?,
                0x101b96 => attack_body_block_a0(e, &mut r, frame, pc)?,
                0x101b9c => attack_body_block_a1(e, &mut r, frame, pc)?,
                0x101bbc => attack_body_block_a2(e, &mut r, frame, pc)?,
                0x101bdc => attack_body_block_a3(e, &mut r, frame, pc)?,
                0x101bec => attack_body_block_a4(e, &mut r, frame, pc)?,
                0x101bf2 => attack_body_block_a5(e, &mut r, frame, pc)?,
                0x101bfa => attack_body_block_a6(e, &mut r, frame, pc)?,
                0x101c76 => attack_body_block_a7(e, &mut r, frame, pc)?,
                0x101c8e => attack_body_block_a8(e, &mut r, frame, pc)?,
                0x101caa => attack_body_block_a9(e, &mut r, frame, pc)?,
                0x101cb8 => attack_body_block_aa(e, &mut r, frame, pc)?,
                0x101cd4 => attack_body_block_ab(e, &mut r, frame, pc)?,
                0x101cdc => attack_body_block_ac(e, &mut r, frame, pc)?,
                0x101ce2 => attack_body_block_ad(e, &mut r, frame, pc)?,
                0x101cfe => attack_body_block_ae(e, &mut r, frame, pc)?,
                0x101d1c => attack_body_block_af(e, &mut r, frame, pc)?,
                0x101d2c => attack_body_block_b0(e, &mut r, frame, pc)?,
                0x101d3e => attack_body_block_b1(e, &mut r, frame, pc)?,
                0x101d52 => attack_body_block_b2(e, &mut r, frame, pc)?,
                0x101db2 => attack_body_block_b3(e, &mut r, frame, pc)?,
                0x101dc6 => attack_body_block_b4(e, &mut r, frame, pc)?,
                0x101dda => attack_body_block_b5(e, &mut r, frame, pc)?,
                0x101de0 => attack_body_block_b6(e, &mut r, frame, pc)?,
                0x101dfc => attack_body_block_b7(e, &mut r, frame, pc)?,
                0x101e0a => attack_body_block_b8(e, &mut r, frame, pc)?,
                0x101e1c => attack_body_block_b9(e, &mut r, frame, pc)?,
                0x101e22 => attack_body_block_ba(e, &mut r, frame, pc)?,
                0x101e28 => attack_body_block_bb(e, &mut r, frame, pc)?,
                0x101e44 => attack_body_block_bc(e, &mut r, frame, pc)?,
                0x101e52 => attack_body_block_bd(e, &mut r, frame, pc)?,
                0x101e62 => attack_body_block_be(e, &mut r, frame, pc)?,
                0x101e6a => attack_body_block_bf(e, &mut r, frame, pc)?,
                0x101e70 => attack_body_block_c0(e, &mut r, frame, pc)?,
                0x101e92 => attack_body_block_c1(e, &mut r, frame, pc)?,
                0x101ea4 => attack_body_block_c2(e, &mut r, frame, pc)?,
                _ => attack_body_block_c3(e, &mut r, frame, pc)?,
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
crate::recovered_application_control::entry!(attack, 0x100af0);
mod attack_body_block_0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100af0: link.w a6, #$f274
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, -3468)?;
        // 00100af4: movem.l d3-d7/a2, -(a7)
        e.step()?;
        r.save_register(e, 10)?;
        r.save_register(e, 7)?;
        r.save_register(e, 6)?;
        r.save_register(e, 5)?;
        r.save_register(e, 4)?;
        r.save_register(e, 3)?;
        // 00100af8: move.w $8(a6), d6
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0x8_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(6, 16, value);
        r.move_flags(value, 16);
        // 00100afc: move.w $a(a6), d3
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xa_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(3, 16, value);
        r.move_flags(value, 16);
        // 00100b00: clr.b -$d6c(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffff294_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00100b04: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100b06: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100b0a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100b10: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100b12: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100b14: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100b18: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 00100b1e: adda.l d0, a1
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(1)?;
        let value = destination.wrapping_add(source);
        r.put_address(1, value);
        // 00100b20: move.w $acc(a1), d0
        e.step()?;
        let value_address = r.address(1)?.wrapping_add(0xacc_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100b24: cmp.w $aca(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0xaca_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00100b28: blt.b $100b3c
        e.step()?;
        pc = if r.less()? {
            0x100b3c_u32
        } else {
            0x100b2a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_0_unit::attack_body_block_0;

mod attack_body_block_1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100b2a: move.w #$2d, -(a7)
        e.step()?;
        let value = 0x2d_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100b2e: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100b34_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 00100b34: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00100b36: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00100b38: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_1_unit::attack_body_block_1;

mod attack_body_block_2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100b3c: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100b3e: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100b42: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100b48: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100b4a: addq.w #$1, $acc(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xacc_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00100b4e: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100b50: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100b54: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100b5a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100b5c: lea.l -$c42(a6), a1
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff3be_u32);
        r.put_address(1, value);
        // 00100b60: move.w #$30f, d0
        e.step()?;
        let value = 0x30f_u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        pc = 0x100b64_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_2_unit::attack_body_block_2;

mod attack_body_block_3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100b64: move.l (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = e.l(value_address)?;
        r.offset_address(0, 4)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.offset_address(1, 4)?;
        r.move_flags(value, 32);
        // 00100b66: dbra d0, $100b64
        e.step()?;
        let counter = r.data(0, 16)?;
        let value = counter.wrapping_sub(1) & 0xffff;
        r.put_data(0, 16, value);
        pc = if value != 0xffff {
            0x100b64_u32
        } else {
            0x100b6a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_3_unit::attack_body_block_3;

mod attack_body_block_4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100b6a: move.w (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(0, 2)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.offset_address(1, 2)?;
        r.move_flags(value, 16);
        // 00100b6c: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00100b70: ble.b $100b8e
        e.step()?;
        pc = if r.less_equal()? {
            0x100b8e_u32
        } else {
            0x100b72_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_4_unit::attack_body_block_4;

mod attack_body_block_5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100b72: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00100b74: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100b76: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100b7a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100b80: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100b82: lea.l -$d62(a6), a1
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff29e_u32);
        r.put_address(1, value);
        // 00100b86: moveq #$47, d0
        e.step()?;
        r.put_data(0, 32, 0x47_u32);
        r.move_flags(0x47_u32, 32);
        pc = 0x100b88_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_5_unit::attack_body_block_5;

mod attack_body_block_6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100b88: move.l (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = e.l(value_address)?;
        r.offset_address(0, 4)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.offset_address(1, 4)?;
        r.move_flags(value, 32);
        // 00100b8a: dbra d0, $100b88
        e.step()?;
        let counter = r.data(0, 16)?;
        let value = counter.wrapping_sub(1) & 0xffff;
        r.put_data(0, 16, value);
        pc = if value != 0xffff {
            0x100b88_u32
        } else {
            0x100b8e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_6_unit::attack_body_block_6;

mod attack_body_block_7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100b8e: tst.b $43565f.l
        e.step()?;
        let value_address = 0x43565f_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00100b94: beq.w $1010e2
        e.step()?;
        pc = if r.equal()? {
            0x1010e2_u32
        } else {
            0x100b98_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_7_unit::attack_body_block_7;

mod attack_body_block_8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100b98: bsr.l $1368c8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100b9e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1368c8_u32)?;
        // 00100b9e: subi.w #$a, d3
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(3, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        // 00100ba2: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100ba4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100ba8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100bae: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100bb0: move.b $b0(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb0_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00100bb4: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100bb6: addq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100bb8: beq.b $100bd4
        e.step()?;
        pc = if r.equal()? {
            0x100bd4_u32
        } else {
            0x100bba_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_8_unit::attack_body_block_8;

mod attack_body_block_9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100bba: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100bbc: beq.b $100bc2
        e.step()?;
        pc = if r.equal()? {
            0x100bc2_u32
        } else {
            0x100bbe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_9_unit::attack_body_block_9;
mod attack_body_block_a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100bbe: bra.w $100e9e
        e.step()?;
        pc = 0x100e9e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a_unit::attack_body_block_a;

mod attack_body_block_b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100bc2: move.w #$10, -(a7)
        e.step()?;
        let value = 0x10_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100bc6: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100bcc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 00100bcc: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00100bce: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00100bd0: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b_unit::attack_body_block_b;

mod attack_body_block_c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100bd4: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100bda: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100bde: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100be4: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100be6: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100bec: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100bf0: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 00100bf6: adda.l d0, a1
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(1)?;
        let value = destination.wrapping_add(source);
        r.put_address(1, value);
        // 00100bf8: move.w $f6(a1), d0
        e.step()?;
        let value_address = r.address(1)?.wrapping_add(0xf6_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100bfc: add.w $b64(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0xb64_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100c00: move.w d0, -$d64(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100c04: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100c06: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100c0a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100c10: move.w (a0, d0.l), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100c14: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100c16: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100c18: sub.w d0, -$d64(a6)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29c_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00100c1c: move.l #$3005a, -(a7)
        e.step()?;
        let value = 0x3005a_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00100c22: move.w -$d64(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff29c_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100c26: bsr.l $13f65e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100c2c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13f65e_u32)?;
        // 00100c2c: move.w d0, -$d64(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100c30: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100c32: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100c36: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100c3c: lea.l $40(a0, d0.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00100c40: subq.w #$2, (a0)
        e.step()?;
        let source = 0x2_u32;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00100c42: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100c48: bsr.l $11f1a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100c4e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11f1a8_u32)?;
        // 00100c4e: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100c52: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100c58_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00100c58: cmp.w -$d64(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff29c_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00100c5c: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 00100c60: bgt.w $100e7c
        e.step()?;
        pc = if r.greater()? {
            0x100e7c_u32
        } else {
            0x100c64_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_c_unit::attack_body_block_c;

mod attack_body_block_d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100c64: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100c66: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100c6a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100c70: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00100c72: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00100c76: lea.l $439d3c.l, a1
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(1, value);
        // 00100c7c: move.w $e(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00100c80: cmp.w $10(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00100c84: beq.w $100e7c
        e.step()?;
        pc = if r.equal()? {
            0x100e7c_u32
        } else {
            0x100c88_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_d_unit::attack_body_block_d;

mod attack_body_block_e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100c88: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100c8a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100c8e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100c94: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100c96: moveq #$3, d0
        e.step()?;
        r.put_data(0, 32, 0x3_u32);
        r.move_flags(0x3_u32, 32);
        // 00100c98: muls.w $112(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0x112_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100c9c: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00100c9e: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00100ca2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100ca8: moveq #$f, d2
        e.step()?;
        r.put_data(2, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00100caa: muls.w (a0, d1.l), d2
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(2, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(2, 32, value);
        r.move_flags(value, 32);
        // 00100cae: add.w d0, d2
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(2, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(2, 16, value);
        // 00100cb0: ext.l d2
        e.step()?;
        let source = r.data(2, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(2, 32, value);
        r.move_flags(value, 32);
        // 00100cb2: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100cb4: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100cb8: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100cbe: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100cc0: sub.l d2, $9ce(a0)
        e.step()?;
        let source = r.data(2, 32)?;
        let destination_address = r.address(0)?.wrapping_add(0x9ce_u32);
        let destination = e.l(destination_address)?;
        let value = r.subtract_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00100cc4: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100cc6: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100cca: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100cd0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100cd2: moveq #$3, d0
        e.step()?;
        r.put_data(0, 32, 0x3_u32);
        r.move_flags(0x3_u32, 32);
        // 00100cd4: muls.w $112(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0x112_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100cd8: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00100cda: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00100cde: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100ce4: moveq #$f, d2
        e.step()?;
        r.put_data(2, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00100ce6: muls.w (a0, d1.l), d2
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(2, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(2, 32, value);
        r.move_flags(value, 32);
        // 00100cea: add.w d0, d2
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(2, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(2, 16, value);
        // 00100cec: ext.l d2
        e.step()?;
        let source = r.data(2, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(2, 32, value);
        r.move_flags(value, 32);
        // 00100cee: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100cf0: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100cf4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100cfa: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100cfc: add.l d2, $9d2(a0)
        e.step()?;
        let source = r.data(2, 32)?;
        let destination_address = r.address(0)?.wrapping_add(0x9d2_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00100d00: move.w #$ff2f, -(a7)
        e.step()?;
        let value = 0xff2f_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100d04: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100d0a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00100d0a: move.w #$d0, -(a7)
        e.step()?;
        let value = 0xd0_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100d0e: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100d14_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00100d14: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100d16: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100d1a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100d20: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100d22: move.b #$1, $b0(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00100d28: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100d2a: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100d2e: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100d34: move.w $10(a0, d0.l), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100d38: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00100d3a: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00100d3c: beq.b $100d42
        e.step()?;
        pc = if r.equal()? {
            0x100d42_u32
        } else {
            0x100d3e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_e_unit::attack_body_block_e;

mod attack_body_block_f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100d3e: bra.w $100e3e
        e.step()?;
        pc = 0x100e3e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_f_unit::attack_body_block_f;

mod attack_body_block_10_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_10(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100d42: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100d44: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100d48: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100d4e: moveq #$32, d1
        e.step()?;
        r.put_data(1, 32, 0x32_u32);
        r.move_flags(0x32_u32, 32);
        // 00100d50: muls.w (a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00100d54: move.w #$1388, d0
        e.step()?;
        let value = 0x1388_u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100d58: sub.w d1, d0
        e.step()?;
        let source = r.data(1, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100d5a: move.w d0, $4434b6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100d60: move.l #$32079e, -(a7)
        e.step()?;
        let value = 0x32079e_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00100d66: move.w $4434b6.l, -(a7)
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100d6c: bsr.l $13f65e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100d72_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13f65e_u32)?;
        // 00100d72: move.w d0, $4434b6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100d78: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100d7a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100d7e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100d84: move.w (a0, d0.l), -$d8c(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff274_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100d8a: move.w $4434b6.l, -(a7)
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100d90: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100d96_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00100d96: cmp.w -$d8c(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff274_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00100d9a: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00100d9c: bgt.b $100dea
        e.step()?;
        pc = if r.greater()? {
            0x100dea_u32
        } else {
            0x100d9e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_10_unit::attack_body_block_10;

mod attack_body_block_11_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_11(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100d9e: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100da4: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100da8: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100dae: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100db0: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100db2: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100db6: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 00100dbc: adda.l d0, a1
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(1)?;
        let value = destination.wrapping_add(source);
        r.put_address(1, value);
        // 00100dbe: move.w $f6(a1), d0
        e.step()?;
        let value_address = r.address(1)?.wrapping_add(0xf6_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100dc2: add.w $b64(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0xb64_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100dc6: cmpi.w #$5f, d0
        e.step()?;
        let source = 0x5f_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00100dca: bge.b $100dea
        e.step()?;
        pc = if r.greater_equal()? {
            0x100dea_u32
        } else {
            0x100dcc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_11_unit::attack_body_block_11;

mod attack_body_block_12_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_12(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100dcc: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100dce: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100dd2: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100dd8: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100dda: addq.w #$1, $f6(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xf6_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00100dde: move.w #$8f, -(a7)
        e.step()?;
        let value = 0x8f_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100de2: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100de8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 00100de8: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x100dea_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_12_unit::attack_body_block_12;

mod attack_body_block_13_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_13(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100dea: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100dec: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100df0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100df6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100df8: clr.b $b3(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0xb3_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00100dfc: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00100dfe: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100e00: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00100e02: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00100e06: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100e0c: move.w d0, $e(a0, d1.l)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100e10: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100e12: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100e16: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100e1c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100e1e: move.b #$1, $b0(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00100e24: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100e26: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100e2a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100e30: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100e32: addq.l #$1, $a6e(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xa6e_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00100e36: bsr.l $16043e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100e3c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16043e_u32)?;
        // 00100e3c: bra.b $100e62
        e.step()?;
        pc = 0x100e62_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_13_unit::attack_body_block_13;

mod attack_body_block_14_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_14(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100e3e: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100e40: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100e44: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100e4a: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00100e4c: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00100e50: lea.l $439d3c.l, a1
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(1, value);
        // 00100e56: move.w $10(a0, d0.l), $e(a1, d1.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(1)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100e5c: bsr.l $16043e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100e62_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16043e_u32)?;
        pc = 0x100e62_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_14_unit::attack_body_block_14;

mod attack_body_block_15_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_15(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100e62: move.w $408572.l, -(a7)
        e.step()?;
        let value_address = 0x408572_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100e68: move.w $408570.l, -(a7)
        e.step()?;
        let value_address = 0x408570_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100e6e: bsr.l $10e6b6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100e74_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10e6b6_u32)?;
        // 00100e74: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00100e76: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00100e78: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_15_unit::attack_body_block_15;

mod attack_body_block_16_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_16(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100e7c: move.l #$3c1770, -(a7)
        e.step()?;
        let value = 0x3c1770_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00100e82: move.l #$1e012c, -(a7)
        e.step()?;
        let value = 0x1e012c_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00100e88: pea.l $404088.l
        e.step()?;
        let value = 0x404088_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00100e8e: bsr.l $123d1c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100e94_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x123d1c_u32)?;
        // 00100e94: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00100e96: lea.l $c(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xc_u32);
        r.put_address(7, value);
        // 00100e9a: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_16_unit::attack_body_block_16;

mod attack_body_block_17_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_17(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100e9e: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100ea4: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100ea8: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100eae: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100eb0: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100eb6: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100eba: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 00100ec0: adda.l d0, a1
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(1)?;
        let value = destination.wrapping_add(source);
        r.put_address(1, value);
        // 00100ec2: move.w $f4(a1), d0
        e.step()?;
        let value_address = r.address(1)?.wrapping_add(0xf4_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100ec6: add.w $b62(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0xb62_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100eca: move.w d0, -$d64(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100ece: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100ed0: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100ed4: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100eda: move.w (a0, d0.l), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100ede: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100ee0: sub.w d0, -$d64(a6)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29c_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00100ee4: move.l #$5005f, -(a7)
        e.step()?;
        let value = 0x5005f_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00100eea: move.w -$d64(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff29c_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100eee: bsr.l $13f65e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100ef4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13f65e_u32)?;
        // 00100ef4: move.w d0, -$d64(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100ef8: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100efa: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100efe: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100f04: lea.l $40(a0, d0.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00100f08: subq.w #$2, (a0)
        e.step()?;
        let source = 0x2_u32;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00100f0a: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100f10: bsr.l $11f1a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100f16_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11f1a8_u32)?;
        // 00100f16: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100f1a: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100f20_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00100f20: cmp.w -$d64(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff29c_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00100f24: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 00100f28: bgt.w $1010c0
        e.step()?;
        pc = if r.greater()? {
            0x1010c0_u32
        } else {
            0x100f2c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_17_unit::attack_body_block_17;

mod attack_body_block_18_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_18(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00100f2c: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100f2e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100f32: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100f38: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100f3a: move.w $112(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x112_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100f3e: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100f40: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00100f42: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00100f46: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100f4c: moveq #$a, d2
        e.step()?;
        r.put_data(2, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00100f4e: muls.w (a0, d1.l), d2
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(2, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(2, 32, value);
        r.move_flags(value, 32);
        // 00100f52: add.w d0, d2
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(2, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(2, 16, value);
        // 00100f54: ext.l d2
        e.step()?;
        let source = r.data(2, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(2, 32, value);
        r.move_flags(value, 32);
        // 00100f56: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100f58: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100f5c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100f62: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100f64: sub.l d2, $9ce(a0)
        e.step()?;
        let source = r.data(2, 32)?;
        let destination_address = r.address(0)?.wrapping_add(0x9ce_u32);
        let destination = e.l(destination_address)?;
        let value = r.subtract_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00100f68: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100f6a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100f6e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100f74: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100f76: move.w $112(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x112_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100f7a: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00100f7c: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00100f7e: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00100f82: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100f88: moveq #$a, d2
        e.step()?;
        r.put_data(2, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00100f8a: muls.w (a0, d1.l), d2
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(2, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(2, 32, value);
        r.move_flags(value, 32);
        // 00100f8e: add.w d0, d2
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(2, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(2, 16, value);
        // 00100f90: ext.l d2
        e.step()?;
        let source = r.data(2, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(2, 32, value);
        r.move_flags(value, 32);
        // 00100f92: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100f94: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100f98: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00100f9e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100fa0: add.l d2, $9d2(a0)
        e.step()?;
        let source = r.data(2, 32)?;
        let destination_address = r.address(0)?.wrapping_add(0x9d2_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00100fa4: move.w #$ff32, -(a7)
        e.step()?;
        let value = 0xff32_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100fa8: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100fae_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00100fae: move.w #$cd, -(a7)
        e.step()?;
        let value = 0xcd_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100fb2: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100fb8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00100fb8: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100fba: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100fbe: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100fc4: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100fc6: move.b #$ff, $b0(a0)
        e.step()?;
        let value = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb0_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00100fcc: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100fce: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100fd2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00100fd8: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00100fda: move.b #$ff, $b3(a0)
        e.step()?;
        let value = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb3_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00100fe0: move.w $408572.l, -(a7)
        e.step()?;
        let value_address = 0x408572_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100fe6: move.w $408570.l, -(a7)
        e.step()?;
        let value_address = 0x408570_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00100fec: bsr.l $10e6b6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x100ff2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10e6b6_u32)?;
        // 00100ff2: subq.w #$1, $4435c4.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4435c4_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00100ff8: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00100ffa: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00100ffe: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00101004: moveq #$4b, d1
        e.step()?;
        r.put_data(1, 32, 0x4b_u32);
        r.move_flags(0x4b_u32, 32);
        // 00101006: muls.w (a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010100a: move.w #$2710, d0
        e.step()?;
        let value = 0x2710_u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010100e: sub.w d1, d0
        e.step()?;
        let source = r.data(1, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101010: move.w d0, $4434b6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101016: move.l #$4b0b6d, -(a7)
        e.step()?;
        let value = 0x4b0b6d_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010101c: move.w $4434b6.l, -(a7)
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101022: bsr.l $13f65e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101028_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13f65e_u32)?;
        // 00101028: move.w d0, $4434b6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010102e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101030: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101034: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010103a: move.w (a0, d0.l), -$d8c(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff274_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101040: move.w $4434b6.l, -(a7)
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101046: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10104c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 0010104c: cmp.w -$d8c(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff274_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101050: lea.l $10(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0x10_u32);
        r.put_address(7, value);
        // 00101054: bgt.b $1010a2
        e.step()?;
        pc = if r.greater()? {
            0x1010a2_u32
        } else {
            0x101056_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_18_unit::attack_body_block_18;

mod attack_body_block_19_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_19(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101056: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010105c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101060: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101066: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101068: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010106a: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010106e: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 00101074: adda.l d0, a1
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(1)?;
        let value = destination.wrapping_add(source);
        r.put_address(1, value);
        // 00101076: move.w $f4(a1), d0
        e.step()?;
        let value_address = r.address(1)?.wrapping_add(0xf4_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010107a: add.w $b62(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0xb62_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010107e: cmpi.w #$5f, d0
        e.step()?;
        let source = 0x5f_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101082: bge.b $1010a2
        e.step()?;
        pc = if r.greater_equal()? {
            0x1010a2_u32
        } else {
            0x101084_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_19_unit::attack_body_block_19;
mod attack_body_block_1a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_1a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101084: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101086: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010108a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101090: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101092: addq.w #$1, $f4(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xf4_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00101096: move.w #$8e, -(a7)
        e.step()?;
        let value = 0x8e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010109a: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1010a0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 001010a0: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1010a2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_1a_unit::attack_body_block_1a;

mod attack_body_block_1b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_1b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001010a2: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001010a4: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001010a8: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001010ae: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001010b0: addq.l #$1, $a6a(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xa6a_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 001010b4: bsr.l $16043e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1010ba_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16043e_u32)?;
        // 001010ba: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001010bc: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_1b_unit::attack_body_block_1b;

mod attack_body_block_1c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_1c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001010c0: move.l #$3c1770, -(a7)
        e.step()?;
        let value = 0x3c1770_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001010c6: move.l #$1e012c, -(a7)
        e.step()?;
        let value = 0x1e012c_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001010cc: pea.l $404088.l
        e.step()?;
        let value = 0x404088_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001010d2: bsr.l $123d1c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1010d8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x123d1c_u32)?;
        // 001010d8: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001010da: lea.l $c(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xc_u32);
        r.put_address(7, value);
        // 001010de: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_1c_unit::attack_body_block_1c;

mod attack_body_block_1d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_1d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001010e2: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001010e8: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 001010ea: clr.w -$d6a(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffff296_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001010ee: clr.w -$d72(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffff28e_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001010f2: clr.w -$d70(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffff290_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001010f6: clr.w -$d6e(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffff292_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001010fa: moveq #$0, d5
        e.step()?;
        r.put_data(5, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001010fc: move.b d5, d7
        e.step()?;
        let value = r.data(5, 8)?;
        r.put_data(7, 8, value);
        r.move_flags(value, 8);
        // 001010fe: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101100: move.b d5, d0
        e.step()?;
        let value = r.data(5, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00101102: move.w d0, $4434b8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101108: move.b #$1, $438582.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x438582_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00101110: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101112: moveq #$1, d0
        e.step()?;
        r.put_data(0, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 00101114: move.w d0, $43877e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x43877e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010111a: clr.w $438786.l
        e.step()?;
        let value = 0;
        let destination_address = 0x438786_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101120: clr.w $4427a6.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4427a6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101126: lea.l $443788.l, a0
        e.step()?;
        let value = 0x443788_u32;
        r.put_address(0, value);
        // 0010112c: lea.l $21c2(a0), a0
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x21c2_u32);
        r.put_address(0, value);
        // 00101130: lea.l $44374e.l, a1
        e.step()?;
        let value = 0x44374e_u32;
        r.put_address(1, value);
        // 00101136: moveq #$d, d0
        e.step()?;
        r.put_data(0, 32, 0xd_u32);
        r.move_flags(0xd_u32, 32);
        pc = 0x101138_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_1d_unit::attack_body_block_1d;

mod attack_body_block_1e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_1e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101138: move.l (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = e.l(value_address)?;
        r.offset_address(0, 4)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.offset_address(1, 4)?;
        r.move_flags(value, 32);
        // 0010113a: dbra d0, $101138
        e.step()?;
        let counter = r.data(0, 16)?;
        let value = counter.wrapping_sub(1) & 0xffff;
        r.put_data(0, 16, value);
        pc = if value != 0xffff {
            0x101138_u32
        } else {
            0x10113e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_1e_unit::attack_body_block_1e;

mod attack_body_block_1f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_1f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010113e: move.w (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(0, 2)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.offset_address(1, 2)?;
        r.move_flags(value, 16);
        // 00101140: clr.w $4471ea.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4471ea_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101146: moveq #$0, d4
        e.step()?;
        r.put_data(4, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101148: bra.b $10117a
        e.step()?;
        pc = 0x10117a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_1f_unit::attack_body_block_1f;

mod attack_body_block_20_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_20(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010114a: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010114c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101150: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101156: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101158: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0010115c: beq.b $101178
        e.step()?;
        pc = if r.equal()? {
            0x101178_u32
        } else {
            0x10115e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_20_unit::attack_body_block_20;

mod attack_body_block_21_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_21(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010115e: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101160: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101164: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010116a: tst.w $10(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010116e: bne.b $101178
        e.step()?;
        pc = if r.not_equal()? {
            0x101178_u32
        } else {
            0x101170_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_21_unit::attack_body_block_21;

mod attack_body_block_22_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_22(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101170: move.w #$1, $4471ea.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x4471ea_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x101178_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_22_unit::attack_body_block_22;

mod attack_body_block_23_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_23(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101178: addq.w #$1, d4
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(4, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(4, 16, value);
        pc = 0x10117a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_23_unit::attack_body_block_23;

mod attack_body_block_24_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_24(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010117a: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00101180: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101182: cmp.w d0, d4
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101184: ble.b $10114a
        e.step()?;
        pc = if r.less_equal()? {
            0x10114a_u32
        } else {
            0x101186_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_24_unit::attack_body_block_24;

mod attack_body_block_25_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_25(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101186: tst.w $4471ea.l
        e.step()?;
        let value_address = 0x4471ea_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010118c: bne.b $10119e
        e.step()?;
        pc = if r.not_equal()? {
            0x10119e_u32
        } else {
            0x10118e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_25_unit::attack_body_block_25;

mod attack_body_block_26_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_26(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010118e: move.w $4435c4.l, $44374c.l
        e.step()?;
        let value_address = 0x4435c4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x44374c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101198: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010119a: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_26_unit::attack_body_block_26;

mod attack_body_block_27_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_27(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010119e: move.l #$10000, -(a7)
        e.step()?;
        let value = 0x10000_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001011a4: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001011aa: bsr.l $120c14
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1011b0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x120c14_u32)?;
        // 001011b0: tst.w -$b78(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff488_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001011b4: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001011b6: beq.w $1012fe
        e.step()?;
        pc = if r.equal()? {
            0x1012fe_u32
        } else {
            0x1011ba_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_27_unit::attack_body_block_27;

mod attack_body_block_28_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_28(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001011ba: clr.b -$d6b(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffff295_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001011be: move.w -$b78(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff488_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001011c2: bsr.l $130d38
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1011c8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x130d38_u32)?;
        // 001011c8: cmpi.w #$44c, $453794.l
        e.step()?;
        let source = 0x44c_u32;
        let destination_address = 0x453794_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001011d0: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001011d2: ble.w $10129e
        e.step()?;
        pc = if r.less_equal()? {
            0x10129e_u32
        } else {
            0x1011d6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_28_unit::attack_body_block_28;

mod attack_body_block_29_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_29(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001011d6: move.w $453794.l, -(a7)
        e.step()?;
        let value_address = 0x453794_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001011dc: bsr.l $13145c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1011e2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13145c_u32)?;
        // 001011e2: cmpi.b #$9, $443762.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x443762_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001011ea: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001011ec: bne.w $10129e
        e.step()?;
        pc = if r.not_equal()? {
            0x10129e_u32
        } else {
            0x1011f0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_29_unit::attack_body_block_29;
mod attack_body_block_2a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_2a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001011f0: tst.w -$bb2(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff44e_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001011f4: beq.b $10124e
        e.step()?;
        pc = if r.equal()? {
            0x10124e_u32
        } else {
            0x1011f6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_2a_unit::attack_body_block_2a;

mod attack_body_block_2b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_2b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001011f6: move.w -$c36(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff3ca_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001011fa: add.w $447d96.l, d0
        e.step()?;
        let source_address = 0x447d96_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101200: cmpi.w #$8, d0
        e.step()?;
        let source = 0x8_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101204: bge.b $101248
        e.step()?;
        pc = if r.greater_equal()? {
            0x101248_u32
        } else {
            0x101206_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_2b_unit::attack_body_block_2b;

mod attack_body_block_2c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_2c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101206: move.w d6, -(a7)
        e.step()?;
        let value = r.data(6, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101208: bsr.l $15ef50
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10120e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15ef50_u32)?;
        // 0010120e: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00101210: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101212: beq.b $101242
        e.step()?;
        pc = if r.equal()? {
            0x101242_u32
        } else {
            0x101214_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_2c_unit::attack_body_block_2c;

mod attack_body_block_2d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_2d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101214: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101216: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010121a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101220: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101222: lea.l -$c42(a6), a1
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff3be_u32);
        r.put_address(1, value);
        // 00101226: move.w #$30f, d0
        e.step()?;
        let value = 0x30f_u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        pc = 0x10122a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_2d_unit::attack_body_block_2d;

mod attack_body_block_2e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_2e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010122a: move.l (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = e.l(value_address)?;
        r.offset_address(0, 4)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.offset_address(1, 4)?;
        r.move_flags(value, 32);
        // 0010122c: dbra d0, $10122a
        e.step()?;
        let counter = r.data(0, 16)?;
        let value = counter.wrapping_sub(1) & 0xffff;
        r.put_data(0, 16, value);
        pc = if value != 0xffff {
            0x10122a_u32
        } else {
            0x101230_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_2e_unit::attack_body_block_2e;

mod attack_body_block_2f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_2f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101230: move.w (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(0, 2)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.offset_address(1, 2)?;
        r.move_flags(value, 16);
        // 00101232: move.w -$b78(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff488_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101236: bsr.l $130d38
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10123c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x130d38_u32)?;
        // 0010123c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010123e: bra.w $10130a
        e.step()?;
        pc = 0x10130a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_2f_unit::attack_body_block_2f;

mod attack_body_block_30_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_30(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101242: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101244: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_30_unit::attack_body_block_30;

mod attack_body_block_31_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_31(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101248: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010124a: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_31_unit::attack_body_block_31;

mod attack_body_block_32_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_32(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010124e: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00101254: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010125a: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00101260: cmp.w (a0, d0.w * 2), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101264: bne.b $101298
        e.step()?;
        pc = if r.not_equal()? {
            0x101298_u32
        } else {
            0x101266_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_32_unit::attack_body_block_32;

mod attack_body_block_33_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_33(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101266: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010126c: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 0010126e: pea.l $21.w
        e.step()?;
        let value = 0x21_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101272: MacToolbox 0xa862
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa862_u32 as u16))?;
        // 00101274: pea.l $1e.w
        e.step()?;
        let value = 0x1e_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101278: MacToolbox 0xa863
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa863_u32 as u16))?;
        // 0010127a: move.w #$27, -(a7)
        e.step()?;
        let value = 0x27_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010127e: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101284_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 00101284: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010128a: bsr.l $11f1a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101290_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11f1a8_u32)?;
        // 00101290: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101292: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101294: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_33_unit::attack_body_block_33;

mod attack_body_block_34_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_34(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101298: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010129a: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_34_unit::attack_body_block_34;

mod attack_body_block_35_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_35(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010129e: cmpi.w #$fff6, $453792.l
        e.step()?;
        let source = 0xfff6_u32;
        let destination_address = 0x453792_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001012a6: bne.b $10130a
        e.step()?;
        pc = if r.not_equal()? {
            0x10130a_u32
        } else {
            0x1012a8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_35_unit::attack_body_block_35;

mod attack_body_block_36_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_36(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001012a8: cmpi.w #$1, $453794.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x453794_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001012b0: bne.b $1012c8
        e.step()?;
        pc = if r.not_equal()? {
            0x1012c8_u32
        } else {
            0x1012b2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_36_unit::attack_body_block_36;

mod attack_body_block_37_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_37(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001012b2: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001012b4: move.w $453798.l, -(a7)
        e.step()?;
        let value_address = 0x453798_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001012ba: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1012c0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 001012c0: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 001012c2: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001012c4: beq.b $1012e6
        e.step()?;
        pc = if r.equal()? {
            0x1012e6_u32
        } else {
            0x1012c6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_37_unit::attack_body_block_37;

mod attack_body_block_38_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_38(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001012c6: bra.b $10130a
        e.step()?;
        pc = 0x10130a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_38_unit::attack_body_block_38;

mod attack_body_block_39_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_39(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001012c8: cmpi.w #$2, $453794.l
        e.step()?;
        let source = 0x2_u32;
        let destination_address = 0x453794_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001012d0: bne.b $1012e6
        e.step()?;
        pc = if r.not_equal()? {
            0x1012e6_u32
        } else {
            0x1012d2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_39_unit::attack_body_block_39;
mod attack_body_block_3a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_3a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001012d2: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001012d6: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1012dc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001012dc: cmp.w $453798.l, d0
        e.step()?;
        let source_address = 0x453798_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001012e2: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001012e4: bgt.b $10130a
        e.step()?;
        pc = if r.greater()? {
            0x10130a_u32
        } else {
            0x1012e6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_3a_unit::attack_body_block_3a;

mod attack_body_block_3b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_3b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001012e6: move.w #$1, -$d70(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff290_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001012ec: move.w $45379a.l, -$d72(a6)
        e.step()?;
        let value_address = 0x45379a_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff28e_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001012f4: move.w $453796.l, -$d6a(a6)
        e.step()?;
        let value_address = 0x453796_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff296_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001012fc: bra.b $10130a
        e.step()?;
        pc = 0x10130a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_3b_unit::attack_body_block_3b;

mod attack_body_block_3c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_3c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001012fe: move.b #$1, -$d6b(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff295_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00101304: bsr.l $136398
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10130a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x136398_u32)?;
        pc = 0x10130a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_3c_unit::attack_body_block_3c;

mod attack_body_block_3d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_3d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010130a: moveq #$21, d0
        e.step()?;
        r.put_data(0, 32, 0x21_u32);
        r.move_flags(0x21_u32, 32);
        // 0010130c: add.w -$c3c(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff3c4_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101310: add.w -$e8(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff18_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101314: add.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 00101316: tst.w -$be0(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff420_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010131a: beq.b $10132a
        e.step()?;
        pc = if r.equal()? {
            0x10132a_u32
        } else {
            0x10131c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_3d_unit::attack_body_block_3d;

mod attack_body_block_3e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_3e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010131c: move.w -$be0(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff420_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101320: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101326_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00101326: sub.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 00101328: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10132a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_3e_unit::attack_body_block_3e;

mod attack_body_block_3f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_3f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010132a: tst.w -$bd8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff428_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010132e: beq.b $101334
        e.step()?;
        pc = if r.equal()? {
            0x101334_u32
        } else {
            0x101330_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_3f_unit::attack_body_block_3f;

mod attack_body_block_40_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_40(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101330: subi.w #$f, d5
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        pc = 0x101334_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_40_unit::attack_body_block_40;

mod attack_body_block_41_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_41(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101334: tst.w -$bae(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff452_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101338: beq.b $10133e
        e.step()?;
        pc = if r.equal()? {
            0x10133e_u32
        } else {
            0x10133a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_41_unit::attack_body_block_41;

mod attack_body_block_42_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_42(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010133a: subi.w #$f, d5
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        pc = 0x10133e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_42_unit::attack_body_block_42;

mod attack_body_block_43_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_43(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010133e: tst.w -$baa(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff456_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101342: beq.b $101348
        e.step()?;
        pc = if r.equal()? {
            0x101348_u32
        } else {
            0x101344_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_43_unit::attack_body_block_43;

mod attack_body_block_44_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_44(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101344: subi.w #$a, d5
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        pc = 0x101348_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_44_unit::attack_body_block_44;

mod attack_body_block_45_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_45(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101348: tst.w -$b9c(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff464_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010134c: beq.b $10135c
        e.step()?;
        pc = if r.equal()? {
            0x10135c_u32
        } else {
            0x10134e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_45_unit::attack_body_block_45;

mod attack_body_block_46_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_46(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010134e: move.w -$b9c(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff464_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101352: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101358_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00101358: sub.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 0010135a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10135c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_46_unit::attack_body_block_46;

mod attack_body_block_47_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_47(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010135c: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101360: ble.b $1013a6
        e.step()?;
        pc = if r.less_equal()? {
            0x1013a6_u32
        } else {
            0x101362_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_47_unit::attack_body_block_47;

mod attack_body_block_48_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_48(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101362: moveq #$0, d4
        e.step()?;
        r.put_data(4, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101364: bra.b $1013a0
        e.step()?;
        pc = 0x1013a0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_48_unit::attack_body_block_48;

mod attack_body_block_49_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_49(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101366: lea.l -$d62(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff29e_u32);
        r.put_address(0, value);
        // 0010136a: adda.w d4, a0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.address(0)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010136c: tst.b $a8(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xa8_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00101370: beq.b $10139e
        e.step()?;
        pc = if r.equal()? {
            0x10139e_u32
        } else {
            0x101372_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_49_unit::attack_body_block_49;
mod attack_body_block_4a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_4a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101372: movea.w d4, a0
        e.step()?;
        let value = r.data(4, 16)?;
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00101374: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101376: add.l d0, d0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00101378: lea.l -$c42(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff3be_u32);
        r.put_address(0, value);
        // 0010137c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010137e: add.w $ae(a0), d5
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0xae_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 00101382: movea.w d4, a0
        e.step()?;
        let value = r.data(4, 16)?;
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00101384: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101386: add.l d0, d0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00101388: lea.l -$c42(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff3be_u32);
        r.put_address(0, value);
        // 0010138c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010138e: movea.w $ae(a0), a1
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xae_u32);
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00101392: move.l a1, d0
        e.step()?;
        let value = r.address(1)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101394: divs.w #$5, d0
        e.step()?;
        let divisor = 0x5_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00101398: add.w d0, $4434b8.l
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = 0x4434b8_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x10139e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_4a_unit::attack_body_block_4a;

mod attack_body_block_4b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_4b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010139e: addq.w #$1, d4
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(4, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(4, 16, value);
        pc = 0x1013a0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_4b_unit::attack_body_block_4b;

mod attack_body_block_4c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_4c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013a0: cmpi.w #$8, d4
        e.step()?;
        let source = 0x8_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001013a4: blt.b $101366
        e.step()?;
        pc = if r.less()? {
            0x101366_u32
        } else {
            0x1013a6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_4c_unit::attack_body_block_4c;

mod attack_body_block_4d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_4d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013a6: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001013aa: ble.w $101470
        e.step()?;
        pc = if r.less_equal()? {
            0x101470_u32
        } else {
            0x1013ae_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_4d_unit::attack_body_block_4d;

mod attack_body_block_4e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_4e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013ae: sub.w -$d5a(a6), d5
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff2a6_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 001013b2: move.b -$ca9(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff357_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001013b6: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001013b8: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001013ba: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1013c0_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001013c0: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001013c2: sub.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 001013c4: tst.b -$cae(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff352_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001013c8: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001013ca: beq.b $1013de
        e.step()?;
        pc = if r.equal()? {
            0x1013de_u32
        } else {
            0x1013cc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_4e_unit::attack_body_block_4e;

mod attack_body_block_4f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_4f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013cc: move.b -$cae(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff352_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001013d0: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001013d2: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001013d4: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1013da_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001013da: add.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 001013dc: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1013de_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_4f_unit::attack_body_block_4f;

mod attack_body_block_50_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_50(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013de: tst.b -$caa(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff356_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001013e2: beq.b $1013e8
        e.step()?;
        pc = if r.equal()? {
            0x1013e8_u32
        } else {
            0x1013e4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_50_unit::attack_body_block_50;

mod attack_body_block_51_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_51(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013e4: addi.w #$f, d5
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        pc = 0x1013e8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_51_unit::attack_body_block_51;

mod attack_body_block_52_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_52(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013e8: tst.b -$c95(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff36b_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001013ec: beq.b $1013f2
        e.step()?;
        pc = if r.equal()? {
            0x1013f2_u32
        } else {
            0x1013ee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_52_unit::attack_body_block_52;

mod attack_body_block_53_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_53(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013ee: addi.w #$f, d5
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        pc = 0x1013f2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_53_unit::attack_body_block_53;

mod attack_body_block_54_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_54(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013f2: tst.b -$c93(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff36d_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001013f6: beq.b $1013fc
        e.step()?;
        pc = if r.equal()? {
            0x1013fc_u32
        } else {
            0x1013f8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_54_unit::attack_body_block_54;

mod attack_body_block_55_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_55(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013f8: addi.w #$a, d5
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        pc = 0x1013fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_55_unit::attack_body_block_55;

mod attack_body_block_56_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_56(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001013fc: tst.b -$c8b(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff375_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00101400: beq.b $101414
        e.step()?;
        pc = if r.equal()? {
            0x101414_u32
        } else {
            0x101402_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_56_unit::attack_body_block_56;

mod attack_body_block_57_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_57(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101402: move.b -$c8b(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff375_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00101406: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101408: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010140a: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101410_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00101410: add.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 00101412: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x101414_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_57_unit::attack_body_block_57;

mod attack_body_block_58_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_58(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101414: tst.b -$c8a(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff376_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00101418: beq.b $10142c
        e.step()?;
        pc = if r.equal()? {
            0x10142c_u32
        } else {
            0x10141a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_58_unit::attack_body_block_58;

mod attack_body_block_59_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_59(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010141a: move.b -$c8a(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff376_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010141e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101420: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101422: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101428_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00101428: sub.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 0010142a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10142c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_59_unit::attack_body_block_59;
mod attack_body_block_5a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_5a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010142c: moveq #$5, d0
        e.step()?;
        r.put_data(0, 32, 0x5_u32);
        r.move_flags(0x5_u32, 32);
        // 0010142e: muls.w $442576.l, d0
        e.step()?;
        let source_address = 0x442576_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101434: moveq #$5a, d1
        e.step()?;
        r.put_data(1, 32, 0x5a_u32);
        r.move_flags(0x5a_u32, 32);
        // 00101436: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00101438: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010143a: moveq #$5, d0
        e.step()?;
        r.put_data(0, 32, 0x5_u32);
        r.move_flags(0x5_u32, 32);
        // 0010143c: muls.w $442576.l, d0
        e.step()?;
        let source_address = 0x442576_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101442: moveq #$f, d1
        e.step()?;
        r.put_data(1, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00101444: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00101446: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101448: move.w d5, -(a7)
        e.step()?;
        let value = r.data(5, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010144a: bsr.l $13f65e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101450_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13f65e_u32)?;
        // 00101450: move.w d0, d5
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(5, 16, value);
        r.move_flags(value, 16);
        // 00101452: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101456: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10145c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 0010145c: cmp.w d5, d0
        e.step()?;
        let source = r.data(5, 16)?;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010145e: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101460: ble.b $10146a
        e.step()?;
        pc = if r.less_equal()? {
            0x10146a_u32
        } else {
            0x101462_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_5a_unit::attack_body_block_5a;

mod attack_body_block_5b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_5b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101462: tst.b -$caf(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff351_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00101466: beq.w $1015d4
        e.step()?;
        pc = if r.equal()? {
            0x1015d4_u32
        } else {
            0x10146a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_5b_unit::attack_body_block_5b;

mod attack_body_block_5c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_5c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010146a: moveq #$1, d7
        e.step()?;
        r.put_data(7, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 0010146c: bra.w $1015d4
        e.step()?;
        pc = 0x1015d4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_5c_unit::attack_body_block_5c;

mod attack_body_block_5d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_5d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101470: cmpi.w #$ffff, d3
        e.step()?;
        let source = 0xffff_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101474: ble.w $1015d4
        e.step()?;
        pc = if r.less_equal()? {
            0x1015d4_u32
        } else {
            0x101478_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_5d_unit::attack_body_block_5d;

mod attack_body_block_5e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_5e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101478: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010147a: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010147e: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101484: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101486: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101488: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010148c: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 00101492: move.w $a(a1, d0.l), d0
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0xa_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101496: add.w $b5e(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0xb5e_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010149a: sub.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 0010149c: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010149e: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001014a2: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001014a8: move.w $6c(a0, d0.l), -(a7)
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
        // 001014ac: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1014b2_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001014b2: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001014b4: sub.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 001014b6: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001014b8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001014bc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001014c2: tst.w $62(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x62_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001014c6: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001014c8: beq.b $1014e4
        e.step()?;
        pc = if r.equal()? {
            0x1014e4_u32
        } else {
            0x1014ca_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_5e_unit::attack_body_block_5e;

mod attack_body_block_5f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_5f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001014ca: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001014cc: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001014d0: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001014d6: move.w $62(a0, d0.l), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x62_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001014da: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1014e0_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001014e0: add.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 001014e2: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1014e4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_5f_unit::attack_body_block_5f;

mod attack_body_block_60_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_60(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001014e4: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001014e6: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001014ea: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001014f0: tst.w $6a(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x6a_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001014f4: beq.b $1014fa
        e.step()?;
        pc = if r.equal()? {
            0x1014fa_u32
        } else {
            0x1014f6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_60_unit::attack_body_block_60;

mod attack_body_block_61_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_61(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001014f6: addi.w #$f, d5
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        pc = 0x1014fa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_61_unit::attack_body_block_61;

mod attack_body_block_62_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_62(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001014fa: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001014fc: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101500: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101506: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101508: tst.w $98(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x98_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010150c: beq.b $101512
        e.step()?;
        pc = if r.equal()? {
            0x101512_u32
        } else {
            0x10150e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_62_unit::attack_body_block_62;

mod attack_body_block_63_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_63(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010150e: addi.w #$a, d5
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        pc = 0x101512_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_63_unit::attack_body_block_63;

mod attack_body_block_64_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_64(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101512: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101514: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101518: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010151e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101520: tst.w $94(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101524: beq.b $10152a
        e.step()?;
        pc = if r.equal()? {
            0x10152a_u32
        } else {
            0x101526_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_64_unit::attack_body_block_64;

mod attack_body_block_65_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_65(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101526: addi.w #$f, d5
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        pc = 0x10152a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_65_unit::attack_body_block_65;

mod attack_body_block_66_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_66(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010152a: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010152c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101530: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101536: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101538: tst.w $a8(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xa8_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010153c: beq.b $10155a
        e.step()?;
        pc = if r.equal()? {
            0x10155a_u32
        } else {
            0x10153e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_66_unit::attack_body_block_66;

mod attack_body_block_67_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_67(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010153e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101540: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101544: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010154a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010154c: move.w $a8(a0), -(a7)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xa8_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101550: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101556_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00101556: add.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 00101558: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10155a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_67_unit::attack_body_block_67;

mod attack_body_block_68_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_68(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010155a: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010155c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101560: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101566: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101568: tst.w $aa(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xaa_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010156c: beq.b $10158a
        e.step()?;
        pc = if r.equal()? {
            0x10158a_u32
        } else {
            0x10156e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_68_unit::attack_body_block_68;

mod attack_body_block_69_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_69(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010156e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101570: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101574: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010157a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010157c: move.w $aa(a0), -(a7)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xaa_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101580: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101586_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00101586: sub.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        // 00101588: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10158a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_69_unit::attack_body_block_69;
mod attack_body_block_6a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_6a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010158a: moveq #$5, d0
        e.step()?;
        r.put_data(0, 32, 0x5_u32);
        r.move_flags(0x5_u32, 32);
        // 0010158c: muls.w $442576.l, d0
        e.step()?;
        let source_address = 0x442576_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101592: moveq #$5a, d1
        e.step()?;
        r.put_data(1, 32, 0x5a_u32);
        r.move_flags(0x5a_u32, 32);
        // 00101594: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00101596: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101598: moveq #$5, d0
        e.step()?;
        r.put_data(0, 32, 0x5_u32);
        r.move_flags(0x5_u32, 32);
        // 0010159a: muls.w $442576.l, d0
        e.step()?;
        let source_address = 0x442576_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001015a0: moveq #$f, d1
        e.step()?;
        r.put_data(1, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 001015a2: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001015a4: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001015a6: move.w d5, -(a7)
        e.step()?;
        let value = r.data(5, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001015a8: bsr.l $13f65e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1015ae_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13f65e_u32)?;
        // 001015ae: move.w d0, d5
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(5, 16, value);
        r.move_flags(value, 16);
        // 001015b0: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001015b4: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1015ba_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001015ba: cmp.w d5, d0
        e.step()?;
        let source = r.data(5, 16)?;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001015bc: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001015be: ble.b $1015d2
        e.step()?;
        pc = if r.less_equal()? {
            0x1015d2_u32
        } else {
            0x1015c0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_6a_unit::attack_body_block_6a;

mod attack_body_block_6b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_6b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001015c0: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001015c2: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001015c6: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001015cc: tst.w $60(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x60_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001015d0: beq.b $1015d4
        e.step()?;
        pc = if r.equal()? {
            0x1015d4_u32
        } else {
            0x1015d2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_6b_unit::attack_body_block_6b;

mod attack_body_block_6c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_6c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001015d2: moveq #$1, d7
        e.step()?;
        r.put_data(7, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        pc = 0x1015d4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_6c_unit::attack_body_block_6c;

mod attack_body_block_6d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_6d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001015d4: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001015d8: ble.b $101622
        e.step()?;
        pc = if r.less_equal()? {
            0x101622_u32
        } else {
            0x1015da_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_6d_unit::attack_body_block_6d;

mod attack_body_block_6e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_6e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001015da: cmpi.w #$ffff, -$d56(a6)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff2aa_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001015e0: bne.b $101622
        e.step()?;
        pc = if r.not_equal()? {
            0x101622_u32
        } else {
            0x1015e2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_6e_unit::attack_body_block_6e;

mod attack_body_block_6f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_6f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001015e2: tst.b d7
        e.step()?;
        let value = r.data(7, 8)?;
        r.move_flags(value, 8);
        // 001015e4: beq.b $101622
        e.step()?;
        pc = if r.equal()? {
            0x101622_u32
        } else {
            0x1015e6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_6f_unit::attack_body_block_6f;

mod attack_body_block_70_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_70(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001015e6: cmpi.w #$ffff, $4537cc.l
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = 0x4537cc_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001015ee: beq.b $101622
        e.step()?;
        pc = if r.equal()? {
            0x101622_u32
        } else {
            0x1015f0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_70_unit::attack_body_block_70;

mod attack_body_block_71_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_71(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001015f0: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001015f2: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001015f6: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001015fc: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001015fe: tst.w $ca(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xca_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101602: beq.b $101622
        e.step()?;
        pc = if r.equal()? {
            0x101622_u32
        } else {
            0x101604_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_71_unit::attack_body_block_71;

mod attack_body_block_72_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_72(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101604: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010160a: moveq #$0, d7
        e.step()?;
        r.put_data(7, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010160c: move.w d6, -(a7)
        e.step()?;
        let value = r.data(6, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010160e: move.w #$ffce, -(a7)
        e.step()?;
        let value = 0xffce_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101612: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101614: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10161a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 0010161a: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010161c: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010161e: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_72_unit::attack_body_block_72;

mod attack_body_block_73_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_73(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101622: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101626: ble.b $101670
        e.step()?;
        pc = if r.less_equal()? {
            0x101670_u32
        } else {
            0x101628_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_73_unit::attack_body_block_73;

mod attack_body_block_74_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_74(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101628: cmpi.w #$fffe, -$d56(a6)
        e.step()?;
        let source = 0xfffe_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff2aa_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010162e: bne.b $101670
        e.step()?;
        pc = if r.not_equal()? {
            0x101670_u32
        } else {
            0x101630_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_74_unit::attack_body_block_74;

mod attack_body_block_75_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_75(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101630: tst.b d7
        e.step()?;
        let value = r.data(7, 8)?;
        r.move_flags(value, 8);
        // 00101632: beq.b $101670
        e.step()?;
        pc = if r.equal()? {
            0x101670_u32
        } else {
            0x101634_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_75_unit::attack_body_block_75;

mod attack_body_block_76_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_76(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101634: cmpi.w #$fffe, $4537cc.l
        e.step()?;
        let source = 0xfffe_u32;
        let destination_address = 0x4537cc_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010163c: beq.b $101670
        e.step()?;
        pc = if r.equal()? {
            0x101670_u32
        } else {
            0x10163e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_76_unit::attack_body_block_76;

mod attack_body_block_77_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_77(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010163e: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101640: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101644: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010164a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010164c: tst.w $ca(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xca_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101650: beq.b $101670
        e.step()?;
        pc = if r.equal()? {
            0x101670_u32
        } else {
            0x101652_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_77_unit::attack_body_block_77;

mod attack_body_block_78_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_78(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101652: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101658: moveq #$0, d7
        e.step()?;
        r.put_data(7, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010165a: move.w d6, -(a7)
        e.step()?;
        let value = r.data(6, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010165c: move.w #$ffcd, -(a7)
        e.step()?;
        let value = 0xffcd_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101660: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101662: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101668_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00101668: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0010166a: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010166c: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_78_unit::attack_body_block_78;

mod attack_body_block_79_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_79(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101670: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101674: ble.b $1016be
        e.step()?;
        pc = if r.less_equal()? {
            0x1016be_u32
        } else {
            0x101676_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_79_unit::attack_body_block_79;
mod attack_body_block_7a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_7a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101676: tst.w -$d56(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff2aa_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010167a: beq.b $1016be
        e.step()?;
        pc = if r.equal()? {
            0x1016be_u32
        } else {
            0x10167c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_7a_unit::attack_body_block_7a;

mod attack_body_block_7b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_7b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010167c: cmpi.w #$ffff, -$d56(a6)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff2aa_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00101682: beq.b $1016be
        e.step()?;
        pc = if r.equal()? {
            0x1016be_u32
        } else {
            0x101684_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_7b_unit::attack_body_block_7b;

mod attack_body_block_7c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_7c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101684: cmpi.w #$fffe, -$d56(a6)
        e.step()?;
        let source = 0xfffe_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff2aa_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010168a: beq.b $1016be
        e.step()?;
        pc = if r.equal()? {
            0x1016be_u32
        } else {
            0x10168c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_7c_unit::attack_body_block_7c;

mod attack_body_block_7d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_7d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010168c: tst.b d7
        e.step()?;
        let value = r.data(7, 8)?;
        r.move_flags(value, 8);
        // 0010168e: beq.b $1016be
        e.step()?;
        pc = if r.equal()? {
            0x1016be_u32
        } else {
            0x101690_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_7d_unit::attack_body_block_7d;

mod attack_body_block_7e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_7e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101690: move.w $4537ce.l, d0
        e.step()?;
        let value_address = 0x4537ce_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101696: addi.w #$fc00, d0
        e.step()?;
        let source = 0xfc00_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010169a: cmp.w -$d56(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff2aa_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010169e: beq.b $1016be
        e.step()?;
        pc = if r.equal()? {
            0x1016be_u32
        } else {
            0x1016a0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_7e_unit::attack_body_block_7e;

mod attack_body_block_7f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_7f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001016a0: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001016a6: moveq #$0, d7
        e.step()?;
        r.put_data(7, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001016a8: move.w d6, -(a7)
        e.step()?;
        let value = r.data(6, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001016aa: move.w #$ffd3, -(a7)
        e.step()?;
        let value = 0xffd3_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001016ae: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001016b0: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1016b6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001016b6: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001016b8: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001016ba: bra.w $101ea4
        e.step()?;
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_7f_unit::attack_body_block_7f;

mod attack_body_block_80_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_80(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001016be: tst.b d7
        e.step()?;
        let value = r.data(7, 8)?;
        r.move_flags(value, 8);
        // 001016c0: beq.w $101e62
        e.step()?;
        pc = if r.equal()? {
            0x101e62_u32
        } else {
            0x1016c4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_80_unit::attack_body_block_80;

mod attack_body_block_81_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_81(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001016c4: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001016c8: ble.b $1016e6
        e.step()?;
        pc = if r.less_equal()? {
            0x1016e6_u32
        } else {
            0x1016ca_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_81_unit::attack_body_block_81;

mod attack_body_block_82_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_82(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001016ca: tst.b -$c91(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff36f_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001016ce: beq.b $1016e6
        e.step()?;
        pc = if r.equal()? {
            0x1016e6_u32
        } else {
            0x1016d0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_82_unit::attack_body_block_82;

mod attack_body_block_83_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_83(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001016d0: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001016d4: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1016da_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001016da: cmpi.w #$22, d0
        e.step()?;
        let source = 0x22_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001016de: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001016e0: bge.b $1016e6
        e.step()?;
        pc = if r.greater_equal()? {
            0x1016e6_u32
        } else {
            0x1016e2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_83_unit::attack_body_block_83;

mod attack_body_block_84_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_84(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001016e2: move.w d6, d3
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(3, 16, value);
        r.move_flags(value, 16);
        // 001016e4: bra.b $10171a
        e.step()?;
        pc = 0x10171a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_84_unit::attack_body_block_84;

mod attack_body_block_85_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_85(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001016e6: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001016ea: bge.b $10171a
        e.step()?;
        pc = if r.greater_equal()? {
            0x10171a_u32
        } else {
            0x1016ec_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_85_unit::attack_body_block_85;

mod attack_body_block_86_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_86(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001016ec: cmpi.w #$ffff, d3
        e.step()?;
        let source = 0xffff_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001016f0: ble.b $10171a
        e.step()?;
        pc = if r.less_equal()? {
            0x10171a_u32
        } else {
            0x1016f2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_86_unit::attack_body_block_86;

mod attack_body_block_87_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_87(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001016f2: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001016f6: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1016fc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001016fc: cmpi.w #$22, d0
        e.step()?;
        let source = 0x22_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101700: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101702: bge.b $10171a
        e.step()?;
        pc = if r.greater_equal()? {
            0x10171a_u32
        } else {
            0x101704_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_87_unit::attack_body_block_87;

mod attack_body_block_88_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_88(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101704: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101706: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010170a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101710: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101712: tst.w $9c(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9c_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101716: beq.b $10171a
        e.step()?;
        pc = if r.equal()? {
            0x10171a_u32
        } else {
            0x101718_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_88_unit::attack_body_block_88;

mod attack_body_block_89_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_89(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101718: move.w d6, d3
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(3, 16, value);
        r.move_flags(value, 16);
        pc = 0x10171a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_89_unit::attack_body_block_89;
mod attack_body_block_8a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_8a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010171a: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101720: clr.w $44376e.l
        e.step()?;
        let value = 0;
        let destination_address = 0x44376e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101726: move.w -$ba4(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff45c_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010172a: add.w d0, $44376e.l
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = 0x44376e_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00101730: move.w $44376e.l, $443770.l
        e.step()?;
        let value_address = 0x44376e_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x443770_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010173a: tst.w -$b78(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff488_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010173e: beq.w $101b0a
        e.step()?;
        pc = if r.equal()? {
            0x101b0a_u32
        } else {
            0x101742_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_8a_unit::attack_body_block_8a;

mod attack_body_block_8b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_8b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101742: addq.w #$1, $44376e.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x44376e_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00101748: move.w -$c36(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff3ca_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010174c: add.w $447d96.l, d0
        e.step()?;
        let source_address = 0x447d96_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101752: add.w d0, $443770.l
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = 0x443770_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00101758: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010175e: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101762: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101768: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010176a: move.b $15a(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x15a_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010176e: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101770: movea.l d0, a2
        e.step()?;
        let value = r.data(0, 32)?;
        r.put_address(2, value);
        // 00101772: moveq #$64, d0
        e.step()?;
        r.put_data(0, 32, 0x64_u32);
        r.move_flags(0x64_u32, 32);
        // 00101774: add.l a2, d0
        e.step()?;
        let source = r.address(2)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00101776: move.l d0, -$d8a(a6)
        e.step()?;
        let value = r.data(0, 32)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010177a: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010177e: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101782: move.w #$280e, -(a7)
        e.step()?;
        let value = 0x280e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101786: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101788: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010178c: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101790: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101794: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101796: move.l -$d8a(a6), -$d68(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff276_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010179c: move.l #$42c80000, -$d80(a6)
        e.step()?;
        let value = 0x42c80000_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001017a4: pea.l -$d68(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017a8: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017ac: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001017b0: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001017b2: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017b6: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017ba: move.w #$1006, -(a7)
        e.step()?;
        let value = 0x1006_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001017be: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001017c0: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017c4: pea.l -$d68(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017c8: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001017cc: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001017ce: move.w $453790.l, -$d8a(a6)
        e.step()?;
        let value_address = 0x453790_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001017d6: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017da: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017de: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001017e2: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001017e4: pea.l -$d68(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017e8: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017ec: move.w #$1004, -(a7)
        e.step()?;
        let value = 0x1004_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001017f0: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001017f2: pea.l $443770.l
        e.step()?;
        let value = 0x443770_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017f8: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001017fc: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101800: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101802: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101806: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010180a: move.w #$0, -(a7)
        e.step()?;
        let value = 0x0_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010180e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101810: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101814: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101818: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010181a: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010181e: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101822: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101826: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101828: move.w -$d7a(a6), $443770.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff286_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x443770_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101830: tst.w $4537f4.l
        e.step()?;
        let value_address = 0x4537f4_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101836: beq.b $101898
        e.step()?;
        pc = if r.equal()? {
            0x101898_u32
        } else {
            0x101838_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_8b_unit::attack_body_block_8b;

mod attack_body_block_8c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_8c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101838: move.w $4537f4.l, -$d7c(a6)
        e.step()?;
        let value_address = 0x4537f4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101840: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101844: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101848: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010184c: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010184e: pea.l -$d68(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101852: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101856: move.w #$1004, -(a7)
        e.step()?;
        let value = 0x1004_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010185a: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010185c: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101860: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101864: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101868: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010186a: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010186e: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101872: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101876: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101878: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010187c: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101880: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101882: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101886: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010188a: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010188e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101890: move.b -$d87(a6), $443765.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff279_u32);
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x443765_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x101898_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_8c_unit::attack_body_block_8c;

mod attack_body_block_8d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_8d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101898: tst.w $4537a8.l
        e.step()?;
        let value_address = 0x4537a8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010189e: beq.b $101900
        e.step()?;
        pc = if r.equal()? {
            0x101900_u32
        } else {
            0x1018a0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_8d_unit::attack_body_block_8d;

mod attack_body_block_8e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_8e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001018a0: move.w $4537a8.l, -$d7c(a6)
        e.step()?;
        let value_address = 0x4537a8_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001018a8: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018ac: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018b0: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001018b4: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001018b6: pea.l -$d68(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018ba: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018be: move.w #$1004, -(a7)
        e.step()?;
        let value = 0x1004_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001018c2: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001018c4: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018c8: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018cc: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001018d0: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001018d2: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018d6: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018da: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001018de: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001018e0: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018e4: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001018e8: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001018ea: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018ee: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001018f2: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001018f6: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001018f8: move.b -$d87(a6), $443766.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff279_u32);
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x443766_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x101900_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_8e_unit::attack_body_block_8e;

mod attack_body_block_8f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_8f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101900: tst.w $4537aa.l
        e.step()?;
        let value_address = 0x4537aa_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101906: beq.b $101968
        e.step()?;
        pc = if r.equal()? {
            0x101968_u32
        } else {
            0x101908_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_8f_unit::attack_body_block_8f;

mod attack_body_block_90_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_90(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101908: move.w $4537aa.l, -$d7c(a6)
        e.step()?;
        let value_address = 0x4537aa_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101910: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101914: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101918: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010191c: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010191e: pea.l -$d68(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101922: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101926: move.w #$1004, -(a7)
        e.step()?;
        let value = 0x1004_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010192a: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010192c: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101930: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101934: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101938: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 0010193a: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010193e: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101942: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101946: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101948: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010194c: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101950: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101952: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101956: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010195a: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010195e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101960: move.b -$d87(a6), $443767.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff279_u32);
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x443767_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x101968_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_90_unit::attack_body_block_90;

mod attack_body_block_91_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_91(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101968: tst.w $4537ac.l
        e.step()?;
        let value_address = 0x4537ac_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010196e: beq.b $1019d0
        e.step()?;
        pc = if r.equal()? {
            0x1019d0_u32
        } else {
            0x101970_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_91_unit::attack_body_block_91;

mod attack_body_block_92_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_92(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101970: move.w $4537ac.l, -$d7c(a6)
        e.step()?;
        let value_address = 0x4537ac_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101978: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010197c: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101980: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101984: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101986: pea.l -$d68(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010198a: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010198e: move.w #$1004, -(a7)
        e.step()?;
        let value = 0x1004_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101992: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101994: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101998: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0010199c: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001019a0: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001019a2: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001019a6: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001019aa: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001019ae: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001019b0: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001019b4: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001019b8: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001019ba: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001019be: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001019c2: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001019c6: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001019c8: move.b -$d87(a6), $443768.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff279_u32);
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x443768_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x1019d0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_92_unit::attack_body_block_92;

mod attack_body_block_93_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_93(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001019d0: tst.w $4537ae.l
        e.step()?;
        let value_address = 0x4537ae_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001019d6: beq.b $101a38
        e.step()?;
        pc = if r.equal()? {
            0x101a38_u32
        } else {
            0x1019d8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_93_unit::attack_body_block_93;

mod attack_body_block_94_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_94(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001019d8: move.w $4537ae.l, -$d7c(a6)
        e.step()?;
        let value_address = 0x4537ae_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001019e0: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001019e4: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001019e8: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001019ec: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001019ee: pea.l -$d68(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001019f2: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001019f6: move.w #$1004, -(a7)
        e.step()?;
        let value = 0x1004_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001019fa: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 001019fc: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a00: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a04: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a08: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101a0a: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a0e: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a12: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a16: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101a18: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a1c: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a20: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101a22: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a26: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a2a: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a2e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101a30: move.b -$d87(a6), $443769.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff279_u32);
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x443769_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x101a38_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_94_unit::attack_body_block_94;

mod attack_body_block_95_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_95(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101a38: tst.w $4537b0.l
        e.step()?;
        let value_address = 0x4537b0_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101a3e: beq.b $101aa0
        e.step()?;
        pc = if r.equal()? {
            0x101aa0_u32
        } else {
            0x101a40_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_95_unit::attack_body_block_95;

mod attack_body_block_96_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_96(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101a40: move.w $4537b0.l, -$d7c(a6)
        e.step()?;
        let value_address = 0x4537b0_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a48: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a4c: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a50: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a54: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101a56: pea.l -$d68(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a5a: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a5e: move.w #$1004, -(a7)
        e.step()?;
        let value = 0x1004_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a62: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101a64: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a68: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a6c: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a70: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101a72: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a76: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a7a: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a7e: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101a80: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a84: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a88: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101a8a: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a8e: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101a92: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101a96: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101a98: move.b -$d87(a6), $44376a.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff279_u32);
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x44376a_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x101aa0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_96_unit::attack_body_block_96;

mod attack_body_block_97_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_97(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101aa0: tst.w $4537b2.l
        e.step()?;
        let value_address = 0x4537b2_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101aa6: beq.b $101b20
        e.step()?;
        pc = if r.equal()? {
            0x101b20_u32
        } else {
            0x101aa8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_97_unit::attack_body_block_97;

mod attack_body_block_98_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_98(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101aa8: move.w $4537b2.l, -$d7c(a6)
        e.step()?;
        let value_address = 0x4537b2_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101ab0: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101ab4: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101ab8: move.w #$200e, -(a7)
        e.step()?;
        let value = 0x200e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101abc: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101abe: pea.l -$d68(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101ac2: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101ac6: move.w #$1004, -(a7)
        e.step()?;
        let value = 0x1004_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101aca: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101acc: pea.l -$d7c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff284_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101ad0: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101ad4: move.w #$1010, -(a7)
        e.step()?;
        let value = 0x1010_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101ad8: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101ada: pea.l -$d80(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff280_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101ade: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101ae2: move.w #$100e, -(a7)
        e.step()?;
        let value = 0x100e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101ae6: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101ae8: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101aec: move.w #$16, -(a7)
        e.step()?;
        let value = 0x16_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101af0: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101af2: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101af6: pea.l -$d8a(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff276_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101afa: move.w #$2810, -(a7)
        e.step()?;
        let value = 0x2810_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101afe: MacToolbox 0xa9eb
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9eb_u32 as u16))?;
        // 00101b00: move.b -$d87(a6), $44376b.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff279_u32);
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x44376b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00101b08: bra.b $101b20
        e.step()?;
        pc = 0x101b20_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_98_unit::attack_body_block_98;

mod attack_body_block_99_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_99(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101b0a: addq.w #$1, $44376e.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x44376e_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00101b10: move.w -$c36(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff3ca_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101b14: add.w $447d96.l, d0
        e.step()?;
        let source_address = 0x447d96_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101b1a: add.w d0, $443770.l
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = 0x443770_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x101b20_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_99_unit::attack_body_block_99;
mod attack_body_block_9a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_9a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101b20: move.l #$10000, -(a7)
        e.step()?;
        let value = 0x10000_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00101b26: pea.l $1.w
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00101b2a: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101b2c: bsr.l $15075a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101b32_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15075a_u32)?;
        // 00101b32: move.w d0, $4434b8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101b38: tst.w $4434b8.l
        e.step()?;
        let value_address = 0x4434b8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101b3e: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 00101b42: ble.w $101e62
        e.step()?;
        pc = if r.less_equal()? {
            0x101e62_u32
        } else {
            0x101b46_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_9a_unit::attack_body_block_9a;

mod attack_body_block_9b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_9b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101b46: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101b4a: ble.b $101b62
        e.step()?;
        pc = if r.less_equal()? {
            0x101b62_u32
        } else {
            0x101b4c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_9b_unit::attack_body_block_9b;

mod attack_body_block_9c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_9c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101b4c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00101b4e: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101b50: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101b54: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00101b5a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101b5c: move.b #$1, $105(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x105_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x101b62_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_9c_unit::attack_body_block_9c;

mod attack_body_block_9d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_9d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101b62: move.w -$b48(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff4b8_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101b66: add.w -$da(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff26_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101b6a: move.w d0, -$d8c(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff274_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101b6e: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101b72: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101b78_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00101b78: cmp.w -$d8c(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff274_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101b7c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101b7e: bgt.b $101b8e
        e.step()?;
        pc = if r.greater()? {
            0x101b8e_u32
        } else {
            0x101b80_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_9d_unit::attack_body_block_9d;

mod attack_body_block_9e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_9e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101b80: move.w $4434b8.l, d0
        e.step()?;
        let value_address = 0x4434b8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101b86: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101b88: move.w d0, $4434b8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x101b8e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_9e_unit::attack_body_block_9e;

mod attack_body_block_9f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_9f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101b8e: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101b92: ble.w $101cd4
        e.step()?;
        pc = if r.less_equal()? {
            0x101cd4_u32
        } else {
            0x101b96_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_9f_unit::attack_body_block_9f;

mod attack_body_block_a0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101b96: tst.w -$d70(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff290_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101b9a: beq.b $101bec
        e.step()?;
        pc = if r.equal()? {
            0x101bec_u32
        } else {
            0x101b9c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a0_unit::attack_body_block_a0;

mod attack_body_block_a1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101b9c: moveq #$ec, d0
        e.step()?;
        r.put_data(0, 32, 0xffffffec_u32);
        r.move_flags(0xffffffec_u32, 32);
        // 00101b9e: add.w -$d6a(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff296_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101ba2: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00101ba4: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00101ba6: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00101baa: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00101bb0: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101bb2: adda.w d0, a0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.address(0)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101bb4: cmpi.b #$ff, $b2(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb2_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00101bba: ble.b $101bdc
        e.step()?;
        pc = if r.less_equal()? {
            0x101bdc_u32
        } else {
            0x101bbc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a1_unit::attack_body_block_a1;

mod attack_body_block_a2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101bbc: moveq #$ec, d0
        e.step()?;
        r.put_data(0, 32, 0xffffffec_u32);
        r.move_flags(0xffffffec_u32, 32);
        // 00101bbe: add.w -$d6a(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff296_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101bc2: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00101bc4: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00101bc6: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00101bca: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00101bd0: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101bd2: adda.w d0, a0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.address(0)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101bd4: move.b -$d71(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff28f_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00101bd8: add.b d0, $b2(a0)
        e.step()?;
        let source = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0xb2_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        pc = 0x101bdc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a2_unit::attack_body_block_a2;

mod attack_body_block_a3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101bdc: move.w d6, -(a7)
        e.step()?;
        let value = r.data(6, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101bde: move.w #$ffd8, -(a7)
        e.step()?;
        let value = 0xffd8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101be2: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101be4: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101bea_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00101bea: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x101bec_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a3_unit::attack_body_block_a3;

mod attack_body_block_a4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101bec: tst.b -$caf(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff351_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00101bf0: beq.b $101bfa
        e.step()?;
        pc = if r.equal()? {
            0x101bfa_u32
        } else {
            0x101bf2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a4_unit::attack_body_block_a4;

mod attack_body_block_a5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101bf2: move.w -$c52(a6), $4434b8.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff3ae_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x101bfa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a5_unit::attack_body_block_a5;

mod attack_body_block_a6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101bfa: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00101bfc: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101bfe: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101c02: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00101c08: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101c0a: move.b #$1, $105(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x105_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00101c10: move.w $4434b8.l, d0
        e.step()?;
        let value_address = 0x4434b8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101c16: add.w -$d6e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff292_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101c1a: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00101c1c: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00101c1e: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00101c22: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00101c28: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101c2a: sub.w d0, $110(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x110_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00101c2e: move.w $4434b8.l, d0
        e.step()?;
        let value_address = 0x4434b8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101c34: add.w -$d6e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff292_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101c38: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101c3a: move.w d6, d1
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00101c3c: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00101c40: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101c46: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101c48: add.l d0, $a46(a0)
        e.step()?;
        let source = r.data(0, 32)?;
        let destination_address = r.address(0)?.wrapping_add(0xa46_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00101c4c: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101c4e: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101c52: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101c58: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101c5a: addq.l #$1, $a4a(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xa4a_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00101c5e: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00101c60: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101c62: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101c66: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00101c6c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101c6e: tst.w $110(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x110_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101c72: bgt.w $101e1c
        e.step()?;
        pc = if r.greater()? {
            0x101e1c_u32
        } else {
            0x101c76_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a6_unit::attack_body_block_a6;

mod attack_body_block_a7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101c76: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101c78: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101c7c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101c82: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101c84: addq.l #$1, $a5a(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xa5a_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00101c88: tst.b -$d6b(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff295_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00101c8c: beq.b $101caa
        e.step()?;
        pc = if r.equal()? {
            0x101caa_u32
        } else {
            0x101c8e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a7_unit::attack_body_block_a7;

mod attack_body_block_a8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101c8e: move.w #$2, -(a7)
        e.step()?;
        let value = 0x2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101c92: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101c98_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00101c98: addi.w #$191, d0
        e.step()?;
        let source = 0x191_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101c9c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101c9e: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101ca0: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101ca6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00101ca6: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101ca8: bra.b $101cb8
        e.step()?;
        pc = 0x101cb8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a8_unit::attack_body_block_a8;

mod attack_body_block_a9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_a9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101caa: move.w $453776.l, -(a7)
        e.step()?;
        let value_address = 0x453776_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101cb0: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101cb6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00101cb6: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x101cb8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_a9_unit::attack_body_block_a9;

mod attack_body_block_aa_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_aa(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101cb8: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101cba: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101cbc: bsr.l $12e4e4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101cc2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x12e4e4_u32)?;
        // 00101cc2: clr.w $438778.l
        e.step()?;
        let value = 0;
        let destination_address = 0x438778_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101cc8: move.b #$1, -$d6c(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff294_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00101cce: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101cd0: bra.w $101e1c
        e.step()?;
        pc = 0x101e1c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_aa_unit::attack_body_block_aa;

mod attack_body_block_ab_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_ab(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101cd4: cmpi.w #$ffff, d3
        e.step()?;
        let source = 0xffff_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101cd8: ble.w $101e1c
        e.step()?;
        pc = if r.less_equal()? {
            0x101e1c_u32
        } else {
            0x101cdc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_ab_unit::attack_body_block_ab;

mod attack_body_block_ac_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_ac(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101cdc: tst.w -$d70(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff290_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101ce0: beq.b $101d2c
        e.step()?;
        pc = if r.equal()? {
            0x101d2c_u32
        } else {
            0x101ce2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_ac_unit::attack_body_block_ac;

mod attack_body_block_ad_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_ad(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101ce2: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101ce4: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101ce8: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101cee: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101cf0: moveq #$ec, d0
        e.step()?;
        r.put_data(0, 32, 0xffffffec_u32);
        r.move_flags(0xffffffec_u32, 32);
        // 00101cf2: add.w -$d6a(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff296_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101cf6: cmpi.w #$ffff, $5e(a0, d0.w * 2)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x5e_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00101cfc: ble.b $101d1c
        e.step()?;
        pc = if r.less_equal()? {
            0x101d1c_u32
        } else {
            0x101cfe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_ad_unit::attack_body_block_ad;

mod attack_body_block_ae_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_ae(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101cfe: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101d00: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101d04: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101d0a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101d0c: moveq #$ec, d0
        e.step()?;
        r.put_data(0, 32, 0xffffffec_u32);
        r.move_flags(0xffffffec_u32, 32);
        // 00101d0e: add.w -$d6a(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff296_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101d12: lea.l $5e(a0, d0.w * 2), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x5e_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.put_address(0, value);
        // 00101d16: move.w -$d72(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff28e_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101d1a: add.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x101d1c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_ae_unit::attack_body_block_ae;

mod attack_body_block_af_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_af(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101d1c: move.w d6, -(a7)
        e.step()?;
        let value = r.data(6, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101d1e: move.w #$ffd8, -(a7)
        e.step()?;
        let value = 0xffd8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101d22: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101d24: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101d2a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00101d2a: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x101d2c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_af_unit::attack_body_block_af;

mod attack_body_block_b0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101d2c: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101d2e: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101d32: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101d38: tst.w $60(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x60_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101d3c: beq.b $101d52
        e.step()?;
        pc = if r.equal()? {
            0x101d52_u32
        } else {
            0x101d3e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b0_unit::attack_body_block_b0;

mod attack_body_block_b1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101d3e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101d40: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101d44: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101d4a: move.w $50(a0, d0.l), $4434b8.l
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x101d52_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b1_unit::attack_body_block_b1;

mod attack_body_block_b2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101d52: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101d54: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101d58: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101d5e: move.w #$1, $14(a0, d0.l)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x14_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101d64: move.w $4434b8.l, d0
        e.step()?;
        let value_address = 0x4434b8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101d6a: add.w -$d6e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff292_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101d6e: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00101d70: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00101d74: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101d7a: lea.l $50(a0, d1.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00101d7e: sub.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00101d80: move.w $4434b8.l, d0
        e.step()?;
        let value_address = 0x4434b8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101d86: add.w -$d6e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff292_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101d8a: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101d8c: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00101d8e: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00101d92: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101d98: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101d9a: add.l d0, $a42(a0)
        e.step()?;
        let source = r.data(0, 32)?;
        let destination_address = r.address(0)?.wrapping_add(0xa42_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00101d9e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101da0: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101da4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101daa: cmpi.w #$1, $50(a0, d0.l)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00101db0: bge.b $101e1c
        e.step()?;
        pc = if r.greater_equal()? {
            0x101e1c_u32
        } else {
            0x101db2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b2_unit::attack_body_block_b2;

mod attack_body_block_b3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101db2: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101db4: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101db8: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101dbe: cmpi.w #$fff6, $50(a0, d0.l)
        e.step()?;
        let source = 0xfff6_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00101dc4: ble.b $101dda
        e.step()?;
        pc = if r.less_equal()? {
            0x101dda_u32
        } else {
            0x101dc6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b3_unit::attack_body_block_b3;

mod attack_body_block_b4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101dc6: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101dc8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101dcc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101dd2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101dd4: move.b #$1, $9de(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x9de_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x101dda_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b4_unit::attack_body_block_b4;

mod attack_body_block_b5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101dda: tst.b -$d6b(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff295_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00101dde: beq.b $101dfc
        e.step()?;
        pc = if r.equal()? {
            0x101dfc_u32
        } else {
            0x101de0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b5_unit::attack_body_block_b5;

mod attack_body_block_b6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101de0: move.w #$2, -(a7)
        e.step()?;
        let value = 0x2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101de4: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101dea_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00101dea: addi.w #$191, d0
        e.step()?;
        let source = 0x191_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101dee: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101df0: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101df2: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101df8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00101df8: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101dfa: bra.b $101e0a
        e.step()?;
        pc = 0x101e0a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b6_unit::attack_body_block_b6;

mod attack_body_block_b7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101dfc: move.w $453776.l, -(a7)
        e.step()?;
        let value_address = 0x453776_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e02: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101e08_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00101e08: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x101e0a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b7_unit::attack_body_block_b7;

mod attack_body_block_b8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101e0a: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e0c: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e0e: bsr.l $12e4e4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101e14_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x12e4e4_u32)?;
        // 00101e14: move.b #$1, -$d6c(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff294_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00101e1a: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x101e1c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b8_unit::attack_body_block_b8;

mod attack_body_block_b9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_b9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101e1c: tst.b -$d6c(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff294_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00101e20: bne.b $101e92
        e.step()?;
        pc = if r.not_equal()? {
            0x101e92_u32
        } else {
            0x101e22_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_b9_unit::attack_body_block_b9;

mod attack_body_block_ba_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_ba(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101e22: tst.b -$d6b(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff295_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00101e26: beq.b $101e44
        e.step()?;
        pc = if r.equal()? {
            0x101e44_u32
        } else {
            0x101e28_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_ba_unit::attack_body_block_ba;

mod attack_body_block_bb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_bb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101e28: move.w #$2, -(a7)
        e.step()?;
        let value = 0x2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e2c: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101e32_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00101e32: addi.w #$191, d0
        e.step()?;
        let source = 0x191_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101e36: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101e38: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e3a: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101e40_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00101e40: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101e42: bra.b $101e52
        e.step()?;
        pc = 0x101e52_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_bb_unit::attack_body_block_bb;

mod attack_body_block_bc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_bc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101e44: move.w $453776.l, -(a7)
        e.step()?;
        let value_address = 0x453776_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e4a: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101e50_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00101e50: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x101e52_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_bc_unit::attack_body_block_bc;

mod attack_body_block_bd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_bd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101e52: move.w d6, -(a7)
        e.step()?;
        let value = r.data(6, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e54: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e56: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e58: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101e5e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00101e5e: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101e60: bra.b $101e92
        e.step()?;
        pc = 0x101e92_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_bd_unit::attack_body_block_bd;

mod attack_body_block_be_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_be(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101e62: tst.w $4434b8.l
        e.step()?;
        let value_address = 0x4434b8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101e68: bgt.b $101e70
        e.step()?;
        pc = if r.greater()? {
            0x101e70_u32
        } else {
            0x101e6a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_be_unit::attack_body_block_be;

mod attack_body_block_bf_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_bf(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101e6a: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x101e70_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_bf_unit::attack_body_block_bf;

mod attack_body_block_c0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_c0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101e70: move.w d6, -(a7)
        e.step()?;
        let value = r.data(6, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e72: move.w #$ffff, -(a7)
        e.step()?;
        let value = 0xffff_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e76: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e78: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101e7e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00101e7e: move.w d6, d0
        e.step()?;
        let value = r.data(6, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101e80: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101e84: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101e8a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101e8c: addq.l #$1, $a52(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xa52_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00101e90: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x101e92_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_c0_unit::attack_body_block_c0;

mod attack_body_block_c1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_c1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101e92: clr.w $442bc8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e98: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101e9e: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101ea0: move.b -$d6c(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff294_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        pc = 0x101ea4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_c1_unit::attack_body_block_c1;

mod attack_body_block_c2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_c2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101ea4: movem.l (a7)+, d3-d7/a2
        e.step()?;
        r.restore_register(e, 3)?;
        r.restore_register(e, 4)?;
        r.restore_register(e, 5)?;
        r.restore_register(e, 6)?;
        r.restore_register(e, 7)?;
        r.restore_register(e, 10)?;
        // 00101ea8: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 00101eaa: rts
        e.step()?;
        return finish(e, r).map(|_| std::ops::ControlFlow::Break(()));

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_body_block_c2_unit::attack_body_block_c2;

mod attack_body_block_c3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_body_block_c3(
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
use attack_body_block_c3_unit::attack_body_block_c3;
