use safa_api::alloc::GLOBAL_SYSTEM_ALLOCATOR;

use crate::alloc::{GlobalAlloc, Layout, System};

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let results = unsafe { GLOBAL_SYSTEM_ALLOCATOR.alloc(layout) };
        assert!(
            (results as usize).is_multiple_of(layout.align()),
            "SYSTEM ALLLOCATOR ALLOCATED UNALIGNED: {:?}, layout: {:?}",
            results,
            layout
        );
        results
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            GLOBAL_SYSTEM_ALLOCATOR.dealloc(ptr, layout);
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { GLOBAL_SYSTEM_ALLOCATOR.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { GLOBAL_SYSTEM_ALLOCATOR.realloc(ptr, layout, new_size) }
    }
}
