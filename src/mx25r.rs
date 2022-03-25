use embedded_hal::blocking::spi::{Transfer, Write};

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

impl From<Address> for [u8; 3] {
    fn from(addr: Address) -> [u8; 3] {
        let val: u32 = addr.into();
        let bytes: [u8; 4] = val.to_be_bytes();
        [bytes[0], bytes[1], bytes[2]]
    }
}

pub struct MX25R<SPI>
where
    SPI: Write<u8> + Transfer<u8>,
{
    spi: SPI,
}

impl<SPI> MX25R<SPI>
where
    SPI: Write<u8> + Transfer<u8>,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub fn read(&mut self, addr: Address, buf: &[u8]) {
        let addr_val: [u8; 3] = addr.into();
        let cmd: [u8; 5] = [
            Command::Read as u8,
            addr_val[0],
            addr_val[1],
            addr_val[2],
            0,
        ];
        self.spi.transfer(addr_val);
    }
}
