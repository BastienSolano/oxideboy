pub struct Registers {
    // 8 bits registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    pub f:u8, // flags register

    // 16 bits registers
    pub sp: u16,
    pub pc: u16,
}

pub enum CpuFlag {
    C = 0b0001_0000,
    H = 0b0010_0000,
    N = 0b0100_0000,
    Z = 0b1000_0000,
}

impl Registers {
    pub fn new() -> Self {
        Self {
           a: 1,
           b: 0,
           c: 0x13,
           d: 0,
           e: 0xD8,
           h: 1,
           l: 0x4D,
           f: CpuFlag::Z as u8, //TODO: check pandocs boot sequence to set this properly
           sp: 0xFFFE,
           pc: 0x100,
        }
    }

    pub fn af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }

    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0b0000_1111) as u8;
    }

    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = (val & 0b0000_1111) as u8;
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = (val & 0b0000_1111) as u8;
    }

    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = (val & 0b0000_1111) as u8;
    }
}
