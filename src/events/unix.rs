use std::mem::size_of;

use log::*;
use ::libc::{
    //Events
    pthread_cond_t,
    pthread_cond_init,
    pthread_cond_wait,
    pthread_condattr_t,
    pthread_cond_signal,
    pthread_condattr_init,
    pthread_cond_broadcast,
    pthread_cond_timedwait,
    pthread_condattr_setpshared,

    PTHREAD_PROCESS_SHARED,
};

use std::sync::atomic::{AtomicU8, Ordering};
use super::{EventImpl, EventInit, EventState};
use crate::{Result, Timeout};

struct InnerEvent {
    ptr: pthread_cond_t,
    signal: AtomicU8,
}

struct ManualEvent {
    inner: *mut InnerEvent,
}

pub struct Event {
    ptr: *mut pthread_cond_t,
}
impl EventInit for Event {
    fn size_of() -> usize {
        size_of::<pthread_cond_t>()
    }

    unsafe fn new(mem: *mut u8, auto_reset: bool) -> Result<(Box<dyn EventImpl>, usize)> {
        Err(From::from("Not implemented yet !".to_string()))
    }

    unsafe fn from_existing(mem: *mut u8) -> Result<(Box<dyn EventImpl>, usize)> {
        Err(From::from("Not implemented yet !".to_string()))
    }
}

impl EventImpl for Event {
    fn wait(&self, timeout: Timeout) -> Result<()> {
        Err(From::from("Not implemented yet !".to_string()))
    }

    fn set(&self, state: EventState) -> Result<()> {
        Err(From::from("Not implemented yet !".to_string()))
    }
}