use spin::Mutex;
use uart_16550::SerialPort;

static SERIAL1: Mutex<Option<SerialPort>> = Mutex::new(None);

pub fn init() {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    *SERIAL1.lock() = Some(serial_port);
}

pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    if let Some(serial) = SERIAL1.lock().as_mut() {
        serial.write_fmt(args).ok();
    }
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
