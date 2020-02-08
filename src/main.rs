#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use nrf52832_hal::nrf52832_pac as pac;


#[entry]
fn main() -> ! {
    let _cp = cortex_m::Peripherals::take().unwrap();
    let _p = pac::Peripherals::take().unwrap();

    loop {
        continue;
    }
}
