#![no_std]
#![no_main]

mod paging;
mod pmm;
mod trap;
mod uart;

use core::arch::global_asm;
use core::panic::PanicInfo;

use paging::{PageTable, PageTableEntryFlags};
use core::sync::atomic::{AtomicUsize, Ordering};

pub static TICK_COUNT: AtomicUsize = AtomicUsize::new(0);


global_asm!(include_str!("entry.s"));

unsafe extern "C" {
    unsafe static _heap_start: u8;
    unsafe static _start: u8;
    unsafe fn trap_vector();
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    uart::uart_print("Hello World!\n");

    let heap_start: usize = unsafe { core::ptr::addr_of!(_heap_start) as usize };
    let mut pmm: pmm::PhysicalMemoryManager = pmm::PhysicalMemoryManager::new(heap_start);

    let root_ptr: *mut PageTable = pmm.alloc_page().expect("OOM") as *mut PageTable;
    let root_table: &mut PageTable = unsafe { &mut *root_ptr };

    root_table.entries = [paging::PageTableEntry { entry: 0 }; 512];

    uart::uart_print("Building Page Tables..\n");

    root_table.map(&mut pmm, 0x1000_0000, 0x1000_0000, PageTableEntryFlags::RWX);

    let kernel_start: usize = unsafe { core::ptr::addr_of!(_start) as usize };
    let mut addr: usize = kernel_start;

    //Identity mapping
    while addr < heap_start + 4096 * 100 {
        root_table.map(&mut pmm, addr, addr, PageTableEntryFlags::RWX);
        addr += 4096;
    }

    uart::uart_print("Page Tables Built!\n");

    uart::uart_print("Enabling Trap Handling...\n");

    
    let trap_addr: usize = trap_vector as *const() as usize; // *const() is a raw pointer
    

    assert!(trap_addr % 4 == 0, "Trap handler must be 4-byte aligned!");
 
    
    unsafe {
        core::arch::asm!("csrw stvec, {}", in(reg) trap_addr);

        uart::uart_print("stvec initalised!\n");
        
        core::arch::asm!("csrrsi x0, sstatus, 2", options(nostack, nomem)); //set SIE (bit 1) as 1
        
        uart::uart_print("sstatus initalised!\n");
        
        core::arch::asm!(
            "li t0, 32",
            "csrrs x0, sie, t0",
            options(nostack, nomem)
        ); //set STIE (bit 5) as 1

        uart::uart_print("STIE initialed\n");

    }

    uart::uart_print("Trap Handling Enabled!\n");

    uart::uart_print("Enabling MMU...\n");

    let root_ppn: usize = (root_ptr as usize) >> 12;
    let satp_val: usize = (8 << 60) | root_ppn; //for Sv39, mode = 8

    unsafe {
        core::arch::asm!("csrw satp, {}", in(reg) satp_val);
        core::arch::asm!("sfence.vma"); //Clear TLB
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

    let mut last_tick: usize = 0;
    loop {
        let current_tick: usize = TICK_COUNT.load(Ordering::Relaxed);
        if current_tick != last_tick {
            uart::uart_print("Tick: ");
            uart::uart_print_hex(current_tick);
            uart::uart_print("!\n");
            last_tick = current_tick;
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart::uart_print("KERNEL PANIC\n");
    loop {}
}
