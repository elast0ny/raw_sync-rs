# raw_sync
This crate is provides a lightweight wrapper around OS synchronization primitives.

It was mainly developed to be used with the `shared_memory` crate where a user needs to share locks/events between processes using shared memory.


## Features
### Locks
| Feature| Description | Linux | Windows|  Mac<sup>**</sup>| FreeBSD<sup>**</sup> |
|--------|-------------|:-----:|:------:|:----:| :-----: |
|LockType::Mutex|Mutually exclusive lock|✔|✔</sup>|✔|✔|
|LockType::RwLock|Exlusive write/shared read|✔|X<sup>[#1](https://github.com/elast0ny/shared_memory-rs/issues/1)</sup>|✔|✔|


### Events

| Feature| Description | Linux | Windows|  Mac<sup>**</sup>| FreeBSD<sup>**</sup> |
|--------|-------------|:-----:|:------:|:----:| :-----: |
|EventType::Auto/Manual| Generic event : [pthread_cond](https://linux.die.net/man/3/pthread_cond_init) on Unix and [Event Objects](https://msdn.microsoft.com/en-us/library/windows/desktop/ms682655.aspx) on windows. |✔|✔|X<sup>[#14](https://github.com/elast0ny/shared_memory-rs/issues/14)</sup>|✔|
|EventType::*Busy|Busy event managed by polling an AtomicBool in a loop|✔|✔|✔|✔|
|EventType::*EventFd|[Linux specific event type](http://man7.org/linux/man-pages/man2/eventfd.2.html)|✔|N/A|N/A|N/A|