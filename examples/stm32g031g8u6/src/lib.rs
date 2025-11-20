#![no_std]
#![no_main]

// STM32G031G8U6 example using PB6/PB7 (I2C1) and PB5 (EN).
// By default, the crate builds a minimal stub. Enable `--features hw` to compile the HAL-based code.

#[cfg(not(feature = "hw"))]
#[cortex_m_rt::entry]
fn main() -> ! {
    // Stub build (no HAL) to keep CI green; enable `hw` feature for target build.
    loop {}
}

#[cfg(feature = "hw")]
mod hw {
    use cortex_m_rt::entry;
    use embedded_hal::delay::DelayNs;
    use embedded_hal::i2c::I2c;
    use stm32g0xx_hal as hal;

    use hal::gpio::gpiob::PB5;
    use hal::gpio::{Output, PushPull};
    use hal::i2c::blocking::I2c; // blocking I2C implementation
    use hal::pac;
    use hal::prelude::*;
    use hal::rcc::Config;
    use hal::timer::Timer;

    use tps55288_rs::data_types::{CableCompLevel, CableCompOption, FeedbackSource, FaultStatus, InternalFeedbackRatio, OcpDelay, OperatingStatus, VoutSlewRate};
    use tps55288_rs::driver::Tps55288;

    #[entry]
    fn main() -> ! {
        let dp = pac::Peripherals::take().unwrap();
        let mut rcc = dp.RCC.freeze(Config::pll()); // simple clock config

        let gpiob = dp.GPIOB.split(&mut rcc);
        // I2C1 pins PB6/PB7
        let scl = gpiob.pb6.into_open_drain_output();
        let sda = gpiob.pb7.into_open_drain_output();
        let mut i2c = I2c::i2c1(dp.I2C1, sda, scl, 100.kHz(), &mut rcc);

        // EN pin PB5
        let mut en: PB5<Output<PushPull>> = gpiob.pb5.into_push_pull_output();
        en.set_high();

        let mut tim14 = Timer::tim14(dp.TIM14, 1.hz(), &mut rcc);
        run_example(&mut i2c, &mut tim14);
    }

    fn run_example<I2C, D>(i2c: &mut I2C, delay: &mut D)
    where
        I2C: I2c,
        D: DelayNs,
        I2C::Error: core::fmt::Debug,
    {
        let mut dev = Tps55288::new(i2c);
        let _ = dev.init();
        let _ = dev.set_ilim_ma(3_000, true);
        let _ = dev.set_feedback(FeedbackSource::Internal, InternalFeedbackRatio::R0_0564);
        let _ = dev.set_cable_comp(CableCompOption::Internal, CableCompLevel::V0p0, true, true, true);
        let _ = dev.set_vout_sr(VoutSlewRate::Sr2p5MvPerUs, OcpDelay::Us128);

        let mut mv = 3_300u16;
        loop {
            let _ = dev.set_vout_mv(mv);
            mv = if mv + 20 <= 21_000 { mv + 20 } else { 3_300 };
            if let Ok((mode, faults)) = dev.read_status() {
                log_status(mv, mode, faults);
            }
            delay.delay_ms(1000);
        }
    }

    fn log_status(mv: u16, mode: OperatingStatus, faults: FaultStatus) {
        #[cfg(feature = "defmt")]
        defmt::info!("vset={}mV mode={:?} sc:{} oc:{} ov:{}", mv, mode, faults.short_circuit, faults.over_current, faults.over_voltage);
    }
}
