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
    unsafe static _bss_end: u8;
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    uart::uart_print("Booting Kernel...\n");

    let heap_start: usize =  core::ptr::addr_of!(_heap_start) as usize;
    const RAM_END: usize = 0x88000000;
    let mut pmm: pmm::PhysicalMemoryManager = pmm::PhysicalMemoryManager::new(heap_start, RAM_END);

    let root_ptr: *mut PageTable = pmm.alloc_page().expect("OOM") as *mut PageTable;
    let root_table: &mut PageTable = unsafe { &mut *root_ptr };

    root_table.entries = [paging::PageTableEntry { entry: 0 }; 512];

    uart::uart_print("Building Page Tables..\n");

    root_table.map(&mut pmm, 0x1000_0000, 0x1000_0000, PageTableEntryFlags::RWX);

    let kernel_start: usize =  core::ptr::addr_of!(_start) as usize ;
    let bss_end: usize =  core::ptr::addr_of!(_bss_end) as usize;

    let mut addr: usize = kernel_start;
    
    uart::uart_print("\n");
    uart::uart_print("Kernel start address: ");
    uart::uart_print_hex(kernel_start);
    uart::uart_print("\n");

    uart::uart_print("BSS End Address: ");
    uart::uart_print_hex(bss_end);
    uart::uart_print("\n");

    uart::uart_print("Heap Start Address: ");
    uart::uart_print_hex(heap_start);
    uart::uart_print("\n");

    uart::uart_print("\n");

    //Identity mapping
    while addr <= bss_end {
        root_table.map(&mut pmm, addr, addr, PageTableEntryFlags::RWX);
        addr += 4096;
    }

    uart::uart_print("Page Tables Built!\n");

    uart::uart_print("Enabling MMU...\n");

    let root_ppn: usize = (root_ptr as usize) >> 12;
    let satp_val: usize = (8 << 60) | root_ppn; //for Sv39, mode = 8

    unsafe {
        core::arch::asm!("csrw satp, {}", in(reg) satp_val);
        core::arch::asm!("sfence.vma"); //Clear TLB
    }

    uart::uart_print("MMU Enabled! We are still alive!\n");

    uart::uart_print("Enabling Trap Handling...\n");

    
    let trap_addr: usize = trap_vector as *const() as usize; // *const() is a raw pointer
    

    assert!(trap_addr % 4 == 0, "Trap handler must be 4-byte aligned!");
 
    
    unsafe {
        let now: usize;
        core::arch::asm!("rdtime {}", out(reg) now);
        
        let next: usize = now + 10_000_000;
        // core::arch::asm!(
        //     "ecall",
        //     in("a0") next,
        //     in("a6") 0usize,
        //     in("a7") 0x54494D45usize, 
        //     lateout("a0") _,
        //     lateout("a1") _,
        // );

        core::arch::asm!("csrw stimecmp, {}", in(reg) next);

        core::arch::asm!("csrw stvec, {}", in(reg) trap_addr); 

        uart::uart_print("stvec initalised!\n");
        
        core::arch::asm!("csrrsi x0, sstatus, 2", options(nostack, nomem)); //set SIE (bit 1) as 1 (set immediate)
        
        uart::uart_print("sstatus initalised!\n");
        
        core::arch::asm!(
            "csrrs x0, sie, {}",
            in(reg) 32usize,
            options(nostack, nomem)
        ); //set STIE (bit 5) as 1

        uart::uart_print("STIE initialed\n");

    }

    uart::uart_print("Trap Handling Enabled!\n");

    uart::uart_print("Calling ebreak...\n\n");
    unsafe{ core::arch::asm!("ebreak"); }
    uart::uart_print("ebreak resolved!\n\n");


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
