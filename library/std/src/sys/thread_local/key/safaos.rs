pub type Key = usize;
pub type Dtor = unsafe extern "C" fn(*mut u8);

#[inline]
pub unsafe fn set(_key: Key, _value: *mut u8) {
    todo!()
}

#[inline]
pub unsafe fn get(_key: Key) -> *mut u8 {
    todo!()
}

#[inline]
pub fn create(_dtor: Option<Dtor>) -> Key {
    todo!()
}

#[inline]
pub unsafe fn destroy(_key: Key) {
    todo!()
}
