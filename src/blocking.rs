use crate::{
    command::Command,
    error::Error,
    register::*,
    {BLOCK64_SIZE, SECTOR_SIZE},
};
use bit::BitIndex;
use embedded_hal::spi::Operation;
use embedded_hal::spi::SpiDevice;

/// Type alias for the MX25R512F
pub type MX25R512F<SPI> = MX25R<0x00FFFF, SPI>;

/// Type alias for the MX25R1035F
pub type MX25R1035F<SPI> = MX25R<0x01FFFF, SPI>;

/// Type alias for the MX25R2035F
pub type MX25R2035F<SPI> = MX25R<0x03FFFF, SPI>;

/// Type alias for the MX25R4035F
pub type MX25R4035F<SPI> = MX25R<0x07FFFF, SPI>;

/// Type alias for the MX25R8035F
pub type MX25R8035F<SPI> = MX25R<0x0FFFFF, SPI>;

/// Type alias for the MX25R1635F
pub type MX25R1635F<SPI> = MX25R<0x1FFFFF, SPI>;

/// Type alias for the MX25R3235F
pub type MX25R3235F<SPI> = MX25R<0x3FFFFF, SPI>;

/// Type alias for the MX25R6435F
pub type MX25R6435F<SPI> = MX25R<0x7FFFFF, SPI>;

/// The generic low level MX25R driver
pub struct MX25R<const SIZE: u32, SPI>
where
    SPI: SpiDevice,
{
    spi: SPI,
}

impl<const SIZE: u32, SPI, E> MX25R<SIZE, SPI>
where
    SPI: SpiDevice<Error = E>,
{
    pub const CAPACITY: usize = SIZE as usize + 1;

    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Read the wip bit, just less noisy than the `read_status().unwrap().wip_bit`
    pub fn poll_wip(&mut self) -> Result<(), Error<E>> {
        if self.read_status()?.wip_bit {
            return Err(Error::Busy);
        }
        Ok(())
    }

    pub fn wait_wip(&mut self) -> Result<(), Error<E>> {
        loop {
            let res = self.poll_wip();
            match res {
                Ok(()) => return Ok(()),
                Err(Error::Busy) => continue,
                err @ Err(_) => return err,
            }
        }
    }

    pub fn verify_addr(addr: u32) -> Result<u32, Error<E>> {
        if addr > SIZE {
            return Err(Error::OutOfBounds);
        }
        Ok(addr)
    }

    fn command_write(&mut self, bytes: &[u8]) -> Result<(), Error<E>> {
        self.spi.write(bytes).map_err(Error::Spi)
    }
    fn command_transfer(&mut self, bytes: &mut [u8]) -> Result<(), Error<E>> {
        self.spi.transfer_in_place(bytes).map_err(Error::Spi)
    }

    fn addr_command(&mut self, addr: u32, cmd: Command) -> Result<(), Error<E>> {
        let addr_val = Self::verify_addr(addr)?;
        let cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        self.spi.write(&cmd).map_err(Error::Spi)
    }

    fn write_read_base(&mut self, write: &[u8], read: &mut [u8]) -> Result<(), Error<E>> {
        self.spi
            .transaction(&mut [Operation::Write(write), Operation::Read(read)])
            .map_err(Error::Spi)
    }

    fn read_base(&mut self, addr: u32, cmd: Command, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.wait_wip()?;
        let addr_val = Self::verify_addr(addr)?;
        let cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];

        let res = self.write_read_base(&cmd, buff);
        #[cfg(feature = "defmt")]
        if res.is_ok() {
            defmt::trace!("Read from {=u32}, {=usize}: {:?}", addr, buff.len(), buff);
        } else {
            defmt::trace!("Failed to read");
        }
        res
    }

    fn read_base_dummy(
        &mut self,
        addr: u32,
        cmd: Command,
        buff: &mut [u8],
    ) -> Result<(), Error<E>> {
        let addr_val = Self::verify_addr(addr)?;
        self.wait_wip()?;

        let cmd: [u8; 5] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
            Command::Dummy as u8,
        ];
        let res = self.write_read_base(&cmd, buff);
        #[cfg(feature = "defmt")]
        if res.is_ok() {
            defmt::trace!("Read from {=u32}, {=usize}: {:?}", addr, buff.len(), buff);
        } else {
            defmt::trace!("Failed to read");
        }
        res
    }

    fn write_base(&mut self, addr: u32, cmd: Command, buff: &[u8]) -> Result<(), Error<E>> {
        let addr_val: u32 = Self::verify_addr(addr)?;
        let cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];

        let res = self
            .spi
            .transaction(&mut [Operation::Write(&cmd), Operation::Write(buff)])
            .map_err(Error::Spi);

        #[cfg(feature = "defmt")]
        if res.is_ok() {
            defmt::trace!("write from {=u32}, {=usize}: {:?}", addr, buff.len(), buff);
        } else {
            defmt::trace!("Failed to write");
        }
        res
    }

    fn prepare_write(&mut self) -> Result<(), Error<E>> {
        self.wait_wip()?;
        self.write_enable()
    }

    /// Read n bytes from an addresss, note that you should maybe use [`Self::read_fast`] instead
    pub fn read(&mut self, addr: u32, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.read_base(addr, Command::Read, buff)
    }

    /// Read n bytes quickly from an address
    pub fn read_fast(&mut self, addr: u32, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.read_base_dummy(addr, Command::ReadF, buff)
    }

    /// Write n bytes to a page. [`Self::write_enable`] is called internally
    pub fn write_page(&mut self, addr: u32, buff: &[u8]) -> Result<(), Error<E>> {
        self.prepare_write()?;
        self.write_base(addr, Command::ProgramPage, buff)
    }

    /// Erase a 4kB sector. [`Self::write_enable`] is called internally
    pub fn erase_sector(&mut self, addr: u32) -> Result<(), Error<E>> {
        if addr % SECTOR_SIZE != 0 {
            return Err(Error::NotAligned);
        }
        self.prepare_write()?;
        self.addr_command(addr, Command::SectorErase)?;
        #[cfg(feature = "defmt")]
        defmt::trace!("Erase sector {:?}", addr);
        Ok(())
    }

    /// Erase a 64kB block. [`Self::write_enable`] is called internally
    pub fn erase_block64(&mut self, addr: u32) -> Result<(), Error<E>> {
        if addr % BLOCK64_SIZE != 0 {
            return Err(Error::NotAligned);
        }
        self.prepare_write()?;
        self.addr_command(addr, Command::BlockErase)?;
        #[cfg(feature = "defmt")]
        defmt::trace!("Erase block 64 {:?}", addr);
        Ok(())
    }

    /// Erase a 32kB block. [`Self::write_enable`] is called internally
    pub fn erase_block32(&mut self, addr: u32) -> Result<(), Error<E>> {
        if addr % SECTOR_SIZE != 0 {
            return Err(Error::NotAligned);
        }
        self.prepare_write()?;
        self.addr_command(addr, Command::BlockErase32)?;
        #[cfg(feature = "defmt")]
        defmt::trace!("Erase block 32 {:?}", addr);
        Ok(())
    }

    /// Erase the whole chip. [`Self::write_enable`] is called internally
    pub fn erase_chip(&mut self) -> Result<(), Error<E>> {
        self.prepare_write()?;
        self.command_write(&[Command::ChipErase as u8])?;
        #[cfg(feature = "defmt")]
        defmt::trace!("Erase chip");
        Ok(())
    }

    /// Read using the Serial Flash Discoverable Parameter instruction
    pub fn read_sfdp(&mut self, addr: u32, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.read_base_dummy(addr, Command::ReadSfdp, buff)
    }

    /// Enable write operation, though you shouldn't need this function since it's already handled in the write/erase operations.
    fn write_enable(&mut self) -> Result<(), Error<E>> {
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

    /// Write configuration to the configuration register. [`Self::write_enable`] is called internally
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
        self.prepare_write()?;
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
    /// which is independent of the main array. Note that it cannot be updated once locked down. See [`Self::write_security_register`]
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

    /// Enable reset, though you shouldn't need this function since it's already handled in the reset operation.
    pub fn reset_enable(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ResetEnable as u8])
    }

    /// Reset the chip. [`Self::reset_enable`] is called internally
    pub fn reset(&mut self) -> Result<(), Error<E>> {
        self.reset_enable()?;
        self.command_write(&[Command::ResetMemory as u8])
    }
}

/// Implementation of the [`NorFlash`](embedded_storage::nor_flash) trait of the  crate
mod es {

    use crate::error::Error;
    use crate::{check_erase, check_write};
    use crate::{BLOCK32_SIZE, BLOCK64_SIZE, PAGE_SIZE, SECTOR_SIZE};
    use embedded_hal::spi::SpiDevice;
    use embedded_storage::nor_flash::{MultiwriteNorFlash, NorFlash, ReadNorFlash};

    use super::MX25R;

    impl<const SIZE: u32, SPI: SpiDevice> embedded_storage::nor_flash::ErrorType for MX25R<SIZE, SPI> {
        type Error = Error<SPI::Error>;
    }

    impl<const SIZE: u32, SPI: SpiDevice> ReadNorFlash for MX25R<SIZE, SPI> {
        const READ_SIZE: usize = 1;

        fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
            self.read_fast(offset, bytes)
        }

        fn capacity(&self) -> usize {
            Self::CAPACITY
        }
    }

    impl<const SIZE: u32, SPI: SpiDevice> NorFlash for MX25R<SIZE, SPI> {
        const WRITE_SIZE: usize = 1;
        const ERASE_SIZE: usize = SECTOR_SIZE as usize;

        fn erase(&mut self, mut from: u32, to: u32) -> Result<(), Self::Error> {
            check_erase(self.capacity(), from, to)?;

            while from < to {
                self.wait_wip()?;
                let addr_diff = to - from;
                match addr_diff {
                    SECTOR_SIZE => {
                        let sector = from / SECTOR_SIZE;
                        self.erase_sector(sector)
                    }
                    BLOCK32_SIZE => {
                        let block = from / BLOCK32_SIZE;
                        self.erase_block32(block)
                    }
                    BLOCK64_SIZE => {
                        let block = from / BLOCK64_SIZE;
                        self.erase_block64(block)
                    }
                    _ => Err(Error::NotAligned),
                }?;
                from += addr_diff;
            }
            Ok(())
        }

        fn write(&mut self, mut offset: u32, mut bytes: &[u8]) -> Result<(), Self::Error> {
            check_write(self.capacity(), offset, bytes.len())?;

            // Write first chunk, taking into account that given addres might
            // point to a location that is not on a page boundary,
            let chunk_len = (PAGE_SIZE - (offset & 0x000000FF)) as usize;
            let mut chunk_len = chunk_len.min(bytes.len());
            self.write_page(offset, &bytes[..chunk_len])?;

            loop {
                bytes = &bytes[chunk_len..];
                offset += chunk_len as u32;
                chunk_len = bytes.len().min(PAGE_SIZE as usize);
                if chunk_len == 0 {
                    break;
                }
                self.write_page(offset, &bytes[..chunk_len])?;
            }

            Ok(())
        }
    }

    impl<const SIZE: u32, SPI: SpiDevice> MultiwriteNorFlash for MX25R<SIZE, SPI> {}
}
