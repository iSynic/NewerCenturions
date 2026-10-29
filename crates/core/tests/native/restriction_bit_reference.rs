//! Independent signed-bit instruction controls.
pub(super) fn compare_bits() {
    let fixture = crate::test_support::fixture("restricted-npc-spell.json");
    let controls = fixture["historical_bit_boundary"]["controls"]
        .as_array()
        .unwrap();
    for control in controls {
        let input = crate::test_support::bytes(&control["input_bytes"]);
        let mut packet = vec![0; 58];
        packet[48..48 + input.len()].copy_from_slice(&input);
        let index = control["index"].as_i64().unwrap() as i16;
        assert_eq!(
            super::super::restrictions::restriction_bit(&packet, index, None).unwrap(),
            control["expected_result"].as_i64().unwrap() != 0,
            "original signed index {index}"
        );
    }
}

#[test]
fn signed_restriction_bits_match_all_16_original_source_instruction_controls() {
    compare_bits();
}
