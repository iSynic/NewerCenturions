use centurions_content::{Content, read_resource};
use centurions_core::Event;
use macroquad::audio::{
    PlaySoundParams, Sound, load_sound_from_bytes, play_sound, set_sound_volume, stop_sound,
};
use std::{cell::Cell, collections::HashMap, path::Path};

pub struct Audio {
    sounds: HashMap<i16, Sound>,
    volume: Cell<f32>,
    music: Vec<(i16, Sound, f64)>,
    current_music: Cell<Option<usize>>,
    music_until: Cell<f64>,
}
impl Audio {
    pub async fn load(content: &Content, resource_root: &Path) -> Result<Self, String> {
        let mut sounds = HashMap::new();
        for id in [
            82, 83, 85, 87, 88, 100, 101, 102, 104, 105, 107, 108, 109, 133, 134, 137, 138, 139,
            141, 144, 145, 148, 200, 204, 205, 206, 214, 215, 231, 232, 240, 241, 242, 344, 350,
            354, 355, 357, 363, 373, 389, 397, 398, 402, 403, 404, 405, 406, 409, 422, 430, 431,
            450, 500, 502, 503, 505, 507, 508, 550, 551, 552, 553, 601, 602, 605, 615, 619, 624,
            636, 637, 642, 644, 650, 631, 651, 655, 663, 681, 694, 1000, 1101, 3002, 6000, 6001,
            6002, 10003, 10004, 10105, 10107, 10121, 10122, 10129, 10136, 14250, 20001, 20002,
            20005, 30000, 30001, 30002, 30003, 20003,
        ] {
            let Some(resource) = content
                .resources
                .iter()
                .find(|r| r.r#type == "snd " && r.id == id)
            else {
                continue;
            };
            let wave = centurions_content::audio::decode_snd(
                &read_resource(resource_root, resource)
                    .map_err(|e| format!("Sound {id} from {}: {e}", resource.original))?,
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
        let mut music = Vec::new();
        for resource in content
            .resources
            .iter()
            .filter(|r| r.r#type == "snd " && (20128..=20140).contains(&r.id))
        {
            let sample = centurions_content::audio::decode_snd(
                &read_resource(resource_root, resource).map_err(|e| {
                    format!("Music {} from {}: {e}", resource.id, resource.original)
                })?,
            )
            .map_err(|e| format!("Music {}: {e}", resource.id))?;
            let seconds = sample.samples.len() as f64
                / (sample.bits_per_sample / 8 * sample.channels) as f64
                / (sample.rate_fixed as f64 / 65536.);
            let sound = load_sound_from_bytes(&sample.wave())
                .await
                .map_err(|e| e.to_string())?;
            music.push((resource.id, sound, seconds));
        }
        music.sort_by_key(|track| track.0);
        Ok(Self {
            sounds,
            volume: Cell::new(1.),
            music,
            current_music: Cell::new(None),
            music_until: Cell::new(0.),
        })
    }
    pub fn set_volume(&self, value: usize) {
        self.volume.set(value.min(10) as f32 / 10.);
        if let Some(index) = self.current_music.get() {
            set_sound_volume(&self.music[index].1, self.volume.get());
        }
    }
    pub fn music(&self, enabled: bool, requested: Option<i16>, now: f64) {
        if !enabled {
            if let Some(index) = self.current_music.take() {
                stop_sound(&self.music[index].1);
            }
            return;
        }
        if self.music.is_empty() {
            return;
        }
        let requested = requested.and_then(|id| self.music.iter().position(|track| track.0 == id));
        let current = self.current_music.get();
        if requested.is_none() && current.is_some() && now < self.music_until.get() {
            return;
        }
        let index = requested.unwrap_or_else(|| current.map_or(0, |i| (i + 1) % self.music.len()));
        if let Some(previous) = current {
            stop_sound(&self.music[previous].1);
        }
        play_sound(
            &self.music[index].1,
            PlaySoundParams {
                looped: false,
                volume: self.volume.get(),
            },
        );
        self.current_music.set(Some(index));
        self.music_until.set(now + self.music[index].2);
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
