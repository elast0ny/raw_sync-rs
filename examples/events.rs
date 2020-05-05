use std::thread;
use std::time;

use env_logger::Env;
use log::*;
use raw_sync::{
    Timeout,
    events::*
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let mut mem = [0u8; 64];

    info!("----------------");
    info!("AutoEvent");
    info!("----------------");
    test_autoevent(mem.as_mut_ptr())?;

    info!("----------------");
    info!("ManualEvent");
    info!("----------------");
    test_manualevent(mem.as_mut_ptr())?;
    /*
    #[cfg(not(windows))]
    info!("RWLock");
    #[cfg(not(windows))]
    test_rwlock(mem.as_mut_ptr())?;
    */
    Ok(())
}

fn test_autoevent(mem: *mut u8) -> Result<()> {
    let (obj, _) = unsafe{Event::new(mem, true)?};

    let mem_ptr = mem as usize;

    let child = thread::spawn(move || {
        let (obj, _) = unsafe { Event::from_existing(mem_ptr as _).unwrap() };
        info!("Waiting for event to be signaled !");
        obj.wait(Timeout::Infinite).unwrap();
        info!("Waiting until timeout");
        if let Ok(_) = obj.wait(Timeout::Val(time::Duration::from_secs(1))) {
            panic!("This should have timed out !");
        };
        info!("timed out !");
        obj.set(EventState::Signaled).unwrap();
        obj.set(EventState::Signaled).unwrap();
        obj.set(EventState::Clear).unwrap();
        info!("Done");
    });
    
    info!("Setting event to signaled");
    obj.set(EventState::Signaled)?;
    thread::sleep(time::Duration::from_secs(3));
    info!("Waiting until timeout");
    if let Ok(_) = obj.wait(Timeout::Val(time::Duration::from_secs(1))) {
        panic!("This should have timed out !");
    };
    info!("timed out !");
    info!("Done");

    let _ = child.join();
    Ok(())
}

fn test_manualevent(mem: *mut u8) -> Result<()> {
    let (obj, _) = unsafe{Event::new(mem, false)?};

    let mem_ptr = mem as usize;

    obj.set(EventState::Signaled)?;

    let child = thread::spawn(move || {
        let (obj, _) = unsafe { Event::from_existing(mem_ptr as _).unwrap() };
        info!("Waiting for event to be signaled !");
        obj.wait(Timeout::Infinite).unwrap();
        info!("Waiting on same event");
        if let Err(_) = obj.wait(Timeout::Val(time::Duration::from_secs(1))) {
            panic!("This shouldnt time out!");
        };
        info!("Done");
    });
    
    thread::sleep(time::Duration::from_secs(3));
    info!("Waiting until timeout");
    if let Err(_) = obj.wait(Timeout::Val(time::Duration::from_secs(1))) {
        panic!("This shouldnt time out!");
    };
    obj.set(EventState::Clear)?;
    info!("Waiting until timeout");
    if let Ok(_) = obj.wait(Timeout::Val(time::Duration::from_secs(1))) {
        panic!("This shouldve timed out!");
    };
    info!("Timed out !");
    info!("Done");

    let _ = child.join();
    Ok(())
}
