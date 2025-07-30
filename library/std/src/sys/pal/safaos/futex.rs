use crate::sync::atomic::Atomic;
use crate::time::Duration;

/// An atomic for use as a futex that is at least 32-bits but may be larger
pub type Futex = Atomic<Primitive>;
/// Must be the underlying type of Futex
pub type Primitive = u32;

/// An atomic for use as a futex that is at least 8-bits but may be larger.
pub type SmallFutex = Atomic<SmallPrimitive>;
/// Must be the underlying type of SmallFutex
pub type SmallPrimitive = u32;

pub fn futex_wait(futex: &Atomic<u32>, expected: u32, timeout: Option<Duration>) -> bool {
    // FIXME: Infinite timeout is just the max for now
    let timeout_duration = timeout.unwrap_or(Duration::MAX);
    let results = safa_api::syscalls::futex::futex_wait(futex, expected, timeout_duration)
        .expect("FATAL System error while waiting for Futex");
    results
}

#[inline]
pub fn futex_wake(futex: &Atomic<u32>) -> bool {
    let results = safa_api::syscalls::futex::futex_wake(futex, 1)
        .expect("FATAL System error while waking 1 Futex")
        > 0;
    results
}

#[inline]
pub fn futex_wake_all(futex: &Atomic<u32>) {
    safa_api::syscalls::futex::futex_wake(futex, usize::MAX)
        .expect("FATAL System error while waking all Futexes");
}
