#![no_std]
//! This is a platform agnostic library for the Macronix MX25R NOR flash series using [embedded-hal](https://github.com/rust-embedded/embedded-hal).
//!
//! Multiple chips are supported:
//! * [MX25R512F](https://www.macronix.com/Lists/Datasheet/Attachments/7399/MX25R512F,%20Wide%20Range,%20512Kb,%20v1.3.pdf)
//! * [MX25R1035F](https://www.macronix.com/Lists/Datasheet/Attachments/7400/MX25R1035F,%20Wide%20Range,%201Mb,%20v1.4.pdf)
//! * [MX25R2035F](https://www.macronix.com/Lists/Datasheet/Attachments/7478/MX25R2035F,%20Wide%20Range,%202Mb,%20v1.6.pdf)
//! * [MX25R4035F](https://www.macronix.com/Lists/Datasheet/Attachments/7425/MX25R4035F,%20Wide%20Range,%204Mb,%20v1.4.pdf)
//! * [MX25R8035F](https://www.macronix.com/Lists/Datasheet/Attachments/7934/MX25R8035F,%20Wide%20Range,%208Mb,%20v1.6.pdf)
//! * [MX25R1635F](https://www.macronix.com/Lists/Datasheet/Attachments/7595/MX25R1635F,%20Wide%20Range,%2016Mb,%20v1.6.pdf)
//! * [MX25R3235F](https://www.macronix.com/Lists/Datasheet/Attachments/7966/MX25R3235F,%20Wide%20Range,%2032Mb,%20v1.8.pdf)
//! * [MX25R6435F](https://www.macronix.com/Lists/Datasheet/Attachments/7913/MX25R6435F,%20Wide%20Range,%2064Mb,%20v1.5.pdf)

pub mod asynchronous;
pub mod blocking;
mod command;
pub mod error;
pub mod register;

use crate::error::Error;

pub const BLOCK64_SIZE: u32 = 0x010000;
pub const BLOCK32_SIZE: u32 = BLOCK64_SIZE / 2;

pub const SECTOR_SIZE: u32 = 0x1000;
pub const PAGE_SIZE: u32 = 0x100;

pub(crate) fn check_erase<E>(capacity: usize, from: u32, to: u32) -> Result<(), Error<E>> {
    let capacity = capacity as u32;
    if from > to || to > capacity {
        return Err(Error::OutOfBounds);
    }
    if from % SECTOR_SIZE != 0 || to % SECTOR_SIZE != 0 {
        return Err(Error::NotAligned);
    }
    Ok(())
}

pub(crate) fn check_write<E>(capacity: usize, offset: u32, length: usize) -> Result<(), Error<E>> {
    let capacity = capacity as u32;
    let length = length as u32;
    if length > capacity || offset > capacity - length {
        return Err(Error::OutOfBounds);
    }
    Ok(())
}
