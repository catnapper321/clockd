mod futurestream;
pub use futurestream::*;
use crate::*;

pub fn unix_seconds_now() -> Duration {
    let sys_t = std::time::SystemTime::now();
    sys_t.duration_since(std::time::UNIX_EPOCH).unwrap()
}

pub fn humanize_seconds(mut s: i64) -> String {
    let ago = if s < 0 {
        s *= -1;
        " ago"
    } else {
        ""
    };
    let d = s / 86400;
    s -= d * 86400;
    let h = s / 3600;
    s -= h * 3600;
    if d > 0 {
        return format!("{d}d {h}h{ago}");
    }
    let m = s / 60;
    s -= m * 60;
    if h > 0 {
        return format!("{h}h {m}m{ago}");
    }
    if m > 0 {
        return format!("{m}m {s}s{ago}");
    }
    format!("{s}s{ago}")
}
