use embedded_hal::{
    blocking::spi::{Transfer, Write},
    digital::v2::OutputPin,
};

use crate::command::Command;

const SECTOR_SIZE: u32 = 0x1000;
const PAGE_SIZE: u32 = 0x100;
const BLOCK_SIZE: u32 = 0x010000;

pub struct Address {
    pub sector: u16,
    pub page: u8,
}

impl Address {
    pub fn new(sector: u16, page: u8) -> Self {
        Self { sector, page }
    }
}

impl From<Address> for u32 {
    fn from(addr: Address) -> u32 {
        addr.sector as u32 * SECTOR_SIZE + addr.page as u32 * PAGE_SIZE
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Error<SPI: Transfer<u8>, GPIO: OutputPin> {
    /// An SPI transfer failed.
    Spi(SPI::Error),

    /// A GPIO could not be set.
    Gpio(GPIO::Error),
}

pub struct MX25R<SPI, CS>
where
    SPI: Transfer<u8>,
    CS: OutputPin,
{
    spi: SPI,
    cs: CS,
}

impl<SPI, CS> MX25R<SPI, CS>
where
    SPI: Transfer<u8>,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS) -> Self {
        Self { spi, cs }
    }

    fn command(&mut self, bytes: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.cs.set_low().map_err(Error::Gpio)?;
        let spi_result = self.spi.transfer(bytes).map_err(Error::Spi);
        self.cs.set_high().map_err(Error::Gpio)?;
        spi_result?;
        Ok(())
    }

    pub fn read(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        let addr_val: u32 = addr.into();

        self.cs.set_low().map_err(Error::Gpio)?;
        let mut cmd: [u8; 4] = [
            Command::Read as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        self.spi.transfer(&mut cmd).map_err(Error::Spi);
        self.spi.transfer(&mut buff).map_err(Error::Spi);
        self.cs.set_high().map_err(Error::Gpio)?;
        Ok(())
    }

    pub fn write_enable(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command(&[Command::WriteEnable as u8])
    }

    pub fn write_disable(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command(&[Command::Disable as u8])
    }
}
