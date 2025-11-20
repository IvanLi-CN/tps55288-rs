#![no_std]
#![no_main]

//! Minimal STM32G031G8U6 example sketch using `stm32g0xx-hal` (blocking I2C).
//!
//! Wiring (adjust to your board):
//! - I2C1: PB6 = SCL, PB7 = SDA (with pull-ups)
//! - EN pin of TPS55288 tied to MCU GPIO (optional; if tied high, skip the GPIO step)
//! - FB/INT optional to a GPIO input for fault indication
//!
//! Build (example):
//! ```
//! cargo build --release -p tps55288-stm32g031g8u6 --target thumbv6m-none-eabi
//! ```
//! Flash (probe-rs example):
//! ```
//! probe-rs run --chip STM32G031G8Ux target/thumbv6m-none-eabi/release/tps55288-stm32g031g8u6
//! ```

use cortex_m_rt::entry;
use embedded_hal::i2c::I2c;
use stm32g0xx_hal as hal;

use hal::gpio::{gpiob::PB, Output, PushPull};
use hal::i2c::I2c as HalI2c;
use hal::pac;
use hal::prelude::*;

use tps55288_rs::driver::Tps55288;
use tps55288_rs::{CableCompLevel, CableCompOption, FeedbackSource, InternalFeedbackRatio, OcpDelay, VoutSlewRate};

type EnaPin = PB<Output<PushPull>>;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    let clocks = rcc.config().sysclk(16.mhz()).freeze(&mut flash);

    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);

    // I2C pins PB6/PB7 (AF1 for I2C1)
    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

    let i2c: HalI2c<pac::I2C1, _> = HalI2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), &mut rcc);

    // Optional EN pin (e.g., PA0) if TPS55288 EN is tied to MCU.
    let mut en = gpioa.pa0.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    en.set_high();

    let mut dev = Tps55288::new(i2c);
    dev.init().ok();
    dev.set_vout_mv(5_000).ok();
    dev.set_ilim_ma(3_000, true).ok();
    dev.set_vout_sr(VoutSlewRate::Sr2p5MvPerUs, OcpDelay::Us128).ok();
    dev.set_feedback(FeedbackSource::Internal, InternalFeedbackRatio::R0_0564).ok();
    dev.set_cable_comp(CableCompOption::Internal, CableCompLevel::V0p0, true, true, true)
        .ok();

    loop {
        // TODO: add status/fault poll and VOUT stepping.
    }
}
