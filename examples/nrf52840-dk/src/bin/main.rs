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
    config,
    gpio::{Level, Output, OutputDrive},
    interrupt,
    spim::{self, Spim},
    Peripherals,
};
use mx25r::{
    self,
    mx25r::{Address, MX25R},
};
use panic_probe as _;

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let mut spi_config = spim::Config::default();
    spi_config.frequency = spim::Frequency::M16;

    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    let spi = Spim::new(p.TWISPI0, irq, p.P0_19, p.P0_21, p.P0_20, spi_config);
    let cs = Output::new(p.P0_17, Level::High, OutputDrive::Standard);
    let mut memory = MX25R::new(spi, cs);
    memory.chip_erase().unwrap();
    let mut buff = [0];

    info!("Status {}", memory.read_status().unwrap());

    memory.write_enable().unwrap();
    info!("Writing 42");
    memory
        .write_page(Address { sector: 0, page: 0 }, &[42])
        .unwrap();

    info!("Status {}", memory.read_status().unwrap());
    info!("Value before {}", buff);
    memory
        .read(Address { sector: 0, page: 0 }, &mut buff)
        .unwrap();
    info!("Value after {}", buff);

    loop {
        info!("Status {}", memory.read_status().unwrap());
        Timer::after(Duration::from_millis(1000)).await;
    }
}
