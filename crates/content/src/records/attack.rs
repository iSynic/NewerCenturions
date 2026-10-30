use crate::{ContentError, word};

/// The spell info packet shared by firearms and Psi scans, held in the item
/// and scan tables. Unknown target types compare exactly as the game does.
pub struct SpellInfoRecord<'a>(pub &'a [u8]);

impl SpellInfoRecord<'_> {
    /// Kind of lingering field the attack lays down; zero lays none.
    pub fn field_kind(&self) -> i16 {
        i16::from(self.0[4] as i8)
    }
    /// The animation shown where the attack lands.
    pub fn effect_animation(&self) -> i16 {
        i16::from(self.0[20] as i8)
    }

    /// The area pattern the attack covers; one and up index the shapes of `Data AD`.
    pub fn area_size(&self) -> u8 {
        self.0[31]
    }
    /// Whether the attack may be used in combat, or in camp when `combat` is false.
    pub fn mode_allowed(&self, combat: bool) -> bool {
        self.0[if combat { 21 } else { 22 }] != 0
    }

    /// Range gained for each power level.
    pub fn range_per_power(&self) -> i16 {
        i16::from(self.0[3] as i8)
    }

    /// Range at power level zero.
    pub fn range_base(&self) -> i16 {
        i16::from(self.0[2] as i8)
    }

    /// Psi points one power level costs.
    pub fn cost(&self) -> i16 {
        i16::from(self.0[30])
    }

    /// The attack's effect code; 57 is healing.
    pub fn effect(&self) -> Result<i16, ContentError> {
        word(self.0, 40)
    }
    /// Byte 12, whose meaning is unknown.
    pub fn unidentified_byte12(&self) -> u8 {
        self.0[12]
    }
    /// The seven elemental damage bytes, in packet order.
    pub fn elemental_damage(&self) -> &[u8] {
        &self.0[23..30]
    }
    /// Largest physical damage the attack deals.
    pub fn damage_maximum(&self) -> Result<i16, ContentError> {
        word(self.0, 34)
    }
    /// The race the attack is limited to; zero is no limit.
    pub fn race_restriction(&self) -> Result<i16, ContentError> {
        word(self.0, 50)
    }
    /// The kind of defense the attack is made against; 9 is a weapon's direct damage.
    pub fn defense_category(&self) -> u8 {
        self.0[19]
    }
    /// The saving-throw mode: above one, a spell's target gets no saving throw.
    pub fn cannot(&self) -> u8 {
        self.0[9]
    }
    /// Added to the target's saving throw chance for each power level of the use.
    pub fn save_bonus_per_attack(&self) -> i16 {
        i16::from(self.0[8] as i8)
    }
    /// Added to the target's resistance chance for each power level of the use.
    pub fn resistance_bonus_per_attack(&self) -> i16 {
        i16::from(self.0[10] as i8)
    }
    /// Lowest and highest duration drawn for each power level, added to the base duration.
    pub fn power_duration_range(&self) -> (i16, i16) {
        (i16::from(self.0[15] as i8), i16::from(self.0[16] as i8))
    }
    /// How the attack chooses its targets.
    pub fn target_type(&self) -> u8 {
        self.0[1]
    }
    /// Targeting mode 6: a ray that walks from the caster.
    pub fn is_ray(&self) -> bool {
        self.target_type() == 6
    }
    /// Targeting mode 7: party durations and the other party-wide effects.
    pub fn is_party_duration(&self) -> bool {
        self.target_type() == 7
    }
    /// Targeting modes 9, 10 and 12: every body of one side, or every body of both.
    pub fn is_global(&self) -> bool {
        matches!(self.target_type(), 9 | 10 | 12)
    }
    /// Global targeting mode 9: every body on the party's side (faction zero).
    pub fn is_global_own_side(&self) -> bool {
        self.target_type() == 9
    }
    /// Global targeting mode 12: every body regardless of side.
    pub fn is_global_all(&self) -> bool {
        self.target_type() == 12
    }
    /// The count the use runs at: its power level, which scales the range and the shots.
    pub fn attack_count(&self) -> i16 {
        i16::from(self.0[0] as i8)
    }
    /// Range added to the attack's reach.
    pub fn range_modifier(&self) -> i16 {
        i16::from(self.0[2] as i8)
    }
    /// Whether the attack is a weapon's plain damage with no further effect.
    pub fn is_direct_weapon_damage(&self) -> Result<bool, ContentError> {
        Ok(self.0[19] == 9 && self.0[20] == 9 && word(self.0, 40)? == 0)
    }
    /// Whether the target type is 0, which limits the use to one shot rather than the weapon's
    /// whole shot count.
    pub fn divides_shots_between_targets(&self) -> bool {
        self.0[1] == 0
    }
    /// Whether the packet covers an area (target type 2 and up). The type is read as signed, so
    /// types 128..255 target bodies like 0 and 1.
    pub fn requires_area_mask(&self) -> bool {
        self.0[1] as i8 >= 2
    }
    /// Whether the attack turns by a random rotation each use. The draw is consumed even when
    /// no animation is drawn.
    pub fn can_rotate(&self) -> bool {
        self.0[7] != 0
    }
    /// Lowest and highest base duration of the attack's condition.
    pub fn duration_range(&self) -> (i16, i16) {
        (i16::from(self.0[13] as i8), i16::from(self.0[14] as i8))
    }
    /// The sound played as the attack is launched.
    pub fn launch_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 36)
    }
    /// The sound played where the attack lands.
    pub fn impact_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 38)
    }
    /// Whether the impact sound plays when the target is wounded.
    pub fn plays_impact_when_wounded(&self) -> bool {
        self.0[19] == 9
    }
    /// The effect on health the attack carries.
    pub fn health_effect(&self) -> i8 {
        self.0[18] as i8
    }
}

/// The spell info packet as a view that can change it.
pub struct SpellInfoRecordMut<'a>(pub &'a mut [u8]);

impl SpellInfoRecordMut<'_> {
    /// Chooses the area pattern.
    pub fn set_area_size(&mut self, shape: u8) {
        self.0[31] = shape;
    }
    /// Turns the area pattern by `rotation` steps; `None` when the pattern number overflows.
    pub fn rotate_area(&mut self, rotation: u8) -> Option<()> {
        self.0[31] = self.0[31].checked_add(rotation)?;
        Some(())
    }
    /// Readies the packet for a collision with a field: direct weapon damage becomes
    /// an ordinary attack and no new field is laid.
    pub fn prepare_field_collision(&mut self) {
        if self.0[19] == 9 {
            self.0[19] = 1;
            self.0[20] = 1;
        }
        self.0[4] = 0;
    }
    /// Adds `bonus` to the saving throw chance gained for each power level.
    pub fn add_save_bonus(&mut self, bonus: i16) {
        self.0[8] = self.0[8].wrapping_add(bonus as u8);
    }
    /// Sets the saving-throw mode.
    pub fn set_saving_throw_mode(&mut self, mode: u8) {
        self.0[9] = mode;
    }

    /// Sets the effect on health.
    pub fn set_health_effect(&mut self, effect: u8) {
        self.0[18] = effect;
    }
    /// Sets the damage of element `element`.
    pub fn set_elemental_damage(&mut self, element: usize, amount: i16) {
        self.0[23 + element] = amount as u8;
    }
    /// Sets the smallest and largest physical damage.
    pub fn set_damage_bounds(&mut self, minimum: i16, maximum: i16) {
        super::put_word(self.0, 32, minimum);
        super::put_word(self.0, 34, maximum);
    }
    /// Sets the sound played where the attack lands.
    pub fn set_impact_sound(&mut self, sound: i16) {
        super::put_word(self.0, 38, sound);
    }
    /// Adds `modifier` to the range.
    pub fn add_range_modifier(&mut self, modifier: u8) {
        self.0[2] = self.0[2].wrapping_add(modifier);
    }
    /// Clears the defense category, so the attack no longer counts as weapon damage.
    pub fn reset_turn_attack_kind(&mut self) {
        self.0[19] = 0;
    }
}
