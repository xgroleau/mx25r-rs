use core::fmt::Debug;

use embedded_storage_async::nor_flash::{NorFlashErrorKind, NorFlashError};

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

    /// The device is busy
    Busy,
}

impl<SpiError: Debug> NorFlashError for Error<SpiError> {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Error::Spi(_) => NorFlashErrorKind::Other,
            Error::Value => NorFlashErrorKind::Other,
            Error::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            Error::NotAligned => NorFlashErrorKind::NotAligned,
            Error::Busy => NorFlashErrorKind::Other,
        }
    }
}
