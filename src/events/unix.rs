use std::mem::size_of;
use std::ptr::null_mut;

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

use crate::{Result, Timeout};
use crate::locks::*;
use crate::events::*;


struct InnerEvent {
    cond: pthread_cond_t,
    auto_reset: u8,
    signal: u8,
}
pub struct Event {
    mutex: Box<dyn LockImpl>,
    inner: *mut InnerEvent,
}
impl EventInit for Event {
    fn size_of() -> usize {
        Mutex::size_of() + size_of::<InnerEvent>()
    }

    unsafe fn new(mem: *mut u8, auto_reset: bool) -> Result<(Box<dyn EventImpl>, usize)> {
        let (mutex, used_bytes) = Mutex::new(mem, null_mut())?;
        let ptr = mem.add(used_bytes + size_of::<*mut u8>() as usize) as *mut InnerEvent;
        let inner = &mut *ptr;

        let mut attrs: pthread_condattr_t = std::mem::zeroed();
        if pthread_condattr_init(&mut attrs) != 0 {
            return Err(From::from(
                "Failed to initialize pthread_condattr_init".to_string(),
            ));
        }
        if pthread_condattr_setpshared(&mut attrs, PTHREAD_PROCESS_SHARED) != 0 {
            return Err(From::from(
                "Failed to set pthread_condattr_setpshared(PTHREAD_PROCESS_SHARED)".to_string(),
            ));
        }

        trace!("pthread_cond_init({:p})", ptr);
        if pthread_cond_init(&mut inner.cond, &attrs) != 0 {
            return Err(From::from(
                "Failed to initialize pthread_cond_init".to_string(),
            ));
        }
        inner.auto_reset = if auto_reset {
            1
        } else {
            0
        };
        inner.signal = 0;

        let obj = Box::new(Self {
            mutex,
            inner,
        });

        Ok((obj, ptr as usize - mem as usize))
    }

    unsafe fn from_existing(mem: *mut u8) -> Result<(Box<dyn EventImpl>, usize)> {
        let (mutex, used_bytes) = Mutex::from_existing(mem, null_mut())?;
        let ptr = mem.add(used_bytes + size_of::<*mut u8>() as usize) as *mut InnerEvent;
        let inner = &mut *ptr;

        if inner.auto_reset > 1 || inner.signal > 1 {
            return Err(From::from("Existing Event is corrupted"));
        }

        let obj = Box::new(Self {
            mutex,
            inner,
        });

        Ok((obj, ptr as usize - mem as usize))
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