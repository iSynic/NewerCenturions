use crate::RuleError;

/// Toolbox TickCount/Button replies belong to the presentation adapter.
pub trait TimingServices {
    fn tick_count(&mut self) -> Result<u32, RuleError>;
    fn button(&mut self) -> Result<bool, RuleError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimingEffect {
    TickCount(u32),
    Button(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DelayResult {
    pub effects: Vec<TimingEffect>,
    pub elapsed: u32,
    pub interrupted: bool,
    pub native_result: u32,
}

fn tick(
    services: &mut impl TimingServices,
    effects: &mut Vec<TimingEffect>,
) -> Result<u32, RuleError> {
    let value = services.tick_count()?;
    effects.push(TimingEffect::TickCount(value));
    Ok(value)
}

/// CODE 1 delay, 0x1205f8. The unsigned elapsed comparison is strictly greater.
pub fn delay(
    duration: i16,
    default_duration: i16,
    services: &mut impl TimingServices,
) -> Result<DelayResult, RuleError> {
    let duration = if duration == 0 {
        default_duration
    } else {
        duration
    };
    let limit = i32::from(duration) as u32;
    let mut effects = Vec::new();
    let start = tick(services, &mut effects)?;
    loop {
        let elapsed = tick(services, &mut effects)?.wrapping_sub(start);
        if elapsed > limit {
            return Ok(DelayResult {
                effects,
                elapsed,
                interrupted: false,
                native_result: elapsed,
            });
        }
    }
}

/// CODE 1 delay2, 0x157ea4. Button is queried before each follow-up TickCount.
pub fn delay_interruptible(
    duration: i16,
    default_duration: i16,
    services: &mut impl TimingServices,
) -> Result<DelayResult, RuleError> {
    let duration = if duration == 0 {
        default_duration
    } else {
        duration
    };
    let limit = i32::from(duration) as u32;
    let mut effects = Vec::new();
    let start = tick(services, &mut effects)?;
    let mut elapsed = 0;
    loop {
        let pressed = services.button()?;
        effects.push(TimingEffect::Button(pressed));
        if pressed {
            return Ok(DelayResult {
                effects,
                elapsed,
                interrupted: true,
                native_result: 0,
            });
        }
        elapsed = tick(services, &mut effects)?.wrapping_sub(start);
        if elapsed > limit {
            return Ok(DelayResult {
                effects,
                elapsed,
                interrupted: false,
                native_result: 0,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    struct Replies(VecDeque<TimingEffect>);
    impl TimingServices for Replies {
        fn tick_count(&mut self) -> Result<u32, RuleError> {
            match self.0.pop_front() {
                Some(TimingEffect::TickCount(value)) => Ok(value),
                _ => Err(RuleError::Invalid(
                    "missing expected TickCount reply".into(),
                )),
            }
        }
        fn button(&mut self) -> Result<bool, RuleError> {
            match self.0.pop_front() {
                Some(TimingEffect::Button(value)) => Ok(value),
                _ => Err(RuleError::Invalid("missing expected Button reply".into())),
            }
        }
    }

    #[test]
    fn delay_preserves_default_strict_comparison_and_unsigned_clock_wrap() {
        let replies = vec![
            TimingEffect::TickCount(u32::MAX - 1),
            TimingEffect::TickCount(1),
            TimingEffect::TickCount(2),
        ];
        let result = delay(0, 3, &mut Replies(replies.clone().into())).unwrap();
        assert_eq!(result.effects, replies);
        assert_eq!(result.elapsed, 4);
        assert_eq!(result.native_result, 4);
    }

    #[test]
    fn button_short_circuits_before_a_second_tick_even_for_negative_duration() {
        let replies = vec![TimingEffect::TickCount(100), TimingEffect::Button(true)];
        let result = delay_interruptible(-1, 0, &mut Replies(replies.clone().into())).unwrap();
        assert_eq!(result.effects, replies);
        assert!(result.interrupted);
        assert_eq!(result.native_result, 0);
        let mut exhausted = Replies(
            vec![
                TimingEffect::TickCount(0),
                TimingEffect::TickCount(u32::MAX),
            ]
            .into(),
        );
        assert!(delay(-1, 0, &mut exhausted).is_err());
    }
}
