#![no_std]
#![no_main]

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
         peripherals,
        spim::{self, Spim},
        Peripherals,
    };

    use embassy_time::Delay;
    use embedded_hal_bus::spi::ExclusiveDevice;
    use mx25r::{
        address::{Address, Page, Sector},
        asynchronous::AsyncMX25R6435F, blocking::MX25R6435F,
    };

    bind_interrupts!(struct Irqs {
        SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    });

    #[init]
    async fn init() -> Peripherals {
        let cfg = Config::default();
        embassy_nrf::init(cfg)
    }

    #[test]
    async fn asynchronous(p: Peripherals) {
        let spi_config = spim::Config::default();

        // See https://infocenter.nordicsemi.com/index.jsp?topic=%2Fug_nrf52840_dk%2FUG%2Fdk%2Fhw_external_memory.html
        let spi = Spim::new(p.SPI3, Irqs, p.P0_19, p.P0_21, p.P0_20, spi_config);
        let cs = Output::new(p.P0_17, Level::High, OutputDrive::Standard);
        let spi_dev = ExclusiveDevice::new(spi, cs, Delay).unwrap();

        let mut memory = AsyncMX25R6435F::new(spi_dev);

        let mut buff = [0];
        let page = Page(0);
        let sector = Sector(0);
        let addr = Address::from_page(sector, page);

        memory.read(addr, &mut buff).await.unwrap();
        defmt::info!("Value before erase {}", buff);

        defmt::info!("Erasing first sector");
        memory.erase_sector(sector).await.unwrap();

        defmt::info!("red 2: {}", buff);
        let res = memory.read(addr, &mut buff).await;
        defmt::info!("red 2: {}, {}", defmt::Debug2Format(&res), buff);
        defmt::assert_eq!(buff[0], 0xff);

        defmt::info!("Writing 42");
        memory.write_page(sector, page, &[42]).await.unwrap();

        memory.read(addr, &mut buff).await.unwrap();
        defmt::assert_eq!(buff[0], 42);

        memory.erase_sector(sector).await.unwrap();
    }

    #[test]
    async fn synchronous(p: Peripherals) {
        let spi_config = spim::Config::default();

        // See https://defmt::infocenter.nordicsemi.com/index.jsp?topic=%2Fug_nrf52840_dk%2FUG%2Fdk%2Fhw_external_memory.html
        let spi = Spim::new(p.SPI3, Irqs, p.P0_19, p.P0_21, p.P0_20, spi_config);
        let cs = Output::new(p.P0_17, Level::High, OutputDrive::Standard);
        let spi_dev = ExclusiveDevice::new(spi, cs, Delay).unwrap();

        let mut memory = MX25R6435F::new(spi_dev);

        let mut buff = [0];
        let page = Page(0);
        let sector = Sector(0);
        let addr = Address::from_page(sector, page);

        memory.read(addr, &mut buff).unwrap();
        defmt::info!("Value before erase {}", buff);

        defmt::info!("Erasing first sector");
        memory.erase_sector(sector).unwrap();

        memory.read(addr, &mut buff).unwrap();
        defmt::assert_eq!(buff[0], 0xff);

        defmt::info!("Writing 42");
        memory.write_page(sector, page, &[42]).unwrap();

        memory.read(addr, &mut buff).unwrap();
        defmt::assert_eq!(buff[0], 42);
    }
}
