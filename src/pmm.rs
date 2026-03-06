use core::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::paging::{KERNEL_PAGE_TABLE, PageTable, PageTableEntryFlags};
use spin::{Mutex, MutexGuard};

struct FreePage {
    next: Option<*mut FreePage>,
}
pub struct PhysicalMemoryManager {
    next_free_page: usize,
    free_list: Option<*mut FreePage>,
    limit: usize,
}

impl PhysicalMemoryManager {
    pub const fn new(mut start: usize, limit: usize) -> Self {
        if start % 4096 != 0 {
            start += 4096 - (start % 4096);
        }

        Self {
            next_free_page: start,
            free_list: None,
            limit,
        }
    }

    pub fn alloc_page(&mut self) -> Option<usize> {
        if self.next_free_page + 4096 > self.limit {
            return None;
        }

        if let Some(page_ptr) = self.free_list {
            unsafe {
                let page: &mut FreePage = &mut *page_ptr;
                self.free_list = page.next;

                return Some(page_ptr as usize);
            }
        }

        let address: usize = self.next_free_page;
        // The "Bump" Allocator
        self.next_free_page += 4096;
        Some(address)
    }

    pub fn dealloc_page(&mut self, ptr: usize) {
        let page_ptr: *mut FreePage = ptr as *mut FreePage;
        unsafe {
            (*page_ptr).next = self.free_list;
            self.free_list = Some(page_ptr);
        }
    }
}

struct HeapAllocator {
    pmm: &'static Mutex<PhysicalMemoryManager>,
    next_va: AtomicUsize,
}

impl HeapAllocator{
    pub const fn new(pmm: &'static Mutex<PhysicalMemoryManager> ) -> Self {
        Self {
            pmm,
            next_va: AtomicUsize::new(0x4000_0000)
        }
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size: usize = layout.size();
        let pages_needed: usize = (size + 4095) / 4096;

        let start_va: usize = self
            .next_va
            .fetch_add(pages_needed * 4096, Ordering::SeqCst);

        let mut pmm_lock: MutexGuard<'_, PhysicalMemoryManager> = self.pmm.lock();
        let mut pt_lock: MutexGuard<'_, Option<&'static mut PageTable>> = KERNEL_PAGE_TABLE.lock();

        if let Some(ref mut root_table) = *pt_lock {
            for i in 0..pages_needed {
                let va: usize = start_va + (i * 4096); // implement VMM later

                if let Some(pa) = pmm_lock.alloc_page() {
                    root_table.map(&mut *pmm_lock, va, pa, PageTableEntryFlags::RWX);
                } else {
                    return core::ptr::null_mut();
                }
            }
        }
        unsafe {
            core::arch::asm!("sfence.vma"); // Flush TLB
        }

        start_va as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

unsafe impl Send for PhysicalMemoryManager {}
unsafe impl Sync for PhysicalMemoryManager {}

unsafe impl Send for HeapAllocator {}
unsafe impl Sync for HeapAllocator {}


pub static PMM: Mutex<PhysicalMemoryManager> = Mutex::new(PhysicalMemoryManager::new(0, 0));

#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator {
    pmm: &PMM,
    next_va: core::sync::atomic::AtomicUsize::new(0x4000_0000),
};

extern crate alloc;