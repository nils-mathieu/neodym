use nd_log::Verbosity;
use nd_x86_64::{inb, outb, RFlags};

use core::fmt;
use core::fmt::Write as _;

/// Initializes the logging facade.
///
/// This function simply initializes the `nd_log` crate with a function that writes to the serial
/// COM1 port.
///
/// # Safety
///
/// This function must not be called more than once.
///
/// The serial I/O ports must not be used anywere else in the program (and must never be).
pub unsafe fn initialize_logger() {
    // SAFETY:
    //  The `initialize` function must only be called once, ensuring that the serial port has not
    //  been initialized yet.
    //
    //  This file is the only place of the code that uses the serial port, ensuring exclusivity.
    unsafe { SerialOut::init() };

    nd_log::set_global_logger(|record| {
        let prefix = match record.verbosity {
            Verbosity::Error => "  \x1B[31mError\x1B[0m ",
            Verbosity::Warn => "   \x1B[33mWarn\x1B[0m ",
            Verbosity::Info => "   \x1B[36mInfo\x1B[0m ",
            Verbosity::Trace => "  Trace ",
        };

        let restore_interrupts = unsafe { nd_x86_64::rflags().contains(RFlags::INTERRUPT) };

        if restore_interrupts {
            // Prevent interrupts while we are writing, ensuring that
            // the output is not corrupted.
            unsafe { nd_x86_64::cli() };
        }

        // SAFETY:
        //  We're setting the global logger *after* having initialized the serial output port,
        //  ensuring that the `get_unchecked` function is safe.
        let mut serial_out = unsafe { SerialOut::get_unchecked() };

        let _ = writeln!(serial_out, "{prefix}{}", record.message);

        if restore_interrupts {
            unsafe { nd_x86_64::sti() };
        }
    });

    nd_log::trace!("Logger initialized.");
}

/// Represents the output serial port.
#[derive(Clone, Copy)]
struct SerialOut {
    _private: (),
}

impl SerialOut {
    /// The port that we're using to log.
    pub const COM1: u16 = 0x3F8;

    /// Returns a new [`SerialOut`] instance.
    ///
    /// # Safety
    ///
    /// The [`SerialOut::init`] function must've been called previously.
    #[inline(always)]
    pub unsafe fn get_unchecked() -> Self {
        Self { _private: () }
    }

    /// # Safety
    ///
    /// This function must not be called more than once.
    ///
    /// The serial I/O ports must not be used anywere else in the program (and must never be).
    pub unsafe fn init() -> Self {
        // More or less taken from:
        //   https://wiki.osdev.org/Serial_Ports

        unsafe {
            // Disable interrupts.
            outb(Self::COM1 + 1, 0x00);

            // Set baud rate to 38400 baud.
            outb(Self::COM1 + 3, 0x80);
            outb(Self::COM1, 0x03);
            outb(Self::COM1 + 1, 0x00);

            // Confiture the UART. 8 bits, no parity bit, only one stop bit. This also includes
            // more configuration.
            outb(Self::COM1 + 3, 0x03);
            outb(Self::COM1 + 2, 0xC7);
            outb(Self::COM1 + 4, 0x1E);

            // Normal operation mode.
            outb(Self::COM1 + 4, 0x0F);
        }

        Self { _private: () }
    }

    /// Returns whether the transmition buffer is currently empty.
    #[inline(always)]
    pub fn is_transmit_empty(self) -> bool {
        unsafe { inb(Self::COM1 + 5) & 0x20 != 0 }
    }

    /// Writes a specific byte to the output port.
    pub fn write_byte(self, byte: u8) {
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }

        unsafe { outb(Self::COM1, byte) };
    }
}

impl fmt::Write for SerialOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            self.write_byte(b);
        }

        Ok(())
    }
}
