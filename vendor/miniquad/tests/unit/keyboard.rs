use super::*;

#[test]
fn virtual_input_and_physical_scan_codes_agree() {
    for (vk, scan) in [
        (0x46, 0x21),
        (0x52, 0x13),
        (0x0d, 0x1c),
        (0x20, 0x39),
        (0x25, 0x14b),
        (0x27, 0x14d),
        (0x26, 0x148),
        (0x28, 0x150),
    ] {
        assert_eq!(message_scancode(vk, 0), scan);
        assert_eq!(message_scancode(vk, (scan as isize) << 16), scan);
    }
}

#[test]
fn unknown_virtual_input_stays_unknown() {
    assert_eq!(message_scancode(0, 0), 0);
}
