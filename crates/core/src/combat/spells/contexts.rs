//! The packet, caster and field-usage inputs a spell resolution reads.

/// A spell being cast on bodies: its packet, power level, count, whether armor filters its damage,
/// and its weapon sound.
pub struct DirectSpell<'a> {
    pub packet: &'a [u8],
    pub level: i16,
    pub count: i16,
    pub armor_filter: bool,
    pub weapon_sound: i16,
}

/// Which spell fields have already struck this turn: the field being examined and the usage marks.
pub struct FieldUsage {
    pub slot: usize,
    pub used: Vec<u8>,
}

/// A spell being resolved: its packet, number, count, caster, the caster's initiative, and whether
/// it is a field collision.
pub struct SpellContext<'a> {
    pub packet: &'a [u8],
    pub id: i16,
    pub count: i16,
    pub caster: i16,
    pub initiative: i16,
    pub collision: bool,
}

/// The bodies a spell reaches, one mark per body, and the table of spell fields, including the
/// field the spell itself lays.
pub struct SpellMask {
    pub mask: [i16; 110],
    pub fields: Vec<u8>,
}
