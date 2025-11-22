use defmt::{info, warn};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    i2c::{Config as I2cConfig, I2c},
    mode::Async,
    time::Hertz,
};
use embassy_time::{Duration, Timer};

use tps55288_rs::data_types::{
    CableCompLevel, CableCompOption, FeedbackSource, FaultStatus, InternalFeedbackRatio,
    OcpDelay, OperatingStatus, VoutSlewRate,
};
use tps55288_rs::driver::Tps55288;

/// Concrete I2C type for I2C1 on PB6/PB7 using DMA1 channels (async mode).
pub type BoardI2c = I2c<'static, Async>;

pub struct Board {
    pub i2c: BoardI2c,
    pub en: Output<'static>,
    pub led: Output<'static>,
}

/// Initialize peripherals following the sc8815-rs STM32G0 pattern.
pub fn init_board() -> Board {
    let p = embassy_stm32::init(Default::default());

    let mut i2c_cfg = I2cConfig::default();
    i2c_cfg.scl_pullup = true;
    i2c_cfg.sda_pullup = true;

    let i2c = I2c::new(
        p.I2C1,
        p.PB6, // SCL
        p.PB7, // SDA
        super::Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        Hertz(100_000),
        i2c_cfg,
    );

    // EN is wired to PB5 on current hardware; start high to allow I2C access.
    let en = Output::new(p.PB5, Level::High, Speed::Low);
    // PB8 is available for a heartbeat LED.
    let led = Output::new(p.PB8, Level::High, Speed::Low);

    Board { i2c, en, led }
}

/// Apply a safe baseline configuration using internal DAC feedback (FB pin unused).
pub async fn setup_device<I2C>(dev: &mut Tps55288<I2C>)
where
    I2C: embedded_hal_async::i2c::I2c,
{
    info!("Configuring TPS55288 with internal DAC feedback");

    if let Err(e) = dev.init_async().await {
        warn!("init failed: {:?}", defmt::Debug2Format(&e));
    }
    if let Err(e) = dev.set_ilim_ma_async(3_000, true).await {
        warn!("set_ilim failed: {:?}", defmt::Debug2Format(&e));
    }
    // Use the smallest internal divider (R0_0564) so the REF DAC maps 0.8–21 V correctly.
    if let Err(e) = dev
        .set_feedback_async(FeedbackSource::Internal, InternalFeedbackRatio::R0_0564)
        .await
    {
        warn!("set_feedback failed: {:?}", defmt::Debug2Format(&e));
    }
    if let Err(e) = dev
        .set_cable_comp_async(CableCompOption::Internal, CableCompLevel::V0p0, true, true, true)
        .await
    {
        warn!("set_cable_comp failed: {:?}", defmt::Debug2Format(&e));
    }
    if let Err(e) = dev
        .set_vout_sr_async(VoutSlewRate::Sr2p5MvPerUs, OcpDelay::Us128)
        .await
    {
        warn!("set_vout_sr failed: {:?}", defmt::Debug2Format(&e));
    }
}

pub fn log_status(mv: u16, mode: OperatingStatus, faults: FaultStatus) {
    if faults.short_circuit || faults.over_current || faults.over_voltage {
        warn!(
            "vset={}mV mode={:?} sc:{} oc:{} ov:{}",
            mv, mode, faults.short_circuit, faults.over_current, faults.over_voltage
        );
    } else {
        info!(
            "vset={}mV mode={:?} sc:{} oc:{} ov:{}",
            mv, mode, faults.short_circuit, faults.over_current, faults.over_voltage
        );
    }
}

pub async fn heartbeat(led: &mut Output<'_>) {
    led.toggle();
    Timer::after(Duration::from_secs(1)).await;
}
