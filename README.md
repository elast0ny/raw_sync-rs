# raw_sync
This crate is provides a lightweight wrapper around OS synchronization primitives.

It was mainly developed to be used with the `shared_memory` crate for when cross-process synchronization is required through shared memory.


## Features
### Locks
| Feature| Description | Linux | Windows| Mac| FreeBSD |
|--------|-------------|:-----:|:------:|:----:| :-----: |
|Mutex|Mutually exclusive lock|✔|✔</sup>|✔|✔|
|RwLock|Exclusive write/shared read|✔|X|✔|✔|


### Events

| Feature| Description | Linux | Windows|  Mac| FreeBSD |
|--------|-------------|:-----:|:------:|:----:| :-----: |
|Event| Generic event : [pthread_cond](https://linux.die.net/man/3/pthread_cond_init) on Unix and [Event Objects](https://msdn.microsoft.com/en-us/library/windows/desktop/ms682655.aspx) on windows. |✔|✔|X|✔|
|BusyEvent|Busy event managed by polling an AtomicU8 in a loop|✔|✔|✔|✔|
|EventFd|[Linux specific event type](http://man7.org/linux/man-pages/man2/eventfd.2.html)|✔|N/A|N/A|N/A|