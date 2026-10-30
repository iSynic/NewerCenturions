use crate::{ContentError, mission::MissionStage, word};

/// The data that lies after the tactical map (`field`) in the game's globals:
/// object layer, events and the mission table. A spell's target scan can index
/// past the map into them.
pub struct SpellMapAliases<'a> {
    objects: &'a [i16],
    events: &'a [i32],
    stages: &'a [MissionStage],
}

impl<'a> SpellMapAliases<'a> {
    /// Gathers the object layer, events and mission stages that follow the map.
    pub fn new(objects: &'a [i16], events: &'a [i32], stages: &'a [MissionStage]) -> Self {
        Self {
            objects,
            events,
            stages,
        }
    }

    /// A word read at a signed index from the start of the tactical map. It
    /// never writes.
    pub fn tactical_word(&self, index: i32) -> Result<Option<i16>, ContentError> {
        let address = 0x4085b8_i64 + i64::from(index) * 2;
        if (0x40c500..0x410448).contains(&address) {
            return Ok(self
                .objects
                .get(((address - 0x40c500) / 2) as usize)
                .copied());
        }
        if (0x410448..0x4182d8).contains(&address) {
            let offset = (address - 0x410448) as usize;
            return Ok(self.events.get(offset / 4).map(|value| {
                if offset.is_multiple_of(4) {
                    (*value >> 16) as i16
                } else {
                    *value as i16
                }
            }));
        }
        // The whole Data Mission table is loaded when the mission name is shown.
        if (0x41d1c4..0x41f604).contains(&address) {
            let offset = (address - 0x41d1c4) as usize;
            return self
                .stages
                .get(offset / 464)
                .map(|stage| word(&stage.record, offset % 464))
                .transpose();
        }
        Ok(None)
    }
}
