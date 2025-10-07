const RAM_SIZE: usize = 8192; 

// TODO: remove trait if not necessary (if CPU hold an instance of Mmu directly instead of
// something more generic like Box<dyn MemoryBus>)
pub trait MemoryBus {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, val: u8);
    fn read_word(&self, addr: u16) -> u16;
    fn write_word(&mut self, addr: u16, val: u16);
}

pub struct Mmu {
    ram: [u8; RAM_SIZE],
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            ram: [0; RAM_SIZE]
        }
    }
}

impl MemoryBus for Mmu {
    fn read_byte(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn read_word(&self, addr: u16) -> u16 {
        ((self.ram[addr as usize] as u16) << 8) | self.ram[addr as usize + 1] as u16
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
    }

    fn write_word(&mut self, addr: u16, val: u16) {
        let msbs: u8 = (val >> 8) as u8;
        let lsbs: u8 = (val & 0b0000_1111) as u8;

        self.write_byte(addr, msbs);
        self.write_byte(addr+1, lsbs);
    }
}
