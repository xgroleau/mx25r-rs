#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Command {
    Read = 0x03,
    ReadF = 0x0B,
    Read2 = 0xBB,
    ReadD = 0x3B,
    Read4 = 0xEB,
    ReadQ = 0x6B,
    ProgramPage = 0x02,
    ProgramPage4 = 0x38,
    SectorErase = 0x20,
    BlockErase32 = 0x52,
    BlockErase = 0xD8,
    ChipErase = 0x60,
    ReadSfdp = 0x5A,
    WriteEnable = 0x06,
    WriteDisable = 0x04,
    ReadStatus = 0x05,
    ReadConfig = 0x15,
    WriteStatus = 0x01,
    ProgramEraseSuspend = 0xB0,
    ProgramEraseResume = 0x30,
    DeepPowerDown = 0xB9,
    SetBurstLength = 0xC0,
    ReadIdentification = 0x9F,
    ReadManufacturerId = 0x90,
    ReadElectronicId = 0xAB,
    EnterSecureOTP = 0xB1,
    ExitSecureOTP = 0xC1,
    ReadSecurityRegister = 0x2B,
    WriteSecurityRegister = 0x2F,
    Nop = 0x00,
    ResetEnable = 0x66,
    ResetMemory = 0x99,
}
