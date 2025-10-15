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
    // Carry flag, set if an operation results in a carry or a borrow
    // - a 8-bit addtion that results in a value > 0xFF
    // - a 16-bit addition that results in a value > 0xFFFF
    // - the result of a subtractionor comparison is < 0
    // - a rotate or shift operation shifts out a 1
    C = 0b0001_0000,

    // Half Carry flag
    // indicates a carry for the lower nibble (4 bits)
    H = 0b0010_0000,

    // Subtract flag, set if the last operation was a subtraction
    N = 0b0100_0000,

    // Zero flag, set if the result of an operation is zero
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
        self.f = (val & 0xFF) as u8;
    }

    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = (val & 0xFF) as u8;
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = (val & 0xFF) as u8;
    }

    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = (val & 0xFF) as u8;
    }

    pub fn clear_flags(&mut self) {
        self.f = 0;
    }

    pub fn get_flag(&self, flag: CpuFlag) -> bool {
        (self.f & (flag as u8)) != 0
    }

    pub fn set_flag(&mut self, flag: CpuFlag, condition: bool) {
        if condition {
            self.f |= flag as u8;
        } else {
            self.f &= !(flag as u8);
        }
    }
}

pub fn add8_needs_carry(a: u8, b: u8) -> bool {
    (a as u16 + b as u16) > 0xFF
    // equivalent to: (a as u16 + b as u16) & 0x100 == 0x100
}

pub fn add16_needs_carry(a: u16, b: u16) -> bool {
    (a as u32 + b as u32) > 0xFFFF
    // equivalent to: (a as u32 + b as u32) & 0x10000 == 0x10000
}

pub fn add8_needs_half_carry(a: u8, b: u8) -> bool {
    ((a & 0x0F) + (b & 0x0F)) > 0x0F
    // equivalent to: ((a & 0x0F) + (b & 0x0F)) & 0x10 == 0x10
}

pub fn add16_needs_half_carry(a: u16, b: u16) -> bool {
    ((a & 0x0FFF) + (b & 0x0FFF)) > 0x0FFF
    // equivalent to: ((a & 0x0FFF) + (b & 0x0FFF)) & 0x1000 == 0x1000
}