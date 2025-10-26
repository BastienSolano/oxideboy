pub trait MemoryBus {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, val: u8);
    fn read_word(&self, addr: u16) -> u16;
    fn write_word(&mut self, addr: u16, val: u16);
    fn tick(&mut self, num_cycles: u8);

    /// Records an internal CPU operation that doesn't access memory (1 M-cycle).
    /// This is important for cycle-accurate emulation and timing synchronization.
    /// Examples: internal ALU operations, SP increment/decrement, etc.
    fn tick_internal(&mut self) {
        // Default implementation does nothing; test mocks can override
    }
}

pub struct Mmu {
    // Cartridge ROM
    rom: Vec<u8>,

    // RAM
    wram: [u8; 0x2000], // Working RAM (8KB: 0xC000 - 0xDFFF)
    hram: [u8; 0x7F], // High RAM (127B: 0xFF80 - 0xFFFE) 

    // Memory-mapped IO registers
    // (simply the ones needed for Blargg's tests for now)
    sb: u8,   // 0xFF01 - Serial transfer data
    sc: u8,   // 0xFF02 - Serial transfer control
    if_reg: u8,  // 0xFF0F - Interrupt Flag
    ie_reg: u8,   // 0xFFFF - Interrupt Enable

    // For MBC1
    rom_bank: usize,

    // Capture serial output for test results
    // TODO: create a proper logging mechanism and Serial device emulation
    serial_output: Vec<u8>,
}

impl Mmu {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            wram: [0; 0x2000],
            hram: [0; 0x7F],
            sb: 0,
            sc: 0,
            if_reg: 0,
            ie_reg: 0,
            rom_bank: 1,
            serial_output: Vec::new(),
        }
    }

    pub fn get_serial_output(&self) -> String {
        String::from_utf8_lossy(&self.serial_output).to_string()
    }
}

impl MemoryBus for Mmu {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                // ROM Bank 0
                self.rom.get(addr as usize).copied().unwrap_or(0xFF)
            },
            0x4000..=0x7FFF => {
                // Switchable ROM Bank
                let banked_addr = (self.rom_bank * 0x4000) + ((addr as usize) - 0x4000);
                self.rom.get(banked_addr).copied().unwrap_or(0xFF)
            },
            0x8000..=0xBFFF => 0xFF, // VRAM and external RAM (not implemented yet)
            0xC000..=0xDFFF => {
                // Working RAM
                self.wram[(addr - 0xC000) as usize]
            },
            0xE000..=0xFDFF => {
                // Echo RAM (mirror of C000-DDFF)
                self.wram[(addr - 0xE000) as usize]
            },
            0xFE00..=0xFE9F => 0xFF, // OAM (not implemented yet)
            0xFEA0..=0xFEFF => 0xFF, // Unusable memory
            0xFF00..=0xFF7F => {
                // I/O Registers
                match addr {
                    0xFF01 => self.sb,
                    0xFF02 => self.sc,
                    0xFF0F => self.if_reg,
                    _ => 0xFF, // Other I/O registers not implemented yet
                }
            }
            0xFF80..=0xFFFE => {
                // High RAM
                self.hram[(addr - 0xFF80) as usize]
            },
            0xFFFF => self.ie_reg,
        }
    }

    fn read_word(&self, addr: u16) -> u16 {
        let low = self.read_byte(addr) as u16;
        let high = self.read_byte(addr + 1) as u16;
        (high << 8) | low
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                // MBC1 RAM Enable (not implemented)
            },
            0x2000..=0x3FFF => {
                // MBC1 ROM Bank Number
                let bank = (val & 0x1F) as usize;
                self.rom_bank = if bank == 0 { 1 } else { bank };
            },
            0x4000..=0x7FFF => {
                // RAM Bank Number / Upper Bits of ROM Bank Number (not implemented)
            },
            0x8000..=0xBFFF => {
                // VRAM and external RAM (not implemented yet)
            },
            0xC000..=0xDFFF => {
                // Working RAM
                self.wram[(addr - 0xC000) as usize] = val;
            },
            0xE000..=0xFDFF => {
                // Echo RAM (mirror of C000-DDFF)
                self.wram[(addr - 0xE000) as usize] = val;
            },
            0xFE00..=0xFEFF => {
                // OAM (not implemented yet) + unsusable memory
            },
            0xFF00..=0xFF7F => {
                // I/O Registers
                match addr {
                    0xFF01 => self.sb = val,
                    0xFF02 => {
                        self.sc = val;
                        if val & 0x80 != 0 {
                            // Start serial transfer (for simplicity, we just output the byte)
                            self.serial_output.push(self.sb);
                            self.sc &= 0x7F; // Clear the start bit
                        }
                    },
                    0xFF0F => self.if_reg = val,
                    _ => {}, // Other I/O registers not implemented yet
                }
            },
            0xFF80..=0xFFFE => {
                // High RAM
                self.hram[(addr - 0xFF80) as usize] = val;
            },
            0xFFFF => self.ie_reg = val,
        }
    }

    fn write_word(&mut self, addr: u16, val: u16) {
        let low = (val & 0xFF) as u8;
        let high = (val >> 8) as u8;
        self.write_byte(addr, low);
        self.write_byte(addr + 1, high);
    }

    fn tick(&mut self, num_cycles: u8) {
        // TODO: update timers, etc.
    }
}
