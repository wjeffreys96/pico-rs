#![no_std]
#![no_main]

use bsp::entry;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
    Timer,
};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use panic_probe as _;
use rp_pico::hal::fugit::{ExtU64, Instant};
use rp_pico::hal::gpio::bank0::Gpio25;
use rp_pico::hal::gpio::{FunctionSio, Pin, PullDown, SioOutput};
use rp_pico::{self as bsp};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

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

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();
    let time_start = timer.get_counter();
    let mut last_led_toggle_time = time_start;

    loop {
        if (timer.get_counter() - last_led_toggle_time) >= 500.millis::<1, 1_000_000>() {
            led_pin.toggle();
            last_led_toggle_time = timer.get_counter();
        }
    }
}
