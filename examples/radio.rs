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
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;


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
    write!(uart, "\r\n\r\nStarting...\r\n").unwrap();

    let mut delay = Delay::new(cp.SYST);

    let radio = p.RADIO;

    // Reset radio
    radio.power.write(|w| w.power().disabled());
    delay.delay_ms(1u8);
    radio.power.write(|w| w.power().enabled());
    delay.delay_ms(1u8);

    radio.mode.write(|w| w.mode().nrf_1mbit());
    radio.txpower.write(|w| w.txpower()._0d_bm());
    radio.pcnf0.write(|w| unsafe {
        w.lflen().bits(0);
        w.s0len().clear_bit();
        w.s1len().bits(0);
        w.s1incl().clear_bit();
        w.plen()._8bit()
    });
    radio.pcnf1.write(|w| unsafe {
        w.maxlen().bits(16);
        w.statlen().bits(16); // Important
        w.balen().bits(1); // Base address length==2, hacky!
        w.endian().big(); // Don't care
        w.whiteen().disabled() // No whitening
    });

    // Logical address 0: 55 55 55 (0101..01)
    // Logical address 1: aa aa aa (1010..10)
    //radio.base0.write(|w| unsafe { w.bits(0xdeadbeef) });
    radio.base0.write(|w| unsafe { w.bits(0x55555555) });
    radio.base1.write(|w| unsafe { w.bits(0xaaaaaaaa) });
    //radio.base1.write(|w| unsafe { w.bits(0x55555555) });
    radio.prefix0.modify(|_, w| unsafe {
        w.ap0().bits(0x55);
        w.ap1().bits(0xaa);
        w
    });
    radio.rxaddresses.write(|w| {
        w.addr0().enabled();
        w.addr1().enabled()
    });

    radio.crccnf.write(|w| w.len().disabled());

    // 2432 - WiFi router frequency
    radio.frequency.write(|w| unsafe {
        w.frequency().bits(100);
        w.map().default()
    });

    radio.shorts.modify(|_, w| w.ready_start().enabled());
    radio.shorts.modify(|_, w| w.end_disable().enabled());

    // Start receiver
//    write!(uart, "state: {}\r\n", radio.state.read().state().bits()).unwrap();
//    radio.tasks_rxen.write(|w| unsafe { w.bits(1) });
//    write!(uart, "state: {}\r\n", radio.state.read().state().bits()).unwrap();
//    //while !radio.state.read().state().is_rx_idle() {}
//    write!(uart, "state: {}\r\n", radio.state.read().state().bits()).unwrap();
//    write!(uart, "Started\r\n").unwrap();

    let mut packet = [1u8; 32];

//    compiler_fence(SeqCst);
//    let buf: &[u8] = &packet;
//    radio.packetptr.write(|w| unsafe { w.bits(buf.as_ptr() as u32) });
//    write!(uart, "packetptr: {:08x}\r\n", radio.packetptr.read().bits()).unwrap();
//    radio.tasks_start.write(|w| unsafe { w.bits(1) });
//    write!(uart, "state: {}\r\n", radio.state.read().state().bits()).unwrap();
//    while radio.events_end.read().bits() == 0 {}
//    write!(uart, "state: {}\r\n", radio.state.read().state().bits()).unwrap();
//    radio.events_end.reset();
//    compiler_fence(SeqCst);
//    let addr = radio.rxmatch.read().rxmatch().bits();
//    write!(uart, "{} {:?}\r\n", addr, packet);
//    write!(uart, "state: {}\r\n", radio.state.read().state().bits()).unwrap();

    //loop {}
    radio.events_disabled.reset();

    let mut counter: u32 = 0;
    loop {
        packet = [2u8; 32];
        //core::sync::atomic::fence(SeqCst);
        compiler_fence(SeqCst);
        let buf: &[u8] = &packet;
        radio.packetptr.write(|w| unsafe { w.bits(buf.as_ptr() as u32) });
        radio.tasks_rxen.write(|w| unsafe { w.bits(1) });
        //write!(uart, "state after start: {}\r\n", radio.state.read().state().bits()).unwrap();
        while radio.events_disabled.read().bits() == 0 {}
        radio.events_disabled.reset();

        while !radio.state.read().state().is_disabled() {}

        //core::sync::atomic::fence(SeqCst);
        compiler_fence(SeqCst);
        counter += 1;

        if counter & 0xfff == 0 {
            write!(uart, "{} packets\r\n", counter).unwrap();
        }

        //core::sync::atomic::fence(SeqCst);
        compiler_fence(SeqCst);
        let mut min = 0xff;
        let mut max = 0;
        for i in 0..16 {
            let b = unsafe { packet.as_ptr().offset(i as isize).read_volatile() };
            min = core::cmp::min(min, b);
            max = core::cmp::max(max, b);
        }
        if min == max && (min == 0 || min == 0xff) {
            continue
        }
        if &packet[0..4] == [64, 0, 7, 0] {
            continue
        }
//        if packet[0] == 0x80 && packet[1] == 0 {
//            continue
//        }
        let addr = radio.rxmatch.read().rxmatch().bits();
        write!(uart, "{} {:?}\r\n", addr, packet);
        //write!(uart, "{:?}\r\n", unsafe { packet.as_ptr().read_volatile() });
        //write!(uart, "state: {}\r\n", radio.state.read().state().bits()).unwrap();

//        write!(uart, "counter={}\r\n", counter).unwrap();
//        counter = counter.wrapping_add(1);
//        delay.delay_ms(1000u32);
        delay.delay_ms(10u32);
    }
}
