use std::ffi::CString;
use std::mem::size_of;
use std::ptr::null_mut;

use log::*;
use winapi::{
    shared::ntdef::{FALSE, NULL, TRUE},
    um::{
        handleapi::CloseHandle,
        synchapi::{CreateEventA, OpenEventA, WaitForSingleObject, ResetEvent, SetEvent},
        winnt::{EVENT_MODIFY_STATE, HANDLE, SYNCHRONIZE},
        winbase::{INFINITE, WAIT_OBJECT_0},
    },
};

use super::{EventImpl, EventInit, EventState};
use crate::{Result, Timeout};

unsafe fn new_event(mem: *mut u8, is_auto: bool) -> Result<Box<dyn EventImpl>> {
    let mut handle: HANDLE = NULL;
    let mut id: u32 = 0;
    while handle == NULL {
        id = rand::random::<u32>();
        let path = CString::new(format!("event_{}", id)).unwrap();
        debug!(
            "CreateEventA(NULL, '{:?}', '{}')",
            is_auto,
            path.to_string_lossy(),
        );
        handle = CreateEventA(
            null_mut(),
            if is_auto { FALSE } else { TRUE } as _,
            FALSE as _,
            path.as_ptr() as *mut _,
        );
    }
   
    let obj: Box<dyn EventImpl> = Box::new(Event { handle });
    *(mem as *mut u32) = id;
    Ok(obj)
}

unsafe fn existing_event(mem: *mut u8) -> Result<Box<dyn EventImpl>> {
    let id: u32 = *(mem as *mut u32);
    let path = CString::new(format!("event_{}", id)).unwrap();
    debug!("OpenEventA('{}')", path.to_string_lossy());
    let handle = OpenEventA(
        EVENT_MODIFY_STATE | SYNCHRONIZE, // request full access
        FALSE as _,                       // handle not inheritable
        path.as_ptr() as *mut _,
    );

    if handle == NULL {
        return Err(From::from(format!(
            "Failed to open event {}",
            path.to_string_lossy()
        )));
    }

    Ok(Box::new(Event { handle }))
}

pub struct Event {
    handle: HANDLE,
}
impl Drop for Event {
    fn drop(&mut self) {
        debug!("CloseHandle(0x{:X})", self.handle as usize);
        unsafe { CloseHandle(self.handle) };
    }
}
impl EventInit for Event {
    fn size_of() -> usize {
        size_of::<u32>()
    }
    unsafe fn new(mem: *mut u8) -> Result<(Box<dyn EventImpl>, usize)> {
        Ok((new_event(mem, true)?, Self::size_of()))
    }
    unsafe fn from_existing(mem: *mut u8) -> Result<(Box<dyn EventImpl>, usize)> {
        Ok((existing_event(mem)?, Self::size_of()))
    }
}
impl EventImpl for Event {
    fn wait(&self, timeout: Timeout) -> Result<()> {
        let wait_res = unsafe { WaitForSingleObject(self.handle, match timeout {
            Timeout::Infinite => INFINITE,
            Timeout::Val(dur) => dur.as_millis() as _,
        })};
        
        if wait_res == WAIT_OBJECT_0 {
            Ok(())
        } else {
            Err(From::from(format!(
                "Failed waiting for event : 0x{:X}",
                wait_res
            )))
        }
    }
    fn set(&self, state: EventState) -> Result<()> {
        let res = unsafe {match state {
            EventState::Reset => {
                ResetEvent(self.handle)
            },
            EventState::Signaled => {
                SetEvent(self.handle)
            },
        }};
        
        if res != 0 {
            Ok(())
        } else {
            Err(From::from(format!(
                "Failed setting event state : 0x{:X}",
                res
            )))
        }
    }
}
