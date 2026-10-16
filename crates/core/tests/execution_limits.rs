#[path = "hex.rs"]
mod fixture_hex;
#[path = "support/no_services.rs"]
mod no_services;

use centurions_core::recovered_application_control::*;
use centurions_core::recovered_equipment::EquipmentLimits;
use centurions_core::recovered_prompt_control::PromptLimits;
use centurions_core::recovered_rendering::RenderingMemory;
use centurions_core::{RandomState, RuleError};

struct TinyRoutines;
impl RoutineResolver for TinyRoutines {
    fn dispatch(&self, execution: &mut Execution<'_>, entry: u32, frame: u32) -> Result<()> {
        match entry {
            3 => {
                execution.byte(0, 0xff)?;
                execution.call(3, frame)
            }
            4 => {
                execution.byte(0, 0xff)?;
                execution.rand(10, frame)?;
                execution.call(99, frame)
            }
            _ => Err(RuleError::Unsupported {
                action: "test.unresolved",
                dependency: "routine absent from tiny resolver",
            }
            .into()),
        }
    }
    fn owns_native_stack(&self, _: u32) -> bool {
        false
    }
}

fn limits() -> ApplicationControlLimits {
    ApplicationControlLimits {
        maximum_steps: 8,
        maximum_call_depth: 4,
        maximum_random_draws: 1,
        prompt: PromptLimits {
            maximum_steps: 8,
            maximum_polls: 1,
            filename_capacity: 32,
        },
        equipment: EquipmentLimits {
            maximum_steps: 8,
            maximum_polls: 1,
            filename_capacity: 32,
        },
    }
}
fn state() -> ApplicationControlState {
    let mut memory = RenderingMemory::default();
    memory.insert_region(0, &[0; 8]);
    ApplicationControlState {
        memory,
        random: RandomState::new(42),
    }
}

// Protects bounded recursive dispatch and rollback after candidate writes/RNG draws.
#[test]
fn limits_and_missing_children_reject_without_mutating_input() {
    for (entry, steps, depth, draws, diagnostic) in [
        (3, 2, 4, 1, "processing budget"),
        (3, 8, 2, 1, "call depth"),
        (4, 8, 4, 0, "random budget"),
        (4, 8, 4, 1, "test.unresolved"),
    ] {
        let input = state();
        let before = input.clone();
        let mut bounds = limits();
        bounds.maximum_steps = steps;
        bounds.maximum_call_depth = depth;
        bounds.maximum_random_draws = draws;
        let error = execute(
            &input,
            entry,
            8,
            ApplicationRegisters::default(),
            bounds,
            &mut no_services::NoServices::default(),
            &TinyRoutines,
        )
        .unwrap_err();
        assert!(error.to_string().contains(diagnostic), "{error}");
        assert_eq!(input, before);
    }
}
