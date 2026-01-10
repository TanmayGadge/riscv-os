const UART0: *mut u8 = 0x1000_0000 as *mut u8;

pub fn uart_putc(c: u8) {
    unsafe {
        core::ptr::write_volatile(UART0, c);
    }
}

pub fn uart_print(s: &str) {
    for byte in s.bytes() {
        uart_putc(byte);
    }
}
