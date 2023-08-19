use tz::{timezone, DateTime};

pub fn dow_to_str(dow: u8) -> &'static str {
    match dow {
        0 => "Sun",
        1 => "Mon",
        2 => "Tue",
        3 => "Wed",
        4 => "Thu",
        5 => "Fri",
        6 => "Sat",
        _ => "???",
    }
}

pub fn mon_to_str(mon: u8) -> &'static str {
    match mon {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

pub fn humanize_datetime(dt: DateTime) -> String {
    format!(
        "{:3} {:02} {:3} {:02}:{:02}",
        dow_to_str(dt.week_day()),
        dt.month_day(),
        mon_to_str(dt.month()),
        dt.hour(),
        dt.minute(),
    )
}

pub fn humanize_datetime_full(dt: DateTime) -> String {
    format!(
        "{:3} {:02} {:3} {:02}:{:02} {}",
        dow_to_str(dt.week_day()),
        dt.month_day(),
        mon_to_str(dt.month()),
        dt.hour(),
        dt.minute(),
        dt.local_time_type().time_zone_designation()
    )
}
