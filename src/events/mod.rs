
cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        use windows as os;
    } else if #[cfg(any(target_os = "linux"))] {
        mod unix;
        use unix as os;
    } else {
        unimplemented!("This crate does not support your OS yet !");
    }
}
pub use os::*;
use crate::{Result, Timeout};

pub enum EventState {
    /// Clear's the event state so the next wait() call will block
    Clear,
    /// Sets the event to the signaled state unblocking any waiters
    Signaled,
}

pub trait EventInit {
    /// Size required for the event's internal representation
    fn size_of() -> usize;
    /// Initializes a new instance of the event in the provided buffer and returns the number of used bytes
    unsafe fn new(mem: *mut u8, is_auto: bool) -> Result<(Box<dyn EventImpl>, usize)>;
    /// Re-uses an event from an already initialized location and returns the number of used bytes
    unsafe fn from_existing(mem: *mut u8) -> Result<(Box<dyn EventImpl>, usize)>;
}

pub trait EventImpl {
    /// Wait for the event to be signaled
    fn wait(&self, timeout: Timeout) -> Result<()>;
    /// Set the current state of the event
    fn set(&self, state: EventState) -> Result<()>;
}