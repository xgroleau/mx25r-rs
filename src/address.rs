const BLOCK64_SIZE: u32 = 0x010000;
const SECTOR_SIZE: u32 = 0x1000;
const PAGE_SIZE: u32 = 0x100;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Block32(pub u16);

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Block64(pub u16);

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sector(pub u16);

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page(pub u8);

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct Address(u32);

impl Address {
    pub fn from_addr(sector: Sector, page: Page, offset: u8) -> Self {
        let addr = sector.0 as u32 * SECTOR_SIZE + page.0 as u32 * PAGE_SIZE + offset as u32;
        Address(addr)
    }

    pub fn from_page(sector: Sector, page: Page) -> Self {
        Self::from_addr(sector, page, 0)
    }

    pub fn from_sector(sector: Sector) -> Self {
        Self::from_addr(sector, Page(0), 0)
    }

    pub fn from_block32(block: Block32) -> Self {
        let addr = block.0 as u32 * BLOCK64_SIZE / SECTOR_SIZE / 2;
        Address(addr)
    }

    pub fn from_block64(block: Block64) -> Self {
        let addr = block.0 as u32 * BLOCK64_SIZE / SECTOR_SIZE;
        Address(addr)
    }
}

impl From<Address> for u32 {
    fn from(addr: Address) -> u32 {
        addr.0
    }
}
