#![no_std]
#![no_main]

mod uart;

use core::panic::PanicInfo;
use core::arch::global_asm;

global_asm!(include_str!("entry.s"));

/// Our kernel entry function.
/// This will be called from entry.S.
#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {

    uart::uart_print("Hello World!\n");

    loop {}
}

/// Minimal panic handler.
/// If something panics, just halt forever (spin loop).
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
