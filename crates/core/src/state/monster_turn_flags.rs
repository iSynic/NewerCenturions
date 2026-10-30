use crate::Session;

impl Session {
    /// Keeps the battle's pending note of whether a monster attacked in the party-wide flags.
    pub(crate) fn store_monster_turn_flags(&mut self) {
        if let Some(value) = self.combat.as_mut().and_then(|c| c.did_attack_write.take()) {
            self.globals.monster_turn_flags.did_attack = value;
        }
    }

    /// Sets whether a monster attacked this turn.
    pub(crate) fn write_did_attack(&mut self, value: i16) {
        self.store_monster_turn_flags();
        self.globals.monster_turn_flags.did_attack = value;
    }

    /// Clears both monster turn flags (spell attempts and whether it attacked), as a new turn does.
    pub(crate) fn reset_monster_turn_flags(&mut self) {
        self.store_monster_turn_flags();
        self.globals.monster_turn_flags.getup();
    }
}
