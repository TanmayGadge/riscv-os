use crate::TICK_COUNT;
use crate::uart;

use core::sync::atomic::Ordering;

#[repr(C)]
pub struct TrapContext {
    pub ra: usize,         // offset 0
    pub sp: usize,         // offset 8 (The original SP we saved)
    pub gp: usize,         // offset 16
    pub tp: usize,         // offset 24
    pub regs: [usize; 27], // t0-t6, a0-a7, s0-s11 (offsets 32-240)
}

#[unsafe(no_mangle)]
pub extern "C" fn trap_handler(ctx: &TrapContext) -> () {
    uart::uart_print("\n--- STACK FRAME TRACE ---\n");

    uart::uart_print("RA: ");
    uart::uart_print_hex(ctx.ra);

    uart::uart_print(" | SP: ");
    uart::uart_print_hex(ctx.sp);

    // Print t0 (which is regs[0] if your 'sd t0, 32(sp)' is first in the array)
    uart::uart_print("\nT0: ");
    uart::uart_print_hex(ctx.regs[0]);

    uart::uart_print("\n-------------------------\n");

    uart::uart_print("TRAP ENTERED\n");

    let scause_value: usize;
    let mut sepc_value: usize;
    let stval_value: usize;

    uart::uart_print("reading CSRs...\n");
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) scause_value);
        uart::uart_print("scause read\n");
        core::arch::asm!("csrr {}, sepc",   out(reg) sepc_value);
        uart::uart_print("sepc read\n");
        core::arch::asm!("csrr {}, stval",  out(reg) stval_value);
        uart::uart_print("stval read\n");
    }

    uart::uart_print("checking interrupt bit...\n");
    let is_interrupt: bool = (scause_value >> 63) & 1 == 1;

    uart::uart_print("checking cause code...\n");
    let cause_code: usize = scause_value & 0xfff;

    uart::uart_print("cause_code: ");
    uart::uart_print_hex(cause_code);
    uart::uart_print("\n");
    uart::uart_print("is_interrupt: ");
    uart::uart_print(if is_interrupt { "true\n" } else { "false\n" });

    uart::uart_print("SEPC: ");
    uart::uart_print_hex(sepc_value);
    uart::uart_print("\n");

    let is_interrupt: bool = (scause_value >> 63) & 1 == 1;
    let cause_code: usize = scause_value & 0xfff; //cause code is stored in the low 12 bits

    if is_interrupt {
        match cause_code {
            5 => {
                uart::uart_print("TIMER TRAP\n");
                uart::uart_print("SEPC: ");
                uart::uart_print_hex(sepc_value);
                uart::uart_print("\n");

                TICK_COUNT.fetch_add(1, Ordering::Relaxed);

                let now: usize;
                unsafe {
                    core::arch::asm!("rdtime {}", out(reg) now); //rdtime = pseudo-instruction to read current timer (mtime)
                }

                const TIMER_INTERVAL: usize = 10_000_000; //1 second at 10MHz
                let next: usize = now + TIMER_INTERVAL;

                unsafe {
                    // core::arch::asm!(
                    //     "ecall",
                    //     in("a0") next, // arg0: absolute time
                    //     in("a6") 0usize, // Function: set_timer (update mtimecmp)
                    //     in("a7") 0x54494D45usize, // Extension: TIME
                    //     lateout("a0") _, // stores error value after call
                    //     lateout("a1") _, // stores return value after call
                    // );

                    core::arch::asm!("csrw stimecmp, {}", in(reg) next);
                }
            }

            _ => {}
        }
    } else {
        //Handle Exceptions (page faults, ecall, etc.)

        match cause_code {
            /*
            8: ecall from u-mode
            9: ecall from s-mode
            11: ecall from m-mode
            */
            9 => {
                let mut sepc_value: usize;
                unsafe {
                    core::arch::asm!("csrr {}, sepc", out(reg) sepc_value);
                    core::arch::asm!("csrw sepc, {}", in(reg) (sepc_value + 4));
                }
            }
            3 => {
                uart::uart_print("BREAKPOINT HIT - trap handler works!\n");
                unsafe {
                    // let sepc: usize;
                    // core::arch::asm!("csrr {}, sepc", out(reg) sepc);

                    // let instruction: u16 = core::ptr::read_volatile(sepc as *const u16);
                    // let step: usize = if (instruction & 0b11) != 0b11 { 2 } else { 4 };

                    // core::arch::asm!("csrw sepc, {}", in(reg) (sepc + step));
                    uart::uart_print("Advancing SEPC...\n");
                    advance_sepc();
                    core::arch::asm!("csrr {}, sepc", out(reg) sepc_value);

                    uart::uart_print("SEPC advancded to: ");
                    uart::uart_print_hex(sepc_value);
                    uart::uart_print("\n");
                }
            }
            _ => loop {},
        }
    }

    uart::uart_print("\n--- TRAP DIAGNOSTIC ---\n");
    uart::uart_print("SCAUSE: ");
    uart::uart_print_hex(scause_value);

    uart::uart_print("\nSEPC  : ");
    uart::uart_print_hex(sepc_value);

    uart::uart_print("\nSTVAL : ");
    uart::uart_print_hex(stval_value);
    uart::uart_print("\n-----------------------\n");

    // loop{}
}

pub fn advance_sepc() {
    unsafe {
        let sepc: usize;
        core::arch::asm!("csrr {}, sepc", out(reg) sepc);
        let instruction: u16 = core::ptr::read_volatile(sepc as *const u16);
        let step: usize = if (instruction & 0b11) != 0b11 { 2 } else { 4 };
        core::arch::asm!("csrw sepc, {}", in(reg) (sepc + step));
    }
}
