use crate::{ContentError, InventoryRecord, mission::long, word};

// edititems 0x134034 display/control order, shared by drawing and mutation.
const EDITOR_MODIFIERS: [usize; 10] = [13, 15, 16, 11, 17, 18, 19, 14, 21, 20];

/// Original 150-byte definition, borrowed without eager validation. Callers
/// keep their existing field-read order and malformed-record diagnostics.
#[derive(Clone, Copy)]
pub struct ItemDefinition<'a>(pub &'a [u8]);

impl ItemDefinition<'_> {
    pub fn icon(&self) -> Result<i16, ContentError> {
        word(self.0, 10)
    }
    pub fn description_id(&self) -> Result<i16, ContentError> {
        word(self.0, 102)
    }
    pub fn pickup_sound(&self) -> Result<i16, ContentError> {
        word(self.0, 12)
    }
    pub fn base_weight(&self) -> Result<i16, ContentError> {
        word(self.0, 16)
    }
    pub fn supplied_quantity(&self) -> Result<i16, ContentError> {
        word(self.0, 20)
    }
    pub fn rounds_per_shot(&self) -> Result<i16, ContentError> {
        word(self.0, 22)
    }
    pub fn category(&self) -> Result<i16, ContentError> {
        word(self.0, 26)
    }
    /// Retained by loaditem and saved unchanged; callers must not infer a
    /// single universal meaning for this category-dependent native field.
    pub fn retained_field32(&self) -> Result<i16, ContentError> {
        word(self.0, 32)
    }
    pub fn reserve_ammunition_kind(&self) -> Result<i16, ContentError> {
        word(self.0, 34)
    }
    pub fn unit_weight(&self) -> Result<i16, ContentError> {
        word(self.0, 36)
    }
    pub fn spell(&self) -> Result<i16, ContentError> {
        word(self.0, 44)
    }
    pub fn price(&self) -> Result<i32, ContentError> {
        long(self.0, 142)
    }
    /// installitem 0x12c874 fresh-stock branch. Capacity selection and copied
    /// modifiers stay distinct from updateshop's retained display record.
    pub fn fresh_stock(&self) -> Result<InventoryRecord, ContentError> {
        let mut record = InventoryRecord([0; 22]);
        let quantity = word(self.0, 148)?.wrapping_add(word(self.0, 20)?);
        let capacity = match word(self.0, 90)? {
            0 => match word(self.0, 24)? {
                0 => quantity,
                n => n,
            },
            n => n,
        };
        for (offset, value) in [(0, word(self.0, 102)?), (2, quantity), (6, capacity)] {
            record.0[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        for (destination, source) in [
            (8, 35),
            (10, 27),
            (13, 139),
            (14, 78),
            (15, 81),
            (16, 83),
            (17, 84),
            (18, 85),
            (19, 86),
            (21, 80),
            (20, 79),
        ] {
            record.0[destination] = self.0[source];
        }
        Ok(record)
    }

    /// updateshop 0x139244 deliberately leaves instance bytes 11 and 12 intact.
    pub fn copy_display_stock_into(
        &self,
        record: &mut InventoryRecord,
    ) -> Result<(), ContentError> {
        for (offset, value) in [
            (0, word(self.0, 102)?),
            (2, word(self.0, 20)?),
            (4, 0),
            (6, word(self.0, 20)?),
        ] {
            record.0[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        record.0[9] = 0;
        for (destination, source) in [
            (13, 139),
            (8, 35),
            (10, 27),
            (14, 78),
            (15, 81),
            (16, 83),
            (17, 84),
            (18, 85),
            (19, 86),
            (21, 80),
            (20, 79),
        ] {
            record.0[destination] = self.0[source];
        }
        Ok(())
    }
}

impl InventoryRecord {
    pub fn editor_modifier(&self, row: usize) -> i8 {
        self.0[EDITOR_MODIFIERS[row]] as i8
    }
    pub fn set_editor_modifier(&mut self, row: usize, value: u8) {
        self.0[EDITOR_MODIFIERS[row]] = value;
    }
    pub fn adjust_editor_modifier(&mut self, row: usize, delta: i8) {
        let value = &mut self.0[EDITOR_MODIFIERS[row]];
        *value = value.wrapping_add(delta as u8);
    }
    pub fn set_quantity(&mut self, quantity: i16) {
        super::put_word(&mut self.0, 2, quantity);
    }
    pub fn capacity(&self) -> Result<i16, ContentError> {
        word(&self.0, 6)
    }
    pub fn set_capacity(&mut self, capacity: i16) {
        super::put_word(&mut self.0, 6, capacity);
    }
    /// Stack count or loaded rounds, according to the original item category.
    pub fn quantity(&self) -> Result<i16, ContentError> {
        word(&self.0, 2)
    }
    pub fn ammunition_id(&self) -> Result<i16, ContentError> {
        word(&self.0, 4)
    }
    pub fn equipment_flag(&self) -> u8 {
        self.0[9]
    }
    pub fn category_code(&self) -> u8 {
        self.0[10]
    }
}
