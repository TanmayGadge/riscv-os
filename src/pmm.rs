pub struct PhysicalMemoryManager{
    next_free_page: usize,
}

impl PhysicalMemoryManager {
    pub const fn new(start: usize) -> Self {
        Self{
            next_free_page: start
        }
    }

    pub fn alloc_page(&mut self) -> usize{

        let address: usize = self.next_free_page;
        
        // The "Bump" Allocator
        self.next_free_page += 4096;
        address //No semicolon means return automatically
    }
}