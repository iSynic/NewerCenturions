use centurions_content::{Content, read_resource};
use centurions_core::Event;
use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound};
use std::{cell::Cell, collections::HashMap, path::Path};

pub struct Audio {
    sounds: HashMap<i16, Sound>,
    volume: Cell<f32>,
}
impl Audio {
    pub async fn load(content: &Content, root: &Path) -> Result<Self, String> {
        let mut sounds = HashMap::new();
        for id in [
            82, 83, 85, 87, 88, 100, 101, 102, 104, 105, 107, 108, 109, 134, 137, 138, 139, 141,
            145, 200, 204, 205, 206, 214, 215, 231, 232, 240, 241, 242, 344, 350, 354, 355, 357,
            363, 373, 389, 397, 398, 402, 403, 404, 405, 406, 409, 422, 430, 431, 450, 500, 502,
            503, 505, 507, 508, 550, 551, 552, 553, 601, 602, 605, 615, 619, 624, 636, 637, 642,
            644, 650, 631, 651, 655, 663, 681, 694, 1000, 1101, 3002, 6000, 6001, 6002, 10003,
            10105, 10107, 10121, 10122, 10129, 10136, 14250, 20001, 20002, 20005, 30000, 30001,
            30002, 30003, 20003,
        ] {
            let resource = content
                .resources
                .iter()
                .find(|r| r.r#type == "snd " && r.id == id)
                .ok_or_else(|| format!("sound {id} missing"))?;
            let wave = centurions_content::audio::decode_snd(
                &read_resource(&root.join("analysis/extracted"), resource)
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?
            .wave();
            sounds.insert(
                id,
                load_sound_from_bytes(&wave)
                    .await
                    .map_err(|e| e.to_string())?,
            );
        }
        Ok(Self {
            sounds,
            volume: Cell::new(1.),
        })
    }
    pub fn set_volume(&self, value: usize) {
        self.volume.set(value.min(10) as f32 / 10.);
    }
    pub fn play(&self, events: &[Event]) {
        for event in events {
            if let Event::Sound { resource_id } = event
                && let Some(sound) = self.sounds.get(&resource_id.wrapping_abs())
            {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: false,
                        volume: self.volume.get(),
                    },
                );
            }
        }
    }
}
