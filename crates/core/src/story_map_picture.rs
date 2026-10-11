/// Original opcode 113 stores its signed word before requesting a map redraw.
pub fn assign(word: &mut i16, parameter: i16) {
    *word = parameter;
}
