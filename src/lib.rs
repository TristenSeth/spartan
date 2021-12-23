#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]


use core::panic::PanicInfo;
use core::fmt;
pub mod serial;
pub mod vga_buffer;

/// Color formatting  for test results
struct Green(&'static str);

impl fmt::Display for Green {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        write!(f, "\x1B[32m")?; // prefix code
        write!(f, "{}", self.0)?;
        write!(f, "\x1B[0m")?; // postfix code
        Ok(())
    }
}

struct Red(&'static str);

impl fmt::Display for Red {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        write!(f, "\x1B[31m")?; // prefix code
        write!(f, "{}", self.0)?;
        write!(f, "\x1B[0m")?; // postfix code
        Ok(())
    }
}


/// Trait for testable functions
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("{}", Green("[ok]"));
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("{}", Red("[failed]\n"));
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failure);
    loop {}
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}