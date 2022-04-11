use crate::{
    address::{self, Address, Block32, Block64, Page, Sector},
    command::Command,
    register::*,
    error::Error,
};
use bit::BitIndex;
use embedded_hal::spi::blocking::{SpiBus, SpiBusRead, SpiBusWrite, SpiDevice};

/// Type alias for the MX25R512F
pub type MX25R512F<SPI, P> = MX25R<0x00FFFF, SPI, P>;

/// Type alias for the MX25R1035F
pub type MX25R1035F<SPI, P> = MX25R<0x01FFFF, SPI, P>;

/// Type alias for the MX25R2035F
pub type MX25R2035F<SPI, P> = MX25R<0x03FFFF, SPI, P>;

/// Type alias for the MX25R4035F
pub type MX25R4035F<SPI, P> = MX25R<0x07FFFF, SPI, P>;

/// Type alias for the MX25R8035F
pub type MX25R8035F<SPI, P> = MX25R<0x0FFFFF, SPI, P>;

/// Type alias for the MX25R1635F
pub type MX25R1635F<SPI, P> = MX25R<0x1FFFFF, SPI, P>;

/// Type alias for the MX25R3235F
pub type MX25R3235F<SPI, P> = MX25R<0x3FFFFF, SPI, P>;

/// Type alias for the MX25R6435F
pub type MX25R6435F<SPI, P> = MX25R<0x7FFFFF, SPI, P>;

/// The generic low level MX25R driver
pub struct MX25RLowLevel<const SIZE: u32, SPI>
where
    SPI: SpiDevice,
    SPI::Bus: SpiBus,
{
    spi: SPI,
}

impl<const SIZE: u32, SPI, E> MX25RLowLevel<SIZE, SPI>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub fn verify_addr(addr: Address) -> Result<u32, Error<E>> {
        let val: u32 = addr.into();
        if val > SIZE {
            return Err(Error::OutOfBounds);
        }
        Ok(val)
    }

    fn command_write(&mut self, bytes: &[u8]) -> Result<(), Error<E>> {
        self.spi
            .transaction(|bus| bus.write(bytes))
            .map_err(Error::Spi)
    }

    fn command_transfer(&mut self, bytes: &mut [u8]) -> Result<(), Error<E>> {
        self.spi
            .transaction(|bus| bus.transfer_in_place(bytes))
            .map_err(Error::Spi)
    }

    fn addr_command(&mut self, addr: Address, cmd: Command) -> Result<(), Error<E>> {
        let addr_val: u32 = Self::verify_addr(addr)?;
        let cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        self.spi
            .transaction(|bus| bus.write(&cmd))
            .map_err(Error::Spi)
    }

    fn read_base(&mut self, addr: Address, cmd: Command, buff: &mut [u8]) -> Result<(), Error<E>> {
        let addr_val: u32 = Self::verify_addr(addr)?;
        let cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];

        self.spi
            .transaction(|bus| {
                bus.write(&cmd)?;
                bus.read(buff)
            })
            .map_err(Error::Spi)
    }

    fn read_base_dummy(
        &mut self,
        addr: Address,
        cmd: Command,
        buff: &mut [u8],
    ) -> Result<(), Error<E>> {
        let addr_val: u32 = Self::verify_addr(addr)?;

        let cmd: [u8; 5] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
            Command::Dummy as u8,
        ];

        self.spi
            .transaction(|bus| {
                bus.write(&cmd)?;
                bus.read(buff)
            })
            .map_err(Error::Spi)
    }

    fn write_base(&mut self, addr: Address, cmd: Command, buff: &[u8]) -> Result<(), Error<E>> {
        let addr_val: u32 = Self::verify_addr(addr)?;
        let cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];

        self.spi
            .transaction(|bus| {
                bus.write(&cmd)?;
                bus.write(buff)
            })
            .map_err(Error::Spi)
    }

    /// Read n bytes from an addresss
    pub fn read(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.read_base(addr, Command::Read, buff)
    }

    /// Read n bytes quickly from an address
    pub fn read_fast(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.read_base_dummy(addr, Command::ReadF, buff)
    }

    /// Write n bytes to a page. Write must be enabled, see `write_enable`
    pub fn write_page(&mut self, sector: Sector, page: Page, buff: &[u8]) -> Result<(), Error<E>> {
        let addr = Address::from_page(sector, page);
        self.write_base(addr, Command::ProgramPage, buff)
    }

    /// Erase a 4kB sector. Write must be enabled, see `write_enable`
    pub fn sector_erase(&mut self, sector: Sector) -> Result<(), Error<E>> {
        let addr = Address::from_sector(sector);
        self.addr_command(addr, Command::SectorErase)
    }

    /// Erase a 64kB block. Write must be enabled, see `write_enable`
    pub fn block64_erase(&mut self, block: Block64) -> Result<(), Error<E>> {
        let addr = Address::from_block64(block);
        self.addr_command(addr, Command::BlockErase)
    }

    /// Erase a 32kB block. Write must be enabled, see `write_enable`
    pub fn block32_erase(&mut self, block: Block32) -> Result<(), Error<E>> {
        let addr = Address::from_block32(block);
        self.addr_command(addr, Command::BlockErase32)
    }

    /// Erase the whole chip. Write must be enabled, see `write_enable`
    pub fn chip_erase(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ChipErase as u8])
    }

    /// Read using the Serial Flash Discoverable Parameter instruction
    pub fn read_sfdp(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.read_base_dummy(addr, Command::ReadSfdp, buff)
    }

    /// Enable write operation
    pub fn write_enable(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::WriteEnable as u8])
    }

    /// Disable write
    pub fn write_disable(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::WriteDisable as u8])
    }

    /// Read the status register
    pub fn read_status(&mut self) -> Result<StatusRegister, Error<E>> {
        let mut command: [u8; 2] = [Command::ReadStatus as u8, 0];

        self.command_transfer(&mut command)?;
        Ok(command[1].into())
    }

    /// Read the configuration register
    pub fn read_configuration(&mut self) -> Result<ConfigurationRegister, Error<E>> {
        let mut command: [u8; 3] = [Command::ReadConfig as u8, 0, 0];
        self.command_transfer(&mut command)?;
        Ok(ConfigurationRegister {
            dummmy_cycle: command[1].bit(6),
            protected_section: command[1].bit(3).into(),
            power_mode: command[2].bit(1).into(),
        })
    }

    /// Write configuration to the configuration register
    pub fn write_configuration(
        &mut self,
        block_protected: u8,
        quad_enable: bool,
        status_write_disable: bool,
        dummy_cycle: bool,
        protected_section: ProtectedArea,
        power_mode: PowerMode,
    ) -> Result<(), Error<E>> {
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

    /// Suspend the pogram erase
    pub fn suspend_program_erase(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ProgramEraseSuspend as u8])
    }

    /// Resume program erase
    pub fn resume_program_erase(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ProgramEraseResume as u8])
    }

    /// Deep powerdown the chip
    pub fn deep_power_down(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::DeepPowerDown as u8])
    }

    /// Set the burst length
    pub fn set_burst_length(&mut self, burst_length: u8) -> Result<(), Error<E>> {
        self.command_write(&[Command::SetBurstLength as u8, burst_length])
    }

    /// Read the identification of the device
    pub fn read_identification(
        &mut self,
    ) -> Result<(ManufacturerId, MemoryType, MemoryDensity), Error<E>> {
        let mut command = [Command::ReadIdentification as u8, 0, 0, 0];
        self.command_transfer(&mut command)?;
        Ok((
            ManufacturerId(command[1]),
            MemoryType(command[2]),
            MemoryDensity(command[3]),
        ))
    }

    /// Read the electronic signature of the device
    pub fn read_electronic_id(&mut self) -> Result<ElectronicId, Error<E>> {
        let dummy = Command::Dummy as u8;
        let mut command = [Command::ReadElectronicId as u8, dummy, dummy, dummy, 0];
        self.command_transfer(&mut command)?;
        Ok(ElectronicId(command[4]))
    }

    /// Read the manufacturer ID and the device ID
    pub fn read_manufacturer_id(&mut self) -> Result<(ManufacturerId, DeviceId), Error<E>> {
        let dummy = Command::Dummy as u8;
        let mut command = [Command::ReadManufacturerId as u8, dummy, dummy, 0x00, 0, 0];
        self.command_transfer(&mut command)?;
        Ok((ManufacturerId(command[4]), DeviceId(command[5])))
    }

    /// Enter to access additionnal 8kB of secured memory, 
    /// which is independent of the main array. Note that it cannot be updated once locked down. See [`write_security_register`]
    pub fn enter_secure_opt(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::EnterSecureOTP as u8])
    }

    /// Exit the secured OTP
    pub fn exit_secure_opt(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ExitSecureOTP as u8])
    }

    /// Read the security register
    pub fn read_security_register(&mut self) -> Result<SecurityRegister, Error<E>> {
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

    /// Write the security register, note that this operation is **NON REVERSIBLE**
    pub fn write_security_register(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::WriteSecurityRegister as u8])
    }

    /// No operation, can terminate a reset enabler
    pub fn nop(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::Nop as u8])
    }

    /// Enable reset
    pub fn reset_enable(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ResetEnable as u8])
    }

    /// Reset the chip, note that the chip must have reset enabled. See `reset_enable`
    pub fn reset(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ResetMemory as u8])
    }
}


// Type states chip write enabled
pub struct WriteEnabled;
pub struct WriteDisabled;

/// The generic higher level driver for the mx25r. Safer and check if the wip bit is on for each transaction
pub struct MX25R<const SIZE: u32, SPI, P>
where
    SPI: SpiDevice,
    SPI::Bus: SpiBus,
{
    mx25r_ll: MX25RLowLevel<SIZE, SPI>,
    _write_permission: P,

}

impl<const SIZE: u32, SPI, E, P> MX25R<SIZE, SPI, P>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
{
    /// Create a new instance
    pub fn new(mx25r_ll: MX25RLowLevel<SIZE, SPI>) -> MX25R<SIZE, SPI, WriteDisabled> {
        MX25R {
            mx25r_ll,
            _write_permission: WriteDisabled
        }
    }

    /// Read n bytes quickly from an address
    pub fn read(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.mx25r_ll.read_fast(addr, buff)
    }

    /// Check if the chip has work in progess
    pub fn poll_wip(&mut self) -> Result<(), Error<E>> {
        if self.mx25r_ll.read_status()?.wip_bit {
            return Err(Error::Busy);
        }
        Ok(())
    } 
}

impl<const SIZE: u32, SPI, E> MX25R<SIZE, SPI, WriteDisabled>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
{
    pub fn enable_write(mut self) -> Result<MX25R<SIZE, SPI, WriteEnabled>, Error<E>> {
        self.mx25r_ll.write_enable()?;
        Ok(MX25R {
            mx25r_ll: self.mx25r_ll,
            _write_permission: WriteEnabled
        })
    }
}

impl<const SIZE: u32, SPI, E> MX25R<SIZE, SPI, WriteEnabled>
where
    SPI: SpiDevice<Error = E>,
    SPI::Bus: SpiBus,
{
    pub fn disabe_write(mut self) -> Result<MX25R<SIZE, SPI, WriteDisabled>, Error<E>> {
        self.mx25r_ll.write_disable()?;
        Ok(MX25R {
            mx25r_ll: self.mx25r_ll,
            _write_permission: WriteDisabled
        })
    }

    /// Write n bytes to a page
    pub fn write_page(&mut self, sector: Sector, page: Page, buff: &[u8]) -> Result<(), Error<E>> {
        self.poll_wip()?;
        self.mx25r_ll.write_page(sector, page, buff)
    }

    /// Erase a 4kB sector
    pub fn sector_erase(&mut self, sector: Sector) -> Result<(), Error<E>> {
        self.poll_wip()?;
        self.mx25r_ll.sector_erase(sector)
    }

    /// Erase a 64kB block
    pub fn block64_erase(&mut self, block: Block64) -> Result<(), Error<E>> {
        self.poll_wip()?;
        self.mx25r_ll.block64_erase(block)
    }

    /// Erase a 32kB block
    pub fn block32_erase(&mut self, block: Block32) -> Result<(), Error<E>> {
        self.poll_wip()?;
        self.mx25r_ll.block32_erase(block)
    }

    /// Erase the whole chip
    pub fn chip_erase(&mut self) -> Result<(), Error<E>> {
        self.poll_wip()?;
        self.mx25r_ll.chip_erase()
    } 

}

/// Implementation of the `NorFlash` trait of the `embedded_storage` crate. Note that you should enable write on the chip before using those traits.
/// Or write a wrapper using typestates.
mod es {
    use crate::address::{BLOCK32_SIZE, BLOCK64_SIZE, PAGE_SIZE, SECTOR_SIZE};

    use super::*;
    use core::fmt::Debug;
    use embedded_storage::nor_flash::{
        check_erase, check_read, check_write, ErrorType, MultiwriteNorFlash, NorFlash,
        NorFlashError, NorFlashErrorKind, ReadNorFlash,
    };

    impl<E> From<NorFlashErrorKind> for Error<E> {
        fn from(e: NorFlashErrorKind) -> Self {
            match e {
                NorFlashErrorKind::NotAligned => Error::NotAligned,
                NorFlashErrorKind::OutOfBounds => Error::OutOfBounds,
                _ => Error::Value,
            }
        }
    }

    impl<SpiError> NorFlashError for Error<SpiError>
    where
        SpiError: Debug,
    {
        fn kind(&self) -> NorFlashErrorKind {
            match self {
                Error::OutOfBounds => NorFlashErrorKind::OutOfBounds,
                Error::NotAligned => NorFlashErrorKind::NotAligned,
                Error::Value => NorFlashErrorKind::Other,
                Error::Spi(_) => NorFlashErrorKind::Other,
                Error::Busy => NorFlashErrorKind::Other,
            }
        }
    }

    impl<const SIZE: u32, SPI, E, P> ErrorType for MX25R<SIZE, SPI, P>
    where
        SPI: SpiDevice<Error = E>,
        SPI::Bus: SpiBus,
        E: Debug,
    {
        type Error = Error<E>;
    }

    impl<const SIZE: u32, SPI, E, P> ReadNorFlash for MX25R<SIZE, SPI, P>
    where
        SPI: SpiDevice<Error = E>,
        SPI::Bus: SpiBus,
        E: Debug,
    {
        const READ_SIZE: usize = 1;

        fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
            check_read(self, offset, bytes.len())?;
            if offset > SIZE {
                return Err(Error::OutOfBounds);
            }
            self.read(Address(offset), bytes)
        }

        fn capacity(&self) -> usize {
            SIZE as usize
        }
    }

    impl<const SIZE: u32, SPI, E> NorFlash for MX25R<SIZE, SPI, WriteEnabled>
    where
        SPI: SpiDevice<Error = E>,
        SPI::Bus: SpiBus,
        E: Debug,
    {
        const WRITE_SIZE: usize = address::PAGE_SIZE as usize;
        const ERASE_SIZE: usize = address::SECTOR_SIZE as usize;

        fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            check_erase(self, from, to)?;
            let addr_diff = from - to;
            match addr_diff {
                SECTOR_SIZE => {
                    let sector = Sector((from / SECTOR_SIZE) as u16);
                    self.sector_erase(sector)
                }
                BLOCK32_SIZE => {
                    let block = Block32((from / BLOCK32_SIZE) as u16);
                    self.block32_erase(block)
                }
                BLOCK64_SIZE => {
                    let block = Block64((from / BLOCK64_SIZE) as u16);
                    self.block64_erase(block)
                }
                _ => Err(Error::NotAligned),
            }
        }

        fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
            check_write(self, offset, bytes.len())?;
            let sector_id = (offset / SECTOR_SIZE) as u16;
            let page_id = (offset / PAGE_SIZE) as u8;
            self.write_page(sector_id.into(), page_id.into(), bytes)
        }
    }

    impl<const SIZE: u32, SPI, E> MultiwriteNorFlash for MX25R<SIZE, SPI, WriteEnabled>
    where
        SPI: SpiDevice<Error = E>,
        SPI::Bus: SpiBus,
        E: Debug,
    {
    }
}
