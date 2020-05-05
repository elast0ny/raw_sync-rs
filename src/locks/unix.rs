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

use super::{LockGuard, LockImpl, LockInit, ReadLockGuard};
use crate::Result;

pub struct Mutex {
    ptr: *mut pthread_mutex_t,
    data: UnsafeCell<*mut u8>,
}

impl LockInit for Mutex {
    fn size_of() -> usize {
        size_of::<pthread_mutex_t>()
    }

    unsafe fn new(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = mem.align_offset(size_of::<*mut u8>() as _);

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
        trace!("pthread_mutex_init({:p})", ptr);
        if pthread_mutex_init(ptr, &lock_attr) != 0 {
            return Err(From::from(
                "Failed to initialize mutex pthread_mutex_init".to_string(),
            ));
        }

        let mutex = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((mutex, ptr as usize - mem as usize))
    }

    unsafe fn from_existing(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = mem.align_offset(size_of::<*mut u8>() as _);

        let ptr = mem.offset(padding as _) as *mut _;

        trace!("existing mutex ({:p})", ptr);
        let mutex = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((mutex, ptr as usize - mem as usize))
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {}
}

impl LockImpl for Mutex {
    fn lock(&self) -> Result<LockGuard<'_>> {
        let res = unsafe { pthread_mutex_lock(self.ptr) };
        trace!("pthread_mutex_lock({:p})", self.ptr);
        if res != 0 {
            return Err(From::from(format!("Failed to acquire mutex : {}", res)));
        }

        Ok(LockGuard::new(self))
    }
    fn release(&self) -> Result<()> {
        let res = unsafe { pthread_mutex_unlock(self.ptr) };
        trace!("pthread_mutex_unlock({:p})", self.ptr);
        if res != 0 {
            return Err(From::from(format!("Failed to release mutex : {}", res)));
        }
        Ok(())
    }
    unsafe fn get_inner(&self) -> &mut *mut u8 {
        &mut *self.data.get()
    }
}

pub struct RwLock {
    ptr: *mut pthread_rwlock_t,
    data: UnsafeCell<*mut u8>,
}

impl LockInit for RwLock {
    fn size_of() -> usize {
        size_of::<pthread_rwlock_t>()
    }

    unsafe fn new(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = mem.align_offset(size_of::<*mut u8>() as _);

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
        trace!("pthread_rwlock_init({:p})", ptr);
        if pthread_rwlock_init(ptr, &lock_attr) != 0 {
            return Err(From::from(
                "Failed to initialize pthread_rwlock_init".to_string(),
            ));
        }

        let lock = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((lock, ptr as usize - mem as usize))
    }

    unsafe fn from_existing(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = mem.align_offset(size_of::<*mut u8>() as _);

        let ptr = mem.offset(padding as _) as *mut _;

        trace!("existing rwlock ({:p})", ptr);
        let lock = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((lock, ptr as usize - mem as usize))
    }
}

impl Drop for RwLock {
    fn drop(&mut self) {}
}

impl LockImpl for RwLock {
    fn lock(&self) -> Result<LockGuard<'_>> {
        let res = unsafe { pthread_rwlock_wrlock(self.ptr) };
        trace!("pthread_rwlock_wrlock({:p})", self.ptr);
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
        trace!("pthread_rwlock_rdlock({:p})", self.ptr);
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
        trace!("pthread_rwlock_unlock({:p})", self.ptr);
        if res != 0 {
            return Err(From::from(format!("Failed to release rwlock : {}", res)));
        }
        Ok(())
    }
    unsafe fn get_inner(&self) -> &mut *mut u8 {
        &mut *self.data.get()
    }
}
