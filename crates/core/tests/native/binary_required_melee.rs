#[test]
fn required_melee_corpus_matches_original() {
    crate::binary_endgame_weapons::corpus("required-melee.json", false);
}
