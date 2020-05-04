use std::thread;
use std::time;

use env_logger::Env;
use log::*;
use raw_sync::locks::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn increment_val(id: u8, lock: Box<dyn LockImpl>) {
    loop {
        info!("[T{}] Waiting for lock...", id);
        let data_ptr = lock.lock().unwrap();
        //Cast the lock data to usize
        let data: &mut usize = unsafe { &mut *(*data_ptr as *mut usize) };
        if *data >= 5 {
            break;
        }
        *data += 1;
        info!("[T{}] Got lock, sleeping for 1s", id);
        thread::sleep(time::Duration::from_secs(1));
    }
    info!("[T{}] Done !", id);
}

fn main() -> Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    
    info!("Mutex");
    test_mutex()?;

    #[cfg(not(windows))]
    info!("RWLock");
    #[cfg(not(windows))]
    test_rwlock()?;

    Ok(())
}

fn test_mutex() -> Result<()> {
    let mut buf: [u8; 16] = [0; 16];
    let mut some_data: usize = 0;
    let data_ptr = &mut some_data as *mut _ as usize;
    
    let (mutex, _) = unsafe { Mutex::new(&mut buf, data_ptr as _)? };

    thread::spawn(move || {
        let (mutex, _) = unsafe { Mutex::from_existing(&mut buf, data_ptr as _).unwrap() };

        increment_val(2, mutex);
    });

    increment_val(1, mutex);
    Ok(())
}
