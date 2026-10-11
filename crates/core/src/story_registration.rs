pub fn registration_word_accepted(word: u32) -> bool {
    word != 0 && word & 0x0200_0000 == 0
}
