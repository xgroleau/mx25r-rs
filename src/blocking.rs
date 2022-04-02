use crate::{
    address::{Address, Block32, Block64, Sector},
    command::Command,
    register::*,
};
use bit::BitIndex;
use embedded_hal::{
    blocking::spi::{Transfer, Write},
    digital::v2::OutputPin,
};

pub type MX25R512F<SPI, CS> = MX25R<0x00FFFF, SPI, CS>;
pub type MX25R1035F<SPI, CS> = MX25R<0x01FFFF, SPI, CS>;
pub type MX25R2035F<SPI, CS> = MX25R<0x03FFFF, SPI, CS>;
pub type MX25R4035F<SPI, CS> = MX25R<0x07FFFF, SPI, CS>;
pub type MX25R8035F<SPI, CS> = MX25R<0x0FFFFF, SPI, CS>;
pub type MX25R1635F<SPI, CS> = MX25R<0x1FFFFF, SPI, CS>;
pub type MX25R3235F<SPI, CS> = MX25R<0x3FFFFF, SPI, CS>;
pub type MX25R6435F<SPI, CS> = MX25R<0x7FFFFF, SPI, CS>;

const DUMMY: u8 = 0xFF;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// An SPI transfer failed.
    Transfer,

    /// An SPI write failed.
    Write,

    /// A GPIO could not be set.
    Gpio,

    /// Invalid value
    Value,

    /// Invalid address
    Address,
}

pub struct MX25R<const SIZE: u32, SPI, CS>
where
    SPI: Transfer<u8> + Write<u8>,
    CS: OutputPin,
{
    spi: SPI,
    cs: CS,
}

impl<const SIZE: u32, SPI, CS> MX25R<SIZE, SPI, CS>
where
    SPI: Transfer<u8> + Write<u8>,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS) -> Self {
        Self { spi, cs }
    }

    fn verify_addr(addr: Address) -> Result<u32, Error> {
        let val: u32 = addr.into();
        if val > SIZE {
            return Err(Error::Address);
        }
        Ok(val)
    }

    fn command_write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.cs.set_low().map_err(|_| Error::Gpio)?;
        let spi_result = self.spi.write(bytes).map_err(|_| Error::Write);
        self.cs.set_high().map_err(|_| Error::Gpio)?;
        spi_result?;
        Ok(())
    }

    fn command_transfer(&mut self, bytes: &mut [u8]) -> Result<(), Error> {
        self.cs.set_low().map_err(|_| Error::Gpio)?;
        let spi_result = self.spi.transfer(bytes).map_err(|_| Error::Transfer);
        self.cs.set_high().map_err(|_| Error::Gpio)?;
        spi_result?;
        Ok(())
    }

    fn addr_command(&mut self, addr: Address, cmd: Command) -> Result<(), Error> {
        let addr_val: u32 = Self::verify_addr(addr)?;
        self.cs.set_low().map_err(|_| Error::Gpio)?;
        let mut cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        let res = self.spi.transfer(&mut cmd).map_err(|_| Error::Transfer);
        self.cs.set_high().map_err(|_| Error::Gpio)?;

        res?;
        Ok(())
    }

    fn read_base(&mut self, addr: Address, cmd: Command, buff: &mut [u8]) -> Result<(), Error> {
        let addr_val: u32 = Self::verify_addr(addr)?;

        self.cs.set_low().map_err(|_| Error::Gpio)?;
        let mut cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        let cmd_res = self.spi.transfer(&mut cmd).map_err(|_| Error::Transfer);
        let transfer_res = self.spi.transfer(buff).map_err(|_| Error::Transfer);
        self.cs.set_high().map_err(|_| Error::Gpio)?;

        cmd_res?;
        transfer_res?;
        Ok(())
    }

    fn read_base_dummy(
        &mut self,
        addr: Address,
        cmd: Command,
        buff: &mut [u8],
    ) -> Result<(), Error> {
        let addr_val: u32 = Self::verify_addr(addr)?;

        self.cs.set_low().map_err(|_| Error::Gpio)?;
        let mut cmd: [u8; 5] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
            DUMMY,
        ];
        let cmd_res = self.spi.transfer(&mut cmd).map_err(|_| Error::Transfer);
        let transfer_res = self.spi.transfer(buff).map_err(|_| Error::Transfer);
        self.cs.set_high().map_err(|_| Error::Gpio)?;

        cmd_res?;
        transfer_res?;
        Ok(())
    }

    fn write_base(&mut self, addr: Address, cmd: Command, buff: &[u8]) -> Result<(), Error> {
        let addr_val: u32 = Self::verify_addr(addr)?;

        self.cs.set_low().map_err(|_| Error::Gpio)?;
        let mut cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        let cmd_res = self.spi.transfer(&mut cmd).map_err(|_| Error::Transfer);
        let transfer_res = self.spi.write(buff).map_err(|_| Error::Write);
        self.cs.set_high().map_err(|_| Error::Gpio)?;

        cmd_res?;
        transfer_res?;
        Ok(())
    }

    pub fn read(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error> {
        self.read_base(addr, Command::Read, buff)
    }

    pub fn read_fast(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error> {
        self.read_base_dummy(addr, Command::ReadF, buff)
    }

    pub fn read_2io(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error> {
        self.read_base_dummy(addr, Command::Read2, buff)
    }

    pub fn read_1i2o(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error> {
        self.read_base_dummy(addr, Command::ReadD, buff)
    }

    pub fn read_4io(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error> {
        self.read_base_dummy(addr, Command::Read4, buff)
    }

    pub fn read_1i4o(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error> {
        self.read_base_dummy(addr, Command::ReadQ, buff)
    }

    pub fn write_page(&mut self, addr: Address, buff: &[u8]) -> Result<(), Error> {
        self.write_base(addr, Command::ProgramPage, buff)
    }

    pub fn write_page_quad(&mut self, addr: Address, buff: &[u8]) -> Result<(), Error> {
        self.write_base(addr, Command::ProgramPage4, buff)
    }

    pub fn sector_erase(&mut self, sector: Sector) -> Result<(), Error> {
        let addr = Address::from_sector(sector);
        self.addr_command(addr, Command::SectorErase)
    }

    pub fn block_erase(&mut self, block: Block64) -> Result<(), Error> {
        let addr = Address::from_block64(block);
        self.addr_command(addr, Command::BlockErase)
    }

    pub fn block_erase_32(&mut self, block: Block32) -> Result<(), Error> {
        let addr = Address::from_block32(block);
        self.addr_command(addr, Command::BlockErase32)
    }

    pub fn chip_erase(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::ChipErase as u8])
    }

    pub fn read_sfpd(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error> {
        self.read_base_dummy(addr, Command::ReadSfdp, buff)
    }

    pub fn write_enable(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::WriteEnable as u8])
    }

    pub fn write_disable(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::WriteDisable as u8])
    }

    pub fn read_status(&mut self) -> Result<StatusRegister, Error> {
        let mut command: [u8; 2] = [Command::ReadStatus as u8, 0];

        self.command_transfer(&mut command)?;
        Ok(command[1].into())
    }

    pub fn read_configuration(&mut self) -> Result<ConfigurationRegister, Error> {
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
    ) -> Result<(), Error> {
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
        self.command_write(&command)?;
        Ok(())
    }

    pub fn suspend_program_erase(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::ProgramEraseSuspend as u8])
    }

    pub fn resume_program_erase(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::ProgramEraseResume as u8])
    }

    pub fn deep_power_down(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::DeepPowerDown as u8])
    }

    pub fn set_burst_length(&mut self, burst_length: u8) -> Result<(), Error> {
        self.command_write(&[Command::SetBurstLength as u8, burst_length])
    }

    pub fn read_identification(
        &mut self,
    ) -> Result<(ManufacturerId, MemoryType, MemoryDensity), Error> {
        let mut command = [Command::ReadIdentification as u8, 0, 0, 0];
        self.command_transfer(&mut command)?;
        Ok((
            ManufacturerId(command[1]),
            MemoryType(command[2]),
            MemoryDensity(command[3]),
        ))
    }

    pub fn read_electronic_id(&mut self) -> Result<ElectronicId, Error> {
        let mut command = [Command::ReadElectronicId as u8, DUMMY, DUMMY, DUMMY, 0];
        self.command_transfer(&mut command)?;
        Ok(ElectronicId(command[4]))
    }

    pub fn read_manufacturer_id(&mut self) -> Result<(ManufacturerId, DeviceId), Error> {
        let mut command = [Command::ReadManufacturerId as u8, DUMMY, DUMMY, 0x00, 0, 0];
        self.command_transfer(&mut command)?;
        Ok((ManufacturerId(command[4]), DeviceId(command[5])))
    }

    pub fn enter_secure_opt(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::EnterSecureOTP as u8])
    }

    pub fn exit_secure_opt(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::ExitSecureOTP as u8])
    }

    pub fn read_security_register(&mut self) -> Result<SecurityRegister, Error> {
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
    pub fn write_security_register(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::WriteSecurityRegister as u8])
    }

    pub fn nop(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::Nop as u8])
    }

    pub fn reset_enable(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::ResetEnable as u8])
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        self.command_write(&[Command::ResetMemory as u8])
    }
}
