use safa_api::alloc::GLOBAL_SYSTEM_ALLOCATOR;

use crate::ptr::NonNull;
use crate::{
    alloc::{GlobalAlloc, Layout, System},
    ptr,
};

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match GLOBAL_SYSTEM_ALLOCATOR.allocate(layout.size()) {
            Some(data) => data.as_ptr() as *mut u8,
            None => ptr::null_mut(),
        }
    }
    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        unsafe { GLOBAL_SYSTEM_ALLOCATOR.deallocate(NonNull::new_unchecked(ptr)) }
    }
}
