use std::{
    task::Poll,
    pin::Pin,
    time::{Instant, Duration},
    path::PathBuf,
    sync::{Arc, RwLock},
};
use async_io::Timer;
use async_std::stream::Stream;
use futures_lite::{future::FutureExt, StreamExt};
use serde::{Serialize, Deserialize};

mod unixmoment;
pub use unixmoment::*;

impl UnixMoment {
    pub fn timer_for(&self, now: UnixMoment) -> Option<Timer> {
        now.duration_until(*self).map(Timer::after)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AlarmState {
    Waiting,
    Started,
    Done,
}

#[derive(Debug, Clone)]
pub enum AlarmEvent {
    Started(Alarm),
    Now(Alarm),
}

#[derive(Debug, Clone)]
pub struct Alarm {
    pub name: String,
    pub soundfile: Option<PathBuf>,
    pub end_t: UnixMoment,
    pub creation_t: UnixMoment,
    pub state: AlarmState,
}

impl Alarm {
    pub fn new_from_unixmoment(name: impl Into<String>, soundfile: Option<PathBuf>, end_t: UnixMoment) -> Self {
        let creation_t = UnixMoment::now();
        Self {
            name: name.into(),
            soundfile,
            end_t,
            creation_t,
            state: AlarmState::Waiting,
        }
    }
    pub fn new_from_durations(name: impl Into<String>, soundfile: Option<PathBuf>, end_in: Duration) -> Self {
        let creation_t = UnixMoment::now();
        Self {
            name: name.into(),
            soundfile,
            end_t: creation_t + end_in,
            creation_t,
            state: AlarmState::Waiting,
        }
    }
    pub fn is_running(&self) -> bool {
        ! matches!(self.state, AlarmState::Done)
    }
    /// returns an AlarmEvent upon state change
    pub fn update_with_current_time(&mut self, now: UnixMoment) -> Option<AlarmEvent> {
        let s_until = now.seconds_until(self.end_t);
        if s_until <= 0 {
            if ! matches!(self.state, AlarmState::Done) {
                self.state = AlarmState::Done;
                return Some(AlarmEvent::Now(self.clone()));
            }
        } else {
            if ! matches!(self.state, AlarmState::Started) {
                self.state = AlarmState::Started;
                return Some(AlarmEvent::Started(self.clone()));
            }
        }
        None
    }
}

pub struct AlarmList(Vec<Alarm>, Vec<Alarm>);
impl AlarmList {
    pub fn new() -> Self {
        Self(Vec::new(), Vec::new())
    }
    pub fn add(&mut self, a: Alarm) {
        self.0.push(a);
        self.0.sort_by(|a, b| b.end_t.cmp(&a.end_t));
    }
    pub fn remove(&mut self, creation_t: UnixMoment) {
        self.0.retain(|a| a.creation_t != creation_t && a.is_running());
    }
    pub fn acknowledge(&mut self) {
        self.1.clear();
    }
    pub fn lead_alarming(&self) -> Option<&Alarm> {
        self.1.first()
    }
    pub fn next_alarm(&self) -> Option<&Alarm> {
        self.0.last()
    }
    pub fn alarming(&self) -> impl Iterator<Item = &Alarm> {
        self.1.iter()
    }
    pub fn pending(&self) -> impl Iterator<Item = &Alarm> {
        self.0.iter()
    }
    pub fn pending_len(&self) -> usize {
        self.0.len()
    }
    pub fn alarming_len(&self) -> usize {
        self.1.len()
    }
    fn promote_next(&mut self) {
        if let Some(a) = self.0.pop() {
            self.1.push(a);
        }
    }
    pub fn update_with_current_time(&mut self, now: UnixMoment) -> Vec<AlarmEvent> {
        let mut events = Vec::new();
        let mut promote_counter = 0;
        for a in self.0.iter_mut() {
            if let Some(ev) = a.update_with_current_time(now) {
                if matches!(ev, AlarmEvent::Now(_)) { promote_counter += 1; };
                events.push(ev);
            }
        }
        while promote_counter > 0 { self.promote_next(); promote_counter -= 1; }
        events
    }
}
