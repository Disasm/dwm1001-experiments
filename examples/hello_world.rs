#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use nrf52832_hal::{nrf52832_pac as pac, Delay};
use nrf52832_hal::gpio::Level;
use nrf52832_hal::gpio::*;
use nrf52832_hal::uarte::{Pins, Uarte, Parity, Baudrate};
use embedded_hal::blocking::delay::DelayMs;
use core::fmt::Write;


#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = pac::Peripherals::take().unwrap();
    let port0 = p.P0.split();

    let uart_pins = Pins {
        rxd: port0.p0_11.into_floating_input().degrade(),
        txd: port0.p0_05.into_push_pull_output(Level::High).degrade(),
        cts: None,
        rts: None
    };
    let mut uart = Uarte::new(p.UARTE0, uart_pins, Parity::EXCLUDED, Baudrate::BAUD115200);
    write!(uart, "Hello, world!\r\n").unwrap();

    let mut delay = Delay::new(cp.SYST);
    let mut counter: u32 = 0;
    loop {
        write!(uart, "counter={}\r\n", counter).unwrap();
        counter = counter.wrapping_add(1);
        delay.delay_ms(1000u32);
    }
}
