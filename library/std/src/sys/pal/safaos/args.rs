use crate::ffi::OsString;
use crate::fmt;

pub struct Args(safa_api::process::args::ArgsIter);

pub fn args() -> Args {
    Args(safa_api::process::args::ArgsIter::get())
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        let len = self.len();
        for i in 0..len {
            // safe because index is always in len and raw_args is initialized (otherwise len is going to be 0)
            let arg = unsafe { self.0.get_index(i).unwrap_unchecked() };
            list.entry(&unsafe { arg.into_slice_mut() });
        }
        list.finish()
    }
}

impl Iterator for Args {
    type Item = OsString;
    fn next(&mut self) -> Option<OsString> {
        self.0.next().map(|arg| unsafe {
            OsString::from_encoded_bytes_unchecked(arg.into_slice_mut().to_vec())
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<OsString> {
        todo!("next_back on Args is not implemented yet")
    }
}
