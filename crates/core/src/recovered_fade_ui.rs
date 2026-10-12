//! CODE 1 fadeinout composition with actual recovered gamma child bodies.

use crate::recovered_gamma::{
    self as gamma, GammaEffect, GammaError, GammaMemory, GammaPlan, GammaReturn,
};
use crate::recovered_gamma_control::{
    self as control, GammaControlEffect, GammaControlPlan, GammaControlServices,
};

fn merge_gamma(result: &mut GammaControlPlan, child: GammaPlan) -> GammaReturn {
    result.memory = child.memory;
    result
        .effects
        .extend(child.effects.into_iter().map(GammaControlEffect::Gamma));
    child.native_return
}

fn merge_control(result: &mut GammaControlPlan, child: GammaControlPlan) {
    result.memory = child.memory;
    result.effects.extend(child.effects);
}

fn write_flag(result: &mut GammaControlPlan, value: u8) -> Result<(), GammaError> {
    result.memory.byte(0x4042f0)?;
    result.memory.insert_region(0x4042f0, &[value]);
    result
        .effects
        .push(GammaControlEffect::Gamma(GammaEffect::Write {
            address: 0x4042f0,
            bytes: vec![value],
        }));
    Ok(())
}

/// fadeinout, 001315a2. Status WORDs are discarded by the original caller.
pub fn fadeinout(
    memory: &GammaMemory,
    frame_pointer: u32,
    steps: u16,
    direction: i16,
    services: &(impl GammaControlServices + ?Sized),
) -> Result<GammaControlPlan, GammaError> {
    let mut result = GammaControlPlan {
        memory: memory.clone(),
        effects: Vec::new(),
        native_return: GammaReturn::DataRegister(0),
    };
    if result.memory.byte(0x438587)? != 0 || result.memory.byte(0x438588)? != 0 {
        return Ok(result);
    }
    let availability = gamma::is_color_gamma_available(&result.memory, services)?;
    let GammaReturn::Byte(available) = merge_gamma(&mut result, availability) else {
        unreachable!()
    };
    if available == 0 {
        return Ok(result);
    }
    if direction == 1 && result.memory.byte(0x4042f0)? == 0 {
        let child = control::get_monitor_state(
            &result.memory,
            frame_pointer.wrapping_sub(14),
            0x4042ec,
            services,
        )?;
        merge_control(&mut result, child);
        let child = control::fade_to_constant_color(
            &result.memory,
            frame_pointer.wrapping_sub(14),
            4,
            steps,
            services,
        )?;
        merge_control(&mut result, child);
        write_flag(&mut result, 1)?;
    } else if direction == 0 && result.memory.byte(0x4042f0)? != 0 {
        let target = result.memory.long(0x4042ec)?;
        let child = control::fade_to_monitor_state(
            &result.memory,
            frame_pointer.wrapping_sub(18),
            4,
            50,
            target,
            services,
        )?;
        merge_control(&mut result, child);
        // The original rereads the handle after the fade child returns.
        let head = result.memory.long(0x4042ec)?;
        let child = control::release_monitor_state(
            &result.memory,
            frame_pointer.wrapping_sub(14),
            1,
            head,
            services,
        )?;
        merge_control(&mut result, child);
        write_flag(&mut result, 0)?;
    }
    Ok(result)
}
