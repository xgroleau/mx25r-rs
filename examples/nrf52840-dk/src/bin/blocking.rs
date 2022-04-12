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
    blocking::{WriteEnabled, MX25R6435F, MX25RLowLevel},
    error::Error,
};
use panic_probe as _;

type DkMX25R<'a> = MX25R6435F<ExclusiveDevice<Spim<'a, TWISPI0>, Output<'a, P0_17>>, WriteEnabled>;

async fn wait_wip(mx25r: &mut DkMX25R<'_>) {
    while let Err(Error::Busy) = mx25r.poll_wip() {
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut spi_config = spim::Config::default();
    spi_config.frequency = spim::Frequency::M16;

    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);

    // See https://infocenter.nordicsemi.com/index.jsp?topic=%2Fug_nrf52840_dk%2FUG%2Fdk%2Fhw_external_memory.html
    let mut spi = Spim::new(p.TWISPI0, irq, p.P0_19, p.P0_21, p.P0_20, spi_config);
    let cs = Output::new(p.P0_17, Level::High, OutputDrive::Standard);
    
    cs.is_set_high();
    spi.blocking_write(&[2, 0, 0, 0, 42]).unwrap();
    cs.is_set_low();
    Timer::after(Duration::from_millis(1000)).await;
    cs.is_set_high();
    let mut buff = [3, 0, 0, 0, 0, 0, 0 , 0];
    spi.blocking_transfer_in_place(&mut buff).unwrap();
    cs.is_set_low();
    info!("Value {}", buff);

    let spi_dev = ExclusiveDevice::new(spi, cs);


    let mut memory = MX25RLowLevel::<0x7FFFFF, ExclusiveDevice<Spim<TWISPI0>, Output<P0_17>>>::new(spi_dev); //MX25R6435F::new(spi_dev).enable_write().unwrap();

    let mut buff = [0];
    let page = Page(0);
    let sector = Sector(0);
    let addr = Address::from_page(sector, page);

    memory.read(addr, &mut buff).unwrap();
    info!("Value before erase {}", buff);

    info!("Erasing first sector");
    //memory.erase_sector(sector).unwrap();
    //wait_wip(&mut memory).await;

    memory.read(addr, &mut buff).unwrap();
    info!("Value after erase {}", buff);

    info!("Writing 42");
    memory.write_page(sector, page, &[42]).unwrap();
    info!("Status {}", memory.read_status().unwrap());
    //wait_wip(&mut memory).await;

    Timer::after(Duration::from_millis(1000)).await;
    memory.read(addr, &mut buff).unwrap();
    info!("Value after write {}", buff);
}
