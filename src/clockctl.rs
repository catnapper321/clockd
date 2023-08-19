#![allow(unused)]
use clockd::*;
mod timepart;
use timepart::TimePart;
use std::time::Duration;
use std::io::Write;

fn next_day(dt: tz::DateTime) -> Result<tz::DateTime, tz::error::TzError> {
    let us = dt.unix_time() + 86400;
    let ltt = dt.local_time_type().clone();
    Ok(tz::DateTime::from_timespec_and_local(us, 0, ltt)?)
}

fn find_weekday(starting_dt: tz::DateTime, weekday: u8) -> Result<tz::DateTime, tz::error::TzError> {
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

fn  timeparts_to_unixmoment(mut current_dt: tz::DateTime, tps: &[TimePart]) -> Result<UnixMoment, tz::error::TzError> {
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
    let new_dt = tz::DateTime::new(year, month, day, hour, minute, second, 0, *ltt)?;
    Ok(UnixMoment::new(new_dt.unix_time() + interval_seconds))
}


type Anything<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn main() -> Anything<()> {
    // println!("HELLO WORLD");
    let tz = tz::TimeZone::local()?;
    let dt = tz::DateTime::now(tz.as_ref())?;
    let mut buf = String::new();
    loop {
        buf.clear();
        print!("Enter time spec: ");
        std::io::stdout().flush();
        std::io::stdin().read_line(&mut buf)?;
        let (_, tps) = TimePart::parse_line(&buf).map_err(|_| "input error" )?;
        let um = timeparts_to_unixmoment(dt.clone(), tps.as_slice())?;
        let new_dt = um.as_datetime(tz.as_ref());
        println!("{:?}", new_dt);
    }
    Ok(())
}

