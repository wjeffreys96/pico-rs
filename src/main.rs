#![no_std]
#![no_main]

extern crate alloc;

use bsp::entry;
use bsp::hal::{clocks::init_clocks_and_plls, pac, sio::Sio, watchdog::Watchdog, Timer};
use defmt::{debug, info};
use embedded_alloc::LlffHeap as Heap;
use embedded_hal::digital::StatefulOutputPin;
use panic_probe as _;
use rp_pico::hal::fugit::ExtU64;
use rp_pico::hal::Clock;
use rp_pico::{self as bsp};
use rtt_target::rtt_init;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[entry]
fn main() -> ! {
    info!("Program start");

    unsafe {
        embedded_alloc::init!(HEAP, 1024);
    }

    debug!("allocator initialized {}", alloc::vec![1].as_slice());

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    debug!("pac initialized");

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    debug!("watchdog initialized");

    let sio = Sio::new(pac.SIO);

    debug!("sio initialized");

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    debug!("clocks initialized");

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    debug!("pins initialized");

    let mut channels = rtt_init! {
        up: {
            0: { size: 1024, name: "Terminal" }
            1: { size: 1024, name: "defmt" }
        }
        down: {
            0: { size: 128, name: "Terminal" }
        }
    };

    rtt_target::set_defmt_channel(channels.up.1);

    debug!("rtt initialized");

    let mut rtt_buf = [0u8; 128];
    let mut led_pin = pins.led.into_push_pull_output();
    let time_start = timer.get_counter();
    let mut last_led_toggle_time = time_start;

    info!("Entering main loop");
    loop {
        if (timer.get_counter() - last_led_toggle_time) >= 500.millis::<1, 1_000_000>() {
            led_pin.toggle();
            last_led_toggle_time = timer.get_counter();
        }
        let read = channels.down.0.read(&mut rtt_buf);
        if read > 0 {
            info!("{:?}", core::str::from_utf8(&rtt_buf[..read - 1]).unwrap());
        }
        delay.delay_ms(100);
    }
}
