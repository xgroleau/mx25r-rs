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

pub struct ManufacturerId(u8);
pub struct MemoryType(u8);
pub struct MemoryDensity(u8);
pub struct ElectronicId(u8);
pub struct DeviceId(u8);

pub struct SecurityRegister {
    erase_failed: bool,
    program_failed: bool,
    erase_suspended: bool,
    program_suspended: bool,
    locked_down: bool,
    secured_otp: bool,
}

pub enum Error<SPI, GPIO>
where
    SPI: Transfer<u8> + Write<u8>,
    GPIO: OutputPin,
{
    /// An SPI transfer failed.
    Transfer(<SPI as Transfer<u8>>::Error),
    Write(<SPI as Write<u8>>::Error),

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
        let spi_result = self.spi.write(bytes).map_err(Error::Write);
        self.cs.set_high().map_err(Error::Gpio)?;
        spi_result?;
        Ok(())
    }

    fn command_transfer(&mut self, bytes: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.cs.set_low().map_err(Error::Gpio)?;
        let spi_result = self.spi.transfer(bytes).map_err(Error::Transfer);
        self.cs.set_high().map_err(Error::Gpio)?;
        spi_result?;
        Ok(())
    }

    fn addr_command(&mut self, addr: Address, cmd: Command) -> Result<(), Error<SPI, CS>> {
        let addr_val: u32 = addr.into();
        self.cs.set_low().map_err(Error::Gpio)?;
        let mut cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        let res = self.spi.transfer(&mut cmd).map_err(Error::Transfer);
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
        let mut cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        let res1 = self.spi.transfer(&mut cmd).map_err(Error::Transfer);
        let res2 = self.spi.transfer(buff).map_err(Error::Transfer);
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
        let mut cmd: [u8; 5] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
            DUMMY,
        ];
        let res1 = self.spi.transfer(&mut cmd).map_err(Error::Transfer);
        let res2 = self.spi.transfer(buff).map_err(Error::Transfer);
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
        let mut cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        let res1 = self.spi.transfer(&mut cmd).map_err(Error::Transfer);
        let res2 = self.spi.write(buff).map_err(Error::Write);
        self.cs.set_high().map_err(Error::Gpio)?;

        res1?;
        res2?;
        Ok(())
    }

    pub fn read(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base(addr, Command::Read, buff)
    }

    pub fn read_fast(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::ReadF, buff)
    }

    pub fn read_2io(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::Read2, buff)
    }

    pub fn read_1i2o(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::ReadD, buff)
    }

    pub fn read_4io(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::Read4, buff)
    }

    pub fn read_1i4o(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::ReadQ, buff)
    }

    pub fn write_page(&mut self, addr: Address, buff: &[u8]) -> Result<(), Error<SPI, CS>> {
        self.write_base(addr, Command::ProgramPage, buff)
    }

    pub fn write_page_quad(&mut self, addr: Address, buff: &[u8]) -> Result<(), Error<SPI, CS>> {
        self.write_base(addr, Command::ProgramPage4, buff)
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
        self.addr_command(addr, Command::BlockErase32)
    }

    pub fn chip_erase(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::ChipErase as u8])
    }

    pub fn read_sfpd(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<SPI, CS>> {
        self.read_base_dummy(addr, Command::ReadSfdp, buff)
    }

    pub fn write_enable(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::WriteEnable as u8])
    }

    pub fn write_disable(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::WriteDisable as u8])
    }

    pub fn read_status(&mut self) -> Result<StatusRegister, Error<SPI, CS>> {
        let mut command: [u8; 2] = [Command::ReadStatus as u8, 0];

        self.command_transfer(&mut command)?;
        return Ok(command[1].into());
    }

    pub fn read_configuration(&mut self) -> Result<ConfigurationRegister, Error<SPI, CS>> {
        let mut command: [u8; 3] = [Command::ReadConfig as u8, 0, 0];
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

        let mut command: [u8; 4] = [Command::WriteStatus as u8, 0, 0, 0];
        command[1].set_bit_range(2..6, block_protected);
        command[1].set_bit(6, quad_enable);
        command[1].set_bit(7, status_write_disable);
        command[2].set_bit(3, protected_section.into());
        command[2].set_bit(6, dummy_cycle);
        command[3].set_bit(1, power_mode.into());
        Ok(())
    }

    pub fn suspend_program_erase(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::ProgramEraseSuspend as u8])
    }

    pub fn resume_program_erase(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::ProgramEraseResume as u8])
    }

    pub fn deep_power_down(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::DeepPowerDown as u8])
    }

    pub fn set_burst_length(&mut self, burst_length: u8) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::SetBurstLength as u8, burst_length])
    }

    pub fn read_identification(
        &mut self,
    ) -> Result<(ManufacturerId, MemoryType, MemoryDensity), Error<SPI, CS>> {
        let mut command = [Command::ReadIdentification as u8, 0, 0, 0];
        self.command_transfer(&mut command)?;
        Ok((
            ManufacturerId(command[1]),
            MemoryType(command[2]),
            MemoryDensity(command[3]),
        ))
    }

    pub fn read_electronic_id(&mut self) -> Result<ElectronicId, Error<SPI, CS>> {
        let mut command = [Command::ReadElectronicId as u8, DUMMY, DUMMY, DUMMY, 0];
        self.command_transfer(&mut command)?;
        Ok(ElectronicId(command[4]))
    }

    pub fn read_manufacturer_id(&mut self) -> Result<(ManufacturerId, DeviceId), Error<SPI, CS>> {
        let mut command = [Command::ReadManufacturerId as u8, DUMMY, DUMMY, 0x00, 0, 0];
        self.command_transfer(&mut command)?;
        Ok((ManufacturerId(command[4]), DeviceId(command[5])))
    }

    pub fn enter_secure_opt(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::EnterSecureOTP as u8])
    }

    pub fn exit_secure_opt(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::ExitSecureOTP as u8])
    }

    pub fn read_security_register(&mut self) -> Result<SecurityRegister, Error<SPI, CS>> {
        let mut command = [Command::ReadSecurityRegister as u8, 0];
        self.command_transfer(&mut command)?;
        Ok(SecurityRegister {
            erase_failed: command[1].bit(6),
            program_failed: command[1].bit(5),
            erase_suspended: command[1].bit(3),
            program_suspended: command[1].bit(2),
            locked_down: command[1].bit(1),
            secured_otp: command[1].bit(0),
        })
    }

    // TODO: Check the right way to put a warning
    #[deprecated(
        note = "Warning: This function will lock your security register, make sure you understand the implications"
    )]
    pub fn write_security_register(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::WriteSecurityRegister as u8])
    }

    pub fn nop(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::NOP as u8])
    }

    pub fn reset_enable(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::ResetEnable as u8])
    }

    pub fn reset(&mut self) -> Result<(), Error<SPI, CS>> {
        self.command_write(&[Command::ResetMemory as u8])
    }
}
