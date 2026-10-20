//! Contexts for combat/spells.

pub struct DirectSpell<'a> {
    pub packet: &'a [u8],
    pub level: i16,
    pub count: i16,
    pub armor_filter: bool,
    pub weapon_sound: i16,
}

pub struct FieldUsage {
    pub slot: usize,
    pub used: Vec<u8>,
}

pub struct SpellContext<'a> {
    pub packet: &'a [u8],
    pub id: i16,
    pub count: i16,
    pub caster: i16,
    pub initiative: i16,
    pub collision: bool,
}

pub struct SpellMask {
    pub mask: [i16; 110],
    pub fields: Vec<u8>,
}
