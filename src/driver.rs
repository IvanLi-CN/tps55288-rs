//! Driver scaffold for TPS55288.
//! Provides blocking I2C helpers; async version will mirror this API behind the `async` feature.

use crate::error::Error;
use crate::registers::{
    addr, code_to_ilim_ma, code_to_vout_mv, decode_status_mode, ilim_ma_to_code, vout_mv_to_code, ALT_I2C_ADDRESS,
    CdcBits, DEFAULT_I2C_ADDRESS, IoutLimitBits, ModeBits, StatusBits, VoutFsBits, VoutSrBits,
};
use crate::data_types::{
    CableCompLevel, CableCompOption, FeedbackSource, FaultStatus, InternalFeedbackRatio, OcpDelay, OperatingStatus,
    VoutSlewRate,
};

/// TPS55288 driver placeholder.
pub struct Tps55288<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Tps55288<I2C> {
    /// Create a new driver instance with the default I2C address (0x74).
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: DEFAULT_I2C_ADDRESS,
        }
    }

    /// Create a new driver instance with a custom I2C address.
    pub fn with_address(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Return the 7-bit I2C address configured for this instance.
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Switch between default and alternate address (helper for MODE/I2CADD flows).
    pub fn set_address(&mut self, address: u8) {
        self.address = address;
    }

    /// Quick helper: select default address (0x74).
    pub fn select_default_address(&mut self) {
        self.address = DEFAULT_I2C_ADDRESS;
    }

    /// Quick helper: select alternate address (0x75).
    pub fn select_alt_address(&mut self) {
        self.address = ALT_I2C_ADDRESS;
    }
}

impl<I2C> Tps55288<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    /// Initialize device with safe defaults (enable OE, current limit enabled, default VOUT).
    pub fn init(&mut self) -> Result<(), Error<I2C::Error>> {
        // Enable current limit with default 50 mV (datasheet reset value) to avoid uncontrolled current.
        self.write_reg(addr::IOUT_LIMIT, IoutLimitBits::EN.bits() | 0b1100100)?;
        // Set default VOUT to datasheet reset (REF reset = 0x0000 -> ~0.8 V). Caller should override for actual use.
        self.set_vout_mv(crate::registers::VOUT_MIN_MV)?;
        // Ensure MODE: OE=0 initially, leave other bits at reset; then enable OE explicitly.
        let mut mode = ModeBits::from_bits_truncate(self.read_reg(addr::MODE)?);
        mode.remove(ModeBits::OE);
        self.write_reg(addr::MODE, mode.bits())?;
        // Enable output (OE=1) keeping other bits unchanged.
        mode.insert(ModeBits::OE);
        self.write_reg(addr::MODE, mode.bits())
    }

    /// Write a single register.
    pub fn write_reg(&mut self, reg: u8, value: u8) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[reg, value])
            .map_err(Error::I2c)
    }

    /// Read a single register.
    pub fn read_reg(&mut self, reg: u8) -> Result<u8, Error<I2C::Error>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .map_err(Error::I2c)?;
        Ok(buf[0])
    }

    /// Update masked bits in a register (read-modify-write).
    pub fn update_reg(&mut self, reg: u8, mask: u8, value: u8) -> Result<(), Error<I2C::Error>> {
        let cur = self.read_reg(reg)?;
        let new = (cur & !mask) | (value & mask);
        self.write_reg(reg, new)
    }

    /// Write a burst starting at a register (for multi-byte REF DAC etc.).
    pub fn write_regs(&mut self, start_reg: u8, data: &[u8]) -> Result<(), Error<I2C::Error>> {
        let mut buf = [0u8; 8];
        if data.len() + 1 > buf.len() {
            // Small helper only; larger writes can stream directly in future.
            return Err(Error::InvalidConfig);
        }
        buf[0] = start_reg;
        buf[1..=data.len()].copy_from_slice(data);
        self.i2c
            .write(self.address, &buf[..=data.len()])
            .map_err(Error::I2c)
    }

    /// Read a burst starting at a register.
    pub fn read_regs(&mut self, start_reg: u8, data: &mut [u8]) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write_read(self.address, &[start_reg], data)
            .map_err(Error::I2c)
    }

    /// Set output voltage (mV) using internal DAC (writes REF0/REF1).
    pub fn set_vout_mv(&mut self, mv: u16) -> Result<(), Error<I2C::Error>> {
        let code = vout_mv_to_code(mv);
        let bytes = code.to_le_bytes();
        self.write_regs(addr::REF0, &bytes)
    }

    /// Read current VOUT setting (mV) from DAC registers.
    pub fn get_vout_mv(&mut self) -> Result<u16, Error<I2C::Error>> {
        let mut buf = [0u8; 2];
        self.read_regs(addr::REF0, &mut buf)?;
        let code = u16::from_le_bytes(buf);
        Ok(code_to_vout_mv(code))
    }

    /// Configure output current limit (mA) and enable bit.
    pub fn set_ilim_ma(&mut self, ma: u16, enable: bool) -> Result<(), Error<I2C::Error>> {
        let code = ilim_ma_to_code(ma) & 0x7F;
        let mut val = code;
        if enable {
            val |= IoutLimitBits::EN.bits();
        }
        self.write_reg(addr::IOUT_LIMIT, val)
    }

    /// Read output current limit configuration (mA, enable flag).
    pub fn get_ilim_ma(&mut self) -> Result<(u16, bool), Error<I2C::Error>> {
        let val = self.read_reg(addr::IOUT_LIMIT)?;
        let enable = (val & IoutLimitBits::EN.bits()) != 0;
        let code = val & 0x7F;
        Ok((code_to_ilim_ma(code), enable))
    }

    /// Configure VOUT slew rate and OCP delay.
    pub fn set_vout_sr(&mut self, slew: VoutSlewRate, ocp_delay: OcpDelay) -> Result<(), Error<I2C::Error>> {
        let mut bits = VoutSrBits::empty();
        bits |= match slew {
            VoutSlewRate::Sr1p25MvPerUs => VoutSrBits::empty(),
            VoutSlewRate::Sr2p5MvPerUs => VoutSrBits::SR0,
            VoutSlewRate::Sr5MvPerUs => VoutSrBits::SR1,
            VoutSlewRate::Sr10MvPerUs => VoutSrBits::SR0 | VoutSrBits::SR1,
        };
        bits |= match ocp_delay {
            OcpDelay::Us128 => VoutSrBits::empty(),
            OcpDelay::Ms3_072 => VoutSrBits::OCP_DELAY0,
            OcpDelay::Ms6_144 => VoutSrBits::OCP_DELAY1,
            OcpDelay::Ms12_288 => VoutSrBits::OCP_DELAY0 | VoutSrBits::OCP_DELAY1,
        };
        self.write_reg(addr::VOUT_SR, bits.bits())
    }

    /// Configure feedback source and internal divider ratio.
    pub fn set_feedback(&mut self, source: FeedbackSource, ratio: InternalFeedbackRatio) -> Result<(), Error<I2C::Error>> {
        let mut bits = VoutFsBits::empty();
        if matches!(source, FeedbackSource::External) {
            bits |= VoutFsBits::FB_EXT;
        }
        bits |= match ratio {
            InternalFeedbackRatio::R0_2256 => VoutFsBits::empty(),
            InternalFeedbackRatio::R0_1128 => VoutFsBits::INTFB0,
            InternalFeedbackRatio::R0_0752 => VoutFsBits::INTFB1,
            InternalFeedbackRatio::R0_0564 => VoutFsBits::INTFB0 | VoutFsBits::INTFB1,
        };
        self.write_reg(addr::VOUT_FS, bits.bits())
    }

    /// Configure cable droop compensation and fault masks.
    pub fn set_cable_comp(
        &mut self,
        option: CableCompOption,
        level: CableCompLevel,
        mask_sc: bool,
        mask_ocp: bool,
        mask_ovp: bool,
    ) -> Result<(), Error<I2C::Error>> {
        let mut bits = CdcBits::empty();
        if mask_sc {
            bits |= CdcBits::SC_MASK;
        }
        if mask_ocp {
            bits |= CdcBits::OCP_MASK;
        }
        if mask_ovp {
            bits |= CdcBits::OVP_MASK;
        }
        if matches!(option, CableCompOption::External) {
            bits |= CdcBits::CDC_OPT;
        }
        let level_bits = match level {
            CableCompLevel::V0p0 => CdcBits::empty(),
            CableCompLevel::V0p1 => CdcBits::CDC0,
            CableCompLevel::V0p2 => CdcBits::CDC1,
            CableCompLevel::V0p3 => CdcBits::CDC0 | CdcBits::CDC1,
            CableCompLevel::V0p4 => CdcBits::CDC2,
            CableCompLevel::V0p5 => CdcBits::CDC2 | CdcBits::CDC0,
            CableCompLevel::V0p6 => CdcBits::CDC2 | CdcBits::CDC1,
            CableCompLevel::V0p7 => CdcBits::CDC2 | CdcBits::CDC1 | CdcBits::CDC0,
        };
        bits |= level_bits;
        self.write_reg(addr::CDC, bits.bits())
    }

    /// Read STATUS register raw bits.
    pub fn read_status_raw(&mut self) -> Result<StatusBits, Error<I2C::Error>> {
        let val = self.read_reg(addr::STATUS)?;
        Ok(StatusBits::from_bits_truncate(val))
    }

    /// Decode STATUS into user-friendly enums.
    pub fn read_status(&mut self) -> Result<(OperatingStatus, FaultStatus), Error<I2C::Error>> {
        let bits = self.read_status_raw()?;
        let mode_bits = decode_status_mode(&bits);
        let operating = match mode_bits {
            0b00 => OperatingStatus::Boost,
            0b01 => OperatingStatus::Buck,
            0b10 => OperatingStatus::BuckBoost,
            _ => OperatingStatus::Reserved,
        };
        let faults = FaultStatus {
            short_circuit: bits.contains(StatusBits::SCP),
            over_current: bits.contains(StatusBits::OCP),
            over_voltage: bits.contains(StatusBits::OVP),
        };
        Ok((operating, faults))
    }
}

#[cfg(feature = "async")]
impl<I2C> Tps55288<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    /// Async version of [`init`].
    pub async fn init_async(&mut self) -> Result<(), Error<I2C::Error>> {
        self.write_reg_async(addr::IOUT_LIMIT, IoutLimitBits::EN.bits() | 0b1100100)
            .await?;
        self.set_vout_mv_async(crate::registers::VOUT_MIN_MV).await?;
        let mut mode = ModeBits::from_bits_truncate(self.read_reg_async(addr::MODE).await?);
        mode.remove(ModeBits::OE);
        self.write_reg_async(addr::MODE, mode.bits()).await?;
        mode.insert(ModeBits::OE);
        self.write_reg_async(addr::MODE, mode.bits()).await
    }

    pub async fn write_reg_async(&mut self, reg: u8, value: u8) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[reg, value])
            .await
            .map_err(Error::I2c)
    }

    pub async fn read_reg_async(&mut self, reg: u8) -> Result<u8, Error<I2C::Error>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .await
            .map_err(Error::I2c)?;
        Ok(buf[0])
    }

    pub async fn update_reg_async(&mut self, reg: u8, mask: u8, value: u8) -> Result<(), Error<I2C::Error>> {
        let cur = self.read_reg_async(reg).await?;
        let new = (cur & !mask) | (value & mask);
        self.write_reg_async(reg, new).await
    }

    pub async fn write_regs_async(&mut self, start_reg: u8, data: &[u8]) -> Result<(), Error<I2C::Error>> {
        let mut buf = [0u8; 8];
        if data.len() + 1 > buf.len() {
            return Err(Error::InvalidConfig);
        }
        buf[0] = start_reg;
        buf[1..=data.len()].copy_from_slice(data);
        self.i2c
            .write(self.address, &buf[..=data.len()])
            .await
            .map_err(Error::I2c)
    }

    pub async fn read_regs_async(&mut self, start_reg: u8, data: &mut [u8]) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write_read(self.address, &[start_reg], data)
            .await
            .map_err(Error::I2c)
    }

    pub async fn set_vout_mv_async(&mut self, mv: u16) -> Result<(), Error<I2C::Error>> {
        let code = vout_mv_to_code(mv);
        let bytes = code.to_le_bytes();
        self.write_regs_async(addr::REF0, &bytes).await
    }

    pub async fn get_vout_mv_async(&mut self) -> Result<u16, Error<I2C::Error>> {
        let mut buf = [0u8; 2];
        self.read_regs_async(addr::REF0, &mut buf).await?;
        let code = u16::from_le_bytes(buf);
        Ok(code_to_vout_mv(code))
    }

    pub async fn set_ilim_ma_async(&mut self, ma: u16, enable: bool) -> Result<(), Error<I2C::Error>> {
        let code = ilim_ma_to_code(ma) & 0x7F;
        let mut val = code;
        if enable {
            val |= IoutLimitBits::EN.bits();
        }
        self.write_reg_async(addr::IOUT_LIMIT, val).await
    }

    pub async fn get_ilim_ma_async(&mut self) -> Result<(u16, bool), Error<I2C::Error>> {
        let val = self.read_reg_async(addr::IOUT_LIMIT).await?;
        let enable = (val & IoutLimitBits::EN.bits()) != 0;
        let code = val & 0x7F;
        Ok((code_to_ilim_ma(code), enable))
    }

    pub async fn set_vout_sr_async(&mut self, slew: VoutSlewRate, ocp_delay: OcpDelay) -> Result<(), Error<I2C::Error>> {
        let mut bits = VoutSrBits::empty();
        bits |= match slew {
            VoutSlewRate::Sr1p25MvPerUs => VoutSrBits::empty(),
            VoutSlewRate::Sr2p5MvPerUs => VoutSrBits::SR0,
            VoutSlewRate::Sr5MvPerUs => VoutSrBits::SR1,
            VoutSlewRate::Sr10MvPerUs => VoutSrBits::SR0 | VoutSrBits::SR1,
        };
        bits |= match ocp_delay {
            OcpDelay::Us128 => VoutSrBits::empty(),
            OcpDelay::Ms3_072 => VoutSrBits::OCP_DELAY0,
            OcpDelay::Ms6_144 => VoutSrBits::OCP_DELAY1,
            OcpDelay::Ms12_288 => VoutSrBits::OCP_DELAY0 | VoutSrBits::OCP_DELAY1,
        };
        self.write_reg_async(addr::VOUT_SR, bits.bits()).await
    }

    pub async fn set_feedback_async(
        &mut self,
        source: FeedbackSource,
        ratio: InternalFeedbackRatio,
    ) -> Result<(), Error<I2C::Error>> {
        let mut bits = VoutFsBits::empty();
        if matches!(source, FeedbackSource::External) {
            bits |= VoutFsBits::FB_EXT;
        }
        bits |= match ratio {
            InternalFeedbackRatio::R0_2256 => VoutFsBits::empty(),
            InternalFeedbackRatio::R0_1128 => VoutFsBits::INTFB0,
            InternalFeedbackRatio::R0_0752 => VoutFsBits::INTFB1,
            InternalFeedbackRatio::R0_0564 => VoutFsBits::INTFB0 | VoutFsBits::INTFB1,
        };
        self.write_reg_async(addr::VOUT_FS, bits.bits()).await
    }

    pub async fn set_cable_comp_async(
        &mut self,
        option: CableCompOption,
        level: CableCompLevel,
        mask_sc: bool,
        mask_ocp: bool,
        mask_ovp: bool,
    ) -> Result<(), Error<I2C::Error>> {
        let mut bits = CdcBits::empty();
        if mask_sc {
            bits |= CdcBits::SC_MASK;
        }
        if mask_ocp {
            bits |= CdcBits::OCP_MASK;
        }
        if mask_ovp {
            bits |= CdcBits::OVP_MASK;
        }
        if matches!(option, CableCompOption::External) {
            bits |= CdcBits::CDC_OPT;
        }
        let level_bits = match level {
            CableCompLevel::V0p0 => CdcBits::empty(),
            CableCompLevel::V0p1 => CdcBits::CDC0,
            CableCompLevel::V0p2 => CdcBits::CDC1,
            CableCompLevel::V0p3 => CdcBits::CDC0 | CdcBits::CDC1,
            CableCompLevel::V0p4 => CdcBits::CDC2,
            CableCompLevel::V0p5 => CdcBits::CDC2 | CdcBits::CDC0,
            CableCompLevel::V0p6 => CdcBits::CDC2 | CdcBits::CDC1,
            CableCompLevel::V0p7 => CdcBits::CDC2 | CdcBits::CDC1 | CdcBits::CDC0,
        };
        bits |= level_bits;
        self.write_reg_async(addr::CDC, bits.bits()).await
    }

    pub async fn read_status_raw_async(&mut self) -> Result<StatusBits, Error<I2C::Error>> {
        let val = self.read_reg_async(addr::STATUS).await?;
        Ok(StatusBits::from_bits_truncate(val))
    }

    pub async fn read_status_async(&mut self) -> Result<(OperatingStatus, FaultStatus), Error<I2C::Error>> {
        let bits = self.read_status_raw_async().await?;
        let mode_bits = decode_status_mode(&bits);
        let operating = match mode_bits {
            0b00 => OperatingStatus::Boost,
            0b01 => OperatingStatus::Buck,
            0b10 => OperatingStatus::BuckBoost,
            _ => OperatingStatus::Reserved,
        };
        let faults = FaultStatus {
            short_circuit: bits.contains(StatusBits::SCP),
            over_current: bits.contains(StatusBits::OCP),
            over_voltage: bits.contains(StatusBits::OVP),
        };
        Ok((operating, faults))
    }
}
