//! Source-derived newland7 replacement destinations and saved continuation.
use crate::{prelude::*, support};

#[test]
fn encounter_replacement_preserves_original_read_and_write_strides_across_reload() {
    for target in [-1_i16, -2] {
        let mut content = support::content();
        let map = &mut content.scenario.as_mut().unwrap().maps[0];
        let tables = Arc::make_mut(&mut map.shared);
        let extra = &mut tables.combat.as_mut().unwrap().encounters[..10];
        for (offset, value) in [(0, target), (2, 1), (4, 1), (8, 2)] {
            extra[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        let script = &mut tables.door_scripts[42..84];
        for action in 0..8 {
            script[8 + action * 2..10 + action * 2]
                .copy_from_slice(&(256_i16 + action as i16).to_be_bytes());
            script[24 + action * 2..26 + action * 2]
                .copy_from_slice(&(-256_i16 - action as i16).to_be_bytes());
        }
        let (stride, header, table) = if target == -1 {
            (426, 106, &tables.menus)
        } else {
            (516, 156, &tables.typed_encounters)
        };
        let mut expected = table.clone();
        let mut replaced = table[stride..stride + header].to_vec();
        for action in 0..8 {
            replaced[16 + action] = action as u8;
            replaced[64 + action * 2..66 + action * 2]
                .copy_from_slice(&(-256_i16 - action as i16).to_be_bytes());
        }
        expected[426..426 + header].copy_from_slice(&replaced);
        let mut state = support::mission_session(&content);
        let source = &content.map(0).unwrap().menus;
        let tail = source.as_chunks::<426>().1;
        assert_eq!(
            &state.globals.encounter_headers.header(false)[..tail.len()],
            tail
        );
        let random = state.random.clone();
        let record = support::script_record(&[(7, 0), (1, 1)]);
        support::pause_script(&mut state, record, 0);
        support::saved_command(Command::Continue, &mut state, &content);
        assert_eq!(
            if target == -1 {
                &state.menus
            } else {
                &state.typed_encounters
            },
            &expected
        );
        assert_eq!(support::script(&state).cursor, 2);
        state = support::checkpoint(&state, &content, "encounter-replacement");
        support::saved_command(Command::Continue, &mut state, &content);
        assert!(state.exploration.as_ref().unwrap().script.is_none());
        assert_eq!(state.random, random);
    }
}
