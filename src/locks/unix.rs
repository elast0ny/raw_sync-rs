use std::cell::UnsafeCell;
use std::mem::{size_of, MaybeUninit};
use std::time::Duration;

use libc::{
    clock_gettime,
    pthread_mutex_init,
    pthread_mutex_lock,
    //Mutex defs
    pthread_mutex_t,
    //pthread_mutex_timedlock,
    pthread_mutex_unlock,

    pthread_mutexattr_init,
    pthread_mutexattr_setpshared,
    pthread_mutexattr_t,
    pthread_rwlock_init,
    pthread_rwlock_rdlock,
    //Rwlock defs
    pthread_rwlock_t,
    //pthread_rwlock_timedwrlock,
    pthread_rwlock_unlock,

    //pthread_rwlock_timedrdlock,
    pthread_rwlock_wrlock,
    pthread_rwlockattr_init,
    pthread_rwlockattr_setpshared,
    pthread_rwlockattr_t,
    timespec,
    CLOCK_REALTIME,

    PTHREAD_PROCESS_SHARED,
};
//use log::*;

extern "C" {
    fn pthread_rwlock_timedrdlock(attr: *mut pthread_rwlock_t, host: *const timespec) -> i32;
    fn pthread_rwlock_timedwrlock(attr: *mut pthread_rwlock_t, host: *const timespec) -> i32;
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        unsafe fn pthread_mutex_timedlock(lock: *mut pthread_mutex_t, abstime: &timespec) -> i32 {
            let mut timenow: timespec = timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            let timesleep: timespec = timespec {
                tv_sec: 0,
                tv_nsec: 10_000_000, // 10ms
            };
            let mut res: i32;
            loop {
                res = libc::pthread_mutex_trylock(lock);
                if res == libc::EBUSY {
                    // Check timeout before sleeping
                    clock_gettime(CLOCK_REALTIME, &mut timenow);
                    if timenow.tv_sec >= abstime.tv_sec && timenow.tv_nsec >= abstime.tv_nsec {
                        return libc::ETIMEDOUT;
                    }
                    // Sleep for a bit
                    libc::nanosleep(&timesleep, std::ptr::null_mut());
                    continue;
                }
                break;
            }
            res
       }
   } else {
       use libc::pthread_mutex_timedlock;
   }
}

use super::{LockGuard, LockImpl, LockInit, ReadLockGuard};
use crate::{Result, Timeout};

/// Adds a duration to the current time
pub(crate) fn abs_timespec_from_duration(d: Duration) -> timespec {
    unsafe {
        #[allow(clippy::uninit_assumed_init)]
        let mut cur_time: timespec = MaybeUninit::uninit().assume_init();
        // Get current time
        clock_gettime(CLOCK_REALTIME, &mut cur_time);
        // Add duration
        cur_time.tv_sec += d.as_secs() as nix::sys::time::time_t;
        cur_time.tv_nsec += d.subsec_nanos() as nix::sys::time::time_t;
        cur_time
    }
}

pub struct Mutex {
    ptr: *mut pthread_mutex_t,
    data: UnsafeCell<*mut u8>,
}

impl LockInit for Mutex {
    fn size_of(addr: Option<*mut u8>) -> usize {
        let padding = match addr {
            Some(mem) => mem.align_offset(size_of::<*mut u8>() as _),
            None => 0,
        };
        padding + size_of::<pthread_mutex_t>()
    }

    #[allow(clippy::new_ret_no_self)]
    unsafe fn new(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = mem.align_offset(size_of::<*mut u8>() as _);
        #[allow(clippy::uninit_assumed_init)]
        let mut lock_attr: pthread_mutexattr_t = MaybeUninit::uninit().assume_init();

        let ptr = mem.add(padding) as *mut _;
        debug!("Mutex new {:p}", ptr);
        
        debug!("pthread_mutexattr_init({:p})", &lock_attr);
        let res = pthread_mutexattr_init(&mut lock_attr);
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::InitFailed(std::io::Error::from_raw_os_error(res)));
        }
        
        debug!("pthread_mutexattr_setpshared({:p})", &lock_attr);
        let res = pthread_mutexattr_setpshared(&mut lock_attr, PTHREAD_PROCESS_SHARED);
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::InitFailed(std::io::Error::from_raw_os_error(res)));
        }
        
        debug!("pthread_mutex_init({:p}, {:p})", ptr, &lock_attr);
        let res = pthread_mutex_init(ptr, &lock_attr);
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::InitFailed(std::io::Error::from_raw_os_error(res)));
        }

        let mutex = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((mutex, (ptr as usize - mem as usize) + Self::size_of(None)))
    }

    unsafe fn from_existing(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = mem.align_offset(size_of::<*mut u8>() as _);
    
        let ptr = mem.add(padding) as *mut _;

        debug!("Mutex from {:p}", ptr);

        let mutex = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((mutex, (ptr as usize - mem as usize) + Self::size_of(None)))
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {}
}

impl LockImpl for Mutex {
    fn as_raw(&self) -> *mut std::ffi::c_void {
        self.ptr as _
    }

    fn lock(&self) -> Result<LockGuard<'_>> {

        debug!("pthread_mutex_lock({:p})", self.ptr);
        let res = unsafe { pthread_mutex_lock(self.ptr) };
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::LockFailed(std::io::Error::from_raw_os_error(res)));
        }

        Ok(LockGuard::new(self))
    }

    fn try_lock(&self, timeout: Timeout) -> Result<LockGuard<'_>> {
        let timespec: timespec = match timeout {
            Timeout::Infinite => return self.lock(),
            Timeout::Val(d) => abs_timespec_from_duration(d),
        };

        debug!("pthread_mutex_timedlock({:p})", self.ptr);
        let res = unsafe { pthread_mutex_timedlock(self.ptr, &timespec) };
        debug!("\tres = {}", res);
        
        if res != 0 {
            return if res == libc::ETIMEDOUT {
                Err(crate::Error::TimedOut)
            } else {
                Err(crate::Error::LockFailed(std::io::Error::from_raw_os_error(res)))
            };
        }

        Ok(LockGuard::new(self))
    }

    fn release(&self) -> Result<()> {
        debug!("pthread_mutex_unlock({:p})", self.ptr);
        let res = unsafe { pthread_mutex_unlock(self.ptr) };
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::ReleaseFailed(std::io::Error::from_raw_os_error(res)));
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
    fn size_of(addr: Option<*mut u8>) -> usize {
        let padding = match addr {
            Some(mem) => mem.align_offset(size_of::<*mut u8>() as _),
            None => 0,
        };
        padding + size_of::<pthread_rwlock_t>()
    }

    #[allow(clippy::new_ret_no_self)]
    unsafe fn new(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = mem.align_offset(size_of::<*mut u8>() as _);
        let ptr = mem.add(padding) as *mut _;

        debug!("RWLock new {:p}", ptr);

        #[allow(clippy::uninit_assumed_init)]
        let mut lock_attr: pthread_rwlockattr_t = MaybeUninit::uninit().assume_init();

        debug!("pthread_rwlockattr_init({:p})", &lock_attr);
        let res = pthread_rwlockattr_init(&mut lock_attr);
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::InitFailed(std::io::Error::from_raw_os_error(res)));
        }
        
        debug!("pthread_rwlockattr_init({:p})", &lock_attr);
        let res = pthread_rwlockattr_setpshared(&mut lock_attr, PTHREAD_PROCESS_SHARED);
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::InitFailed(std::io::Error::from_raw_os_error(res)));
        }
        
        debug!("pthread_rwlock_init({:p})", ptr);
        let res = pthread_rwlock_init(ptr, &lock_attr);
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::InitFailed(std::io::Error::from_raw_os_error(res)));
        }

        let lock = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((lock, (ptr as usize - mem as usize) + Self::size_of(None)))
    }

    unsafe fn from_existing(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)> {
        let padding = mem.align_offset(size_of::<*mut u8>() as _);

        let ptr = mem.add(padding) as *mut _;
        debug!("RWLock from {:p}", ptr);

        let lock = Box::new(Self {
            ptr,
            data: UnsafeCell::new(data),
        });

        Ok((lock, (ptr as usize - mem as usize) + Self::size_of(None)))
    }
}

impl Drop for RwLock {
    fn drop(&mut self) {}
}

impl LockImpl for RwLock {
    fn as_raw(&self) -> *mut std::ffi::c_void {
        self.ptr as _
    }

    fn lock(&self) -> Result<LockGuard<'_>> {
        debug!("pthread_rwlock_wrlock({:p})", self.ptr);
        let res = unsafe { pthread_rwlock_wrlock(self.ptr) };
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::LockFailed(std::io::Error::from_raw_os_error(res)));
        }

        Ok(LockGuard::new(self))
    }

    fn try_lock(&self, timeout: Timeout) -> Result<LockGuard<'_>> {
        let timespec: timespec = match timeout {
            Timeout::Infinite => return self.lock(),
            Timeout::Val(d) => abs_timespec_from_duration(d),
        };

        debug!("pthread_rwlock_timedwrlock({:p})", self.ptr);
        let res = unsafe { pthread_rwlock_timedwrlock(self.ptr, &timespec) };
        debug!("\tres = {}", res);
        if res != 0 {
            return if res == libc::ETIMEDOUT {
                Err(crate::Error::TimedOut)
            } else {
                Err(crate::Error::LockFailed(std::io::Error::from_raw_os_error(res)))
            };
        }

        Ok(LockGuard::new(self))
    }

    fn rlock(&self) -> Result<ReadLockGuard<'_>> {
        debug!("pthread_rwlock_rdlock({:p})", self.ptr);
        let res = unsafe { pthread_rwlock_rdlock(self.ptr) };
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::LockFailed(std::io::Error::from_raw_os_error(res)));
        }

        Ok(ReadLockGuard::new(self))
    }

    fn try_rlock(&self, timeout: Timeout) -> Result<ReadLockGuard<'_>> {
        let timespec: timespec = match timeout {
            Timeout::Infinite => return self.rlock(),
            Timeout::Val(d) => abs_timespec_from_duration(d),
        };

        debug!("pthread_rwlock_timedrdlock({:p})", self.ptr);
        let res = unsafe { pthread_rwlock_timedrdlock(self.ptr, &timespec) };
        debug!("\tres = {}", res);
        if res != 0 {
            return if res == libc::ETIMEDOUT {
                Err(crate::Error::TimedOut)
            } else {
                Err(crate::Error::LockFailed(std::io::Error::from_raw_os_error(res)))
            };
        }

        Ok(ReadLockGuard::new(self))
    }

    fn release(&self) -> Result<()> {
        debug!("pthread_rwlock_unlock({:p})", self.ptr);
        let res = unsafe { pthread_rwlock_unlock(self.ptr) };
        debug!("\tres = {}", res);
        if res != 0 {
            return Err(crate::Error::ReleaseFailed(std::io::Error::from_raw_os_error(res)));
        }
        Ok(())
    }
    unsafe fn get_inner(&self) -> &mut *mut u8 {
        &mut *self.data.get()
    }
}
