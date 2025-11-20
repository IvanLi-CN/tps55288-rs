//! Error definitions for TPS55288 driver (placeholder).

/// Error type generic over I2C error.
#[derive(Debug)]
pub enum Error<I2cError> {
    /// Underlying I2C transaction failed.
    I2c(I2cError),
    /// Provided parameter was outside datasheet limits.
    OutOfRange,
    /// Unsupported/invalid configuration for current mode.
    InvalidConfig,
}
