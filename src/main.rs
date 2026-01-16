#![no_std]
#![no_main]

mod uart;
mod pmm;
mod paging;

use core::panic::PanicInfo;
use core::arch::global_asm;

use paging::{PageTable, PageTableEntryFlags};

use crate::paging::PageTableEntry;

global_asm!(include_str!("entry.s"));

unsafe extern "C"{
    unsafe static _heap_start: u8;
    unsafe static _start: u8;
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    uart::uart_print("Hello World!\n");


    let heap_start: usize = unsafe {core::ptr::addr_of!(_heap_start) as usize};
    let mut pmm: pmm::PhysicalMemoryManager = pmm::PhysicalMemoryManager::new(heap_start);

    let root_ptr: *mut PageTable = pmm.alloc_page().expect("OOM") as *mut PageTable;
    let root_table: &mut PageTable = unsafe{&mut *root_ptr};

    root_table.entries = [paging::PageTableEntry {entry: 0}; 512];

    uart::uart_print("Building Page Tables..\n");


    root_table.map(
        &mut pmm,
        0x1000_0000,
        0x1000_0000,
        PageTableEntryFlags::RWX
    );

    let kernel_start: usize = unsafe{core::ptr::addr_of!(_start) as usize};
    let mut addr: usize = kernel_start;

    //Identity mapping
    while addr < heap_start + 4096 * 100{
        root_table.map(
            &mut pmm,
            addr,
            addr,
            PageTableEntryFlags::RWX
        );
        addr += 4096;
    }

    uart::uart_print("Enabling MMU...\n");

    let root_ppn = (root_ptr as usize) >> 12;
    let satp_val = (8 << 60) | root_ppn;

    unsafe{
        core::arch::asm!("csrw satp, {}", in(reg) satp_val);
        core::arch::asm!("sfence.vma");
    }


    uart::uart_print("MMU Enabled! We are still alive!\n");

    // let page1: usize = pmm.alloc_page();
    // let page2: usize = pmm.alloc_page();
    // let page3: usize = pmm.alloc_page();

    // if page2 == page1 + 4096 {
    //     uart::uart_print("Memory Allocation Works!\n");
    // } else {
    //     uart::uart_print("Memory Allocation Failed!\n");
    // }


    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart::uart_print("KERNEL PANIC\n");
    loop {}
}
