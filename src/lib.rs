use std::error::Error;
pub struct LockGuard<'t, T> {
    lock: &'t dyn LockImpl<'t, T>,
}
pub struct ReadLockGuard<'t, T> {
    lock: &'t dyn LockImpl<'t, T>,
}

pub trait LockInit<'t, T: ?Sized + 't> {
    /// Size required for the lock's internal representation
    fn size_of() -> usize;
    /// Potential Alignment requirements for the lock's internal representation
    fn alignment() -> Option<u8>;
    /// Initializes a new instance of the lock in the provided buffer and returns
    /// the remaining unused bytes. 
    /// SAFETY : The caller MUST stop using the slice provided as `dst` and use the returned remainder on success
    unsafe fn new(dst: &mut [u8], data: &'t mut [u8]) -> Result<(Box<dyn LockImpl<'t, T>>, &'t mut [u8]), Box<dyn Error>>;
    /// Creates a lock from an already initialized location
    /// SAFETY : The caller MUST stop using the slice provided as `src` and use the returned remainder on success
    unsafe fn from_existing(src: &mut [u8], data: &'t mut [u8]) -> Result<(Box<dyn LockImpl<'t, T>>, &'t mut [u8]), Box<dyn Error>>;
}

pub trait LockImpl<'t, T: ?Sized + 't> {    
        /// Returns a read and write access to the protected data.
        fn lock(&'t mut self) -> Result<LockGuard<'_, T>, Box<dyn Error>>;

        /// Returns a read only access to the protected data. This function always returns an
        /// error for locks that do not implemented read only access.
        fn rlock(&self) -> Result<ReadLockGuard<'_, T>, Box<dyn Error>> {
            Err(From::from("This lock does not implement rlock()".to_string()))
        }
}