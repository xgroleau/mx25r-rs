#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use defmt_rtt as _;
use embassy::{
    executor::Spawner,
    time::{Duration, Timer},
};
use embassy_nrf::{
    gpio::{Level, Output, OutputDrive},
    interrupt,
    peripherals::{P0_17, TWISPI0},
    spim::{self, Spim},
    Peripherals,
};
use embedded_hal::spi::blocking::ExclusiveDevice;
use mx25r::{
    address::{Address, Page, Sector},
    blocking::MX25R6435F,
};
use panic_probe as _;

type DkMX25R<'a> = MX25R6435F<ExclusiveDevice<Spim<'a, TWISPI0>, Output<'a, P0_17>>>;

async fn wait_wip(mx25r: &mut DkMX25R<'_>) {
    while mx25r.read_status().unwrap().wip_bit {
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let mut spi_config = spim::Config::default();
    spi_config.frequency = spim::Frequency::M16;

    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);

    // See https://infocenter.nordicsemi.com/index.jsp?topic=%2Fug_nrf52840_dk%2FUG%2Fdk%2Fhw_external_memory.html
    let spi = Spim::new(p.TWISPI0, irq, p.P0_19, p.P0_21, p.P0_20, spi_config);
    let cs = Output::new(p.P0_17, Level::High, OutputDrive::Standard);
    let spi_dev = ExclusiveDevice::new(spi, cs);

    let mut memory = MX25R6435F::new(spi_dev);
    let mut buff = [0];

    memory.write_enable().unwrap();
    memory.chip_erase().unwrap();
    wait_wip(&mut memory).await;
    info!("Status {}", memory.read_status().unwrap());

    let page = Page(0);
    let sector = Sector(0);
    let addr = Address::from_page(sector, page);
    info!("Writing 42");
    memory.write_page(sector, page, &[42]).unwrap();

    info!("Status {}", memory.read_status().unwrap());
    info!("Value before {}", buff);
    memory.read(addr, &mut buff).unwrap();
    info!("Value after {}", buff);

    loop {
        info!("Status {}", memory.read_status().unwrap());
        Timer::after(Duration::from_millis(5000)).await;
    }
}
