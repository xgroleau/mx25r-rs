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
    spim::{self, Spim},
    Peripherals,
};
use mx25r::{
    address::{Address, Page, Sector},
    blocking::MX25R6435F,
};
use panic_probe as _;

async fn wait_wip<SPI, CS>(mx25r: &mut MX25R6435F<SPI, CS>)
where
    SPI: embedded_hal::blocking::spi::Transfer<u8> + embedded_hal::blocking::spi::Write<u8>,
    CS: embedded_hal::digital::v2::OutputPin,
{
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
    let mut memory = MX25R6435F::new(spi, cs);
    let mut buff = [0];

    memory.write_enable().unwrap();
    memory.chip_erase().unwrap();
    wait_wip(&mut memory).await;
    info!("Status {}", memory.read_status().unwrap());

    let addr = Address::from_page(Sector(0), Page(0));
    info!("Writing 42");
    memory.write_page(addr, &[42]).unwrap();

    info!("Status {}", memory.read_status().unwrap());
    info!("Value before {}", buff);
    memory.read(addr, &mut buff).unwrap();
    info!("Value after {}", buff);

    loop {
        info!("Status {}", memory.read_status().unwrap());
        Timer::after(Duration::from_millis(5000)).await;
    }
}
