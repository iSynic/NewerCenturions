use centurions_content::{Content, read_resource};
use centurions_core::Event;
use macroquad::audio::{
    PlaySoundParams, Sound, load_sound_from_bytes, play_sound, set_sound_volume, stop_sound,
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    path::Path,
};

pub struct Audio {
    sounds: HashMap<i16, (Sound, f64)>,
    volume: Cell<f32>,
    music_volume: Cell<f32>,
    music: Vec<(i16, Sound, f64)>,
    current_music: Cell<Option<usize>>,
    music_until: Cell<f64>,
    pending_sounds: RefCell<VecDeque<(f64, i16)>>,
    ticks_per_second: Cell<f64>,
}
impl Audio {
    pub async fn load(content: &Content, resource_root: &Path) -> Result<Self, String> {
        let mut sounds = HashMap::new();
        for resource in content
            .resources
            .iter()
            .filter(|r| r.r#type == "snd " && !(20128..=20140).contains(&r.id))
        {
            let id = resource.id;
            let sample = centurions_content::audio::decode_snd(
                &read_resource(resource_root, resource)
                    .map_err(|e| format!("Sound {id} from {}: {e}", resource.original))?,
            )
            .map_err(|e| format!("Sound {id} from {}: {e}", resource.original))?;
            let wave = sample.wave();
            let seconds = sample.samples.len() as f64
                / f64::from(sample.bits_per_sample / 8 * sample.channels)
                / (f64::from(sample.rate_fixed) / 65536.);
            sounds.insert(
                id,
                (
                    load_sound_from_bytes(&wave)
                        .await
                        .map_err(|e| e.to_string())?,
                    seconds,
                ),
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
            music_volume: Cell::new(1.),
            music,
            current_music: Cell::new(None),
            music_until: Cell::new(0.),
            pending_sounds: RefCell::new(VecDeque::new()),
            ticks_per_second: Cell::new(60.),
        })
    }
    pub fn set_volumes(&self, effects: usize, music: usize) {
        let effects = effects.min(10) as f32 / 10.;
        if self.volume.replace(effects) != effects {
            for (sound, _) in self.sounds.values() {
                set_sound_volume(sound, effects);
            }
        }
        let music = music.min(10) as f32 / 10.;
        if self.music_volume.replace(music) != music
            && let Some(index) = self.current_music.get()
        {
            set_sound_volume(&self.music[index].1, music);
        }
    }
    pub fn music(&self, enabled: bool, requested: Option<i16>, playlist: &[bool; 13], now: f64) {
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
        let start = current.map_or(0, |i| (i + 1) % self.music.len());
        let next = requested.or_else(|| {
            (0..self.music.len())
                .map(|offset| (start + offset) % self.music.len())
                .find(|index| {
                    playlist
                        .get((self.music[*index].0 - 20128) as usize)
                        .copied()
                        .unwrap_or(false)
                })
        });
        let Some(index) = next else {
            if let Some(previous) = self.current_music.take() {
                stop_sound(&self.music[previous].1);
            }
            return;
        };
        if let Some(previous) = current {
            stop_sound(&self.music[previous].1);
        }
        play_sound(
            &self.music[index].1,
            PlaySoundParams {
                looped: false,
                volume: self.music_volume.get(),
            },
        );
        self.current_music.set(Some(index));
        self.music_until.set(now + self.music[index].2);
    }
    pub fn current_resource(&self) -> Option<i16> {
        self.current_music.get().map(|index| self.music[index].0)
    }
    pub fn play<'a>(&self, events: impl IntoIterator<Item = &'a Event>) {
        let mut at = self
            .pending_sounds
            .borrow()
            .back()
            .map_or(macroquad::time::get_time(), |(at, _)| {
                at.max(macroquad::time::get_time())
            });
        for event in events {
            match event {
                Event::Delay { ticks } => at += f64::from(*ticks) / self.ticks_per_second.get(),
                Event::Sound { resource_id } => self
                    .pending_sounds
                    .borrow_mut()
                    .push_back((at, *resource_id)),
                _ => {}
            }
        }
        self.tick_sounds(macroquad::time::get_time());
    }

    pub fn tick_sounds(&self, now: f64) {
        let mut pending = self.pending_sounds.borrow_mut();
        while pending.front().is_some_and(|(at, _)| *at <= now) {
            let (_, id) = pending.pop_front().unwrap();
            if let Some((sound, _)) = self.sounds.get(&id.wrapping_abs()) {
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

    pub fn set_fast_delays(&self, fast: bool) {
        self.ticks_per_second.set(if fast { 120. } else { 60. });
    }

    pub fn clear_pending(&self) {
        self.pending_sounds.borrow_mut().clear();
    }

    pub fn duration(&self, resource_id: i16) -> f64 {
        self.sounds
            .get(&resource_id.wrapping_abs())
            .map_or(0., |(_, seconds)| *seconds)
    }
}
