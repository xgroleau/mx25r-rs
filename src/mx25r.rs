use bit::BitIndex;
use embedded_hal::{
    blocking::spi::{Transfer, Write},
    digital::v2::OutputPin,
};

use crate::command::Command;

const SECTOR_SIZE: u32 = 0x1000;
const PAGE_SIZE: u32 = 0x100;
const BLOCK_SIZE: u32 = 0x010000;
const DUMMY: u8 = 0xFF;

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

pub struct StatusRegister {
    pub write_protect_disable: bool,
    pub quad_enable: bool,
    pub protected_block: u8,
    pub write_enable_latch: bool,
    pub wip_bit: bool,
}

impl From<u8> for StatusRegister {
    fn from(val: u8) -> StatusRegister {
        StatusRegister {
            write_protect_disable: val.bit(7),
            quad_enable: val.bit(6),
            protected_block: val.bit_range(2..6),
            write_enable_latch: val.bit(1),
            wip_bit: val.bit(0),
        }
    }
}

pub enum ProtectedArea {
    Top,
    Bottom,
}
impl From<bool> for ProtectedArea {
    fn from(val: bool) -> Self {
        if val {
            ProtectedArea::Bottom
        } else {
            ProtectedArea::Top
        }
    }
}
impl From<ProtectedArea> for bool {
    fn from(val: ProtectedArea) -> Self {
        match val {
            ProtectedArea::Bottom => true,
            ProtectedArea::Top => false,
        }
    }
}

pub enum PowerMode {
    UltraLowPower,
    HighPerformance,
}
impl From<bool> for PowerMode {
    fn from(val: bool) -> Self {
        if val {
            PowerMode::HighPerformance
        } else {
            PowerMode::UltraLowPower
        }
    }
}
impl From<PowerMode> for bool {
    fn from(val: PowerMode) -> Self {
        match val {
            PowerMode::HighPerformance => true,
            PowerMode::UltraLowPower => false,
        }
    }
}

pub struct ConfigurationRegister {
    pub dummmy_cycle: bool,
    pub protected_section: ProtectedArea,
    pub power_mode: PowerMode,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Error<SPI, GPIO>
where
    SPI: Transfer<u8>,
    GPIO: OutputPin,
{
    /// An SPI transfer failed.
    Spi(SPI::Error),

    /// A GPIO could not be set.
    Gpio(GPIO::Error),

    /// Invalid value
    Value,
}

pub struct MX25R<SPI, CS>
where
    SPI: Transfer<u8> + Write<u8>,
    CS: OutputPin,
{
    spi: SPI,
    cs: CS,
}

impl<SPI, CS> MX25R<SPI, CS>
where
    SPI: Transfer<u8> + Write<u8>,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS) -> Self {
        Self { spi, cs }
    }

    fn command_write(&mut self, bytes: &[u8]) -> Result<(), Error<SPI, CS>> {
        self.cs.set_low().map_err(Error::Gpio)?;
        let spi_result = self.spi.write(bytes).map_err(Error::Spi);
        self.cs.set_high().map_err(Error::Gpio)?;
        spi_result?;
        Ok(())
    }

    fn command_transfer(&mut self, bytes: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.cs.set_low().map_err(Error::Gpio)?;
        let spi_result = self.spi.transfer(bytes).map_err(Error::Spi);
        self.cs.set_high().map_err(Error::Gpio)?;
        spi_result?;
        Ok(())
    }

    fn addr_command(&mut self, addr: Address, cmd: Command) -> Result<(), Error<SPI, CS>> {
        let addr_val: u32 = addr.into();
        self.cs.set_low().map_err(Error::Gpio)?;
        let mut cmd: [u8; 3] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        let res = self.spi.transfer(&mut cmd).map_err(Error::Spi);
        self.cs.set_high().map_err(Error::Gpio)?;

        res?;
        Ok(())
    }

    fn read_base(
        &mut self,
        addr: Address,
        cmd: Command,
        buff: &mut [u8],
    ) -> Result<(), Error<SPI, CS>> {
        let addr_val: u32 = addr.into();

        self.cs.set_low().map_err(Error::Gpio)?;
        let mut cmd: [u8; 3] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        let res1 = self.spi.transfer(&mut cmd).map_err(Error::Spi);
        let res2 = self.spi.transfer(&mut buff).map_err(Error::Spi);
        self.cs.set_high().map_err(Error::Gpio)?;

        res1?;
        res2?;
        Ok(())
    }

    fn read_base_dummy(
        &mut self,
        addr: Address,
        cmd: Command,
        buff: &mut [u8],
    ) -> Result<(), Error<SPI, CS>> {
        let addr_val: u32 = addr.into();

        self.cs.set_low().map_err(Error::Gpio)?;
        let mut cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
            DUMMY,
        ];
        let res1 = self.spi.transfer(&mut cmd).map_err(Error::Spi);
        let res2 = self.spi.transfer(&mut buff).map_err(Error::Spi);
        self.cs.set_high().map_err(Error::Gpio)?;

        res1?;
        res2?;
        Ok(())
    }

    fn write_base(
        &mut self,
        addr: Address,
        cmd: Command,
        buff: &[u8],
    ) -> Result<(), Error<SPI, CS>> {
        let addr_val: u32 = addr.into();

        self.cs.set_low().map_err(Error::Gpio)?;
        let mut cmd: [u8; 3] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        let res1 = self.spi.transfer(&mut cmd).map_err(Error::Spi);
        let res2 = self.spi.write(&buff).map_err(Error::Spi);
        self.cs.set_high().map_err(Error::Gpio)?;

        res1?;
        res2?;
        Ok(())
    }

    pub fn read(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base(addr, Command::Read, &mut buff)
    }

    pub fn read_fast(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::Read_F, &mut buff)
    }

    pub fn read_2io(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::Read_2, &mut buff)
    }

    pub fn read_1i2o(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::Read_D, &mut buff)
    }

    pub fn read_4io(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::Read_4, &mut buff)
    }

    pub fn read_1i4o(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::Read_Q, &mut buff)
    }

    pub fn write_page(&mut self, addr: Address, buff: &[u8]) -> Result<(), Error<SPI, CS>> {
        self.write_base(addr, Command::ProgramPage, &buff)
    }

    pub fn write_page_quad(&mut self, addr: Address, buff: &[u8]) -> Result<(), Error<SPI, CS>> {
        self.write_base(addr, Command::ProgrammPage_4, &buff)
    }

    pub fn sector_erase(&mut self, sector: u16) -> Result<(), Error<SPI, CS>> {
        let addr = Address::new(sector, 0);
        self.addr_command(addr, Command::SectorErase)
    }

    pub fn block_erase(&mut self, block: u16) -> Result<(), Error<SPI, CS>> {
        let sector = block as u32 * BLOCK_SIZE / SECTOR_SIZE;
        let addr = Address::new(sector as u16, 0);
        self.addr_command(addr, Command::BlockErase)
    }

    pub fn block_erase_32(&mut self, block: u16) -> Result<(), Error<SPI, CS>> {
        let sector = block as u32 * BLOCK_SIZE / SECTOR_SIZE / 2;
        let addr = Address::new(sector as u16, 0);
        self.addr_command(addr, Command::BlockErase_32)
    }

    pub fn chip_erase(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::ChipErase])
    }

    // TODO: pub fn read_sfpd(&mut self) -> Result<(), Error<SPI, CS>> {}

    pub fn write_enable(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::WriteEnable as u8])
    }

    pub fn write_disable(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::Disable as u8])
    }

    pub fn read_status(&mut self) -> Result<StatusRegister, Error<SPI, CS>> {
        let command: [u8; 2] = [Command::ReadStatus, 0];

        self.command_transfer(&mut command)?;
        return Ok(command[1].into());
    }

    pub fn read_configuration(&mut self) -> Result<ConfigurationRegister, Error<SPI, CS>> {
        let command: [u8; 3] = [Command::ReadConfig, 0, 0];
        self.command_transfer(&mut command)?;
        Ok(ConfigurationRegister {
            dummmy_cycle: command[1].bit(6),
            protected_section: command[1].bit(3).into(),
            power_mode: command[2].bit(1).into(),
        })
    }

    pub fn write_configuration(
        &mut self,
        block_protected: u8,
        quad_enable: bool,
        status_write_disable: bool,
        dummy_cycle: bool,
        protected_section: ProtectedArea,
        power_mode: PowerMode,
    ) -> Result<(), Error<SPI, CS>> {
        if block_protected > 0x0F {
            return Err(Error::Value);
        }

        let mut command: [u8; 4] = [Command::WriteStatus, 0, 0, 0];
        command[1].set_bit_range(2..6, block_protected);
        command[1].set_bit(6, status_write_disable);
        command[2].set_bit(6, dummy_cycle);
        command[2].set_bit(3, protected_section.into());
        command[3].set_bit(1, power_mode.into());
        Ok(())
    }
}
