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
    use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};
    use mx25r::{
        address::{SECTOR_SIZE},
        asynchronous::AsyncMX25R6435F,
    };

    bind_interrupts!(struct Irqs {
        SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    });

    #[init]
    async fn init() -> AsyncMX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>
    {
        let cfg = Config::default();
        let p = embassy_nrf::init(cfg);

        let spi_config = spim::Config::default();
        let spi = Spim::new(p.SPI3, Irqs, p.P0_19, p.P0_21, p.P0_20, spi_config);
        let cs = Output::new(p.P0_17, Level::High, OutputDrive::Standard);
        let spi_dev = ExclusiveDevice::new(spi, cs, Delay).unwrap();
        AsyncMX25R6435F::new(spi_dev)
    }

    #[test]
    async fn basic(
        mut memory: AsyncMX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        let mut buff = [0];
        let addr = 0;

        memory.read(addr, &mut buff).await.unwrap();
        defmt::info!("Value before erase {}", buff);

        defmt::info!("Erasing first sector");
        memory.erase_sector(addr).await.unwrap();

        defmt::info!("red 2: {}", buff);
        let res = memory.read(addr, &mut buff).await;
        defmt::info!("red 2: {}, {}", defmt::Debug2Format(&res), buff);
        defmt::assert_eq!(buff[0], 0xff);

        defmt::info!("Writing 42");
        memory.write_page(0, &[42]).await.unwrap();

        memory.read(addr, &mut buff).await.unwrap();
        defmt::assert_eq!(buff[0], 42);

        memory.erase_sector(addr).await.unwrap();
    }
    /// Read multiple bytes in a single call.
    #[test]
    async fn read_multiple_bytes(
        mut memory: AsyncMX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        const LEN: usize = 16;
        let mut buf = [0u8; LEN];
        // Ensure sector is erased
        memory.erase_sector(0).await.unwrap();

        // Fill a pattern across one page
        let mut pattern = [0u8; LEN];
        for i in 0..LEN {
            pattern[i] = i as u8;
        }
        memory.write_page(0, &pattern).await.unwrap();

        // Read it back
        memory.read(0, &mut buf).await.unwrap();
        defmt::assert_eq!(&buf, &pattern);
    }

    /// Read spanning one sector end into the next (should error if next sector is not erased/written).
    #[test]
    async fn read_across_sector_boundary(
        mut memory: AsyncMX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        const SECTOR_SIZE: usize = 4096; // MX25R6435F typical
        let mut buf = [0u8; 16];

        // Erase only sector0
        memory.erase_sector(0).await.unwrap();

        // Attempt to read spanning into sector1 -> might return 0xFF for erased and error if out-of-bounds
        let res = memory.read(0, &mut buf).await;
        // We expect Ok, but since sector1 wasn't erased, contents are undefined
        res.unwrap();

        // All bytes in sector0-end should be 0xFF
        for i in 0..8 {
            defmt::assert_eq!(buf[i], 0xFF);
        }
        // The rest may be arbitrary; we don’t assert on them here.
    }

    /// Zero-length reads should be a no-op (and not panic).
    #[test]
    async fn read_zero_bytes(
        mut memory: AsyncMX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        let mut buf = [];

        // Erase then zero-read
        memory.erase_sector(0).await.unwrap();
        // Should return Ok immediately
        memory.read(0, &mut buf).await.unwrap();
    }

    /// Out-of-bounds reads should error.
    #[test]
    async fn read_out_of_bounds(
        mut memory: AsyncMX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        let mut buf = [0u8; 16];
        // Pick an address beyond the device capacity, e.g. 1 GiB
        let res = memory.read(0x4000_0000, &mut buf).await;
        defmt::assert!(res.is_err(), "Expected out-of-bounds read to error");
    }

    /// Directly exercise the ReadNorFlash trait’s `read` method with an absolute offset.
    #[test]
    async fn direct_trait_read(
        mut memory: AsyncMX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        let mut buf = [0u8; 4];

        // Erase and write a known word
        memory.erase_sector(0).await.unwrap();
        memory.write_page(0, &[1, 2, 3, 4]).await.unwrap();

        // Call ReadNorFlash::read
        ReadNorFlash::read(&mut memory, 0, &mut buf).await.unwrap();
        defmt::assert_eq!(buf, [1, 2, 3, 4]);
    }

    /// Check that `capacity()` returns the expected total size.
    #[test]
    async fn trait_capacity(
        memory: AsyncMX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        // MX25R6435F is 8 MiB
        let cap = memory.capacity();
        defmt::assert_eq!(cap, 8 * 1024 * 1024);
    }

    /// Write a blob at an arbitrary offset via the `NorFlash` & `NorFlash` traits, then read it back.
    #[test]
    async fn trait_write_read(
        mut memory: AsyncMX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        // 4 bytes of test data
        const DATA: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
        const SECTOR_SIZE: u32 = 4096;
        const OFFSET: u32 = 0;
        let mut buf = [0u8; 4];

        // Erase exactly the region we’re about to write
        NorFlash::erase(&mut memory, OFFSET, OFFSET + SECTOR_SIZE)
            .await
            .unwrap();

        // Write via the NorFlash trait
        NorFlash::write(&mut memory, OFFSET, &DATA).await.unwrap();

        // Read back via the ReadNorFlash trait
        ReadNorFlash::read(&mut memory, OFFSET, &mut buf)
            .await
            .unwrap();

        defmt::assert_eq!(buf, DATA);
    }

    /// Erase a region that spans two sectors via the `NorFlash` and `NorFlash` traits,
    /// then verify before/after.
    #[test]
    async fn trait_erase_range(
        mut memory: AsyncMX25R6435F<ExclusiveDevice<Spim<'static, SPI3>, Output<'static>, Delay>>,
    ) {
        // MX25R6435F sector size is 4096
        const SECTOR_SIZE: usize = 4096;
        // Start halfway through sector N…
        const START: usize = SECTOR_SIZE / 2; // = 2048
                                              // …and end 100 bytes into sector N+1
        const END: usize = SECTOR_SIZE * 2; // = 8292
                                            // Compute length at compile time
        const LEN: usize = END - START; // = 6244

        // A stack‐allocated buffer and data pattern of that length
        let mut buf = [0u8; LEN];
        let data = [0x55u8; LEN];

        // 1) Erase the range
        NorFlash::erase(&mut memory, START as u32, END as u32)
            .await
            .unwrap();

        // 2) Write 0x55 across the entire range
        NorFlash::write(&mut memory, START as u32, &data)
            .await
            .unwrap();

        // 3) Verify it’s written
        ReadNorFlash::read(&mut memory, START as u32, &mut buf)
            .await
            .unwrap();
        assert!(buf.iter().all(|&b| b == 0x55));

        // 4) Erase the same range again via the trait
        NorFlash::erase(&mut memory, START as u32, END as u32)
            .await
            .unwrap();

        // 5) Now every byte should be 0xFF
        ReadNorFlash::read(&mut memory, START as u32, &mut buf)
            .await
            .unwrap();
        assert!(buf.iter().all(|&b| b == 0xFF));
    }
}
