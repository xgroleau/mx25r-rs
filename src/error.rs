/// All possible errors emitted by the driver
#[derive(Debug, Clone, Copy)]
pub enum Error<SpiError> {
    /// Internal Spi error
    Spi(SpiError),

    /// Invalid value passed
    Value,

    /// Address out of bound
    OutOfBounds,

    /// Address not aligned
    NotAligned,
}