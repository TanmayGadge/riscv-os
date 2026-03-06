struct FreePage {
    next: Option<*mut FreePage>,
}
pub struct PhysicalMemoryManager {
    next_free_page: usize,
    free_list: Option<*mut FreePage>,
    limit: usize
}

impl PhysicalMemoryManager {
    pub const fn new(mut start: usize, limit: usize) -> Self {

        if start % 4096 != 0{
            start += 4096 - (start % 4096);
        }

        Self {
            next_free_page: start,
            free_list: None,
            limit
        }
    }

    pub fn alloc_page(&mut self) -> Option<usize> {

        if self.next_free_page + 4096 > self.limit{
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


