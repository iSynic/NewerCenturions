//! Complete provisional gameplay control from canonical CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};

pub fn collide_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x1140ae_u32;
        loop {
            match match pc {
                0x1140ae => collide_body_block_0(e, &mut r, frame, pc)?,
                0x1140c4 => collide_body_block_1(e, &mut r, frame, pc)?,
                0x1140da => collide_body_block_2(e, &mut r, frame, pc)?,
                0x1140e0 => collide_body_block_3(e, &mut r, frame, pc)?,
                0x114134 => collide_body_block_4(e, &mut r, frame, pc)?,
                0x1141b2 => collide_body_block_5(e, &mut r, frame, pc)?,
                0x1141b6 => collide_body_block_6(e, &mut r, frame, pc)?,
                0x1141bc => collide_body_block_7(e, &mut r, frame, pc)?,
                0x1141c0 => collide_body_block_8(e, &mut r, frame, pc)?,
                0x1141c6 => collide_body_block_9(e, &mut r, frame, pc)?,
                0x1141d6 => collide_body_block_a(e, &mut r, frame, pc)?,
                0x1141dc => collide_body_block_b(e, &mut r, frame, pc)?,
                0x1141f8 => collide_body_block_c(e, &mut r, frame, pc)?,
                0x114206 => collide_body_block_d(e, &mut r, frame, pc)?,
                0x11422e => collide_body_block_e(e, &mut r, frame, pc)?,
                0x11423e => collide_body_block_f(e, &mut r, frame, pc)?,
                0x11424e => collide_body_block_10(e, &mut r, frame, pc)?,
                0x114260 => collide_body_block_11(e, &mut r, frame, pc)?,
                0x1142b8 => collide_body_block_12(e, &mut r, frame, pc)?,
                0x1142c8 => collide_body_block_13(e, &mut r, frame, pc)?,
                0x114332 => collide_body_block_14(e, &mut r, frame, pc)?,
                0x114334 => collide_body_block_15(e, &mut r, frame, pc)?,
                0x11433c => collide_body_block_16(e, &mut r, frame, pc)?,
                0x114346 => collide_body_block_17(e, &mut r, frame, pc)?,
                0x11435e => collide_body_block_18(e, &mut r, frame, pc)?,
                0x114364 => collide_body_block_19(e, &mut r, frame, pc)?,
                0x114366 => collide_body_block_1a(e, &mut r, frame, pc)?,
                _ => collide_body_block_1b(e, &mut r, frame, pc)?,
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
crate::recovered_application_control::entry!(collide, 0x1140ae);
mod collide_body_block_0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001140ae: link.w a6, #$ffec
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, -20)?;
        // 001140b2: movem.l d3-d6, -(a7)
        e.step()?;
        r.save_register(e, 6)?;
        r.save_register(e, 5)?;
        r.save_register(e, 4)?;
        r.save_register(e, 3)?;
        // 001140b6: move.b $9(a6), d4
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0x9_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(4, 8, value);
        r.move_flags(value, 8);
        // 001140ba: move.b $b(a6), d5
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xb_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(5, 8, value);
        r.move_flags(value, 8);
        // 001140be: cmpi.b #$9, d4
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(4, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 001140c2: bge.b $114134
        e.step()?;
        pc = if r.greater_equal()? {
            0x114134_u32
        } else {
            0x1140c4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_0_unit::collide_body_block_0;

mod collide_body_block_1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001140c4: move.b d4, d0
        e.step()?;
        let value = r.data(4, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001140c6: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001140c8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001140cc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001140d2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001140d4: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001140d8: bne.b $1140e0
        e.step()?;
        pc = if r.not_equal()? {
            0x1140e0_u32
        } else {
            0x1140da_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_1_unit::collide_body_block_1;

mod collide_body_block_2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001140da: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001140dc: bra.w $114366
        e.step()?;
        pc = 0x114366_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_2_unit::collide_body_block_2;

mod collide_body_block_3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001140e0: move.b d4, d0
        e.step()?;
        let value = r.data(4, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001140e2: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001140e4: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 001140ea: movea.w $2(a0, d0.l * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 001140ee: move.l $442784.l, d0
        e.step()?;
        let value_address = 0x442784_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001140f4: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 001140f6: move.w d0, $4471ee.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4471ee_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001140fc: move.b d4, d0
        e.step()?;
        let value = r.data(4, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001140fe: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114100: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00114106: movea.w (a0, d0.l * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 0011410a: move.l $442780.l, d0
        e.step()?;
        let value_address = 0x442780_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114110: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00114112: move.w d0, $4471f0.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4471f0_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114118: move.l $4471ee.l, -$14(a6)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114120: move.l $4471ee.l, -$10(a6)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114128: move.l $4471ee.l, -$c(a6)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114130: bra.w $1141d6
        e.step()?;
        pc = 0x1141d6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_3_unit::collide_body_block_3;

mod collide_body_block_4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114134: move.b d4, d0
        e.step()?;
        let value = r.data(4, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114136: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114138: addi.w #$fff6, d0
        e.step()?;
        let source = 0xfff6_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011413c: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114140: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114146: move.w $10(a0, d0.l), -(a7)
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
        // 0011414a: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114150_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00114150: move.b d0, d6
        e.step()?;
        let value = r.data(0, 8)?;
        r.put_data(6, 8, value);
        r.move_flags(value, 8);
        // 00114152: move.b d4, d0
        e.step()?;
        let value = r.data(4, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114154: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114156: addi.w #$fff6, d0
        e.step()?;
        let source = 0xfff6_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011415a: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00114160: movea.w $2(a0, d0.w * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00114164: move.l $442784.l, d0
        e.step()?;
        let value_address = 0x442784_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011416a: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 0011416c: move.w d0, $4471ee.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4471ee_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114172: move.b d4, d0
        e.step()?;
        let value = r.data(4, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114174: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114176: addi.w #$fff6, d0
        e.step()?;
        let source = 0xfff6_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011417a: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00114180: movea.w (a0, d0.w * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00114184: move.l $442780.l, d0
        e.step()?;
        let value_address = 0x442780_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011418a: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 0011418c: move.w d0, $4471f0.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4471f0_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114192: move.l $4471ee.l, -$14(a6)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011419a: move.l $4471ee.l, -$10(a6)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001141a2: move.l $4471ee.l, -$c(a6)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001141aa: cmpi.b #$1, d6
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(6, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 001141ae: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001141b0: bne.b $1141b6
        e.step()?;
        pc = if r.not_equal()? {
            0x1141b6_u32
        } else {
            0x1141b2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_4_unit::collide_body_block_4;

mod collide_body_block_5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001141b2: subq.w #$1, -$c(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x1141b6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_5_unit::collide_body_block_5;

mod collide_body_block_6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001141b6: cmpi.b #$2, d6
        e.step()?;
        let source = 0x2_u32;
        let destination = r.data(6, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 001141ba: bne.b $1141c0
        e.step()?;
        pc = if r.not_equal()? {
            0x1141c0_u32
        } else {
            0x1141bc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_6_unit::collide_body_block_6;

mod collide_body_block_7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001141bc: subq.w #$1, -$e(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x1141c0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_7_unit::collide_body_block_7;

mod collide_body_block_8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001141c0: cmpi.b #$3, d6
        e.step()?;
        let source = 0x3_u32;
        let destination = r.data(6, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 001141c4: bne.b $1141d6
        e.step()?;
        pc = if r.not_equal()? {
            0x1141d6_u32
        } else {
            0x1141c6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_8_unit::collide_body_block_8;

mod collide_body_block_9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001141c6: subq.w #$1, -$14(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001141ca: subq.w #$1, -$12(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffee_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001141ce: subq.w #$1, -$c(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001141d2: subq.w #$1, -$e(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x1141d6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_9_unit::collide_body_block_9;
mod collide_body_block_a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001141d6: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001141d8: bra.w $114334
        e.step()?;
        pc = 0x114334_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_a_unit::collide_body_block_a;

mod collide_body_block_b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001141dc: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001141de: addq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001141e0: move.b d0, $4358c2.l
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = 0x4358c2_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001141e6: moveq #$14, d0
        e.step()?;
        r.put_data(0, 32, 0x14_u32);
        r.move_flags(0x14_u32, 32);
        // 001141e8: mulu.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001141ea: lea.l $442bd6.l, a0
        e.step()?;
        let value = 0x442bd6_u32;
        r.put_address(0, value);
        // 001141f0: tst.b $f(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xf_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001141f4: beq.w $114332
        e.step()?;
        pc = if r.equal()? {
            0x114332_u32
        } else {
            0x1141f8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_b_unit::collide_body_block_b;

mod collide_body_block_c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001141f8: lea.l $435884.l, a0
        e.step()?;
        let value = 0x435884_u32;
        r.put_address(0, value);
        // 001141fe: tst.b (a0, d3.w)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(1));
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114202: bne.w $114332
        e.step()?;
        pc = if r.not_equal()? {
            0x114332_u32
        } else {
            0x114206_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_c_unit::collide_body_block_c;

mod collide_body_block_d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114206: moveq #$14, d0
        e.step()?;
        r.put_data(0, 32, 0x14_u32);
        r.move_flags(0x14_u32, 32);
        // 00114208: mulu.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011420a: lea.l $442bd6.l, a0
        e.step()?;
        let value = 0x442bd6_u32;
        r.put_address(0, value);
        // 00114210: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114212: move.l (a0), -$8(a6)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x0_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114216: move.l $4(a0), -$4(a6)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x4_u32);
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011421c: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0011421e: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114224: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00114228: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 0011422a: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011422c: bne.b $114260
        e.step()?;
        pc = if r.not_equal()? {
            0x114260_u32
        } else {
            0x11422e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_d_unit::collide_body_block_d;

mod collide_body_block_e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011422e: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00114230: move.l -$c(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114234: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00114238: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 0011423a: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011423c: bne.b $114260
        e.step()?;
        pc = if r.not_equal()? {
            0x114260_u32
        } else {
            0x11423e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_e_unit::collide_body_block_e;

mod collide_body_block_f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011423e: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00114240: move.l -$10(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114244: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00114248: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 0011424a: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011424c: bne.b $114260
        e.step()?;
        pc = if r.not_equal()? {
            0x114260_u32
        } else {
            0x11424e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_f_unit::collide_body_block_f;

mod collide_body_block_10_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_10(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011424e: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00114250: move.l -$14(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114254: pea.l -$8(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00114258: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 0011425a: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011425c: beq.w $114332
        e.step()?;
        pc = if r.equal()? {
            0x114332_u32
        } else {
            0x114260_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_10_unit::collide_body_block_10;

mod collide_body_block_11_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_11(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114260: clr.b $435662.l
        e.step()?;
        let value = 0;
        let destination_address = 0x435662_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114266: moveq #$14, d0
        e.step()?;
        r.put_data(0, 32, 0x14_u32);
        r.move_flags(0x14_u32, 32);
        // 00114268: mulu.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011426a: lea.l $442bd6.l, a0
        e.step()?;
        let value = 0x442bd6_u32;
        r.put_address(0, value);
        // 00114270: move.w $8(a0, d0.l), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x8_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114274: bsr.l $13145c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11427a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13145c_u32)?;
        // 0011427a: moveq #$14, d0
        e.step()?;
        r.put_data(0, 32, 0x14_u32);
        r.move_flags(0x14_u32, 32);
        // 0011427c: mulu.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011427e: lea.l $442bd6.l, a0
        e.step()?;
        let value = 0x442bd6_u32;
        r.put_address(0, value);
        // 00114284: move.b $10(a0, d0.l), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114288: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011428a: move.w d0, $4435cc.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4435cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114290: moveq #$14, d0
        e.step()?;
        r.put_data(0, 32, 0x14_u32);
        r.move_flags(0x14_u32, 32);
        // 00114292: mulu.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114294: lea.l $442bd6.l, a0
        e.step()?;
        let value = 0x442bd6_u32;
        r.put_address(0, value);
        // 0011429a: move.b $13(a0, d0.l), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x13_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011429e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001142a0: move.w d0, $4435c8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4435c8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001142a6: bsr.l $13822a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1142ac_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13822a_u32)?;
        // 001142ac: cmpi.b #$9, $443761.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x443761_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001142b4: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001142b6: bne.b $1142c8
        e.step()?;
        pc = if r.not_equal()? {
            0x1142c8_u32
        } else {
            0x1142b8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_11_unit::collide_body_block_11;

mod collide_body_block_12_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_12(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001142b8: move.b #$1, $443761.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x443761_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001142c0: move.b #$1, $443762.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x443762_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x1142c8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_12_unit::collide_body_block_12;

mod collide_body_block_13_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_13(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001142c8: clr.b $443752.l
        e.step()?;
        let value = 0;
        let destination_address = 0x443752_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001142ce: moveq #$14, d0
        e.step()?;
        r.put_data(0, 32, 0x14_u32);
        r.move_flags(0x14_u32, 32);
        // 001142d0: mulu.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001142d2: lea.l $442bd6.l, a0
        e.step()?;
        let value = 0x442bd6_u32;
        r.put_address(0, value);
        // 001142d8: move.b $c(a0, d0.l), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001142dc: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001142de: addq.w #$3, d0
        e.step()?;
        let source = 0x3_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001142e0: move.w d0, $443722.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x443722_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001142e6: moveq #$14, d0
        e.step()?;
        r.put_data(0, 32, 0x14_u32);
        r.move_flags(0x14_u32, 32);
        // 001142e8: mulu.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001142ea: lea.l $442bd6.l, a0
        e.step()?;
        let value = 0x442bd6_u32;
        r.put_address(0, value);
        // 001142f0: move.b $d(a0, d0.l), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xd_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001142f4: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001142f6: addq.w #$3, d0
        e.step()?;
        let source = 0x3_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001142f8: move.w d0, $443724.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x443724_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001142fe: move.w #$3, $443726.l
        e.step()?;
        let value = 0x3_u32;
        let destination_address = 0x443726_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114306: moveq #$14, d0
        e.step()?;
        r.put_data(0, 32, 0x14_u32);
        r.move_flags(0x14_u32, 32);
        // 00114308: mulu.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011430a: lea.l $442bd6.l, a0
        e.step()?;
        let value = 0x442bd6_u32;
        r.put_address(0, value);
        // 00114310: move.b $11(a0, d0.l), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x11_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114314: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114316: move.w d0, $44331a.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44331a_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011431c: pea.l $1.w
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00114320: move.b d4, d0
        e.step()?;
        let value = r.data(4, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114322: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114324: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114326: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114328: bsr.l $15cb0e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11432e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15cb0e_u32)?;
        // 0011432e: moveq #$1, d5
        e.step()?;
        r.put_data(5, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 00114330: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x114332_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_13_unit::collide_body_block_13;

mod collide_body_block_14_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_14(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114332: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x114334_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_14_unit::collide_body_block_14;

mod collide_body_block_15_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_15(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114334: cmpi.w #$3c, d3
        e.step()?;
        let source = 0x3c_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114338: blt.w $1141dc
        e.step()?;
        pc = if r.less()? {
            0x1141dc_u32
        } else {
            0x11433c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_15_unit::collide_body_block_15;

mod collide_body_block_16_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_16(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011433c: clr.b $4358c2.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4358c2_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114342: tst.b d5
        e.step()?;
        let value = r.data(5, 8)?;
        r.move_flags(value, 8);
        // 00114344: beq.b $114364
        e.step()?;
        pc = if r.equal()? {
            0x114364_u32
        } else {
            0x114346_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_16_unit::collide_body_block_16;

mod collide_body_block_17_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_17(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114346: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 0011434c: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114352: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00114358: cmp.w (a0, d0.w * 2), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0011435c: bne.b $114364
        e.step()?;
        pc = if r.not_equal()? {
            0x114364_u32
        } else {
            0x11435e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_17_unit::collide_body_block_17;

mod collide_body_block_18_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_18(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011435e: bsr.l $16043e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114364_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16043e_u32)?;
        pc = 0x114364_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_18_unit::collide_body_block_18;

mod collide_body_block_19_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_19(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114364: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        pc = 0x114366_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_19_unit::collide_body_block_19;
mod collide_body_block_1a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_1a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114366: movem.l (a7)+, d3-d6
        e.step()?;
        r.restore_register(e, 3)?;
        r.restore_register(e, 4)?;
        r.restore_register(e, 5)?;
        r.restore_register(e, 6)?;
        // 0011436a: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 0011436c: rts
        e.step()?;
        return finish(e, r).map(|_| std::ops::ControlFlow::Break(()));

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use collide_body_block_1a_unit::collide_body_block_1a;

mod collide_body_block_1b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn collide_body_block_1b(
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
use collide_body_block_1b_unit::collide_body_block_1b;
