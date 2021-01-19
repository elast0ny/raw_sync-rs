use std::thread;
use std::time;

use env_logger::Env;
use log::*;
use raw_sync::{events::*, Timeout};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let mut mem = [0u8; 64];

    // Regular event
    event_example(mem.as_mut_ptr(), true)?;
    event_example(mem.as_mut_ptr(), false)?;

    // Busy event
    busy_example(mem.as_mut_ptr(), true)?;
    busy_example(mem.as_mut_ptr(), false)?;

    // Linux EventFd
    /*
    #[cfg(linux)]
    eventfd_example(mem.as_mut_ptr(), true)?;
    #[cfg(linux)]
    eventfd_example(mem.as_mut_ptr(), false)?;
    */
    Ok(())
}

fn event_example(mem: *mut u8, auto_reset: bool) -> Result<()> {
    info!("----------------");
    info!("Event ({})", if auto_reset { "Auto" } else { "Manual" });
    info!("----------------");

    let (obj, _) = unsafe { Event::new(mem, auto_reset)? };

    let mem_ptr = mem as usize;

    let child = thread::spawn(move || {
        let (obj, _) = unsafe { Event::from_existing(mem_ptr as _).unwrap() };
        info!("\tWaiting for event to be signaled !");
        obj.wait(Timeout::Infinite).unwrap();
        info!("\tSignaled !");

        info!("\tWaiting until timeout");
        if auto_reset {
            if obj.wait(Timeout::Val(time::Duration::from_secs(1))).is_ok() {
                panic!("This should have timed out !");
            };
            info!("\ttimed out !");
        } else {
            if obj
                .wait(Timeout::Val(time::Duration::from_secs(1)))
                .is_err()
            {
                panic!("This shouldn't have timed out !");
            };
            info!("\tSignaled !");
        }

        info!("\tSetting event to signaled");
        obj.set(EventState::Signaled).unwrap();
        info!("\tSetting event to signaled");
        obj.set(EventState::Signaled).unwrap();
        info!("\tClearing event");
        obj.set(EventState::Clear).unwrap();
        info!("\tDone");
    });

    info!("Setting event to signaled");
    obj.set(EventState::Signaled)?;
    thread::sleep(time::Duration::from_secs(3));

    info!("Waiting until timeout");
    if obj.wait(Timeout::Val(time::Duration::from_secs(1))).is_ok() {
        panic!("This should have timed out !");
    };
    info!("timed out !");

    info!("Done");

    let _ = child.join();
    Ok(())
}

fn busy_example(mem: *mut u8, auto_reset: bool) -> Result<()> {
    info!("----------------");
    info!("BusyEvent ({})", if auto_reset { "Auto" } else { "Manual" });
    info!("----------------");

    let (obj, _) = unsafe { BusyEvent::new(mem, auto_reset)? };

    let mem_ptr = mem as usize;

    let child = thread::spawn(move || {
        let (obj, _) = unsafe { BusyEvent::from_existing(mem_ptr as _).unwrap() };
        info!("\tWaiting for event to be signaled !");
        obj.wait(Timeout::Infinite).unwrap();
        info!("\tSignaled !");

        info!("\tWaiting until timeout");
        if auto_reset {
            if obj.wait(Timeout::Val(time::Duration::from_secs(1))).is_ok() {
                panic!("This should have timed out !");
            };
            info!("\ttimed out !");
        } else {
            if obj
                .wait(Timeout::Val(time::Duration::from_secs(1)))
                .is_err()
            {
                panic!("This shouldn't have timed out !");
            };
            info!("\tSignaled !");
        }

        info!("\tSetting event to signaled");
        obj.set(EventState::Signaled).unwrap();
        info!("\tSetting event to signaled");
        obj.set(EventState::Signaled).unwrap();
        info!("\tClearing event");
        obj.set(EventState::Clear).unwrap();
        info!("\tDone");
    });

    info!("Setting event to signaled");
    obj.set(EventState::Signaled)?;
    thread::sleep(time::Duration::from_secs(3));

    info!("Waiting until timeout");
    if obj.wait(Timeout::Val(time::Duration::from_secs(1))).is_ok() {
        panic!("This should have timed out !");
    };
    info!("timed out !");

    info!("Done");

    let _ = child.join();
    Ok(())
}
