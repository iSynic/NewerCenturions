use super::put_word;
use crate::{ContentError, combat::MonsterRecord, word};

impl MonsterRecord {
    /// The monster's record as a read-only view.
    pub fn record(&self) -> EnemyRecord<'_> {
        EnemyRecord(&self.bytes)
    }
    /// The monster's record as a view that can change it.
    pub fn record_mut(&mut self) -> EnemyRecordMut<'_> {
        EnemyRecordMut(&mut self.bytes)
    }
}

/// A monster's record (`monster[n]`), loaded from the level's Data MD bestiary
/// and changed as the battle runs.
pub struct EnemyRecord<'a>(&'a [u8]);

/// One of the four map cells a large monster covers; the anchor is the cell it
/// stands on.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FootprintCell {
    Anchor,
    North,
    West,
    Northwest,
}
impl FootprintCell {
    /// Locates the cell's saved tile in the monster record.
    fn offset(self) -> usize {
        match self {
            Self::Anchor => 0x11e,
            Self::North => 0x11c,
            Self::West => 0x11a,
            Self::Northwest => 0x118,
        }
    }
}

/// One of a monster's natural attacks, as the attack rules read it.
pub struct NaturalAttack {
    /// Smallest damage.
    pub minimum: i16,
    /// Largest damage.
    pub maximum: i16,
    /// The sound played where the attack lands.
    pub impact_sound: i16,
    /// The condition the attack inflicts; zero is none.
    pub condition: u8,
    /// Elemental effects read the selected bound even when physical damage falls back.
    pub effect_maximum: i16,
}

impl<'a> EnemyRecord<'a> {
    /// Views a monster record held in `bytes`.
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
    /// Stamina added to the dice rolled for the monster's stamina.
    pub fn stamina_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// The story script that runs when the monster dies; zero runs none.
    pub fn todo_on_death(&self) -> Result<i16, ContentError> {
        word(self.0, 0x60)
    }
    /// The monster's number in the bestiary.
    pub fn monster_id(&self) -> i8 {
        self.0[0x103] as i8
    }
    /// The monster's level: how many dice of stamina it rolls.
    pub fn hit_dice(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// Most credits the monster drops when it dies.
    pub fn credit_drop_bound(&self) -> Result<i16, ContentError> {
        word(self.0, 0x3a)
    }
    /// Victory points the monster is worth.
    pub fn victory_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x54)
    }
    /// Nonzero when the monster flies.
    pub fn flying_flag(&self) -> u8 {
        self.0[0x7f]
    }
    /// One of the six items the monster carries; booty collects them when it falls.
    pub fn item(&self, slot: usize) -> Result<i16, ContentError> {
        word(self.0, 0x26 + slot * 2)
    }
    /// The monster's agility, which orders it in the battle.
    pub fn agility(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// Movement the monster gets each round.
    pub fn movement_max(&self) -> Result<i16, ContentError> {
        word(self.0, 6)
    }
    /// Percent chance each turn that the monster runs its turn script.
    pub fn turn_macro_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x5c)
    }
    /// Accuracy added to the monster's missile attacks.
    pub fn missile_accuracy_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x66)
    }
    /// Damage added to each of the monster's attacks.
    pub fn damage_plus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x40)
    }
    /// The monster's natural attack `index`; an attack with no damage of its own
    /// borrows the first attack's damage and sound.
    pub fn natural_attack(&self, index: usize) -> NaturalAttack {
        let selected = 0x93 + index * 4;
        let damage = if self.0[selected] == 0 {
            0x93
        } else {
            selected
        };
        NaturalAttack {
            minimum: i16::from(self.0[damage] as i8),
            maximum: i16::from(self.0[damage + 1] as i8),
            impact_sound: i16::from(self.0[damage + 2] as i8),
            // Only damage and sound fall back; the condition stays the selected one.
            condition: self.0[selected + 3],
            effect_maximum: i16::from(self.0[selected + 1] as i8),
        }
    }
    /// Taken off the chance to hit the monster with a missile attack.
    pub fn missile_dodge_bonus(&self) -> Result<i16, ContentError> {
        word(self.0, 0x62)
    }
    /// Reactions clear this byte; an attack reads it as a signed multiplier.
    pub fn guarding(&self) -> i8 {
        self.0[0x104] as i8
    }
    /// Remaining duration of `condition`; a negative duration is permanent.
    pub fn condition(&self, condition: super::Condition) -> i16 {
        self.condition_duration(condition as usize)
    }
    /// Remaining duration of condition `index`; a negative duration is permanent.
    pub fn condition_duration(&self, index: usize) -> i16 {
        i16::from(self.0[0xb2 + index] as i8)
    }
    /// The monster's resistance to scans.
    pub fn psi_resistance(&self) -> Result<i16, ContentError> {
        word(self.0, 0xa)
    }
    /// The monster's saving throw in one category.
    pub fn save(&self, category: usize) -> i16 {
        i16::from(self.0[0x8b + category] as i8)
    }
    /// The monster's protection flag for scan level `level`; zero is none.
    pub fn level_protection(&self, level: usize) -> u8 {
        self.0[0xc2 + level]
    }
    /// `level_protection` for a signed level, as resistance checks index it.
    pub fn signed_level_protection(&self, level: i16) -> Result<u8, ContentError> {
        let offset = usize::try_from(0xc2 + i32::from(level)).unwrap_or(usize::MAX);
        Ok(crate::span(self.0, offset, 1)?[0])
    }
    /// Zero-based entry of Cargo Pod `STR#` 130 "Race Class": "Mechanical".
    pub const MECHANICAL_CATEGORY: usize = 3;
    /// Conditions, faction word and stamina word, in the order the final booty
    /// cleanup zeroes them.
    pub const REWARD_CLEARED_FIELDS: [std::ops::Range<usize>; 3] =
        [0xb2..0xda, 0xe..0x10, 0x110..0x112];
    /// Whether the monster belongs to creature category `category`.
    pub fn has_creature_category(&self, category: usize) -> bool {
        self.0[0xa8 + category] != 0
    }
    /// The monster's name.
    pub fn name(&self) -> Result<String, ContentError> {
        crate::combat::monster_name(self.0)
    }
    /// Which side the monster fights on; nonzero is the monsters', zero the party's.
    pub fn faction(&self) -> Result<i16, ContentError> {
        word(self.0, 0xe)
    }
    /// Current stamina.
    pub fn stamina(&self) -> Result<i16, ContentError> {
        word(self.0, 0x110)
    }
    /// Maximum stamina.
    pub fn stamina_max(&self) -> Result<i16, ContentError> {
        word(self.0, 0x112)
    }
    /// Current Psi points.
    pub fn psi_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x10e)
    }
    /// Maximum Psi points.
    pub fn psi_points_max(&self) -> Result<i16, ContentError> {
        word(self.0, 0x52)
    }
    /// Percent chance that the monster's power shield takes a hit.
    pub fn shield_chance(&self) -> Result<i16, ContentError> {
        word(self.0, 0x56)
    }
    /// Facing byte: 1 draws the mirrored icon (the icon id plus 1000); a monster west of the
    /// party starts at 1.
    pub fn facing_left(&self) -> u8 {
        self.0[0x107]
    }
    /// Charge left in the monster's power shield.
    pub fn shield_points(&self) -> Result<i16, ContentError> {
        word(self.0, 0x58)
    }
    /// Which weapons can hurt the monster in melee: zero any; -1 or -2 only a weapon of that
    /// immunity class; any other value only the weapon whose description it names.
    pub fn weapon_immunity(&self) -> Result<i16, ContentError> {
        word(self.0, 12)
    }
    /// The monster's armor rating.
    pub fn armor_rating(&self) -> Result<i16, ContentError> {
        word(self.0, 8)
    }
    /// The icon that shows the monster on the battle map.
    pub fn icon(&self) -> Result<i16, ContentError> {
        word(self.0, 0x50)
    }
    /// Nonzero lets a surviving friendly monster be offered as an ally after the battle; -1
    /// makes it friendly and pre-selected, with a warning if it is deselected.
    pub fn survivor_marker(&self) -> Result<i16, ContentError> {
        word(self.0, 0x5a)
    }
    /// Whether the monster has been taken out of the battle.
    pub fn removed(&self) -> bool {
        self.0[0xb0] == 255
    }
    /// 0 is an ordinary body, 1 a mechanical body, 255 a disabled body.
    /// Capture and targeting deliberately test different values.
    pub fn mechanical_state(&self) -> u8 {
        self.0[0xb0]
    }
    /// Nonzero when the monster can be captured.
    pub fn capture_flag(&self) -> u8 {
        self.0[0xb1]
    }
    /// The monster's size, 0 to 3 ignoring sign: 0 covers its own cell, 1 adds the cell
    /// north, 2 the cell west, and 3 all four.
    pub fn size(&self) -> Result<i16, ContentError> {
        word(self.0, 0x10)
    }
    /// The tile under the monster's anchor cell, put back when it leaves.
    pub fn underlying_tile(&self) -> Result<i16, ContentError> {
        word(self.0, 0x11e)
    }
    /// The tile under the monster's cell `cell`, put back when it leaves.
    pub fn footprint_tile(&self, cell: FootprintCell) -> Result<i16, ContentError> {
        word(self.0, cell.offset())
    }
    /// The sound the monster's steps make.
    pub fn movement_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 0x6c)
    }
    /// The sound played when the monster dies.
    pub fn death_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 0x6a)
    }
    /// The body the monster is aiming at; -1 is none.
    pub fn target(&self) -> Result<i16, ContentError> {
        word(self.0, 0x10c)
    }
    /// Excludes the monster only while a new opponent is being chosen.
    pub fn excluded_from_target_scan(&self) -> bool {
        self.0[0x82] != 0
    }
    /// Item id of the first item the monster carries, the weapon it starts with.
    pub fn primary_weapon(&self) -> Result<i16, ContentError> {
        word(self.0, 0x26)
    }
    /// Item id of the monster's melee weapon.
    pub fn melee_weapon(&self) -> Result<i16, ContentError> {
        word(self.0, 0x28)
    }
    /// Item id of the weapon the monster fights with now.
    pub fn weapon(&self) -> Result<i16, ContentError> {
        word(self.0, 0x4e)
    }
    /// Item id of the ammunition loaded in the monster's missile weapon; zero is none.
    pub fn ammunition(&self) -> Result<i16, ContentError> {
        word(self.0, 0x4c)
    }
    /// Percent chance that the monster fires its missile weapon.
    pub fn missile_percent(&self) -> Result<i16, ContentError> {
        word(self.0, 0x48)
    }
    /// Percent chance that the monster casts a scan.
    pub fn cast_percent(&self) -> Result<i16, ContentError> {
        word(self.0, 0x42)
    }
    /// Compared, signed, with the monster's stamina as a percentage of its
    /// maximum.
    pub fn run_percent(&self) -> Result<i16, ContentError> {
        word(self.0, 0x44)
    }
    /// Checked at the start of the monster's turn; a monster that leaves by a
    /// map edge is set to 101.
    pub fn surrender_percent(&self) -> Result<i16, ContentError> {
        word(self.0, 0x46)
    }
    /// One of the ten spells (`spells`) a monster chooses from.
    pub fn spell_slot(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 0x12 + index * 2)
    }
    /// The story script the monster runs on its turn.
    pub fn turn_macro(&self) -> Result<i16, ContentError> {
        word(self.0, 0x5e)
    }
    /// Actions the monster has left this round.
    pub fn action_points(&self) -> i8 {
        self.0[0x109] as i8
    }
    /// How many attacks the monster makes each round.
    pub fn attack_count(&self) -> i8 {
        self.0[0x8a] as i8
    }
    /// How many scans the monster may cast each round.
    pub fn magic_attack_count(&self) -> i8 {
        self.0[0x7e] as i8
    }
    /// Rounds left on the monster's Helpless condition, during which it skips its turns.
    pub fn turn_skip_duration(&self) -> u8 {
        self.0[0xb3]
    }
    /// Rounds left on the monster's Retreating condition.
    pub fn fleeing_duration(&self) -> u8 {
        self.0[0xb2]
    }
    /// Nonzero while the monster's departure from a cell draws no reaction attacks.
    pub fn reaction_suppression(&self) -> u8 {
        self.0[0xca]
    }
    /// Movement left this round.
    pub fn movement(&self) -> i8 {
        self.0[0x106] as i8
    }
    /// Whether the monster is scan-blocked, confused or helpless, or has been attacked; any of
    /// these stops it casting before it moves.
    pub fn spell_blocked_before_movement(&self) -> bool {
        [0xb7, 0xcf, 0xd9, 0xb3, 0x105]
            .iter()
            .any(|offset| self.0[*offset] != 0)
    }
    /// The same checks as before moving, made in a different order, after the monster has
    /// moved.
    pub fn spell_blocked_after_movement(&self) -> bool {
        [0xb3, 0xb7, 0xcf, 0xd9, 0x105]
            .iter()
            .any(|offset| self.0[*offset] != 0)
    }
}

/// A monster's record as a view that can change it.
pub struct EnemyRecordMut<'a>(&'a mut [u8]);

impl EnemyRecordMut<'_> {
    /// Forgets which bestiary monster the record was.
    pub fn clear_monster_id(&mut self) {
        self.0[0x103] = 0;
    }

    /// A survivor carried back to the menu leaves its level, stamina and
    /// rewards behind.
    pub fn clear_survivor_rewards(&mut self) {
        put_word(self.0, 0, 0);
        put_word(self.0, 0x110, 0);
        for slot in 0..6 {
            put_word(self.0, 0x26 + slot * 2, 0);
        }
        put_word(self.0, 0x3a, 0);
    }

    /// Sets whether the monster is left out of the next opponent choice.
    pub fn set_target_scan_excluded(&mut self, excluded: bool) {
        self.0[0x82] = u8::from(excluded);
    }
    /// Sets the story script that runs when the monster dies.
    pub fn set_todo_on_death(&mut self, script: i16) {
        put_word(self.0, 0x60, script);
    }
    /// Sets the facing byte; battle start gives 1 to a monster west of the party and -1 to one
    /// east of it.
    pub fn set_horizontal_direction(&mut self, direction: i8) {
        self.0[0x107] = direction as u8;
    }

    /// Sets the resistance to scans.
    pub fn set_psi_resistance(&mut self, value: i16) {
        put_word(self.0, 0xa, value);
    }
    /// Adds `amount` to each of the six saving throws.
    pub fn adjust_elemental_saves(&mut self, amount: i16) {
        for value in &mut self.0[0x8b..0x91] {
            *value = value.wrapping_add(amount as u8);
        }
    }
    /// Sets the armor rating.
    pub fn set_armor_rating(&mut self, value: i16) {
        put_word(self.0, 8, value);
    }
    /// Sets the agility.
    pub fn set_agility(&mut self, value: i16) {
        put_word(self.0, 4, value);
    }
    /// Sets maximum stamina.
    pub fn set_stamina_max(&mut self, value: i16) {
        put_word(self.0, 0x112, value);
    }
    /// Sets maximum Psi points.
    pub fn set_psi_points_max(&mut self, value: i16) {
        put_word(self.0, 0x52, value);
    }

    /// Records the tile under the monster's cell `cell`.
    pub fn set_footprint_tile(&mut self, cell: FootprintCell, tile: i16) {
        put_word(self.0, cell.offset(), tile);
    }
    /// Sets the monster's facing from a step of (`dx`, `dy`).
    pub fn set_movement_direction(&mut self, dx: i8, dy: i8) {
        self.0[0x107] = dx as u8;
        self.0[0x108] = dy as u8;
    }
    /// Sets the percent chance that the monster surrenders.
    pub fn set_surrender_percent(&mut self, chance: i16) {
        put_word(self.0, 0x46, chance);
    }

    /// Sets whether the body is ordinary, mechanical or disabled.
    pub fn set_mechanical_state(&mut self, value: u8) {
        self.0[0xb0] = value;
    }
    /// Ends every condition.
    ///
    /// Original: killbody
    pub fn clear_conditions(&mut self) {
        self.0[0xb2..0xda].fill(0);
    }
    /// Sets the percent chance each turn that the monster runs its turn script.
    pub fn set_turn_macro_chance(&mut self, value: i16) {
        put_word(self.0, 0x5c, value);
    }
    /// Moves the monster to a side.
    pub fn set_faction(&mut self, value: i16) {
        put_word(self.0, 0xe, value);
    }
    /// Sets how long `condition` lasts.
    pub fn set_condition(&mut self, condition: super::Condition, duration: i16) {
        self.set_condition_duration(condition as usize, duration);
    }
    /// Sets how long condition `index` lasts.
    pub fn set_condition_duration(&mut self, index: usize, duration: i16) {
        self.0[0xb2 + index] = duration as u8;
    }
    /// A speedy monster gains copies of its first attack in empty slots only;
    /// a filled slot whose damage later falls back to slot zero is left alone.
    pub fn fill_missing_natural_attack(&mut self, slot: usize) {
        let offset = 0x93 + slot * 4;
        if self.0[offset] == 0 {
            self.0.copy_within(0x93..0x97, offset);
        }
    }
    /// Clears the attacked mark.
    pub fn clear_been_attacked(&mut self) {
        self.0[0x105] = 0;
    }
    /// Counts down every running condition by one; a permanent one stays.
    ///
    /// Original: reduce
    pub fn decrement_temporary_conditions(&mut self) {
        for duration in &mut self.0[0xb2..0xda] {
            if (*duration as i8) > 0 {
                *duration -= 1;
            }
        }
    }
    /// Counts down the six party conditions an ally keeps.
    pub fn decrement_party_conditions(&mut self) {
        for value in &mut self.0[0xb2..0xb8] {
            if (*value as i8) > 0 {
                *value -= 1;
            }
        }
    }
    /// Views a monster record held in `bytes` for changing.
    pub fn from_bytes(bytes: &mut [u8]) -> EnemyRecordMut<'_> {
        EnemyRecordMut(bytes)
    }
    /// Marks the monster as attacked; the round's body count clears the mark.
    pub fn mark_been_attacked(&mut self) {
        self.0[0x105] = 1;
    }
    /// Sets current stamina.
    pub fn set_stamina(&mut self, value: i16) {
        put_word(self.0, 0x110, value);
    }
    /// Sets current Psi points.
    pub fn set_psi_points(&mut self, value: i16) {
        put_word(self.0, 0x10e, value);
    }
    /// Sets the actions left this round.
    pub fn set_action_points(&mut self, value: i8) {
        self.0[0x109] = value as u8;
    }
    /// Records the tile under the monster's anchor cell.
    pub fn set_underlying_tile(&mut self, value: i16) {
        put_word(self.0, 0x11e, value);
    }
    /// Aims the monster at `body`; -1 clears it.
    pub fn set_target(&mut self, value: i16) {
        put_word(self.0, 0x10c, value);
    }
    /// Sets the first item the monster carries.
    pub fn set_primary_weapon(&mut self, value: i16) {
        put_word(self.0, 0x26, value);
    }
    /// Sets the weapon the monster fights with.
    pub fn set_weapon(&mut self, value: i16) {
        put_word(self.0, 0x4e, value);
    }
    /// Turns the monster to face a target, from the x of its own cell to the x of the
    /// target's; it keeps its facing when they are level.
    pub fn face_towards(&mut self, origin_x: i16, target_x: i16) {
        if origin_x < target_x && self.0[0x107] == 0 {
            self.0[0x107] = 1;
        } else if target_x < origin_x && self.0[0x107] == 1 {
            self.0[0x107] = 0;
        }
    }
    /// Sets the guarding byte: 1 while the monster has actions left, else 0.
    pub fn set_reaction_ready(&mut self, ready: bool) {
        self.0[0x104] = u8::from(ready);
    }
    /// Sets the movement left this round.
    pub fn set_movement(&mut self, points: i8) {
        self.0[0x106] = points as u8;
    }
}
