use std::error::Error;
use std::ffi::c_void;
use std::ops::{Deref, DerefMut};

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        use windows as os;
    } else if #[cfg(any(target_os = "linux"))] {
        mod nix;
        use nix as os;
    } else {
        unimplemented!("This crate does not support your OS yet !");
    }
}

pub use os::*;

pub struct LockGuard<'t> {
    lock: &'t dyn LockImpl,
}
impl<'t> Drop for LockGuard<'t> {
    fn drop(&mut self) {
        let _ = self.lock.release();
    }
}
impl<'t> LockGuard<'t> {
    fn new(lock_impl: &'t dyn LockImpl) -> Self {
        Self { lock: lock_impl }
    }
}
impl<'t> Deref for LockGuard<'t> {
    type Target = *mut c_void;
    fn deref(&self) -> &Self::Target {
        unsafe { self.lock.get_inner() }
    }
}
impl<'t> DerefMut for LockGuard<'t> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.lock.get_inner() }
    }
}

pub struct ReadLockGuard<'t> {
    lock: &'t dyn LockImpl,
}
impl<'t> Drop for ReadLockGuard<'t> {
    fn drop(&mut self) {
        let _ = self.lock.release();
    }
}
impl<'t> Deref for ReadLockGuard<'t> {
    type Target = *const c_void;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.lock.get_inner() as *mut *mut c_void as *const *const c_void) }
    }
}

pub trait LockInit {
    /// Size required for the lock's internal representation
    fn size_of() -> usize;
    /// Potential Alignment requirements for the lock's internal representation
    fn alignment() -> Option<u8>;
    /// Initializes a new instance of the lock in the provided buffer and returns
    /// the remaining unused bytes.
    /// SAFETY : The caller MUST stop using the slice provided as `dst` and use the returned remainder on success
    unsafe fn new(
        dst: &mut [u8],
        data: *mut c_void,
    ) -> Result<(Box<dyn LockImpl>, &mut [u8]), Box<dyn Error>>;
    /// Creates a lock from an already initialized location
    /// SAFETY : The caller MUST stop using the slice provided as `src` and use the returned remainder on success
    unsafe fn from_existing(
        src: &mut [u8],
        data: *mut c_void,
    ) -> Result<(Box<dyn LockImpl>, &mut [u8]), Box<dyn Error>>;
}

pub trait LockImpl {
    /// Acquires the lock
    fn lock(&self) -> Result<LockGuard<'_>, Box<dyn Error>>;
    /// Release the lock
    fn release(&self) -> Result<(), Box<dyn Error>>;

    /// Leaks the inner data without acquiring the lock
    #[doc(hidden)]
    unsafe fn get_inner(&self) -> &mut *mut c_void;
    /*
    /// Returns a read only access to the protected data. This function always returns an
    /// error for locks that do not implemented read only access.
    fn rlock(&self) -> Result<ReadLockGuard<'_, T>, Box<dyn Error>> {
        self.lock()
    }
    */
}
