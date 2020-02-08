#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use nrf52832_hal::{nrf52832_pac as pac, Delay};
use nrf52832_hal::gpio;
use nrf52832_hal::gpio::p0::*;
use nrf52832_hal::gpio::Level;
use nrf52832_hal::gpio::*;
use embedded_hal::blocking::delay::DelayMs;


#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = pac::Peripherals::take().unwrap();
    let port0 = p.P0.split();

    let mut led1: P0_30<gpio::Output<PushPull>> = port0.p0_30.into_push_pull_output(Level::High);

    let mut delay = Delay::new(cp.SYST);

    loop {
        led1.set_high();
        delay.delay_ms(500u32);
        led1.set_low();
        delay.delay_ms(500u32);
    }
}
