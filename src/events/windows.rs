use std::ffi::CString;
use std::mem::size_of;
use std::ptr::null_mut;

use winapi::{
    shared::ntdef::{FALSE, NULL, TRUE},
    shared::winerror::WAIT_TIMEOUT,
    um::{
        errhandlingapi::GetLastError,
        handleapi::CloseHandle,
        synchapi::{CreateEventA, OpenEventA, ResetEvent, SetEvent, WaitForSingleObject},
        winbase::{INFINITE, WAIT_OBJECT_0, WAIT_ABANDONED},
        winnt::{EVENT_MODIFY_STATE, HANDLE, SYNCHRONIZE},
    },
};

use super::{EventImpl, EventInit, EventState};
use crate::{Result, Timeout};

pub struct Event {
    handle: HANDLE,
}
impl Drop for Event {
    fn drop(&mut self) {
        debug!("CloseHandle({:p})", self.handle);
        unsafe { CloseHandle(self.handle) };
    }
}
impl EventInit for Event {
    fn size_of(_addr: Option<*mut u8>) -> usize {
        size_of::<u32>()
    }

    #[allow(clippy::new_ret_no_self)]
    unsafe fn new(mem: *mut u8, auto_reset: bool) -> Result<(Box<dyn EventImpl>, usize)> {
        let mut handle: HANDLE = NULL;
        let mut id: u32 = 0;
        while handle == NULL {
            id = rand::random::<u32>();
            let path = CString::new(format!("event_{}", id)).unwrap();

            debug!("CreateEventA(NULL, '{:?}', '{}')",!auto_reset, path.to_string_lossy());
            handle = CreateEventA(
                null_mut(),
                if auto_reset { FALSE } else { TRUE } as _,
                FALSE as _,
                path.as_ptr() as *mut _,
            );
        }
        
        debug!("\tres = {:p}", handle);
        let obj: Box<dyn EventImpl> = Box::new(Event { handle });
        *(mem as *mut u32) = id;
        Ok((obj, Self::size_of(None)))
    }

    unsafe fn from_existing(mem: *mut u8) -> Result<(Box<dyn EventImpl>, usize)> {
        let id: u32 = *(mem as *mut u32);
        let path = CString::new(format!("event_{}", id)).unwrap();
        debug!("Event from '{}'", path.to_string_lossy());
        let handle = OpenEventA(
            EVENT_MODIFY_STATE | SYNCHRONIZE, // request full access
            FALSE as _,                       // handle not inheritable
            path.as_ptr() as *mut _,
        );
        if handle == NULL {
            let err = GetLastError() as _;
            return Err(crate::Error::InitFailed(std::io::Error::from_raw_os_error(err)));
        }
        debug!("\tres = {:p}", handle);

        Ok((Box::new(Event { handle }), Self::size_of(None)))
    }
}
impl EventImpl for Event {
    fn wait(&self, timeout: Timeout) -> Result<()> {
        debug!("WaitForSingleObject({:p})", self.handle);
        let wait_res = unsafe {
            WaitForSingleObject(
                self.handle,
                match timeout {
                    Timeout::Infinite => INFINITE,
                    Timeout::Val(dur) => dur.as_millis() as _,
                },
            )
        };
        
        if wait_res == WAIT_OBJECT_0 {
            Ok(())
        } else if wait_res == WAIT_TIMEOUT {
            Err(crate::Error::TimedOut)
        } else if wait_res == WAIT_ABANDONED {
            Err(crate::Error::ObjectCorrupted)
        } else {
            let err = unsafe{GetLastError()} as _;
            Err(crate::Error::WaitFailed(std::io::Error::from_raw_os_error(err)))
        }
    }

    fn set(&self, state: EventState) -> Result<()> {
        let res = match state {
            EventState::Clear => {
                debug!("ResetEvent({:p})", self.handle);
                unsafe { ResetEvent(self.handle) }
            }
            EventState::Signaled => {
                debug!("SetEvent({:p})", self.handle);
                unsafe { SetEvent(self.handle) }
            }
        };

        if res != 0 {
            Ok(())
        } else {
            let err = unsafe{GetLastError()} as _;
            Err(crate::Error::EventSetFailed(std::io::Error::from_raw_os_error(err)))
        }
    }
}
