use crate::uart;
use crate::TICK_COUNT;
use core::sync::atomic::Ordering;

#[unsafe(no_mangle)]
pub extern "C" fn trap_handler() -> (){


    let scause_value: usize;
    let sepc_value: usize;
    let stval_value: usize;

    unsafe{
        core::arch::asm!("csrr {}, scause ", out(reg) scause_value);
        core::arch::asm!("csrr {}, sepc", out(reg) sepc_value);
        core::arch::asm!("csrr {}, stval", out(reg) stval_value);
    }

    let is_interrupt: bool = (scause_value >> 63) & 1 == 1; 
    let cause_code: usize = scause_value & 0xfff; //cause code is stored in the low 12 bits

    if is_interrupt{
        
        match cause_code{
            5 => { 
                let mtime: usize;
                let mut mtimecmp: usize;

                TICK_COUNT.fetch_add(1, Ordering::Relaxed);

                let now: usize;
                unsafe{
                    core::arch::asm!("rdtime {}", out(reg) now); //rdtime = pseudo-instruction to read current timer
                }

                const TIMER_INTERVAL: usize = 1_000_000;
                let next: usize = now + TIMER_INTERVAL;

                unsafe {
                    core::arch::asm!(
                        // Non legacy calling convention
                        "ecall",
                        in("a0") next, // arg0: absolute time
                        in("a6") 0usize,
                        in("a7") 0x54494D45, 
                    );

                    
                }

                
            }

            _ => {}
        }

    }else{
        //Handle Exceptions (page faults, ecall, etc.)

        match cause_code{
            /*
            8: ecall from u-mode
            9: ecall from s-mode
            11: ecall from m-mode
            */

            9 => {
                let mut sepc_value: usize;
                unsafe{
                    core::arch::asm!("csrr {}, sepc", out(reg) sepc_value);
                    core::arch::asm!("csrw sepc, {}", in(reg) (sepc_value + 4));
                }

            }
           _ => {
                // If it's an unhandled exception, stop here to avoid infinite printing
                loop {}
            }
        }
    }   

    // uart::uart_print("\n--- TRAP DIAGNOSTIC ---\n");
    // uart::uart_print("SCAUSE: ");
    // uart::uart_print_hex(scause_value);

    // uart::uart_print("\nSEPC  : ");
    // uart::uart_print_hex(sepc_value);

    // uart::uart_print("\nSTVAL : ");
    // uart::uart_print_hex(stval_value);
    // uart::uart_print("\n-----------------------\n");

    
    // loop{}

}


