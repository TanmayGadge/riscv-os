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


pub fn uart_print_hex(val: usize) {
    let chars: &[u8; 16] = b"0123456789ABCDEF";
    uart_print("0x");
    for i in (0..16).rev() {
        let nibble = (val >> (i * 4)) & 0xF;
        let c = chars[nibble] as char;
        let mut buf = [0u8; 1];
        c.encode_utf8(&mut buf);
        uart_print(core::str::from_utf8(&buf).unwrap());
    }
}