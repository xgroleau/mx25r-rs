use crate::{
    address::{Address, Block32, Block64, Page, Sector, PAGE_SIZE},
    command::Command,
    error::Error,
    register::*,
};
use bit::BitIndex;
use embassy_futures::yield_now;
use embedded_hal::spi::Operation;
use embedded_hal_async::spi::SpiDevice;
use embedded_storage_async::nor_flash::{MultiwriteNorFlash, NorFlash, ReadNorFlash};

/// Type alias for the AsyncMX25R512F
pub type AsyncMX25R512F<SPI> = AsyncMX25R<0x00FFFF, SPI>;

/// Type alias for the AsyncMX25R1035F
pub type AsyncMX25R1035F<SPI> = AsyncMX25R<0x01FFFF, SPI>;

/// Type alias for the AsyncMX25R2035F
pub type AsyncMX25R2035F<SPI> = AsyncMX25R<0x03FFFF, SPI>;

/// Type alias for the AsyncMX25R4035F
pub type AsyncMX25R4035F<SPI> = AsyncMX25R<0x07FFFF, SPI>;

/// Type alias for the AsyncMX25R8035F
pub type AsyncMX25R8035F<SPI> = AsyncMX25R<0x0FFFFF, SPI>;

/// Type alias for the AsyncMX25R1635F
pub type AsyncMX25R1635F<SPI> = AsyncMX25R<0x1FFFFF, SPI>;

/// Type alias for the AsyncMX25R3235F
pub type AsyncMX25R3235F<SPI> = AsyncMX25R<0x3FFFFF, SPI>;

/// Type alias for the AsyncMX25R6435F
pub type AsyncMX25R6435F<SPI> = AsyncMX25R<0x7FFFFF, SPI>;

/// The generic low level AsyncMX25R driver
pub struct AsyncMX25R<const SIZE: u32, SPI>
where
    SPI: SpiDevice,
{
    spi: SPI,
}

impl<const SIZE: u32, SPI, E> AsyncMX25R<SIZE, SPI>
where
    SPI: SpiDevice<Error = E>,
{
    pub const fn capacity() -> usize {
        SIZE as usize + 1
    }

    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub async fn wait_wip(&mut self) -> Result<(), Error<E>> {
        loop {
            let res = self.poll_wip().await;
            match res {
                Ok(()) => return Ok(()),
                Err(Error::Busy) => yield_now().await,
                err @ Err(_) => return err,
            }
        }
    }

    pub fn verify_addr(addr: Address) -> Result<u32, Error<E>> {
        let val: u32 = addr.into();
        if val > SIZE {
            return Err(Error::OutOfBounds);
        }
        Ok(val)
    }

    async fn command_write(&mut self, bytes: &[u8]) -> Result<(), Error<E>> {
        self.spi.write(bytes).await.map_err(Error::Spi)
    }
    async fn command_transfer(&mut self, bytes: &mut [u8]) -> Result<(), Error<E>> {
        self.spi.transfer_in_place(bytes).await.map_err(Error::Spi)
    }

    async fn addr_command(&mut self, addr: Address, cmd: Command) -> Result<(), Error<E>> {
        let addr_val: u32 = Self::verify_addr(addr)?;
        let cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];
        self.spi.write(&cmd).await.map_err(Error::Spi)
    }

    async fn write_read_base(&mut self, write: &[u8], read: &mut [u8]) -> Result<(), Error<E>> {
        self.spi.transaction(&mut [
            Operation::Write(write),
            Operation::Read(read),
        ]).await.map_err(Error::Spi)
    }

    async fn read_base(
        &mut self,
        addr: Address,
        cmd: Command,
        buff: &mut [u8],
    ) -> Result<(), Error<E>> {
        self.wait_wip().await?;
        let addr_val: u32 = Self::verify_addr(addr)?;
        let cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];

        let res = self.write_read_base(&cmd, buff).await;
        #[cfg(feature = "defmt")]
        if res.is_ok() {
            defmt::info!("Read from {=u32}, {=usize}: {:?}", addr.0, buff.len(), buff);
        } else {
            defmt::error!("ERROR READ");
        }
        res
    }

    async fn read_base_dummy(
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
        let res = self.write_read_base(&cmd, buff).await;
        #[cfg(feature = "defmt")]
        if res.is_ok() {
            defmt::trace!("ReadF from {=u32}, {=usize}: {:?}", addr.0, buff.len(), buff);
        } else {
            defmt::error!("ERROR READ");
        }
        res
    }

    async fn write_base(
        &mut self,
        addr: Address,
        cmd: Command,
        buff: &[u8],
    ) -> Result<(), Error<E>> {
        let addr_val: u32 = Self::verify_addr(addr)?;
        let cmd: [u8; 4] = [
            cmd as u8,
            (addr_val >> 16) as u8,
            (addr_val >> 8) as u8,
            addr_val as u8,
        ];

        self.spi.transaction(&mut [
            Operation::Write(&cmd),
            Operation::Write(buff),
        ]).await.map_err(Error::Spi)
    }

    async fn prepare_write(&mut self) -> Result<(), Error<E>> {
        self.poll_wip().await?;
        self.write_enable().await
    }

    /// Read n bytes from an addresss, note that you should maybe use [`Self::read_fast`] instead
    pub async fn read(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.read_base(addr, Command::Read, buff).await
    }

    /// Read n bytes quickly from an address
    pub async fn read_fast(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.read_base_dummy(addr, Command::ReadF, buff).await
    }

    /// Write n bytes to a page. [`Self::write_enable`] is called internally
    pub async fn write_page(
        &mut self,
        sector: Sector,
        page: Page,
        buff: &[u8],
    ) -> Result<(), Error<E>> {
        let addr = Address::from_page(sector, page);
        self.prepare_write().await?;
        self.write_base(addr, Command::ProgramPage, buff).await
    }

    /// Erase a 4kB sector. [`Self::write_enable`] is called internally
    pub async fn erase_sector(&mut self, sector: Sector) -> Result<(), Error<E>> {
        let addr = Address::from_sector(sector);
        self.prepare_write().await?;
        self.addr_command(addr, Command::SectorErase).await
    }

    /// Erase a 64kB block. [`Self::write_enable`] is called internally
    pub async fn erase_block64(&mut self, block: Block64) -> Result<(), Error<E>> {
        let addr = Address::from_block64(block);
        self.prepare_write().await?;
        self.addr_command(addr, Command::BlockErase).await
    }

    /// Erase a 32kB block. [`Self::write_enable`] is called internally
    pub async fn erase_block32(&mut self, block: Block32) -> Result<(), Error<E>> {
        let addr = Address::from_block32(block);
        self.prepare_write().await?;
        self.addr_command(addr, Command::BlockErase32).await
    }

    /// Erase the whole chip. [`Self::write_enable`] is called internally
    pub async fn erase_chip(&mut self) -> Result<(), Error<E>> {
        self.prepare_write().await?;
        self.command_write(&[Command::ChipErase as u8]).await
    }

    /// Read using the Serial Flash Discoverable Parameter instruction
    pub async fn read_sfdp(&mut self, addr: Address, buff: &mut [u8]) -> Result<(), Error<E>> {
        self.read_base_dummy(addr, Command::ReadSfdp, buff).await
    }

    /// Enable write operation, though you shouldn't need this function since it's already handled in the write/erase operations.
    async fn write_enable(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::WriteEnable as u8]).await
    }

    /// Disable write
    pub async fn write_disable(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::WriteDisable as u8]).await
    }

    /// Read the status register
    pub async fn read_status(&mut self) -> Result<StatusRegister, Error<E>> {
        let mut command: [u8; 2] = [Command::ReadStatus as u8, 0];

        self.command_transfer(&mut command).await?;
        Ok(command[1].into())
    }

    /// Read the wip bit, just less noisy than the `read_status().unwrap().wip_bit`
    pub async fn poll_wip(&mut self) -> Result<(), Error<E>> {
        if self.read_status().await?.wip_bit {
            return Err(Error::Busy);
        }
        Ok(())
    }

    /// Read the configuration register
    pub async fn read_configuration(&mut self) -> Result<ConfigurationRegister, Error<E>> {
        let mut command: [u8; 3] = [Command::ReadConfig as u8, 0, 0];
        self.command_transfer(&mut command).await?;
        Ok(ConfigurationRegister {
            dummmy_cycle: command[1].bit(6),
            protected_section: command[1].bit(3).into(),
            power_mode: command[2].bit(1).into(),
        })
    }

    /// Write configuration to the configuration register. [`Self::write_enable`] is called internally
    pub async fn write_configuration(
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
        self.prepare_write().await?;
        let mut command: [u8; 4] = [Command::WriteStatus as u8, 0, 0, 0];
        command[1].set_bit_range(2..6, block_protected);
        command[1].set_bit(6, quad_enable);
        command[1].set_bit(7, status_write_disable);
        command[2].set_bit(3, protected_section.into());
        command[2].set_bit(6, dummy_cycle);
        command[3].set_bit(1, power_mode.into());
        self.command_write(&command).await?;
        Ok(())
    }

    /// Suspend the pogram erase
    pub async fn suspend_program_erase(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ProgramEraseSuspend as u8])
            .await
    }

    /// Resume program erase
    pub async fn resume_program_erase(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ProgramEraseResume as u8])
            .await
    }

    /// Deep powerdown the chip
    pub async fn deep_power_down(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::DeepPowerDown as u8]).await
    }

    /// Set the burst length
    pub async fn set_burst_length(&mut self, burst_length: u8) -> Result<(), Error<E>> {
        self.command_write(&[Command::SetBurstLength as u8, burst_length])
            .await
    }

    /// Read the identification of the device
    pub async fn read_identification(
        &mut self,
    ) -> Result<(ManufacturerId, MemoryType, MemoryDensity), Error<E>> {
        let mut command = [Command::ReadIdentification as u8, 0, 0, 0];
        self.command_transfer(&mut command).await?;
        Ok((
            ManufacturerId(command[1]),
            MemoryType(command[2]),
            MemoryDensity(command[3]),
        ))
    }

    /// Read the electronic signature of the device
    pub async fn read_electronic_id(&mut self) -> Result<ElectronicId, Error<E>> {
        let dummy = Command::Dummy as u8;
        let mut command = [Command::ReadElectronicId as u8, dummy, dummy, dummy, 0];
        self.command_transfer(&mut command).await?;
        Ok(ElectronicId(command[4]))
    }

    /// Read the manufacturer ID and the device ID
    pub async fn read_manufacturer_id(&mut self) -> Result<(ManufacturerId, DeviceId), Error<E>> {
        let dummy = Command::Dummy as u8;
        let mut command = [Command::ReadManufacturerId as u8, dummy, dummy, 0x00, 0, 0];
        self.command_transfer(&mut command).await?;
        Ok((ManufacturerId(command[4]), DeviceId(command[5])))
    }

    /// Enter to access additionnal 8kB of secured memory,
    /// which is independent of the main array. Note that it cannot be updated once locked down. See [`Self::write_security_register`]
    pub async fn enter_secure_opt(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::EnterSecureOTP as u8]).await
    }

    /// Exit the secured OTP
    pub async fn exit_secure_opt(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ExitSecureOTP as u8]).await
    }

    /// Read the security register
    pub async fn read_security_register(&mut self) -> Result<SecurityRegister, Error<E>> {
        let mut command = [Command::ReadSecurityRegister as u8, 0];
        self.command_transfer(&mut command).await?;
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
    pub async fn write_security_register(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::WriteSecurityRegister as u8])
            .await
    }

    /// No operation, can terminate a reset enabler
    pub async fn nop(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::Nop as u8]).await
    }

    /// Enable reset, though you shouldn't need this function since it's already handled in the reset operation.
    pub async fn reset_enable(&mut self) -> Result<(), Error<E>> {
        self.command_write(&[Command::ResetEnable as u8]).await
    }

    /// Reset the chip. [`Self::reset_enable`] is called internally
    pub async fn reset(&mut self) -> Result<(), Error<E>> {
        self.reset_enable().await?;
        self.command_write(&[Command::ResetMemory as u8]).await
    }
}

impl<const SIZE: u32, SPI: SpiDevice> embedded_storage_async::nor_flash::ErrorType
    for AsyncMX25R<SIZE, SPI>
{
    type Error = Error<SPI::Error>;
}

impl<const SIZE: u32, SPI: SpiDevice> ReadNorFlash for AsyncMX25R<SIZE, SPI> {
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read_fast(Address(offset), bytes).await
    }

    fn capacity(&self) -> usize {
        Self::capacity()
    }
}

impl<const SIZE: u32, SPI: SpiDevice> NorFlash for AsyncMX25R<SIZE, SPI> {
    const WRITE_SIZE: usize = 1;

    const ERASE_SIZE: usize = 4096;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let erase_size = Self::ERASE_SIZE as u32;
        if from >= to {
            return Err(Error::OutOfBounds);
        }
        if (from & (erase_size - 1)) != 0 {
            return Err(Error::NotAligned);
        }

        let to_erase = to - from;
        if (to_erase & (erase_size - 1)) != 0 {
            return Err(Error::NotAligned);
        }

        // Make sure we are not busy
        self.wait_wip().await?;

        // todo: smarter erase that does larger aligned erases if possible
        let mut idx = from;
        while idx < to {
            let sector = idx / erase_size;
            let sector = sector as u16;
            #[cfg(feature = "defmt")]
            defmt::warn!("Erase sector {:?}", sector);
            self.erase_sector(Sector(sector)).await?;

            // Wait for the erase to complete, acting like a flush
            self.wait_wip().await?;

            idx += erase_size;
        }
        Ok(())
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let to_write = bytes.len();
        let to_write = to_write as u32;
        let Some(end) = offset.checked_add(to_write) else {
            return Err(Error::OutOfBounds);
        };
        if end > (Self::capacity() as u32) {
            return Err(Error::OutOfBounds);
        }

        let mut cursor = offset;
        let mut bytes = bytes;

        // Ensure we are not busy
        self.wait_wip().await?;

        while !bytes.is_empty() {
            // Is the START aligned to a page?
            let cursor_offset = cursor & (PAGE_SIZE - 1);
            let cursor_aligned = cursor_offset == 0;

            let address = Address(cursor);

            // Decide how much to send. If we are aligned, we can send up to a
            // whole page. If we are not aligned, we can only send the bytes remaining
            // in the current page.
            let to_send = if cursor_aligned {
                // Can we get a whole slice?
                if let Some((now, later)) = bytes.split_at_checked(PAGE_SIZE as usize) {
                    cursor += PAGE_SIZE;
                    bytes = later;
                    now
                } else {
                    // No, we have less than a page left.
                    cursor = end;
                    core::mem::take(&mut bytes)
                }
            } else {
                // we want to take the lesser of the rest of the size of the page,
                // and the remaining data
                let page_remain = PAGE_SIZE - cursor_offset;

                // Do we have a full remainder of the page?
                if let Some((now, later)) = bytes.split_at_checked(page_remain as usize) {
                    // yes!
                    cursor += now.len() as u32;
                    bytes = later;
                    now
                } else {
                    // No, we have less than a page left.
                    cursor = end;
                    core::mem::take(&mut bytes)
                }
            };

            #[cfg(feature = "defmt")]
            defmt::info!("Write to {=u32} len {=usize}: {:?}", address.0, to_send.len(), to_send);
            self.prepare_write().await?;
            self.write_base(address, Command::ProgramPage, to_send)
                .await?;

            // Wait for the write to complete, to behave like a flush
            self.wait_wip().await?;
        }
        // consumed all bytes to send
        Ok(())
    }
}

impl<const SIZE: u32, SPI: SpiDevice> MultiwriteNorFlash for AsyncMX25R<SIZE, SPI> {}
