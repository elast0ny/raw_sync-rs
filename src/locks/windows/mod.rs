use std::cell::UnsafeCell;
use std::error::Error;
use std::ffi::{c_void, CString};
use std::mem::size_of;
use std::ptr::null_mut;

pub const MUTEX_ALL_ACCESS: u32 = 0x1F0001;
use winapi::{
    shared::ntdef::{FALSE, NULL},
    um::{
        handleapi::CloseHandle,
        synchapi::{CreateMutexExA, ReleaseMutex, WaitForSingleObject, CREATE_MUTEX_INITIAL_OWNER},
        winbase::{OpenMutexA, INFINITE, WAIT_ABANDONED, WAIT_OBJECT_0},
        winnt::{HANDLE, SYNCHRONIZE},
    },
};

use log::*;

use super::{LockGuard, LockImpl, LockInit};

pub struct Mutex {
    handle: HANDLE,
    data: UnsafeCell<*mut c_void>,
}

impl LockInit for Mutex {
    fn size_of() -> usize {
        size_of::<u32>()
    }
    fn alignment() -> Option<u8> {
        None
    }

    unsafe fn new(
        dst: &mut [u8],
        data: *mut c_void,
    ) -> Result<(Box<dyn LockImpl>, usize), Box<dyn Error>> {
        // Make sure theres enough space for the mutex ID
        if dst.len() < Self::size_of() {
            return Err(From::from(format!(
                "Not enough space to hold Mutex {}/{}",
                dst.len(),
                Self::size_of()
            )));
        }

        // Find a mutex id that doesnt collide with another
        let mut mutex_handle: HANDLE = NULL;
        let mut mutex_id: u32 = 0;
        while mutex_handle == NULL {
            mutex_id = rand::random::<u32>();
            let path = CString::new(format!("mutex_{}", mutex_id)).unwrap();
            debug!(
                "CreateMutexExA(NULL, '{}', 0x{:X}, 0x{:X})",
                path.to_string_lossy(),
                CREATE_MUTEX_INITIAL_OWNER,
                MUTEX_ALL_ACCESS
            );
            mutex_handle = CreateMutexExA(
                null_mut(),
                path.as_ptr() as *mut _,
                CREATE_MUTEX_INITIAL_OWNER,
                MUTEX_ALL_ACCESS,
            );
        }

        // Create our mutex struct
        let mutex = Box::new(Self {
            handle: mutex_handle,
            data: UnsafeCell::new(data),
        });
        mutex.release()?;

        // Write the mutex id to the backing memory
        *(dst.as_mut_ptr() as *mut u32) = mutex_id;

        Ok((mutex, Self::size_of()))
    }

    unsafe fn from_existing(
        src: &mut [u8],
        data: *mut c_void,
    ) -> Result<(Box<dyn LockImpl>, usize), Box<dyn Error>> {
        // Make sure theres enough space for the mutex ID
        if src.len() < Self::size_of() {
            return Err(From::from(format!(
                "Not enough space to hold Mutex {}/{}",
                src.len(),
                Self::size_of()
            )));
        }

        let mutex_id = *(src.as_mut_ptr() as *mut u32);
        let path = CString::new(format!("mutex_{}", mutex_id)).unwrap();
        debug!(
            "OpenMutexA(0x{:X}, 0x{:X}, '{}')",
            SYNCHRONIZE,
            FALSE,
            path.to_string_lossy()
        );
        let mutex_handle = OpenMutexA(SYNCHRONIZE, FALSE as _, path.as_ptr() as *mut _);
        if mutex_handle == NULL {
            return Err(From::from(format!(
                "Failed to open mutex {}",
                path.to_string_lossy()
            )));
        }

        let mutex = Box::new(Self {
            handle: mutex_handle,
            data: UnsafeCell::new(data),
        });

        Ok((mutex, Self::size_of()))
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        debug!("CloseHandle(0x{:X})", self.handle as usize);
        unsafe { CloseHandle(self.handle) };
    }
}

impl LockImpl for Mutex {
    fn lock(&self) -> Result<LockGuard<'_>, Box<dyn Error>> {
        debug!("WaitForSingleObject(0x{:X})", self.handle as usize);
        let wait_res = unsafe { WaitForSingleObject(self.handle, INFINITE) };
        if wait_res == WAIT_OBJECT_0 {
            Ok(LockGuard::new(self))
        } else if wait_res == WAIT_ABANDONED {
            panic!("A thread holding the mutex has left it in a poisened state");
        } else {
            Err(From::from(format!(
                "Failed to aquire lock with value : 0x{:X}",
                wait_res
            )))
        }
    }
    fn release(&self) -> Result<(), Box<dyn Error>> {
        debug!("ReleaseMutex(0x{:X})", self.handle as usize);
        if unsafe { ReleaseMutex(self.handle) } == 0 {
            Err(From::from(
                "Could not release mutex as we did not own it".to_string(),
            ))
        } else {
            Ok(())
        }
    }
    unsafe fn get_inner(&self) -> &mut *mut c_void {
        &mut *self.data.get()
    }
}
