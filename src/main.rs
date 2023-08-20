#![allow(unused)]
#![feature(unix_socket_ancillary_data)]
use std::fs::File;
use std::sync::{Arc, RwLock};

use clockd::*;

mod tz_display;
use tz_display::*;
use tz::{DateTime, TimeZone, TimeZoneRef, UtcDateTime};

mod audio;
mod fdrecv;
mod error;
use error::*;
mod cli_config;
use cli_config::*;
mod waybar;
mod util;
use util::*;
mod commands;
mod parker;
mod webapp;
mod prelude;
use prelude::*;

use crate::commands::start_command_socket;


type Anything<T> = Result<T, Box<dyn std::error::Error>>;

pub const ONE_HOUR: Duration = Duration::from_secs(3600);
pub const ONE_SECOND: Duration = Duration::from_secs(1);

fn duration_to_next_minute() -> Result<Duration, TimeError> {
    let now = unix_seconds_now();
    // HACK: +1ms to avoid timers from firing a few nanos early
    let r = 60001 - now.as_millis() % 60000;
    Ok(Duration::from_millis(r as u64))
}

#[derive(Debug, Clone)]
pub struct DisplayUpdate {
    timezone: TimeZone,
    dt: DateTime,
    next_alarm: Option<Alarm>,
}
impl DisplayUpdate {
    pub fn new(now: UnixMoment, timezone: TimeZone, next_alarm: Option<Alarm>) -> Self {
        let dt = now.as_datetime(timezone.as_ref()).unwrap();
        Self {
            dt,
            timezone,
            next_alarm,
        }
    }
}

#[derive(Clone)]
enum MainLoopEvent {
    Minute,
    Tick,
    AlarmTimer,
    Command(AppCommand),
    NewListener,
}

fn d(s: u64) -> Duration { Duration::from_secs(s) }

async fn test_driver(cmd_tx: Sender<AppCommand>) {
    info!("test driver started");
    let now = UnixMoment::now();

    let a = AlarmSpec::new(String::from("test1"), None, now + d(5));
    cmd_tx.send(AppCommand::Add(a)).await;

    let a = AlarmSpec::new(String::from("test2"), Some("/home/jeff/.local/alarms/default".into()), now + d(10));
    cmd_tx.send(AppCommand::Add(a)).await;

    // sleep(d(8)).await;
    // trace!("send ack");
    // cmd_tx.send(AppCommand::Acknowledge).await;

    let a = AlarmSpec::new(String::from("test3"), None, now + d(3620));
    cmd_tx.send(AppCommand::Add(a)).await;

    sleep(d(15)).await;
    trace!("send ack");
    cmd_tx.send(AppCommand::Acknowledge).await;

    // sleep(d(10)).await;
    // trace!("send toggle");
    // cmd_tx.send(AppCommand::ToggleDisplay).await;

    info!("test driver ended");
}


async fn main_loop(
    c: &mut Config, 
    // cmd_tx: Sender<AppCommand>,
    mut cmd_rx: Receiver<AppCommand>,
    mut alarm_list: Arc<RwLock<AlarmList>>,
    mut event_tx: broadcast::Sender<AppEvent>,
    ) -> Anything<()> {
    let local_tz = TimeZone::local().unwrap();
    // fd passing synchronous task
    let (fd_parker, fd_nudger) = parker::Parker::new();
    let fd_socket = c.fd_socket.take().unwrap();
    async_std::task::spawn_blocking(|| fdrecv::start_fd_socket(fd_socket, fd_nudger));

    // let mut waybar_display = waybar::WaybarDisplay::new(local_tz);

    let mut cmd_stream = cmd_rx.map(MainLoopEvent::Command);
    let mut fd_stream = fd_parker.map(|_| MainLoopEvent::NewListener);
    let mut alarm_timer = Timer::never();
    let mut now = UnixMoment::now();
    loop {
        let mut next_minute = Timer::after(duration_to_next_minute()?).map(|_| MainLoopEvent::Minute);
        
        // set timer for next alarm
        let mut x = alarm_list.write().unwrap();
        let alarm_events = x.update_with_current_time(now);
        let next_alarm = x.next_alarm();
        let new_duration = next_alarm
            .map(|a| now.duration_until(a.end_t))
            .flatten()
            .unwrap_or(Duration::MAX);
        // set up tick timer
        let tick_stream = if new_duration <= ONE_HOUR || x.lead_alarming().is_some() {
            Timer::interval(Duration::from_secs(1))
        } else {
            Timer::never()
        }.map(|_| MainLoopEvent::Tick);
        drop(x);

        // process alarm events
        for ae in alarm_events.into_iter() {
            match ae {
                AlarmEvent::Started(a) => {
                },
                AlarmEvent::Now(a) => {
                    event_tx.broadcast(AppEvent::Ring(a)).await;
                },
            }
        }

        alarm_timer.set_after(new_duration);

        let alarm_stream = (&mut alarm_timer).map(|_| MainLoopEvent::AlarmTimer);

        let mut s = next_minute
            .merge(&mut cmd_stream)
            .merge(&mut fd_stream)
            .merge(tick_stream)
            .merge(alarm_stream);

        // poll the event stream and tick in a loop
        let ev = loop {
            let ev = s.next().await.unwrap();
            now = UnixMoment::now();
            if ! matches!(ev, MainLoopEvent::Tick) { break ev; }
            event_tx.broadcast(AppEvent::Tick).await;
        };

        match ev {
            MainLoopEvent::Minute => {
                let dt = now.as_datetime(local_tz.as_ref()).unwrap();
                event_tx.broadcast(AppEvent::Minute(dt)).await;
            }
            MainLoopEvent::AlarmTimer => {
                // NOTE: this event should already have been broadcast
            },
            MainLoopEvent::Command(cmd) => {
                match cmd {
                    AppCommand::Add(spec) => {
                        let alarm = Alarm::try_from(spec);
                        if let Ok(a) = alarm {
                            let mut x = alarm_list.write().unwrap();
                            x.add(a);
                            drop(x);
                            event_tx.broadcast(AppEvent::AlarmListUpdate).await;
                        } else {
                            error!("add: bad alarm spec");
                        }
                    },
                    AppCommand::Acknowledge => {
                        let mut x = alarm_list.write().unwrap();
                        x.acknowledge();
                        drop(x);
                        event_tx.broadcast(AppEvent::Ack).await;
                        
                    }
                    AppCommand::Remove(creation_t) => {
                        let mut x = alarm_list.write().unwrap();
                        x.remove(creation_t);
                        drop(x);
                        event_tx.broadcast(AppEvent::AlarmListUpdate).await;
                    }
                    AppCommand::SwitchDisplay => {
                        event_tx.broadcast(AppEvent::SwitchDisplay).await;
                    }
                    _ => {},
                }
            },
            MainLoopEvent::NewListener => {
                event_tx.broadcast(AppEvent::NewListener).await;
            }
            _ => continue,
        }
    }
    unreachable!()
}

#[async_std::main]
async fn main() -> Anything<()> {
    let mut c: Config = get_config()?;
    setup(&c);
    let cmd_socket = c.cmd_socket.take().unwrap();
    let (cmd_tx, cmd_rx) = channel::unbounded::<AppCommand>();
    // TODO: adjust channel capaacity
    let (event_tx, mut event_rx) = broadcast::broadcast::<AppEvent>(2);

    // TEST:
    // spawn(test_driver(cmd_tx.clone())); 

    spawn(commands::start_command_socket(cmd_socket, cmd_tx.clone()));
    let alarm_list = Arc::new(RwLock::new(AlarmList::new()));
    let tz = TimeZone::local().expect("Could not get local time zone");
    // let webstate = webapp::WebState::new(cmd_tx, alarm_list.alarms(), tz);
    // spawn(webapp::server(webstate, c.port));
    spawn(audio::start_audio_task(event_rx.clone()));
    spawn(waybar::waybar_display_server(alarm_list.clone(), event_rx.clone(), tz::TimeZone::local().unwrap())); 

    // manaully keep the event rx drained
    spawn(async move {
        while let Ok(ev) = event_rx.recv().await {
            trace!("broadcast received: {ev:?}");
        }
        unreachable!()
    });

    main_loop(&mut c, cmd_rx, alarm_list, event_tx).await;
    Ok(())
}

fn setup(config: &Config) {
    let loglevel = match config.verbosity {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    let loglevel = tracing::Level::TRACE;
    // let loglevel = tracing::Level::ERROR;
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(loglevel)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to start tracing");
}
