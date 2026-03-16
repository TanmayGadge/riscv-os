use crate::pmm::PhysicalMemoryManager;
use core::marker::PhantomData;

use crate::uart;
use spin::{Mutex, MutexGuard};

#[derive(Clone)]
pub struct PageTableEntryFlags;

impl PageTableEntryFlags {
    pub const VALID: usize = 1 << 0;
    pub const READ: usize = 1 << 1;
    pub const WRITE: usize = 1 << 2;
    pub const EXECUTE: usize = 1 << 3;
    pub const USER: usize = 1 << 4;

    pub const ACCESS: usize = 1 << 6;
    pub const DIRTY: usize = 1 << 7;
    pub const RWX: usize =
        Self::READ | Self::WRITE | Self::EXECUTE | Self::VALID | Self::ACCESS | Self::DIRTY;
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

    pub fn is_executable(&self) -> bool {
        (self.entry & PageTableEntryFlags::EXECUTE) != 0
    }

    pub fn physical_address(&self) -> usize {
        const MASK: usize = (1usize << 44) - 1;
        let ppn: usize = (self.entry >> 10) & MASK;
        ppn << 12
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
        allocator: &mut PhysicalMemoryManager,
    ) -> Option<&mut PageTable> {
        let entry: &mut PageTableEntry = &mut self.entries[index];

        if entry.is_valid() {
            let table_ptr: *mut PageTable = entry.physical_address() as *mut PageTable;
            unsafe { Some(&mut *table_ptr) }
        } else {
            let new_page_addr: usize = allocator.alloc_page()?;
            let new_table: &mut PageTable = unsafe { &mut *(new_page_addr as *mut PageTable) };

            new_table.entries = [PageTableEntry { entry: 0 }; 512];
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
        // Format: PPN | Flags | Valid
        let ppn: usize = (pa >> 12) << 10;
        table0.entries[vpn0].entry = ppn | flags | PageTableEntryFlags::VALID;

        uart::uart_print("Page table entry value: ");
        uart::uart_print_hex(table0.entries[vpn0].entry);
        uart::uart_print("\n");
    }

    pub fn walk(&mut self, va: usize) -> Option<usize> {
        let vpn2: usize = (va >> 30) & 0x1FF;
        let vpn1: usize = (va >> 21) & 0x1FF;
        let vpn0: usize = (va >> 12) & 0x1FF;

        let offset: usize = va & 0xFFF;

        let l2_entry: PageTableEntry = self.entries[vpn2];
        let mut ppn: usize = PageTableEntry::physical_address(&l2_entry);

        if !PageTableEntry::is_valid(&l2_entry) {
            return None;
        }

        if PageTableEntry::is_readable(&l2_entry)
            | PageTableEntry::is_writeable(&l2_entry)
            | PageTableEntry::is_executable(&l2_entry)
        {
            return Some(ppn | offset);
        } else {
            let l1_table: *mut PageTable = ppn as *mut PageTable;
            let l1_entry: PageTableEntry = unsafe { (*l1_table).entries[vpn1] };

            if !PageTableEntry::is_valid(&l1_entry) {
                return None;
            }

            if PageTableEntry::is_readable(&l1_entry)
                | PageTableEntry::is_writeable(&l1_entry)
                | PageTableEntry::is_executable(&l1_entry)
            {
                ppn = PageTableEntry::physical_address(&l1_entry);
                return Some(ppn | offset);
            } else {
                let l0_table: *mut PageTable = ppn as *mut PageTable;
                let l0_entry: PageTableEntry = unsafe { (*l0_table).entries[vpn0] };

                if !PageTableEntry::is_valid(&l0_entry){
                    return None;
                }

                if PageTableEntry::is_readable(&l0_entry)
                    | PageTableEntry::is_writeable(&l0_entry)
                    | PageTableEntry::is_executable(&l0_entry)
                {
                    ppn = PageTableEntry::physical_address(&l0_entry);
                    return Some(ppn | offset);
                }else{
                    uart::uart_print("Page Fault!: The Virtual Address is not mapped to any Physical Address.");
                }
            }
        }
        None //Replace identity mapping with offset mapping
    }

}

pub static KERNEL_PAGE_TABLE: Mutex<Option<&'static mut PageTable>> = Mutex::new(None);
