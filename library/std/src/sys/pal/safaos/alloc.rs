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
    next: Option<&'static mut Block>,
    data_len: usize,
}

impl Block {
    /// Asks the system for a new memory Block with a size big enough to hold `data_len` bytes
    pub fn create(data_len: usize) -> Option<&'static mut Self> {
        let size = data_len + size_of::<Block>();
        let size = size.div_ceil(align_of::<Block>()) * align_of::<Block>();
        assert!(size <= isize::MAX as usize);

        let ptr = get_data_break() as *mut Block;
        syscalls::syssbrk(size as isize).ok()?;
        unsafe {
            *ptr = Self { free: true, data_len: size - size_of::<Block>(), ..Default::default() };
            Some(&mut *ptr)
        }
    }

    /// Gets the Block metadata of a data ptr,
    /// unsafe because the pointer had to be made by calling `[Block::data_from_ptr]` on a vaild pointer, otherwise the returned value is invaild
    pub unsafe fn block_from_data_ptr(data: NonNull<u8>) -> NonNull<Self> {
        unsafe { NonNull::new_unchecked((data.as_ptr() as *mut Block).offset(-1)) }
    }

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
    head: Option<&'static mut Block>,
}

fn get_data_break() -> *mut u8 {
    syscalls::syssbrk(0).unwrap()
}

impl SystemAllocator {
    const fn new() -> Self {
        Self { head: None }
    }

    fn find_block(&self, data_len: usize) -> Option<*mut Block> {
        let mut current = &self.head;
        while let Some(block) = current {
            if block.data_len <= data_len && block.free {
                return Some(block as *const _ as *mut _);
            }

            current = &block.next;
        }

        None
    }

    pub fn allocate(&mut self, size: usize) -> Option<NonNull<[u8]>> {
        if let Some(block) = self.find_block(size) {
            unsafe {
                (*block).free = false;
                Some(Block::data_from_ptr(block))
            }
        } else {
            let new_block = Block::create(size)?;
            new_block.free = false;
            let data = unsafe { Block::data_from_ptr(new_block as *const _) };

            let stolen_head = self.head.take();

            new_block.next = stolen_head;
            self.head = Some(new_block);

            Some(data)
        }
    }

    pub fn deallocate(&mut self, block_data: NonNull<u8>) {
        unsafe {
            let block_ptr = Block::block_from_data_ptr(block_data);
            (*block_ptr.as_ptr()).free = true;
        }
    }
}

#[unstable(feature = "allocator_api", issue = "32838")]
unsafe impl Allocator for Mutex<SystemAllocator> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        self.lock().unwrap().allocate(layout.size()).ok_or(core::alloc::AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _: Layout) {
        self.lock().unwrap().deallocate(ptr)
    }
}

static GLOBAL_SYSTEM_ALLOCATOR: Mutex<SystemAllocator> = Mutex::new(SystemAllocator::new());

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match GLOBAL_SYSTEM_ALLOCATOR.allocate(layout) {
            Ok(data) => data.as_ptr() as *mut u8,
            Err(core::alloc::AllocError) => ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { GLOBAL_SYSTEM_ALLOCATOR.deallocate(NonNull::new_unchecked(ptr), layout) }
    }
}
