use crate::{prelude::*, support};
use centurions_core::recovered_prompt_control::{PromptControlState, PromptLimits, submit_music};
use centurions_core::recovered_rendering::RenderingMemory;

#[path = "../support/no_services.rs"]
mod no_services;

// Complete native A5/RNG and ordered hardware contracts; no playback or full registry.
#[test]
fn native_music_boundaries_preserve_state_and_selection_draws() {
    let fixture = support::fixture("submitmusic-boundary.json");
    for case in fixture.array("cases") {
        let captured = &case["native_music"][0];
        let buffers = &fixture["shared_a5_buffers"];
        let mut memory = RenderingMemory::default();
        memory.insert_region(0x3fe210, &buffers.bytes(case["before"].text("a5_sha256")));
        let frame = captured.n::<u32>("entry_sp") - 4;
        memory.insert_region(frame - 400, &[0; 420]);
        let input = PromptControlState {
            memory,
            random: RandomState::new(case["before"].n("seed")),
        };
        let mut services = no_services::NoServices {
            music: captured
                .array("services")
                .iter()
                .filter(|service| service.get("service").is_some())
                .cloned()
                .collect(),
            ..Default::default()
        };
        let limits = PromptLimits {
            maximum_steps: 100,
            maximum_polls: 1,
            filename_capacity: 32,
        };
        let plan = submit_music(&input, frame, limits, &mut services).unwrap();
        let expected = buffers.bytes(case["after"].text("a5_sha256"));
        let actual: Vec<_> = (0..expected.len())
            .map(|offset| plan.state.memory.byte(0x3fe210 + offset as u32).unwrap())
            .collect();
        support::assert_bytes(&actual, &expected, case.text("case"));
        assert_eq!(plan.state.random.seed, case["after"].n::<u32>("seed"));
        assert_eq!(plan.state.random.draws, case["after"].n::<u64>("draws"));
        assert_eq!(services.cursor.get(), services.music.len());
    }
}
