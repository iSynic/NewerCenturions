#[test]
fn signed_restriction_bits_match_all_16_original_source_instruction_controls() {
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/restricted-npc-spell.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let controls = fixture["historical_bit_boundary"]["controls"]
        .as_array()
        .unwrap();
    assert_eq!(controls.len(), 16);
    for control in controls {
        let input = control["input_bytes"].as_str().unwrap();
        let mut packet = vec![0; 58];
        for (index, offset) in (0..input.len()).step_by(2).enumerate() {
            packet[48 + index] = u8::from_str_radix(&input[offset..offset + 2], 16).unwrap();
        }
        let index = control["index"].as_i64().unwrap() as i16;
        assert_eq!(
            super::restriction_bit(&packet, index).unwrap(),
            control["expected_result"].as_i64().unwrap() != 0,
            "original signed index {index}"
        );
        assert_eq!(control["result"], control["expected_result"]);
    }
    for index in [-401, 64, i16::MIN, i16::MAX] {
        assert!(super::restriction_bit(&[0; 58], index).is_err());
    }
}
