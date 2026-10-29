#[path = "../native/restriction_bit_reference.rs"]
mod native_bits;
#[test]
fn restriction_bits_past_the_packet_read_data_s() {
    use super::restrictions::restriction_bit;
    // Byte 58 onward is Data S at 443788; races below -399 read before the packet.
    let mut data = vec![0; 0x5f28];
    data[0] = 0x80;
    data[4087] = 0x01;
    for index in [64, i16::MAX] {
        assert!(restriction_bit(&[0; 58], index, Some(&data)).unwrap());
        assert!(restriction_bit(&[0; 58], index, None).is_err());
    }
    for index in [-401, i16::MIN] {
        assert!(restriction_bit(&[0; 58], index, Some(&data)).is_err());
    }
}
