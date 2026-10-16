fn hex_digit(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => panic!("invalid native fixture hex byte"),
    }
}
pub fn decode(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0, "native hex record length");
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| hex_digit(pair[0]) * 16 + hex_digit(pair[1]))
        .collect()
}
