use sp::locks::*;
use sync_primitives as sp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut buf: [u8; 16] = [0; 16];
    let mut some_data: usize = 123;
    println!("Init Mutex in buff {:?}", buf);
    let (mutex, remainder) =
        unsafe { sp::locks::Mutex::new(&mut buf, &mut some_data as *mut _ as _)? };

    println!("Remainder {:?}", remainder);
    println!("Shared data {}", some_data);

    let data = mutex.lock()?;
    println!("Got lock on data {:p}", *data);
    unsafe { *(*data as *mut usize) = 124 };
    println!("Shared data {}", some_data);

    Ok(())
}
