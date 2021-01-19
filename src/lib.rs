#[cfg(feature = "verbose")]
pub(crate) use ::log;
#[macro_use]
macro_rules! debug {
    ($($x:tt)*) => {
        {
            #[cfg(feature = "verbose")]
            crate::log::debug!($($x)*)
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, crate::Error>;
/// Event implementations
pub mod events;
/// Lock implementations
pub mod locks;

pub enum Timeout {
    Infinite,
    Val(std::time::Duration),
}

#[derive(Debug)]
pub enum Error {
    LockFailed(std::io::Error),
    ReleaseFailed(std::io::Error),
    InitFailed(std::io::Error),
    EventSetFailed(std::io::Error),
    WaitFailed(std::io::Error),
    SpuriousWake,
    TimedOut,
    ObjectCorrupted,
}

impl std::fmt::Display for crate::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LockFailed(e) => {
                f.write_str(&format!("Failed to acquire lock with OS error : {}", e))
            }
            Self::ReleaseFailed(e) => {
                f.write_str(&format!("Failed to release lock with OS error : {}", e))
            }
            Self::InitFailed(e) => f.write_str(&format!(
                "Failed to initialize object with OS error : {}",
                e
            )),
            Self::EventSetFailed(e) => f.write_str(&format!(
                "Failed to set state of event event with OS error : {}",
                e
            )),
            Self::WaitFailed(e) => f.write_str(&format!("Wait failed with OS error : {}", e)),
            Self::SpuriousWake => {
                f.write_str("A wait call has returned because of external spurious wake up")
            }
            Self::TimedOut => f.write_str("Wait operation has timed out"),
            Self::ObjectCorrupted => f.write_str("Object is corrupted"),
        }
    }
}

impl std::error::Error for crate::Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LockFailed(e) => Some(e),
            Self::ReleaseFailed(e) => Some(e),
            Self::InitFailed(e) => Some(e),
            Self::EventSetFailed(e) => Some(e),
            Self::WaitFailed(e) => Some(e),
            _ => None,
        }
    }
}
