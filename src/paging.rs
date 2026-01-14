use core::marker::PhantomData;

pub struct PageTableEntryFlags;

impl PageTableEntryFlags {
    pub const VALID: usize = 1 << 0;
    pub const READ: usize = 1 << 1;
    pub const WRITE: usize = 1 << 2;
    pub const EXECUTE: usize = 1 << 3;
    pub const USER: usize = 1 << 4;
    // ... other bits exits, but these are the important ones
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: usize, // PTE is just a 64-bit integer
}

impl PageTableEntry {
    pub fn new(entry: usize) -> Self {
        Self { entry }
    }

    pub fn is_valid(&self) -> bool{
        (self.entry & PageTableEntryFlags::VALID) != 0
    }

    pub fn is_readable(&self) -> bool{
        (self.entry & PageTableEntryFlags::READ) != 0
    }

    pub fn is_writeable(&self) -> bool{
        (self.entry & PageTableEntryFlags::WRITE) != 0
    }

    pub fn physical_address(&self) -> usize{
        (self.entry >> 10) << 12
    }
}


#[repr(align(4096))]
#[repr(C)]
pub struct PageTable{
    pub entries: [PageTableEntry; 512],
}

impl PageTable{
    pub fn new() -> Self{
     Self {
        entries: [PageTableEntry {entry: 0}; 512],
     }   
    }
}