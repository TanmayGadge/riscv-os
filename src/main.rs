#![no_std]
#![no_main]

mod uart;
mod pmm;
mod paging;

use core::panic::PanicInfo;
use core::arch::global_asm;

global_asm!(include_str!("entry.s"));

unsafe extern "C"{
    unsafe static _heap_start: u8;
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    uart::uart_print("Hello World!\n");


    let heap_start: usize = unsafe {core::ptr::addr_of!(_heap_start) as usize};

    let mut pmm = pmm::PhysicalMemoryManager::new(heap_start);

    let page1: usize = pmm.alloc_page();
    let page2: usize = pmm.alloc_page();
    let page3: usize = pmm.alloc_page();

    if page2 == page1 + 4096 {
        uart::uart_print("Memory Allocation Works!\n");
    } else {
        uart::uart_print("Memory Allocation Failed!\n");
    }


    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart::uart_print("KERNEL PANIC\n");
    loop {}
}
