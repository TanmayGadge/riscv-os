const UART0: *mut u8 = 0x1000_0000 as *mut u8;
pub const KERNEL_OFFSET: usize = 0xFFFFFFFF00000000;
pub const UART_VIRT_ADDR: usize = 0x1000_0000 + KERNEL_OFFSET;

pub fn uart_putc(c: u8) {
    unsafe {
        let mut satp: usize;
        unsafe { core::arch::asm!("csrr {}, satp", out(reg) satp); }
        // If mode bit (top bit) is 0, MMU is off, use physical addr
        let addr = if satp == 0 { 0x1000_0000 } else { UART_VIRT_ADDR };
        core::ptr::write_volatile(addr as *mut u8, c);
    }
}

pub fn uart_print(s: &str) {
    for byte in s.bytes() {
        uart_putc(byte);
    }
}


pub fn uart_print_hex(val: usize) {
    let chars: &[u8; 16] = b"0123456789ABCDEF";
    uart_print("0x");
    for i in (0..16).rev() {
        let nibble: usize = (val >> (i * 4)) & 0xF;
        let c: char = chars[nibble] as char;
        let mut buf: [u8; 1] = [0u8; 1];
        c.encode_utf8(&mut buf);
        uart_print(core::str::from_utf8(&buf).unwrap());
    }
}

pub fn uart_print_phys(s: &str) {
    // Manually subtract the offset from the string's pointer
    let phys_ptr = (s.as_ptr() as usize).wrapping_sub(KERNEL_OFFSET) as *const u8;
    for i in 0..s.len() {
        unsafe {
            // Use the physical pointer to read the byte
            let byte = *phys_ptr.add(i);
            uart_putc(byte);
        }
    }
}

pub fn uart_putc_phys(c: u8) {
    unsafe {
        core::ptr::write_volatile(0x1000_0000 as *mut u8, c);
    }
}

// Safe to call before MMU is on — takes a literal byte slice
pub fn uart_print_phys_bytes(bytes: &[u8]) {
    for &b in bytes {
        uart_putc_phys(b);
    }
}


//Debug macro
#[macro_export]
macro_rules! kdbg {
    // Numeric/pointer values — caller passes something castable to usize
    ($val:expr) => {{
        $crate::uart::uart_print("[DBG] ");
        $crate::uart::uart_print(file!());
        $crate::uart::uart_print(":");
        $crate::uart::uart_print_hex(line!() as usize);
        $crate::uart::uart_print(" | ");
        $crate::uart::uart_print(stringify!($val));
        $crate::uart::uart_print(" = ");
        $crate::uart::uart_print_hex({
            let v = $val;
            v as usize
        });
        $crate::uart::uart_print("\n");
    }};
    // String messages — just print the message, no hex
    ($msg:literal, str) => {{
        $crate::uart::uart_print("[DBG] ");
        $crate::uart::uart_print(file!());
        $crate::uart::uart_print(":");
        $crate::uart::uart_print_hex(line!() as usize);
        $crate::uart::uart_print(" | ");
        $crate::uart::uart_print($msg);
        $crate::uart::uart_print("\n");
    }};
}