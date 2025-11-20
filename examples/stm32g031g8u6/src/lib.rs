#![no_std]
#![no_main]

// STM32G031G8U6 + TPS55288 using Embassy stack, PB6/PB7 I2C1, PB5 EN.
// Two modes: step VOUT (3.3→21V, 20mV step per sec) and fixed 5V.
// Build: cargo build --release -p tps55288-stm32g031g8u6 --features hw
// Flash: probe-rs run --chip STM32G031G8Ux target/thumbv6m-none-eabi/release/tps55288-stm32g031g8u6

use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    i2c::{self, Config as I2cConfig, I2c},
    time::Hertz,
};
use embassy_time::{Duration, Timer};
use panic_probe as _;

use tps55288_rs::data_types::{CableCompLevel, CableCompOption, FeedbackSource, FaultStatus, InternalFeedbackRatio, OcpDelay, OperatingStatus, VoutSlewRate};
use tps55288_rs::driver::Tps55288;

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>, i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // EN pin PB5
    let mut en = Output::new(p.PB5, Level::High, Speed::Low);
    // LED on PB8 for simple heartbeat
    let mut led = Output::new(p.PB8, Level::High, Speed::Low);

    // I2C1 on PB6/PB7, 100 kHz
    let mut i2c_cfg = I2cConfig::default();
    i2c_cfg.scl_pullup = true;
    i2c_cfg.sda_pullup = true;
    let i2c = I2c::new(
        p.I2C1,
        p.PB6, // SCL
        p.PB7, // SDA
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        Hertz(100_000),
        i2c_cfg,
    );

    info!("TPS55288 demo start: PB6/PB7 I2C1, PB5 EN");

    // Choose which demo to run
    let run_fixed = false; // set true for fixed 5V, false for stepping

    if run_fixed {
        run_fixed_5v(i2c, &mut en, &mut led).await;
    } else {
        run_step_vout(i2c, &mut en, &mut led).await;
    }
}

async fn setup_common<I2C>(i2c: I2C, en: &mut Output<'_, embassy_stm32::peripherals::PB5>) -> Tps55288<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    en.set_high();
    let mut dev = Tps55288::new(i2c);
    let _ = dev.init_async().await;
    let _ = dev.set_ilim_ma_async(3_000, true).await;
    let _ = dev
        .set_feedback_async(FeedbackSource::Internal, InternalFeedbackRatio::R0_0564)
        .await;
    let _ = dev
        .set_cable_comp_async(CableCompOption::Internal, CableCompLevel::V0p0, true, true, true)
        .await;
    let _ = dev
        .set_vout_sr_async(VoutSlewRate::Sr2p5MvPerUs, OcpDelay::Us128)
        .await;
    dev
}

async fn run_step_vout<I2C>(i2c: I2C, en: &mut Output<'_, embassy_stm32::peripherals::PB5>, led: &mut Output<'_, embassy_stm32::peripherals::PB8>)
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let mut dev = setup_common(i2c, en).await;
    let mut mv: u16 = 3_300;
    loop {
        let _ = dev.set_vout_mv_async(mv).await;
        if let Ok((mode, faults)) = dev.read_status_async().await {
            log_status(mv, mode, faults);
        }
        led.toggle();
        mv = if mv + 20 <= 21_000 { mv + 20 } else { 3_300 };
        Timer::after(Duration::from_secs(1)).await;
    }
}

async fn run_fixed_5v<I2C>(i2c: I2C, en: &mut Output<'_, embassy_stm32::peripherals::PB5>, led: &mut Output<'_, embassy_stm32::peripherals::PB8>)
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let mut dev = setup_common(i2c, en).await;
    let target = 5_000u16;
    let _ = dev.set_vout_mv_async(target).await;
    loop {
        if let Ok((mode, faults)) = dev.read_status_async().await {
            log_status(target, mode, faults);
        }
        led.toggle();
        Timer::after(Duration::from_secs(1)).await;
    }
}

fn log_status(mv: u16, mode: OperatingStatus, faults: FaultStatus) {
    if faults.short_circuit || faults.over_current || faults.over_voltage {
        warn!(
            "vset={}mV mode={:?} FAULT sc:{} oc:{} ov:{}",
            mv, mode, faults.short_circuit, faults.over_current, faults.over_voltage
        );
    } else {
        info!(
            "vset={}mV mode={:?} sc:{} oc:{} ov:{}",
            mv, mode, faults.short_circuit, faults.over_current, faults.over_voltage
        );
    }
}
