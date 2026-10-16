#[test]
fn required_enemy_weapon_corpus_matches_original() {
    for name in ["required-enemy-weapons.json", "thunder-firearms.json"] {
        crate::binary_endgame_weapons::corpus(name, false);
    }
}
