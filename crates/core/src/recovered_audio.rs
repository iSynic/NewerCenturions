//! CODE 1 sound control with observed Sound Manager and resource replies.

use crate::RuleError;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AudioContext {
    pub sound_volume: i32,
    pub last_channel: i16,
    pub queue_index: i16,
    pub channels: Vec<u32>,
    pub resource_handles: Vec<u32>,
    pub last_error: i16,
    pub cycle_selector: i8,
    pub cycle_gate_origin: i16,
    pub cycle_gates: Vec<i16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundCommand {
    pub code: u16,
    pub param1: i16,
    pub param2: u32,
}

impl SoundCommand {
    pub fn bytes(self) -> [u8; 8] {
        let mut result = [0; 8];
        result[..2].copy_from_slice(&self.code.to_be_bytes());
        result[2..4].copy_from_slice(&self.param1.to_be_bytes());
        result[4..].copy_from_slice(&self.param2.to_be_bytes());
        result
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioRequest {
    ChannelStatus {
        channel: u32,
        bytes: u16,
    },
    UnlockHandle(u32),
    GetResource {
        resource_type: [u8; 4],
        id: i16,
    },
    LockHandle(u32),
    Immediate {
        channel: u32,
        command: SoundCommand,
    },
    Play {
        channel: u32,
        handle: u32,
        asynchronous: bool,
    },
    Delay(i16),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioReply {
    ChannelStatus {
        bytes: [u8; 24],
        error: i16,
    },
    Handle(u32),
    Error(i16),
    /// The native routine ignores this service's result.
    Ignored,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioObservation {
    pub request: AudioRequest,
    pub reply: AudioReply,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioEffect {
    Service(AudioObservation),
    QueueIndex(i16),
    ResourceHandle { index: i16, handle: u32 },
    LastError(i16),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioPlan {
    pub effects: Vec<AudioEffect>,
    pub consumed_observations: usize,
}

struct Execution<'a> {
    observations: &'a [AudioObservation],
    consumed: usize,
    effects: Vec<AudioEffect>,
}

impl Execution<'_> {
    fn request(&mut self, request: AudioRequest) -> Result<AudioReply, RuleError> {
        let observation = self
            .observations
            .get(self.consumed)
            .ok_or(RuleError::Unsupported {
                action: "audio.service",
                dependency: "the next Sound Manager/resource/timing result has not been supplied",
            })?;
        if observation.request != request {
            return Err(RuleError::Invalid(format!(
                "audio observation {} does not match request {request:?}",
                self.consumed
            )));
        }
        self.consumed += 1;
        self.effects.push(AudioEffect::Service(observation.clone()));
        Ok(observation.reply.clone())
    }

    fn ignored(&mut self, request: AudioRequest) -> Result<(), RuleError> {
        match self.request(request)? {
            AudioReply::Ignored | AudioReply::Error(_) => Ok(()),
            _ => Err(RuleError::Invalid("unexpected audio service result".into())),
        }
    }

    fn status(&mut self, channel: u32) -> Result<([u8; 24], i16), RuleError> {
        match self.request(AudioRequest::ChannelStatus { channel, bytes: 24 })? {
            AudioReply::ChannelStatus { bytes, error } => Ok((bytes, error)),
            _ => Err(RuleError::Invalid(
                "missing 24-byte channel status result".into(),
            )),
        }
    }

    fn resource(&mut self, id: i16) -> Result<u32, RuleError> {
        match self.request(AudioRequest::GetResource {
            resource_type: *b"snd ",
            id,
        })? {
            AudioReply::Handle(handle) => Ok(handle),
            _ => Err(RuleError::Invalid(
                "missing sound resource handle result".into(),
            )),
        }
    }

    fn immediate(&mut self, channel: u32, code: u16, param2: u32) -> Result<i16, RuleError> {
        match self.request(AudioRequest::Immediate {
            channel,
            command: SoundCommand {
                code,
                param1: 0,
                param2,
            },
        })? {
            AudioReply::Error(error) => Ok(error),
            _ => Err(RuleError::Invalid(
                "missing immediate sound OSErr result".into(),
            )),
        }
    }

    fn error(&mut self, context: &mut AudioContext, value: i16) {
        context.last_error = value;
        self.effects.push(AudioEffect::LastError(value));
    }

    fn play(&mut self, channel: u32, handle: u32, asynchronous: bool) -> Result<(), RuleError> {
        self.ignored(AudioRequest::Play {
            channel,
            handle,
            asynchronous,
        })
    }
}

fn execute(
    context: &mut AudioContext,
    observations: &[AudioObservation],
    body: impl FnOnce(&mut AudioContext, &mut Execution<'_>) -> Result<(), RuleError>,
) -> Result<AudioPlan, RuleError> {
    let mut candidate = context.clone();
    let mut execution = Execution {
        observations,
        consumed: 0,
        effects: vec![],
    };
    body(&mut candidate, &mut execution)?;
    *context = candidate;
    Ok(AudioPlan {
        effects: execution.effects,
        consumed_observations: execution.consumed,
    })
}

fn indexed(values: &[u32], index: i16) -> Result<u32, RuleError> {
    usize::try_from(index)
        .ok()
        .and_then(|index| values.get(index))
        .copied()
        .ok_or_else(|| RuleError::Invalid(format!("sound slot {index} exceeds supplied globals")))
}

fn cycle_enabled(context: &AudioContext) -> Result<bool, RuleError> {
    let offset = i32::from(context.cycle_selector) - i32::from(context.cycle_gate_origin);
    let gate = usize::try_from(offset)
        .ok()
        .and_then(|offset| context.cycle_gates.get(offset))
        .ok_or_else(|| {
            RuleError::Invalid("sound cycle selector exceeds supplied WORD table".into())
        })?;
    Ok(*gate == 0)
}

/// sound, 0x133a18. Every busy reply advances to the next channel before retry.
pub fn sound(
    context: &mut AudioContext,
    id: i16,
    observations: &[AudioObservation],
) -> Result<AudioPlan, RuleError> {
    execute(context, observations, |context, execution| {
        if context.sound_volume == 0 || context.last_channel < 0 {
            if id < 0 {
                execution.ignored(AudioRequest::Delay(20))?;
            }
            return Ok(());
        }
        let channel = loop {
            context.queue_index = context.queue_index.wrapping_add(1);
            execution
                .effects
                .push(AudioEffect::QueueIndex(context.queue_index));
            if context.queue_index > context.last_channel {
                context.queue_index = 2;
                execution.effects.push(AudioEffect::QueueIndex(2));
            }
            let channel = indexed(&context.channels, context.queue_index)?;
            let (status, error) = execution.status(channel)?;
            execution.error(context, error);
            if status[12] != 1 {
                break channel;
            }
        };
        let old_handle = indexed(&context.resource_handles, context.queue_index)?;
        execution.ignored(AudioRequest::UnlockHandle(old_handle))?;
        let handle = execution.resource(id.wrapping_abs())?;
        context.resource_handles[context.queue_index as usize] = handle;
        execution.effects.push(AudioEffect::ResourceHandle {
            index: context.queue_index,
            handle,
        });
        execution.ignored(AudioRequest::LockHandle(handle))?;
        for code in [4, 3] {
            let error = execution.immediate(channel, code, 0)?;
            execution.error(context, error);
        }
        if handle != 0 {
            execution.play(channel, handle, id > 0)?;
        }
        Ok(())
    })
}

/// startsoundcycle, 0x133bb8. `incoming_a2` is the caller's preserved register.
pub fn start_sound_cycle(
    context: &mut AudioContext,
    incoming_a2: u32,
    observations: &[AudioObservation],
) -> Result<AudioPlan, RuleError> {
    execute(context, observations, |context, execution| {
        if !cycle_enabled(context)? {
            return Ok(());
        }
        let channel = indexed(&context.channels, 1)?;
        let (status, error) = execution.status(channel)?;
        execution.error(context, error);
        execution.ignored(AudioRequest::UnlockHandle(incoming_a2))?;
        let handle = execution.resource(10000)?;
        execution.ignored(AudioRequest::LockHandle(handle))?;
        if handle != 0 {
            if status[12] != 1 {
                execution.play(channel, handle, true)?;
            }
            sustain_body(context, handle, execution)?;
        }
        Ok(())
    })
}

/// sustainsoundcycle, 0x133c56. Both playback decisions use the same busy byte.
pub fn sustain_sound_cycle(
    context: &mut AudioContext,
    incoming_a2: u32,
    observations: &[AudioObservation],
) -> Result<AudioPlan, RuleError> {
    execute(context, observations, |context, execution| {
        sustain_body(context, incoming_a2, execution)
    })
}

fn sustain_body(
    context: &mut AudioContext,
    incoming_a2: u32,
    execution: &mut Execution<'_>,
) -> Result<(), RuleError> {
    if !cycle_enabled(context)? {
        return Ok(());
    }
    let channel = indexed(&context.channels, 1)?;
    let (status, error) = execution.status(channel)?;
    execution.error(context, error);
    execution.ignored(AudioRequest::UnlockHandle(incoming_a2))?;
    let first_handle = execution.resource(10001)?;
    execution.ignored(AudioRequest::LockHandle(first_handle))?;
    let error = execution.immediate(channel, 4, 0)?;
    execution.error(context, error);
    if first_handle != 0 && status[12] == 1 {
        execution.play(channel, first_handle, true)?;
    }
    execution.ignored(AudioRequest::UnlockHandle(first_handle))?;
    let second_handle = execution.resource(10002)?;
    execution.ignored(AudioRequest::LockHandle(second_handle))?;
    if second_handle != 0 && status[12] == 1 {
        execution.play(channel, second_handle, true)?;
    }
    Ok(())
}

/// quiet, 0x13b56c. Bulk mode excludes the last channel and keeps error scratch.
pub fn quiet(
    context: &mut AudioContext,
    selector: i16,
    observations: &[AudioObservation],
) -> Result<AudioPlan, RuleError> {
    execute(context, observations, |context, execution| {
        if context.last_channel < 0 {
            return Ok(());
        }
        if selector > 0 {
            let channel = indexed(&context.channels, selector)?;
            execution.immediate(channel, 3, 0)?;
            execution.immediate(channel, 4, 0)?;
        } else {
            for index in 1..context.last_channel {
                let channel = indexed(&context.channels, index)?;
                execution.immediate(channel, 3, 0)?;
                execution.immediate(channel, 4, 0)?;
            }
        }
        Ok(())
    })
}

fn packed_volume(value: i32) -> u32 {
    let level = match value {
        0 => 0,
        1 => 4,
        2 => 26,
        3 => 52,
        4 => 82,
        5 => 113,
        6 => 141,
        7 => 169,
        8 => 197,
        9 => 228,
        10 => 256,
        _ => 113,
    };
    level << 16 | level
}

/// setspearkervolume, 0x153584. Music slot zero precedes sound slots 1..19.
pub fn set_speaker_volume(
    context: &mut AudioContext,
    sound_volume: i32,
    music_volume: i32,
    observations: &[AudioObservation],
) -> Result<AudioPlan, RuleError> {
    execute(context, observations, |context, execution| {
        if music_volume >= 0 {
            let channel = indexed(&context.channels, 0)?;
            let error = execution.immediate(channel, 46, packed_volume(music_volume))?;
            execution.error(context, error);
        }
        if sound_volume >= 0 {
            for index in 1..20 {
                let channel = indexed(&context.channels, index)?;
                let error = execution.immediate(channel, 46, packed_volume(sound_volume))?;
                execution.error(context, error);
            }
        }
        Ok(())
    })
}
