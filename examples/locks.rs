use std::thread;
use std::time;

use env_logger::Env;
use log::*;
use raw_sync::locks::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn increment_val(id: u8, lock: Box<dyn LockImpl>) {
    loop {
        info!("[T{}] Waiting for lock...", id);
        //Read value
        {
            let data_ptr = lock.rlock().unwrap();
            info!("[T{}]\t READ LOCKED", id);
            let data = unsafe { &*(*data_ptr as *const usize) };
            if *data >= 5 {
                break;
            }
            thread::sleep(time::Duration::from_secs(1));
            info!("[T{}]\t READ RELEASED", id);
        }

        // Write to value
        {
            let data_ptr = lock.lock().unwrap();
            info!("[T{}]\t WRITE LOCKED", id);
            let data = unsafe { &mut *(*data_ptr as *mut usize) };
            *data += 1;
            thread::sleep(time::Duration::from_secs(1));
            info!("[T{}]\t WRITE RELEASED", id);
        }
    }
    info!("[T{}] Done !", id);
}

fn main() -> Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let mut mem = [0u8; 64];

    info!("Mutex");
    test_mutex(mem.as_mut_ptr())?;

    #[cfg(not(windows))]
    info!("RWLock");
    #[cfg(not(windows))]
    test_rwlock(mem.as_mut_ptr())?;

    Ok(())
}

fn test_mutex(mem: *mut u8) -> Result<()> {
    let mut some_data: usize = 0;

    let mem_ptr = mem as usize;
    let data_ptr = &mut some_data as *mut _ as usize;

    let (lock, _) = unsafe { Mutex::new(mem, data_ptr as _)? };

    let child = thread::spawn(move || {
        let (lock, _) = unsafe { Mutex::from_existing(mem_ptr as _, data_ptr as _).unwrap() };
        increment_val(2, lock);
    });

    increment_val(1, lock);
    let _ = child.join();
    Ok(())
}

#[cfg(not(windows))]
fn test_rwlock(mem: *mut u8) -> Result<()> {
    let mut some_data: usize = 0;

    let mem_ptr = mem as usize;
    let data_ptr = &mut some_data as *mut _ as usize;

    let (lock, _) = unsafe { RWLock::new(mem, data_ptr as _)? };

    let child = thread::spawn(move || {
        let (lock, _) = unsafe { RWLock::from_existing(mem_ptr as _, data_ptr as _).unwrap() };
        increment_val(2, lock);
    });

    increment_val(1, lock);
    let _ = child.join();
    Ok(())
}
