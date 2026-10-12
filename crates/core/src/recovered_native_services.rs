//! Explicit imported runtime and Toolbox observations for source-lifted control.

use crate::RuleError;
use crate::recovered_application_control::{ApplicationRegisters, RegisterBits};
use crate::recovered_load_control::LoadFlow;
use crate::recovered_rendering::RenderingMemory;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeCall {
    Toolbox(u16),
    Library(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRequest {
    pub call: NativeCall,
    /// Actual native entry SP, including the library return-PC slot.
    pub stack: u32,
    pub registers: ApplicationRegisters,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeObservation {
    pub memory: RenderingMemory,
    /// Changed/observed values only. Unsurfaced scratch outputs remain unknown.
    pub data: [Option<RegisterBits>; 8],
    pub address: [Option<RegisterBits>; 8],
    pub ccr: Option<(u8, u8)>,
    /// Actual stack after the import, including Pascal cleanup or library RTS.
    pub stack_after: u32,
    pub flow: LoadFlow,
}

pub trait NativeServices {
    /// Only imported runtime entries and Toolbox traps may use this boundary.
    /// Application gameplay must call a recovered Rust body instead.
    fn observe_native(
        &mut self,
        _: &NativeRequest,
        _: &RenderingMemory,
    ) -> Result<NativeObservation, RuleError> {
        Err(RuleError::Unsupported {
            action: "system_control.native_import",
            dependency: "the reached native stack, memory and register observation",
        })
    }
}
