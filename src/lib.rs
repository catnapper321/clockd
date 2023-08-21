#![allow(unused)]
mod prelude;
use prelude::*;
use serde::{Serialize, Deserialize};
mod util;
use util::*;
use std::path::PathBuf;
mod alarm;
pub use alarm::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppCommand {
    Add(AlarmSpec),
    Remove(UnixMoment),
    Acknowledge,
    SwitchDisplay,
}

/// Struct used by external processes to pass a new Alarm to the daemon
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlarmSpec {
    name: String,
    soundfile: Option<PathBuf>,
    end_t: UnixMoment, 
}

impl AlarmSpec {
    pub fn new(name: String, soundfile: Option<PathBuf>, end_t: UnixMoment) -> Self {
        Self {name, soundfile, end_t}
    }
}

impl TryFrom<AlarmSpec> for Alarm {
    type Error = AlarmSpecError;

    fn try_from(value: AlarmSpec) -> Result<Self, Self::Error> {
        let now = UnixMoment::now();
        let end_in = now.duration_until(value.end_t)
            .ok_or(AlarmSpecError::EndTimeInPast)?;
        if let Some(ref p) = value.soundfile {
            if ! p.exists() { return Err(AlarmSpecError::SoundfileNotExist); }
        }
        Ok(Alarm::new_from_durations(value.name, value.soundfile, end_in))
    }
}

#[derive(Debug)]
pub enum AlarmSpecError {
    EndTimeInPast,
    SoundfileNotExist,
}
impl std::error::Error for AlarmSpecError {}
impl std::fmt::Display for AlarmSpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Ring(Alarm),
    Minute(tz::DateTime),
    Tick,
    AlarmListUpdate,
    Ack,
    SwitchDisplay,
    NewListener,
}
