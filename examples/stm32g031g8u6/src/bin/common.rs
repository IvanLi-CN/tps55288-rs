use defmt::{info, warn};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    i2c::{Config as I2cConfig, I2c},
    peripherals::I2C1,
    time::Hertz,
    Peripheral,
};
use embassy_time::{Duration, Timer};

use tps55288_rs::data_types::{CableCompLevel, CableCompOption, FeedbackSource, FaultStatus, InternalFeedbackRatio, OcpDelay, OperatingStatus, VoutSlewRate};
use tps55288_rs::driver::Tps55288;

pub struct Board<'d> {
    pub i2c: I2c<'d, I2C1, embassy_stm32::dma::NoDma, embassy_stm32::dma::NoDma>,
    pub en: Output<'d, embassy_stm32::peripherals::PB5>,
    pub led: Output<'d, embassy_stm32::peripherals::PB8>,
}

pub fn init_board() -> Board<'static> {
    let p = embassy_stm32::init(Default::default());

    let mut i2c_cfg = I2cConfig::default();
    i2c_cfg.scl_pullup = true;
    i2c_cfg.sda_pullup = true;

    let i2c = I2c::new(
        p.I2C1,
        p.PB6, // SCL
        p.PB7, // SDA
        super::Irqs,
        embassy_stm32::dma::NoDma,
        embassy_stm32::dma::NoDma,
        Hertz(100_000),
        i2c_cfg,
    );

    let en = Output::new(p.PB5, Level::High, Speed::Low);
    let led = Output::new(p.PB8, Level::High, Speed::Low);

    Board { i2c, en, led }
}

pub async fn setup_device<I2C>(dev: &mut Tps55288<I2C>)
where
    I2C: embedded_hal_async::i2c::I2c,
{
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

pub async fn heartbeat(led: &mut Output<'_, embassy_stm32::peripherals::PB8>) {
    led.toggle();
    Timer::after(Duration::from_secs(1)).await;
}
