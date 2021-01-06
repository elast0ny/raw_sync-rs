use std::thread;
use std::time;

use env_logger::Env;
use log::*;
use raw_sync::{locks::*, Timeout};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn test_timeout(id: u8, lock: &dyn LockImpl) {
    info!("[{}] Waiting for lock for 1 second", id);
    let guard = lock.try_lock(Timeout::Val(time::Duration::from_secs(1)));
    if guard.is_err() {
        info!("[{}] Timed out !", id);
    } else {
        info!("[{}] Holding lock for 2s", id);
        thread::sleep(time::Duration::from_secs(2))
    }
}

fn increment_val(id: u8, lock: Box<dyn LockImpl>) {
    loop {
        info!("[{}] Waiting for lock...", id);
        //Read value
        {
            let data_ptr = lock.rlock().unwrap();
            info!("[{}]\t READ LOCKED", id);
            let data = unsafe { &*(*data_ptr as *const usize) };
            if *data >= 5 {
                info!("[{}]\t READ RELEASED", id);
                break;
            }
            thread::sleep(time::Duration::from_secs(1));
            info!("[{}]\t READ RELEASED", id);
        }

        // Write to value
        {
            let data_ptr = lock.lock().unwrap();
            info!("[{}]\t WRITE LOCKED", id);
            let data = unsafe { &mut *(*data_ptr as *mut usize) };
            *data += 1;
            thread::sleep(time::Duration::from_secs(1));
            info!("[{}]\t WRITE RELEASED", id);
        }
    }
    info!("[{}] Done !", id);
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let mut mem = [0u8; 64];

    test_mutex(mem.as_mut_ptr())?;

    #[cfg(not(windows))]
    test_rwlock(mem.as_mut_ptr())?;

    Ok(())
}

fn test_mutex(mem: *mut u8) -> Result<()> {
    info!("-----------");
    info!("Mutex");
    info!("-----------");
    let mut some_data: usize = 0;

    let mem_ptr = mem as usize;
    let data_ptr = &mut some_data as *mut _ as usize;

    let (lock, _) = unsafe { Mutex::new(mem, data_ptr as _)? };

    let child = thread::spawn(move || {
        let (lock, _) = unsafe { Mutex::from_existing(mem_ptr as _, data_ptr as _).unwrap() };
        test_timeout(2, &*lock);
        increment_val(2, lock);
    });

    test_timeout(1, &*lock);
    increment_val(1, lock);
    let _ = child.join();
    Ok(())
}

#[cfg(not(windows))]
fn test_rwlock(mem: *mut u8) -> Result<()> {
    info!("-----------");
    info!("RwLock");
    info!("-----------");

    let mut some_data: usize = 0;

    let mem_ptr = mem as usize;
    let data_ptr = &mut some_data as *mut _ as usize;

    let (lock, _) = unsafe { RwLock::new(mem, data_ptr as _)? };

    let child = thread::spawn(move || {
        let (lock, _) = unsafe { RwLock::from_existing(mem_ptr as _, data_ptr as _).unwrap() };
        test_timeout(2, &*lock);
        increment_val(2, lock);
    });

    test_timeout(1, &*lock);
    increment_val(1, lock);
    let _ = child.join();
    Ok(())
}
