use core::cell::SyncUnsafeCell;
use core::ptr::NonNull;

use crate::ffi::OsString;
use crate::fmt;
use crate::mem::MaybeUninit;

#[derive(Debug, Clone, Copy)]
pub(super) struct RawArgs {
    pub(super) args: NonNull<[(NonNull<u8>, usize)]>,
}

impl RawArgs {
    fn len(&self) -> usize {
        unsafe { (*self.args.as_ptr()).len() }
    }

    fn get(&self, index: usize) -> Option<&'static str> {
        unsafe {
            let args = &*self.args.as_ptr();
            let (ptr, len) = args.get(index)?;
            Some(core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr.as_ptr(), *len)))
        }
    }
}

unsafe impl Sync for RawArgs {}

pub(super) static RAW_ARGS: SyncUnsafeCell<MaybeUninit<Option<RawArgs>>> =
    SyncUnsafeCell::new(MaybeUninit::uninit());
pub struct Args {
    raw_args: Option<RawArgs>,
    index: usize,
}

pub fn args() -> Args {
    Args { raw_args: unsafe { (*RAW_ARGS.get()).assume_init() }, index: 0 }
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        let len = self.len();
        for i in 0..len {
            // safe because index is always in len and raw_args is initialized (otherwise len is going to be 0)
            let arg = unsafe { self.raw_args.unwrap_unchecked().get(i).unwrap_unchecked() };
            list.entry(&arg);
        }
        list.finish()
    }
}

impl Iterator for Args {
    type Item = OsString;
    fn next(&mut self) -> Option<OsString> {
        if self.index >= self.len() {
            return None;
        }
        // it is gruanted that the raw_args is initialized if len > 0
        let raw_args = unsafe { self.raw_args.unwrap_unchecked() };
        let results = raw_args.get(self.index).map(|arg| OsString::from(arg));
        self.index += 1;
        results
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        if len == 0 {
            (0, Some(0))
        } else {
            let left = len - self.index;
            (left, Some(left))
        }
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.raw_args.map(|s| s.len()).unwrap_or(0)
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<OsString> {
        todo!("next_back on Args is not implemented yet")
    }
}
