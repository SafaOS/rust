use super::syscalls;
use crate::ptr::NonNull;
use crate::sync::Mutex;
use crate::{
    alloc::{GlobalAlloc, Layout, System},
    ptr,
};
use core::alloc::Allocator;

#[derive(Debug, Default)]
struct Block {
    free: bool,
    next: Option<NonNull<Block>>,
    data_len: usize,
}

impl Block {
    /// Asks the system for a new memory Block with a size big enough to hold `data_len` bytes
    pub fn create(data_len: usize) -> Option<NonNull<Self>> {
        let size = data_len + size_of::<Block>();
        let size = size.next_multiple_of(align_of::<Block>());
        assert!(size <= isize::MAX as usize);

        let ptr = get_data_break() as *mut Block;
        syscalls::sbrk(size as isize).ok()?;
        unsafe {
            *ptr = Self { free: true, data_len: size - size_of::<Block>(), ..Default::default() };
            Some(NonNull::new_unchecked(ptr))
        }
    }

    #[inline(always)]
    /// Gets the Block metadata of a data ptr,
    /// unsafe because the pointer had to be made by calling `[Block::data_from_ptr]` on a vaild pointer, otherwise the returned value is invaild
    pub unsafe fn block_from_data_ptr(data: NonNull<u8>) -> NonNull<Self> {
        unsafe { NonNull::new_unchecked((data.as_ptr() as *mut Block).offset(-1)) }
    }

    #[inline(always)]
    /// Gets the data ptr from a pointer to Block
    pub unsafe fn data_from_ptr(ptr: *const Self) -> NonNull<[u8]> {
        unsafe {
            let length = (*ptr).data_len;
            let ptr_to_data = ptr.offset(1) as *const u8 as *mut u8;
            NonNull::new_unchecked(core::slice::from_raw_parts_mut(ptr_to_data, length) as *mut [u8])
        }
    }
}

struct SystemAllocator {
    head: Option<NonNull<Block>>,
}

fn get_data_break() -> *mut u8 {
    // Should never fail
    unsafe { syscalls::sbrk(0).unwrap_unchecked() }
}

impl SystemAllocator {
    const fn new() -> Self {
        Self { head: None }
    }

    /// tries to find a block with enough space for `data_len` bytes
    #[inline]
    fn try_find_block(&self, data_len: usize) -> Option<NonNull<Block>> {
        // To optimize the search for exact size we have to manipulate the data_len a bit
        let size = data_len + size_of::<Block>();
        let size = size.next_multiple_of(align_of::<Block>());
        let data_len = size - size_of::<Block>();

        let mut current = self.head;
        let mut best_block: Option<(NonNull<Block>, usize)> = None;

        while let Some(block_ptr) = current {
            let block = unsafe { &*block_ptr.as_ptr() };
            if !block.free {
                current = block.next;
                continue;
            }

            if block.data_len == data_len {
                return Some(block_ptr);
            }

            if block.data_len > data_len
                && best_block.is_none_or(|(_, bb_len)| bb_len > block.data_len)
            {
                best_block = Some((block_ptr, block.data_len));
            }

            current = block.next;
        }

        best_block.map(|(ptr, _)| ptr)
    }

    /// finds a block with enough space for `data_len` bytes
    /// or creates a new one if there is no enough space
    #[inline]
    fn find_block(&mut self, data_len: usize) -> Option<NonNull<Block>> {
        if let Some(block) = self.try_find_block(data_len) {
            Some(block)
        } else {
            unsafe {
                let new_block = Block::create(data_len)?;
                let stolen_head = self.head.take();

                (*new_block.as_ptr()).next = stolen_head;
                self.head = Some(new_block);

                Some(new_block)
            }
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<NonNull<[u8]>> {
        let block = self.find_block(size)?;
        unsafe {
            let ptr = block.as_ptr();
            (*ptr).free = false;
            Some(Block::data_from_ptr(ptr))
        }
    }

    pub fn deallocate(&mut self, block_data: NonNull<u8>) {
        unsafe {
            let block_ptr = Block::block_from_data_ptr(block_data).as_ptr();
            let block = &mut *block_ptr;
            block.free = true;
        }
    }
}

unsafe impl Send for SystemAllocator {}
unsafe impl Sync for SystemAllocator {}

#[unstable(feature = "allocator_api", issue = "32838")]
unsafe impl Allocator for Mutex<SystemAllocator> {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        self.lock().unwrap().allocate(layout.size()).ok_or(core::alloc::AllocError)
    }
    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, _: Layout) {
        self.lock().unwrap().deallocate(ptr)
    }
}

static GLOBAL_SYSTEM_ALLOCATOR: Mutex<SystemAllocator> = Mutex::new(SystemAllocator::new());

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match GLOBAL_SYSTEM_ALLOCATOR.allocate(layout) {
            Ok(data) => data.as_ptr() as *mut u8,
            Err(core::alloc::AllocError) => ptr::null_mut(),
        }
    }
    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { GLOBAL_SYSTEM_ALLOCATOR.deallocate(NonNull::new_unchecked(ptr), layout) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        match GLOBAL_SYSTEM_ALLOCATOR.allocate_zeroed(layout) {
            Ok(data) => data.as_ptr() as *mut u8,
            Err(core::alloc::AllocError) => ptr::null_mut(),
        }
    }
    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe {
            let ptr = NonNull::new_unchecked(ptr);
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            match GLOBAL_SYSTEM_ALLOCATOR.grow(ptr, layout, new_layout) {
                Ok(data) => data.as_ptr() as *mut u8,
                Err(core::alloc::AllocError) => ptr::null_mut(),
            }
        }
    }
}
