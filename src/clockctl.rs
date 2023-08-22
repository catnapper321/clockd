#![allow(unused)]
use clockd::*;
mod timepart;
use timepart::TimePart;
use std::process::exit;
use std::time::Duration;
use std::io::Write;
use std::path::{PathBuf, Path};
use std::os::unix::net::UnixStream;
use tz::{DateTime, TimeZone, TimeZoneRef};
mod tz_display;
use tz_display::*;

#[derive(Debug, clap::Parser)]
pub struct Config {
    #[clap(short = 's', long = "socket")]
    cmd_socket: Option<PathBuf>,
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum SubCommand {
    Add { 
        #[clap(short = 'n')]
        name: Option<String>,
        #[clap(short = 'f')]
        soundfile: Option<PathBuf>,
        timeparts: Vec<String>
    },
    Ack,
    List,
    SwitchDisplay,
}

type Anything<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn main() -> Anything<()> {
    let tz = tz::TimeZone::local()?;
    let dt = DateTime::now(tz.as_ref())?;
    let mut c: Config = clap::Parser::parse();
    make_socket_path(&mut c.cmd_socket, "clockd.cmd")?;
    let cmd_socket = c.cmd_socket.expect("Could not determine socket path");
    if ! cmd_socket.exists() {
        println!("Command socket {cmd_socket:?} does not exist");
        exit(1);
    }
    match c.subcommand {
        SubCommand::Add { name, soundfile, timeparts } => {
            ensure_soundfile(&soundfile);
            let tps_raw: String = timeparts.join(" ");
            let (_, tps) = TimePart::parse_line(&tps_raw).map_err(|_| "input error" )?;
            let end_t = timeparts_to_unixmoment(dt.clone(), tps.as_slice())?;
            let name = name.unwrap_or(String::from("Anon"));
            let alarm = AlarmSpec::new(name, soundfile, end_t);
            let cmd = AppCommand::Add(alarm);
            send_command(cmd_socket, cmd)?;
            // let new_dt = um.as_datetime(tz.as_ref()).unwrap();
            // println!("{}", humanize_datetime_full(new_dt));
            // println!("name is {name:?}");
            // println!("soundfile is {soundfile:?}");
        },
        SubCommand::Ack => {
            send_command(cmd_socket, AppCommand::Acknowledge)?;
        },
        SubCommand::List => {
            todo!()
        },
        SubCommand::SwitchDisplay => {
            send_command(cmd_socket, AppCommand::SwitchDisplay)?;
        }
    }
    Ok(())
}

fn next_day(dt: DateTime) -> Result<DateTime, tz::error::TzError> {
    let us = dt.unix_time() + 86400;
    let ltt = dt.local_time_type().clone();
    Ok(DateTime::from_timespec_and_local(us, 0, ltt)?)
}

fn find_weekday(starting_dt: DateTime, weekday: u8) -> Result<DateTime, tz::error::TzError> {
    let mut counter = 7;
    let mut dt = next_day(starting_dt)?;
    while counter > 0 {
        if dt.week_day() == weekday { break; }
        dt = next_day(dt)?;
        counter -= 1;
    }
    if counter == 0 {
        let e: std::io::Error = std::io::ErrorKind::InvalidInput.into();
        return Err(tz::error::TzError::from(e));
    }
    Ok(dt)
}

fn  timeparts_to_unixmoment(mut current_dt: DateTime, tps: &[TimePart]) -> Result<UnixMoment, tz::error::TzError> {
    let tz = tz::TimeZone::local().unwrap();
    // find starting day
    for tp in tps {
        match tp {
            TimePart::Tomorrow => current_dt = next_day(current_dt)?,
            TimePart::WeekDay(n) => current_dt = find_weekday(current_dt, *n)?,
            _ => continue,
        }
    }
    let mut year = current_dt.year();
    let mut month = current_dt.month();
    let mut day = current_dt.month_day();
    let mut hour = current_dt.hour();
    let mut minute = current_dt.minute();
    let mut second = current_dt.second();
    let mut interval_seconds = 0i64;

    for tp in tps {
        match tp {
            TimePart::HM(h, m) => {
                hour = *h;
                minute = *m;
            },
            TimePart::MD(m, d) => {
                month = *m;
                day = *d;
            },
            TimePart::Month(m) => month = *m,
            TimePart::Year(y) => year = *y,
            TimePart::YMD(y, m, d) => {
                year = *y;
                month = *m;
                day = *d;
            },
            TimePart::Hours(h) => interval_seconds += (h * 3600),
            TimePart::Minutes(m) => interval_seconds += (m * 60),
            TimePart::Seconds(s) => interval_seconds += s,
            _ => continue,
        }
    }
    let ltt = current_dt.local_time_type();
    let new_dt = DateTime::new(year, month, day, hour, minute, second, 0, *ltt)?;
    Ok(UnixMoment::new(new_dt.unix_time() + interval_seconds))
}


fn verify_soundfile(p: impl AsRef<Path>) -> bool {
    p.as_ref().exists() && p.as_ref().is_file()
}

fn ensure_soundfile(p: &Option<impl AsRef<Path>>) {
    // let sf = soundfile.as_ref().map(verify_soundfile);
    if let Some(path) = p.as_ref() {
        if verify_soundfile(path) { return }
    } else {
        return
    }
    println!("Sound file is not valid");
    exit(1);
}

fn send_command(path: impl AsRef<Path>, cmd: AppCommand) -> Anything<()> {
    let socket = UnixStream::connect(path)?;
    serde_json::to_writer(socket, &cmd)?;
    Ok(())
}

fn make_socket_path(config_path: &mut Option<PathBuf>, default_name: &str) -> Anything<()> {
    if config_path.is_some() {
        return Ok(());
    }
    // try to construct a path from $XDG_RUNTIME_DIR
    if let Ok(d) = std::env::var("XDG_RUNTIME_DIR") {
        let mut socket_path = PathBuf::new();
        socket_path.push(d);
        socket_path.push(default_name);
        *config_path = Some(socket_path);
        Ok(())
    } else {
        let msg = format!("socket path must be specified for {default_name}");
        Err(msg.into())
    }
}

