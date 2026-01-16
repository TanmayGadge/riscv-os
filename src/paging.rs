use crate::pmm::PhysicalMemoryManager;
use core::marker::PhantomData;

pub struct PageTableEntryFlags;

impl PageTableEntryFlags {
    pub const VALID: usize = 1 << 0;
    pub const READ: usize = 1 << 1;
    pub const WRITE: usize = 1 << 2;
    pub const EXECUTE: usize = 1 << 3;
    pub const USER: usize = 1 << 4;

    pub const RWX: usize = Self::READ | Self::WRITE | Self::EXECUTE | Self::VALID; //Read/Write/Execute
    // ... other bits exits, but these are the important ones
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PageTableEntry {
   pub entry: usize, // PTE is just a 64-bit integer
}

impl PageTableEntry {
    pub fn new(entry: usize) -> Self {
        Self { entry }
    }

    pub fn is_valid(&self) -> bool {
        (self.entry & PageTableEntryFlags::VALID) != 0
    }

    pub fn is_readable(&self) -> bool {
        (self.entry & PageTableEntryFlags::READ) != 0
    }

    pub fn is_writeable(&self) -> bool {
        (self.entry & PageTableEntryFlags::WRITE) != 0
    }

    pub fn physical_address(&self) -> usize {
        (self.entry >> 10) << 12
    }
}

#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn new() -> Self {
        Self {
            entries: [PageTableEntry { entry: 0 }; 512],
        }
    }


    pub fn next_table_create(
        &mut self, 
        index: usize, 
        allocator: &mut PhysicalMemoryManager)
        -> Option<&mut PageTable> {

        let entry: &mut PageTableEntry = &mut self.entries[index];

        if entry.is_valid() {
            let table_ptr: *mut PageTable = entry.physical_address() as *mut PageTable;
            unsafe { Some(&mut *table_ptr) }

        }else{
            let new_page_addr = allocator.alloc_page()?;
            let new_table: &mut PageTable = unsafe{ &mut *(new_page_addr as *mut PageTable)};

            new_table.entries = [PageTableEntry {entry: 0}; 512];
            let flags: usize = PageTableEntryFlags::VALID;
            let pfn: usize = (new_page_addr >> 12) << 10;
            entry.entry = pfn | flags;

            Some(new_table)
        }
    }

    pub fn map(
        &mut self,
        allocator: &mut PhysicalMemoryManager,
        va: usize,
        pa: usize,
        flags: usize,
    ) {
        let vpn2: usize = (va >> 30) & 0x1FF;
        let vpn1: usize = (va >> 21) & 0x1FF;
        let vpn0: usize = (va >> 12) & 0x1FF;

        //  Get Level 1 table.
        let table1: &mut PageTable = self
            .next_table_create(vpn2, allocator)
            .expect("Failed to allocate Level 1 Table");

        // From Level 1, get Level 0 table.
        let table0: &mut PageTable = table1
            .next_table_create(vpn1, allocator)
            .expect("Failed to allocate Level 0 Table");

        // 3. Write the Final Entry (Level 0)
        let entry = &mut table0.entries[vpn0];

        // Format: PPN | Flags | Valid
        let pfn: usize = (pa >> 12) << 10;
        table0.entries[vpn0].entry = pfn | flags | PageTableEntryFlags::VALID;
    }
}
