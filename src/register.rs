use bit::BitIndex;

pub struct ManufacturerId(pub u8);
pub struct MemoryType(pub u8);
pub struct MemoryDensity(pub u8);
pub struct ElectronicId(pub u8);
pub struct DeviceId(pub u8);

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct StatusRegister {
    pub write_protect_disable: bool,
    pub quad_enable: bool,
    pub protected_block: u8,
    pub write_enable_latch: bool,
    pub wip_bit: bool,
}

impl From<u8> for StatusRegister {
    fn from(val: u8) -> StatusRegister {
        StatusRegister {
            write_protect_disable: val.bit(7),
            quad_enable: val.bit(6),
            protected_block: val.bit_range(2..6),
            write_enable_latch: val.bit(1),
            wip_bit: val.bit(0),
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum ProtectedArea {
    Top,
    Bottom,
}
impl From<bool> for ProtectedArea {
    fn from(val: bool) -> Self {
        if val {
            ProtectedArea::Bottom
        } else {
            ProtectedArea::Top
        }
    }
}
impl From<ProtectedArea> for bool {
    fn from(val: ProtectedArea) -> Self {
        match val {
            ProtectedArea::Bottom => true,
            ProtectedArea::Top => false,
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum PowerMode {
    UltraLowPower,
    HighPerformance,
}
impl From<bool> for PowerMode {
    fn from(val: bool) -> Self {
        if val {
            PowerMode::HighPerformance
        } else {
            PowerMode::UltraLowPower
        }
    }
}
impl From<PowerMode> for bool {
    fn from(val: PowerMode) -> Self {
        match val {
            PowerMode::HighPerformance => true,
            PowerMode::UltraLowPower => false,
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct ConfigurationRegister {
    pub dummmy_cycle: bool,
    pub protected_section: ProtectedArea,
    pub power_mode: PowerMode,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct SecurityRegister {
    pub erase_failed: bool,
    pub program_failed: bool,
    pub erase_suspended: bool,
    pub program_suspended: bool,
    pub locked_down: bool,
    pub secured_otp: bool,
}
