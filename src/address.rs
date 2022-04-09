pub const BLOCK64_SIZE: u32 = 0x010000;
pub const BLOCK32_SIZE: u32 = BLOCK64_SIZE / 2;
pub const SECTOR_SIZE: u32 = 0x1000;
pub const PAGE_SIZE: u32 = 0x100;

/// A 32kB block address, containing 8 sectors
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Block32(pub u16);

/// A 64kB block address, containing 16 sector
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Block64(pub u16);

/// A sector id, containing 16 pages for a total of 4kB.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sector(pub u16);

/// A page id, containing 256kB
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page(pub u8);

/// An address on the memory chip
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct Address(pub u32);

impl Address {
    /// Represents any address in memory.
    pub fn from_addr(sector: Sector, page: Page, offset: u8) -> Self {
        let addr = sector.0 as u32 * SECTOR_SIZE + page.0 as u32 * PAGE_SIZE + offset as u32;
        Address(addr)
    }

    /// Represents a specific page in memory.
    pub fn from_page(sector: Sector, page: Page) -> Self {
        Self::from_addr(sector, page, 0)
    }

    /// Represents a specific sector in memory.
    pub fn from_sector(sector: Sector) -> Self {
        Self::from_addr(sector, Page(0), 0)
    }

    /// Represents a specific 32kB block in memory.
    pub fn from_block32(block: Block32) -> Self {
        let addr = block.0 as u32 * BLOCK32_SIZE / SECTOR_SIZE;
        Address(addr)
    }

    /// Represents a specific 64kB block in memory.
    pub fn from_block64(block: Block64) -> Self {
        let addr = block.0 as u32 * BLOCK64_SIZE / SECTOR_SIZE;
        Address(addr)
    }
}

impl From<u16> for Block32 {
    fn from(block_id: u16) -> Block32 {
        Block32(block_id)
    }
}

impl From<u16> for Block64 {
    fn from(block_id: u16) -> Block64 {
        Block64(block_id)
    }
}

impl From<u16> for Sector {
    fn from(sector_id: u16) -> Sector {
        Sector(sector_id)
    }
}

impl From<u8> for Page {
    fn from(page_id: u8) -> Page {
        Page(page_id)
    }
}

impl From<Address> for u32 {
    fn from(addr: Address) -> u32 {
        addr.0
    }
}
