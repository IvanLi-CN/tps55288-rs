#![no_std]
#![no_main]

// Placeholder STM32G031G8U6 example wiring sketch.
// Integrator should replace HAL imports, startup attributes, and I2C init per board/RTFM/embassy setup.
use cortex_m_rt::entry;
use embedded_hal::i2c::I2c;
use tps55288_rs::driver::Tps55288;
use tps55288_rs::{CableCompLevel, CableCompOption, FeedbackSource, InternalFeedbackRatio, OcpDelay, OperatingStatus, VoutSlewRate};

#[entry]
fn main() -> ! {
    // TODO: initialize clocks/gpio/i2c
    // let i2c = ...;
    // let mut driver = Tps55288::new(i2c);
    // driver.init().ok();
    // driver.set_vout_mv(5_000).ok();
    // driver.set_ilim_ma(3_000, true).ok();
    // driver.set_vout_sr(VoutSlewRate::Sr2p5MvPerUs, OcpDelay::Us128).ok();
    // driver.set_feedback(FeedbackSource::Internal, InternalFeedbackRatio::R0_0564).ok();
    // driver.set_cable_comp(CableCompOption::Internal, CableCompLevel::V0p0, true, true, true).ok();
    // let _status = driver.read_status().ok();
    loop {}
}
