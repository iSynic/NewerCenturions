use crate::prelude::*;

use centurions_core::walking_header::lookup;

fn long(bytes: &[u8], offset: usize) -> i32 {
    i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn word(bytes: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn current_position(snapshot: &Value) -> (i32, i32, i32) {
    let location = snapshot.bytes("location");
    (
        long(&location, 16),
        long(&location, 0).wrapping_add(long(&location, 20)),
        long(&location, 4).wrapping_add(long(&location, 24)),
    )
}

#[test]
fn original_caller_captures_current_coordinates_and_skips_header_on_exit() {
    let fixture = crate::support::fixture("regional-move-continuation.json");

    for case in fixture.array("cases") {
        let (_, x, y) = current_position(&case["before"]);
        if case.n::<bool>("game_exit_control") {
        } else {
            assert_eq!(case["arguments"], json!([x, y, 0, 0, 0]));

            if !case.n::<bool>("complete_newland_prefix") {}
        }
    }
}

#[test]
fn walking_header_lookup_matches_continuous_original_prefix_and_full_buffers() {
    for case in crate::support::fixture("regional-move-continuation.json")["cases"]
        .as_array()
        .unwrap()
    {
        if !case.n::<bool>("complete_newland_prefix") {
            continue;
        }
        let before = &case["before"];
        let after = &case["after"];
        let headers = before.bytes("headers");
        let original_headers = headers.clone();
        let (map, x, y) = current_position(before);
        let mut random = RandomState {
            seed: before.n::<u32>("seed"),
            draws: before.n::<u64>("draws"),
        };
        let mut receipt = random.clone();
        for draw in case.array("rng") {
            crate::support::native_expect! {
            scalar receipt.seed => &draw["seed_before"], "binary_regional_move_continuation";
            scalar receipt.bound(draw.n::<i16>("bound")) => &draw["value"], "binary_regional_move_continuation";
            scalar receipt.seed => &draw["seed_after"], "binary_regional_move_continuation";
            }
        }
        let selected = lookup(&headers, map, x, y, &mut random).unwrap();
        assert_eq!(headers, original_headers);
        assert_eq!(random, receipt);
        crate::support::native_expect! {
        scalar random.seed => &after["seed"], "binary_regional_move_continuation";
        scalar random.draws => &after["draws"], "binary_regional_move_continuation";
        }
        assert_eq!(after["header_key"], 307);
        assert_eq!(after["current_header"], 7);
        let native_record = after.bytes("working_header");
        assert_eq!(&headers[7 * 42..8 * 42], native_record);
        if case.n::<bool>("inactive_header_control") {
            assert!(selected.is_none());
            assert_eq!(native_record[7], 0);
        } else {
            let selected = selected.unwrap();
            assert_eq!(selected.slot, 7);
            assert_eq!(selected.record, native_record);
            assert_eq!(selected.record[7], 100);
            for index in 0..6 {
                assert_eq!(word(&selected.record, 8 + index * 2), 1);
                assert_eq!(word(&selected.record, 24 + index * 2), 175 + index as i16);
            }
            assert_eq!(word(&selected.record, 20), 0);
            assert_eq!(word(&selected.record, 22), 39);
            assert_eq!(word(&selected.record, 38), 45);
        }
    }
}

#[test]
fn source_boundaries_preserve_first_match_signed_chance_and_atomic_validation() {
    let mut headers = vec![0; 4200];
    headers[..4].copy_from_slice(&10307_i32.to_be_bytes());
    headers[42..46].copy_from_slice(&10307_i32.to_be_bytes());
    headers[42 + 7] = 100;
    let mut random = RandomState::new(12);
    let original_random = random.clone();
    assert!(lookup(&headers, 1, 7, 3, &mut random).unwrap().is_none());
    assert_eq!(random, original_random);
    headers[7] = 255;
    assert!(lookup(&headers, 1, 7, 3, &mut random).unwrap().is_none());
    assert_eq!(random, original_random);
    assert!(lookup(&headers, 2, 7, 3, &mut random).unwrap().is_none());
    assert_eq!(random, original_random);
    assert!(lookup(&headers[..4199], 1, 7, 3, &mut random).is_err());
    assert_eq!(random, original_random);

    let roll = random.clone().bound(100);
    headers[7] = roll as u8;
    assert_eq!(
        lookup(&headers, 1, 7, 3, &mut random)
            .unwrap()
            .unwrap()
            .slot,
        0
    );
    random = original_random;
    headers[7] = (roll - 1) as u8;
    assert!(lookup(&headers, 1, 7, 3, &mut random).unwrap().is_none());
    assert_eq!(random.draws, 1);
}
