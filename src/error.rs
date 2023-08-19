#[derive(Debug)]
pub enum TimeError {
    SystemTimeError,
    TzError,
}

impl std::error::Error for TimeError {}
impl std::fmt::Display for TimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeError::SystemTimeError => write!(f, "SystemTimeError"),
            TimeError::TzError => write!(f, "TzError"),
        }
    }
}

impl From<tz::error::TzError> for TimeError {
    fn from(_: tz::error::TzError) -> Self {
        Self::TzError
    }
}

impl From<std::time::SystemTimeError> for TimeError {
    fn from(_: std::time::SystemTimeError) -> Self {
        Self::SystemTimeError
    }
}
