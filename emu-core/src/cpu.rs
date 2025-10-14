use crate::ld;
use crate::memory::MemoryBus;
use crate::registers::Registers;

pub struct Cpu<M: MemoryBus> {
    pub reg: Registers,
    halted: bool,
    ime: bool,
    setei: u32,
    setdi: u32,
    pub prefetched: u8,
    pub mmu: M,
}

impl<M: MemoryBus> Cpu<M> {
    pub fn new(mmu: M) -> Self {
        Self {
            reg: Registers::new(),
            halted: false,
            ime: false, // TODO: check bootup value
            setei: 0,   // same
            setdi: 0,   // same
            prefetched: 0,
            mmu,
        }
    }

    pub fn tick(&mut self) {
        // TODO: update timers and handle interrupts

        if self.halted {
            return;
        }

        let cycles = self.execute();
        for _ in 0..cycles {
            self.mmu.tick(); // TODO: maybe handle the fact that memory and cpu have different speeds
        }

        // Prefetch next opcode
        self.prefetched = self.read_byte();
    }

    pub fn read_byte(&mut self) -> u8 {
        let val = self.prefetched;
        self.prefetched = self.mmu.read_byte(self.reg.pc);
        self.reg.pc += 1;
        val
    }

    pub fn read_word(&mut self) -> u16 {
        let low = self.read_byte() as u16;
        let high = self.read_byte() as u16;
        (high << 8) | low
    }

    /// Executes the instructions at mem[pc].
    /// Returns the number of cycles
    fn execute(&mut self) -> u8 {
        let opcode = self.prefetched;
        let high = (opcode & 0xF0) >> 4;
        let low = opcode & 0x0F;
        match (high, low) {
            (0, 0) => 1, // NOP

            // -- 8-bit loads
            // register <- constant
            (0x0..=0x2, 0x6) => ld::ld_cst_to_reg(self, opcode),
            (0x0..=0x3, 0xE) => ld::ld_cst_to_reg(self, opcode),

            // register <- register
            (0x4..=0x6, 0x0..=0x5) => ld::ld_reg_to_reg(self, opcode),
            (0x4..=0x6, 0x7..=0xD) => ld::ld_reg_to_reg(self, opcode),
            (0x7, 0x8..=0xD) => ld::ld_reg_to_reg(self, opcode),
            (0x4..=0x7, 0xF) => ld::ld_reg_to_reg(self, opcode),

            // register <- memory
            (0x4..=0x6, 0x6) => ld::ld_mem_to_reg(self, opcode),
            (0x4..=0x7, 0xE) => ld::ld_mem_to_reg(self, opcode),
            (0xF, 0x0) => ld::ld_mem_to_reg(self, opcode),
            (0xF, 0x2) => ld::ld_mem_to_reg(self, opcode),
            (0xF, 0xA) => ld::ld_mem_to_reg(self, opcode),

            // -- 16-bit loads
            // memory <- register
            (0x0..=0x3, 0x2) => ld::ld_reg_to_mem(self, opcode),
            (0x7, 0x0..=0x5) => ld::ld_reg_to_mem(self, opcode),
            (0x7, 0x7) => ld::ld_reg_to_mem(self, opcode),
            (0xE, 0x0) => ld::ld_reg_to_mem(self, opcode),
            (0xE, 0x2) => ld::ld_reg_to_mem(self, opcode),
            (0xE, 0xA) => ld::ld_reg_to_mem(self, opcode),

            // memory <- constant
            (0x3, 0x6) => ld::ld_cst_to_mem(self),

            // -- 8-bit alu

            // -- 16-bit alu

            // -- jumps

            // -- cb prefix
            (0xC, 0xB) => self.execute_cb(),
            _ => panic!("Unknown instruction opcode: Ox{:02X}", opcode),
        }
    }

    /// Executes instructions prefixed by 0xCB
    fn execute_cb(&mut self) -> u8 {
        let opcode = self.read_byte();
        let high = (opcode & 0xF0) >> 4;
        let low = opcode & 0x0F;
        //TODO implement CB logic
        match (high, low) {
            _ => panic!("Unkown CB prefix instruction opcode: 0xCB{:02X}", opcode),
        }
    }
}
