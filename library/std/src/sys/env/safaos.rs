use core::fmt;

use crate::ffi::{OsStr, OsString};
use crate::io;

pub struct Env {
    inner: Vec<(OsString, OsString)>,
    index: usize,
}

impl Env {
    // FIXME(https://github.com/rust-lang/rust/issues/114583): Remove this when <OsStr as Debug>::fmt matches <str as Debug>::fmt.
    pub fn str_debug(&self) -> impl fmt::Debug + '_ {
        self
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.inner.iter()).finish()
    }
}

impl Iterator for Env {
    type Item = (OsString, OsString);
    fn next(&mut self) -> Option<(OsString, OsString)> {
        if self.index >= self.inner.len() {
            return None;
        }
        let (key, value) = self.inner[self.index].clone();
        self.index += 1;
        Some((key, value))
    }
}

pub fn env() -> Env {
    let all = safa_api::process::env::env_get_all();
    let mut results = Vec::with_capacity(all.len());
    for (key, value) in all {
        results.push((unsafe { OsString::from_encoded_bytes_unchecked(key.to_vec()) }, unsafe {
            OsString::from_encoded_bytes_unchecked(value.to_bytes().to_vec())
        }));
    }

    Env { inner: results, index: 0 }
}

pub fn getenv(key: &OsStr) -> Option<OsString> {
    safa_api::process::env::env_get(key.as_encoded_bytes())
        .map(|s| unsafe { OsString::from_encoded_bytes_unchecked(s.to_vec()) })
}

pub unsafe fn setenv(key: &OsStr, value: &OsStr) -> io::Result<()> {
    safa_api::process::env::env_set(key.as_encoded_bytes(), value.as_encoded_bytes());
    Ok(())
}

pub unsafe fn unsetenv(key: &OsStr) -> io::Result<()> {
    safa_api::process::env::env_remove(key.as_encoded_bytes());
    Ok(())
}
