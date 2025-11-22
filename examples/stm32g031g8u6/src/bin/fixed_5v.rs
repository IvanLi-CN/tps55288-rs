#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, i2c};
use {defmt_rtt as _, panic_probe as _};

mod common;
use common::{heartbeat, init_board, log_status, setup_device};
use tps55288_rs::driver::Tps55288;

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>, i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("TPS55288 fixed 5V demo (PB6/PB7 I2C1, PB5 EN)");
    let mut board = init_board();
    board.en.set_high();
    let mut dev = Tps55288::new(board.i2c);
    setup_device(&mut dev).await;

    let target = 5_000u16;
    let _ = dev.set_vout_mv_async(target).await;

    // Avoid spamming identical "vset=5000mV" lines: only log when status bits change
    // or when a fault is present. The TPS55288 does not expose live V/I telemetry over
    // I2C, so we can only report the setpoint and fault flags.
    let mut last_mode = None;
    let mut last_faults = None;

    loop {
        if let Ok((mode, faults)) = dev.read_status_async().await {
            let changed = last_mode.map(|m| m != mode).unwrap_or(true)
                || last_faults.map(|f| f != faults).unwrap_or(true)
                || faults.short_circuit
                || faults.over_current
                || faults.over_voltage;

            if changed {
                log_status(target, mode, faults);
                last_mode = Some(mode);
                last_faults = Some(faults);
            }
        }
        heartbeat(&mut board.led).await;
    }
}
