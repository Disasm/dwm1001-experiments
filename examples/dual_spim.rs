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
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;


fn configure_spim<P1, P2>(spim: &pac::spim0::RegisterBlock, sck: Pin<P1>, mosi: Pin<P2>) {
    spim.psel.sck.write(|w| {
        let w = unsafe { w.pin().bits(sck.pin) };
        w.connect().connected()
    });
    spim.psel.mosi.write(|w| {
        let w = unsafe { w.pin().bits(mosi.pin) };
        w.connect().connected()
    });

    // Enable SPIM instance
    spim.enable.write(|w| w.enable().enabled());

    // Configure mode
    spim.config.write(|w| {
        w.order().msb_first();
        w.cpol().active_high();
        w.cpha().leading()
    });

    // Configure frequency
    spim.frequency.write(|w| unsafe { w.frequency().bits(0x8000_0000) });

    // Set over-read character to `0`
    spim.orc.write(|w| unsafe { w.orc().bits(0x00) });
}


#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = pac::Peripherals::take().unwrap();
    let port0: gpio::p0::Parts = p.P0.split();

    let spim0 = p.SPIM0;
    let spim1 = p.SPIM1;

    let ppi = p.PPI;
    let egu = p.EGU0;
    ppi.ch[0].eep.write(|w| unsafe { w.bits(&egu.events_triggered[0] as *const _ as u32) });
    ppi.ch[0].tep.write(|w| unsafe { w.bits(&spim0.tasks_start as *const _ as u32) });
    ppi.fork[0].tep.write(|w| unsafe { w.bits(&spim1.tasks_start as *const _ as u32) });
    ppi.chenset.write(|w| w.ch0().set_bit());

    let j7_3 = port0.p0_12.into_push_pull_output(Level::Low).degrade();
    let j7_4 = port0.p0_27.into_push_pull_output(Level::Low).degrade();
    let j7_7 = port0.p0_23.into_push_pull_output(Level::Low).degrade();
    let j7_8 = port0.p0_13.into_push_pull_output(Level::Low).degrade();
    let mut led1: P0_30<gpio::Output<PushPull>> = port0.p0_30.into_push_pull_output(Level::High);

    let mut delay = Delay::new(cp.SYST);

    configure_spim(&spim0, j7_3, j7_4);
    configure_spim(&spim1, j7_7, j7_8);

    loop {
        let mut tx_buf1 = [0; 128];
        for (i, b) in tx_buf1.iter_mut().enumerate() {
            *b = (i + 1) as u8;
        }

        let mut tx_buf2 = [0; 128];
        for (i, b) in tx_buf2.iter_mut().enumerate() {
            *b = (0xff - i) as u8;
        }

        led1.set_low().ok(); // Turn on

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(SeqCst);

        // Set up the DMA write
        spim0.txd.ptr.write(|w| unsafe { w.ptr().bits(tx_buf1.as_ptr() as _) });
        spim1.txd.ptr.write(|w| unsafe { w.ptr().bits(tx_buf2.as_ptr() as _) });

        spim0.txd.maxcnt.write(|w|
            // Note that that nrf52840 maxcnt is a wider
            // type than a u8, so we use a `_` cast rather than a `u8` cast.
            // The MAXCNT field is thus at least 8 bits wide and accepts the full
            // range of values that fit in a `u8`.
            unsafe { w.maxcnt().bits(tx_buf1.len() as _ ) });
        spim1.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(tx_buf2.len() as _ ) });

        // Set up the DMA read
        spim0.rxd.ptr.write(|w|
            // This is safe for the same reasons that writing to TXD.PTR is
            // safe. Please refer to the explanation there.
            unsafe { w.ptr().bits(0) });
        spim0.rxd.maxcnt.write(|w|
            // This is safe for the same reasons that writing to TXD.MAXCNT is
            // safe. Please refer to the explanation there.
            unsafe { w.maxcnt().bits(0) });
        spim1.rxd.ptr.write(|w| unsafe { w.ptr().bits(0) });
        spim1.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(0) });

        // Start SPI transaction
        egu.tasks_trigger[0].write(|w| unsafe { w.bits(1) });
        // spim0.tasks_start.write(|w| unsafe { w.bits(1) });
        // spim1.tasks_start.write(|w| unsafe { w.bits(1) });

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed
        compiler_fence(SeqCst);

        // Wait for END event
        //
        // This event is triggered once both transmitting and receiving are
        // done.
        while spim0.events_end.read().bits() == 0 {}
        while spim1.events_end.read().bits() == 0 {}

        // Reset the event, otherwise it will always read `1` from now on.
        spim0.events_end.write(|w| w);
        spim1.events_end.write(|w| w);

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed
        compiler_fence(SeqCst);

        delay.delay_ms(50u32);

        led1.set_high().ok(); // Turn off

        delay.delay_ms(100u32);
    }
}
