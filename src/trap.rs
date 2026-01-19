#[unsafe(no_mangle)]
// #[unsafe(naked)]



pub extern "C" fn trap_handler() -> usize{

    let scause_value: usize;

    unsafe{
        core::arch::asm!("csrr {}, scause ", out(reg) scause_value);
    }

    let is_interrupt: usize = (scause_value >> 62) & 1; //1 = interrupt, 0 = exception
    let cause_code: usize = (scause_value << 1) >> 1; 

    if is_interrupt == 1{
        
        match cause_code{
            5 => { 
                let mtime: usize;
                let mut mtimecmp: usize;

                let now: usize;
                unsafe{
                    core::arch::asm!("rdtime {}", out(reg) now); //rdtime = pseudo-instruction
                }

                const TIMER_INTERVAL: usize = 100_000;
                let next: usize = now + TIMER_INTERVAL;

                unsafe {
                    core::arch::asm!(
                        "ecall",
                        in("a0") next, // arg0: absolute time
                        in("a1") 0usize,
                        in("a2") 0usize,
                        in("a7") 0usize, // SBI_SET_TIMER = 0
                    );
                }

                // unsafe{
                //     core::arch::asm!("csrr {} time", out(reg) mtime); //mtime in M-mode = time in S-Mode CSR
                //     core::arch::asm!("csrr {} mtimecmp", out(reg) mtimecmp);
    
                //     // core::arch::asm!(
                //     //     "csrr t0, mtime",
                //     //     "csrr t1, mtimecmp",
                //     //     ""
                //     // )
                // }
    
                // if mtime >= mtimecmp{
                //     mtimecmp += mtime;
                // }
    
                // unsafe{
                //     core::arch::asm!("csrw mtimecmp {}", in(reg) mtimecmp);
                // }
            }
        }

    }else{
        //Handle Exceptions (page faults, ecall, etc.)
    }        

    0
}
