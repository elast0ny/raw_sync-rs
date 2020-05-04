use std::cell::UnsafeCell;
use std::mem::size_of;

use libc::{
    //Mutex defs
    pthread_mutex_init,
    pthread_mutex_lock,
    pthread_mutex_t,
    pthread_mutex_unlock,
    pthread_mutexattr_init,
    pthread_mutexattr_setpshared,
    pthread_mutexattr_t,
    
    //Rwlock defs
    pthread_rwlock_init,
    pthread_rwlock_rdlock,
    pthread_rwlock_t,
    pthread_rwlock_unlock,
    pthread_rwlock_wrlock,
    pthread_rwlockattr_init,
    pthread_rwlockattr_setpshared,
    pthread_rwlockattr_t,

    PTHREAD_PROCESS_SHARED,
};
use log::*;
/*
cfg_if::cfg_if! {
    if #[cfg(target_os="macos")] {
        mod mac;
        pub use mac::pthread_mutex_timedlock;
    } else {
        use ::libc::pthread_mutex_timedlock;
    }
}
*/
use super::{LockGuard, ReadLockGuard, LockImpl, LockInit};
use crate::Result;

pub struct Mutex {
    ptr: *mut pthread_mutex_t,
    data: UnsafeCell<*mut u8>,
}

impl LockInit for Mutex {
    fn size_of() -> usize {
        size_of::<pthread_mutex_t>()
    }
    fn alignment() -> Option<u8> {
        // Keep it pointer aligned
        Some(size_of::<*mut u8>() as _)
    }

    unsafe fn new(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = match Self::alignment() {
            Some(v) => {
                let padding = mem.align_offset(v as _);
                padding
            }
            None => 0,
        };

        let mut lock_attr: pthread_mutexattr_t = std::mem::zeroed();
        if pthread_mutexattr_init(&mut lock_attr) != 0 {
            return Err(From::from(
                "Failed to initialize pthread_mutexattr_t".to_string(),
            ));
        }
        if pthread_mutexattr_setpshared(&mut lock_attr, PTHREAD_PROCESS_SHARED) != 0 {
            return Err(From::from(
                "Failed to set pthread_mutexattr_setpshared(PTHREAD_PROCESS_SHARED)".to_string(),
            ));
        }
        let ptr = mem.offset(padding as _) as *mut _;
        debug!("pthread_mutex_init({:p})", ptr);
        if pthread_mutex_init(ptr, &lock_attr) != 0 {
            return Err(From::from(
                "Failed to initialize mutex pthread_mutex_init".to_string(),
            ));
        }

        let mutex = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((mutex, Self::size_of()))
    }

    unsafe fn from_existing(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = match Self::alignment() {
            Some(v) => {
                let padding = mem.align_offset(v as _);
                padding
            }
            None => 0,
        };

        let ptr = mem.offset(padding as _) as *mut _;

        debug!("existing mutex ({:p})", ptr);
        let mutex = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((mutex, Self::size_of()))
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {}
}

impl LockImpl for Mutex {
    fn lock(&self) -> Result<LockGuard<'_>> {
        let res = unsafe { pthread_mutex_lock(self.ptr) };
        debug!("pthread_mutex_lock({:p})", self.ptr);
        if res != 0 {
            return Err(From::from(format!("Failed to acquire mutex : {}", res)));
        }

        Ok(LockGuard::new(self))
    }
    fn release(&self) -> Result<()> {
        let res = unsafe { pthread_mutex_unlock(self.ptr) };
        debug!("pthread_mutex_unlock({:p})", self.ptr);
        if res != 0 {
            return Err(From::from(format!("Failed to release mutex : {}", res)));
        }
        Ok(())
    }
    unsafe fn get_inner(&self) -> &mut *mut u8 {
        &mut *self.data.get()
    }
}

pub struct RWLock {
    ptr: *mut pthread_rwlock_t,
    data: UnsafeCell<*mut u8>,
}

impl LockInit for RWLock {
    fn size_of() -> usize {
        size_of::<pthread_rwlock_t>()
    }
    fn alignment() -> Option<u8> {
        // Keep it pointer aligned
        Some(size_of::<*mut u8>() as _)
    }

    unsafe fn new(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = match Self::alignment() {
            Some(v) => {
                let padding = mem.align_offset(v as _);
                padding
            }
            None => 0,
        };

        let mut lock_attr: pthread_rwlockattr_t = std::mem::zeroed();
        if pthread_rwlockattr_init(&mut lock_attr) != 0 {
            return Err(From::from(
                "Failed to initialize pthread_rwlockattr_t".to_string(),
            ));
        }
        if pthread_rwlockattr_setpshared(&mut lock_attr, PTHREAD_PROCESS_SHARED) != 0 {
            return Err(From::from(
                "Failed to set pthread_rwlockattr_setpshared(PTHREAD_PROCESS_SHARED)".to_string(),
            ));
        }
        let ptr = mem.offset(padding as _) as *mut _;
        debug!("pthread_rwlock_init({:p})", ptr);
        if pthread_rwlock_init(ptr, &lock_attr) != 0 {
            return Err(From::from(
                "Failed to initialize pthread_rwlock_init".to_string(),
            ));
        }

        let lock = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((lock, Self::size_of()))
    }

    unsafe fn from_existing(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = match Self::alignment() {
            Some(v) => {
                let padding = mem.align_offset(v as _);
                padding
            }
            None => 0,
        };

        let ptr = mem.offset(padding as _) as *mut _;

        debug!("existing rwlock ({:p})", ptr);
        let lock = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((lock, Self::size_of()))
    }
}

impl Drop for RWLock {
    fn drop(&mut self) {}
}

impl LockImpl for RWLock {
    fn lock(&self) -> Result<LockGuard<'_>> {
        let res = unsafe { pthread_rwlock_wrlock(self.ptr) };
        debug!("pthread_rwlock_wrlock({:p})", self.ptr);
        if res != 0 {
            return Err(From::from(format!(
                "Failed to acquire writeable rwlock : {}",
                res
            )));
        }

        Ok(LockGuard::new(self))
    }
    fn rlock(&self) -> Result<ReadLockGuard<'_>> {
        let res = unsafe { pthread_rwlock_rdlock(self.ptr) };
        debug!("pthread_rwlock_rdlock({:p})", self.ptr);
        if res != 0 {
            return Err(From::from(format!(
                "Failed to acquire readable rwlock : {}",
                res
            )));
        }

        Ok(ReadLockGuard::new(self))
    }
    fn release(&self) -> Result<()> {
        let res = unsafe { pthread_rwlock_unlock(self.ptr) };
        debug!("pthread_rwlock_unlock({:p})", self.ptr);
        if res != 0 {
            return Err(From::from(format!("Failed to release rwlock : {}", res)));
        }
        Ok(())
    }
    unsafe fn get_inner(&self) -> &mut *mut u8 {
        &mut *self.data.get()
    }
}
