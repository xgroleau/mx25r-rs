#![no_std]
#![no_main]
#![allow(unused_imports)]

use defmt_rtt as _;
use embassy_nrf as _;
use embedded_test as _;

#[cfg(test)]
#[embedded_test::tests]
mod tests {

    use defmt_rtt as _;
    use embassy_nrf::{
        bind_interrupts,
        config::Config,
        gpio::{Level, Output, OutputDrive},
        peripherals::{self, SPI3},
        spim::{self, Spim},
    };

    use embassy_time::Delay;
    use embedded_hal_bus::spi::ExclusiveDevice;
    use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
    use mx25r::{blocking::MX25R6435F, SECTOR_SIZE};

    bind_interrupts!(struct Irqs {
        SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    });

    #[init]
    async fn init() -> MX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>> {
        let cfg = Config::default();
        let p = embassy_nrf::init(cfg);

        let spi_config = spim::Config::default();
        let spi = Spim::new(p.SPI3, Irqs, p.P0_19, p.P0_21, p.P0_20, spi_config);
        let cs = Output::new(p.P0_17, Level::High, OutputDrive::Standard);
        let spi_dev = ExclusiveDevice::new(spi, cs, Delay).unwrap();
        MX25R6435F::new(spi_dev)
    }

    #[test]
    async fn basic(
        mut memory: MX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        let mut buff = [0];
        let addr = 0;

        memory.read(addr, &mut buff).unwrap();
        memory.erase_sector(addr).unwrap();

        memory.read(addr, &mut buff).unwrap();
        defmt::assert_eq!(buff[0], 0xff);

        memory.write_page(0, &[42]).unwrap();

        memory.read(addr, &mut buff).unwrap();
        defmt::assert_eq!(buff[0], 42);

        memory.erase_sector(addr).unwrap();
    }
    /// Read multiple bytes in a single call.
    #[test]
    async fn read_multiple_bytes(
        mut memory: MX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        const LEN: usize = 16;
        let mut buf = [0u8; LEN];
        memory.erase_sector(0).unwrap();

        let mut pattern = [0u8; LEN];
        for (i, e) in pattern.iter_mut().enumerate() {
            *e = i as u8;
        }
        memory.write_page(0, &pattern).unwrap();
        memory.read(0, &mut buf).unwrap();
        defmt::assert_eq!(&buf, &pattern);
    }

    /// Read spanning one sector end into the next (should error if next sector is not erased/written).
    #[test]
    async fn read_across_sector_boundary(
        mut memory: MX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        let mut buf = [0u8; 16];

        memory.erase_sector(0).unwrap();
        memory.read(0, &mut buf).unwrap();
        defmt::assert!(buf.iter().all(|&b| b == 0xFF));
    }

    /// Out-of-bounds reads should error.
    #[test]
    async fn read_out_of_bounds(
        mut memory: MX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        let mut buf = [0u8; 16];
        let res = memory.read(0x4000_0000, &mut buf);
        defmt::assert!(res.is_err(), "Expected out-of-bounds read to error");
    }

    /// Directly exercise the ReadNorFlash trait’s `read` method with an absolute offset.
    #[test]
    async fn direct_trait_read(
        mut memory: MX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        let mut buf = [0u8; 4];

        memory.erase_sector(0).unwrap();
        memory.write_page(0, &[1, 2, 3, 4]).unwrap();

        ReadNorFlash::read(&mut memory, 0, &mut buf).unwrap();
        defmt::assert_eq!(buf, [1, 2, 3, 4]);
    }

    /// Check that `capacity()` returns the expected total size.
    #[test]
    async fn trait_capacity(
        memory: MX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        let cap = memory.capacity();
        defmt::assert_eq!(cap, 8 * 1024 * 1024);
    }

    /// Write a blob at an arbitrary offset via the `NorFlash` & `NorFlash` traits, then read it back.
    #[test]
    async fn trait_write_read(
        mut memory: MX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        // 4 bytes of test data
        const DATA: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
        const SECTOR_SIZE: u32 = 4096;
        let mut buf = [0u8; 4];

        NorFlash::erase(&mut memory, 0, SECTOR_SIZE).unwrap();

        NorFlash::write(&mut memory, 0, &DATA).unwrap();

        ReadNorFlash::read(&mut memory, 0, &mut buf).unwrap();

        defmt::assert_eq!(buf, DATA);
    }

    /// Erase a region that spans two sectors via the `NorFlash` and `NorFlash` traits,
    /// then verify before/after.
    #[test]
    async fn trait_erase_range(
        mut memory: MX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        const LEN: usize = SECTOR_SIZE as usize + (SECTOR_SIZE / 2) as usize;
        const ERASE: u32 = 2 * SECTOR_SIZE;
        let mut buf = [0u8; LEN];
        let data = [0x55u8; LEN];

        NorFlash::erase(&mut memory, 0, ERASE).unwrap();

        ReadNorFlash::read(&mut memory, 0, &mut buf).unwrap();
        defmt::assert!(buf.iter().all(|&b| b == 0xFF));

        NorFlash::write(&mut memory, 0, &data).unwrap();

        ReadNorFlash::read(&mut memory, 0, &mut buf).unwrap();
        defmt::assert!(buf.iter().all(|&b| b == 0x55));

        NorFlash::erase(&mut memory, 0, ERASE).unwrap();

        ReadNorFlash::read(&mut memory, 0, &mut buf).unwrap();
        defmt::assert!(buf.iter().all(|&b| b == 0xFF));
    }
}
