use serde::{Deserialize, Serialize, __private::de::AdjacentlyTaggedEnumVariantVisitor};
use tz::TimeZoneRef;
use std::time::Duration;

const NANOSECONDS_PER_SECOND: i64 = 1000000000;

#[derive(Deserialize, Serialize, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct UnixMoment(i64, i64); // seconds and nanos from UNIX_EPOCH

fn deconstruct_duration(d: Duration) -> (i64, i64) {
    let s = d.as_secs() as i64;
    let n = d.subsec_nanos() as i64;
    (s, n) 
}

impl UnixMoment {
    pub fn new(s: impl Into<i64>) -> Self {
        Self(s.into(), 0)
    }
    pub fn now() -> Self {
        let sys_t = std::time::SystemTime::now();
        let d = sys_t
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Could not get current SystemTime");
        Self::from(d)
    }
    pub fn seconds(&self) -> i64 {
        self.0
    }
    fn next_unit(&self, unit_seconds: i64) -> Self {
        let s = self.0 + unit_seconds - self.0 % unit_seconds;
        Self(s, 0)
    }
    pub fn next_minute(&self) -> Self {
        self.next_unit(60)
    }
    pub fn next_hour(&self) -> Self {
        self.next_unit(3600)
    }
    pub fn next_day(&self) -> Self {
        self.next_unit(86400)
    }
    pub fn duration_until_next_minute(&self) -> Duration {
        self.duration_until(self.next_minute())
            .expect("Somehow could not make a duration for the next minute")
    }
    pub fn duration_until_next_hour(&self) -> Duration {
        self.duration_until(self.next_hour())
            .expect("Somehow could not make a duration for the next hour")
    }
    pub fn duration_until_next_day(&self) -> Duration {
        self.duration_until(self.next_day())
            .expect("Somehow could not make a duration for the next day")
    }
    fn adjust_nanos(mut s: i64, mut n: i64) -> (i64, i64) {
        while n > NANOSECONDS_PER_SECOND {
            n-= NANOSECONDS_PER_SECOND;
            s += 1;
        }
        while n < 0 {
            n += NANOSECONDS_PER_SECOND;
            s -= 1;
        }
        (s, n)
    }
    /// Returns seconds between UnixMoments.
    /// Negative seconds indicate that the given UnixMoment is before
    /// (less than) this one.
    pub fn seconds_until(&self, rhs: Self) -> i64 {
        let mut s = rhs.0 - self.0;
        let mut n = rhs.1 - self.1;
        (s, _) = Self::adjust_nanos(s, n);
        s
    }
    /// Returns None if other is less than this UnixMoment
    pub fn duration_until(&self, other: Self) -> Option<Duration> {
        let mut s = other.0 - self.0;
        let mut n = other.1 - self.1;
        (s, n) = Self::adjust_nanos(s, n);
        if s < 0 {
            None
        } else {
            Some(Duration::new(s as u64, n as u32))
        }
    }
    pub fn as_datetime(&self, tzref: TimeZoneRef) -> Option<tz::DateTime> {
        tz::DateTime::from_timespec(self.0, 0, tzref).ok()
    }
}

impl std::fmt::Display for UnixMoment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}s {}ns", self.0, self.1)
    }
}

impl From<Duration> for UnixMoment {
    /// Duration is from UNIX_EPOCH.
    fn from(value: Duration) -> Self {
        let (s, n) = deconstruct_duration(value);
        Self(s, n) 
    }
}

impl From<i64> for UnixMoment {
    /// i64 is unix seconds
    fn from(value: i64) -> Self {
        Self(value, 0)
    }
}

impl std::ops::Add<Duration> for UnixMoment {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let (mut s, mut n) = deconstruct_duration(rhs);
        n = self.1 + n;
        s = self.0 + s;
        (s, n) = Self::adjust_nanos(s, n);
        Self(s, n)
    }
}

impl std::ops::Sub<Duration> for UnixMoment {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        let (mut s, mut n) = deconstruct_duration(rhs);
        n = self.1 - n;
        s = self.0 - s;
        (s, n) = Self::adjust_nanos(s, n);
        Self(s, n)
    }
}

impl std::ops::Sub<UnixMoment> for UnixMoment {
    type Output = Duration;

    fn sub(self, rhs: UnixMoment) -> Self::Output {
        let mut s = self.0 - rhs.0;
        let mut n = self.1 - rhs.1;
        (s, n) = Self::adjust_nanos(s, n);
        if s < 0 || n < 0 { panic!("Negative duration not allowed"); }
        Duration::new(s as u64, n as u32)
    }
}
