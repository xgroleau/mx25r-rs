#![no_std]
#![no_main]
#![allow(unused_imports)]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf as _;
use panic_probe;

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
    asynchronous::AsyncMX25R6435F,
};

bind_interrupts!(struct Irqs {
    SPI2 => spim::InterruptHandler<peripherals::SPI2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let cfg = Config::default();
    let p = embassy_nrf::init(cfg);

    let spi_config = spim::Config::default();
    let spi = Spim::new(p.SPI2, Irqs, p.P0_19, p.P0_21, p.P0_20, spi_config);
    let cs = Output::new(p.P0_17, Level::High, OutputDrive::Standard);
    let spi_dev = ExclusiveDevice::new(spi, cs, Delay).unwrap();
    let mut memory = AsyncMX25R6435F::new(spi_dev);

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

    defmt::info!("DONE!!!!!!!!!!!!!!!!!");
}
