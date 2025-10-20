use crate::ld;
use crate::alu;
//use crate::stack;
use crate::jumps;
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
        self.mmu.tick(cycles);

        // Prefetch next opcode
        self.prefetched = self.read_byte();
    }

    pub fn read_byte(&mut self) -> u8 {
        // At this point, the next instruction is not yet prefetched
        // so self.prefetched contains the current instruction
        let val = self.mmu.read_byte(self.reg.pc);
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

            // -- register manipulations
            (0x2, 0x7) => self.daa(),
            (0x3, 0x7) => self.scf(),
            (0x2, 0xF) => self.cpl(),
            (0x3, 0xF) => self.ccf(),

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
            (0x0..=0x3, 0xA) => ld::ld_mem_to_reg(self, opcode),
            (0xF, 0x0) => ld::ld_mem_to_reg(self, opcode),
            (0xF, 0x2) => ld::ld_mem_to_reg(self, opcode),
            (0xF, 0xA) => ld::ld_mem_to_reg(self, opcode),

            // -- 16-bit loads
            // register <- register
            (0xF, 0x8) => ld::ld_reg_to_reg(self, opcode),
            (0xF, 0x9) => ld::ld_reg_to_reg(self, opcode),

            // 16-bit register <- constant
            (0x0..=0x03, 0x1) => ld::ld_cst16_to_reg(self, opcode),

            // memory <- register
            (0x0..=0x3, 0x2) => ld::ld_reg_to_mem(self, opcode),
            (0x7, 0x0..=0x5) => ld::ld_reg_to_mem(self, opcode),
            (0x7, 0x7) => ld::ld_reg_to_mem(self, opcode),
            (0xE, 0x0) => ld::ld_reg_to_mem(self, opcode),
            (0xE, 0x2) => ld::ld_reg_to_mem(self, opcode),
            (0xE, 0xA) => ld::ld_reg_to_mem(self, opcode),
            (0x0, 0x8) => ld::ld_reg_to_mem(self, opcode),

            // memory <- constant
            (0x3, 0x6) => ld::ld_cst_to_mem(self),

            // -- 8-bit alu
            // increment 8-bit registers
            (0x0..=0x2, 0x4) => alu::incr(self, opcode),
            (0x0..=0x3, 0xC) => alu::incr(self, opcode),
            // decrement 8-bit registers
            (0x0..=0x2, 0x5) => alu::decr(self, opcode),
            (0x0..=0x3, 0xD) => alu::decr(self, opcode),

            // add
            (0x8, 0x0..=0x7) => alu::add(self, opcode),
            (0xc, 0x6) => alu::add(self, opcode),
            (0xe, 0x8) => alu::add(self, opcode),
            (0x0..=0x3, 0x9) => alu::add(self, opcode),

            // sub
            (0x9, 0x0..=0x7) => alu::sub(self, opcode),
            (0xd, 0x6) => alu::sub(self, opcode),

            // adc
            (0x8, 0x8..=0xF) => alu::adc(self, opcode),
            (0xc, 0xE) => alu::adc(self, opcode),

            // sbc
            (0x9, 0x8..=0xF) => alu::sbc(self, opcode),
            (0xd, 0xE) => alu::sbc(self, opcode),

            // and
            (0xA, 0x0..=0x7) => alu::and(self, opcode),
            (0xE, 0x6) => alu::and(self, opcode),

            // or
            (0xB, 0x0..=0x7) => alu::or(self, opcode),
            (0xF, 0x6) => alu::or(self, opcode),

            // xor
            (0xA, 0x8..=0xF) => alu::xor(self, opcode),
            (0xE, 0xE) => alu::xor(self, opcode),

            // cp
            (0xB, 0x8..=0xF) => alu::cp(self, opcode),
            (0xF, 0xE) => alu::cp(self, opcode),

            // -- 16-bit alu
            // increment 16-bit registers
            (0x0..=0x3, 0x3) => alu::incr(self, opcode),
            // increment memory pointed by 16-bit registers
            (0x3, 0x4) => alu::incr(self, opcode),
            // decrement 16-bit registers
            (0x0..=0x3, 0xB) => alu::decr(self, opcode),
            // decrement memory pointed by 16-bit registers
            (0x3, 0x5) => alu::decr(self, opcode),

            // -- jumps
            // relative jumps
            (0x2..=0x3, 0x0) => jumps::jr(self, opcode),
            (0x1..=0x3, 0x8) => jumps::jr(self, opcode),
            // absolute jumps
            (0xC..=0xD, 0x2) => jumps::jp(self, opcode),
            (0xC, 0x3) => jumps::jp(self, opcode),
            (0xE, 0x9) => jumps::jp(self, opcode),
            (0xC..=0xD, 0xA) => jumps::jp(self, opcode),
            

            // -- stack operations
            // pop
            //(0xC..=0xF, 0x1) => stack::pop(self, opcode),
            //(0xC..=0xF, 0x5) => stack::push(self, opcode),

            // push

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

    fn daa(&mut self) -> u8 {
        let mut a = self.reg.a;
        let mut adjust = 0;
        let mut carry = false;

        if (self.reg.f & 0x40) == 0 {
            // After addition
            // Check C flag (bit 4 = 0x10) for upper nibble adjustment
            if (self.reg.f & 0x10) != 0 || a > 0x99 {
                adjust |= 0x60;
                carry = true;
            }
            // Check H flag (bit 5 = 0x20) for lower nibble adjustment
            if (self.reg.f & 0x20) != 0 || (a & 0x0F) > 0x09 {
                adjust |= 0x06;
            }
            a = a.wrapping_add(adjust);
        } else {
            // After subtraction
            // Check C flag for upper nibble adjustment
            if (self.reg.f & 0x10) != 0 {
                adjust |= 0x60;
                carry = true;
            }
            // Check H flag for lower nibble adjustment
            if (self.reg.f & 0x20) != 0 {
                adjust |= 0x06;
            }
            a = a.wrapping_sub(adjust);
        }

        self.reg.a = a;

        // Update flags
        self.reg.f &= 0x40; // Keep only the N flag
        if a == 0 {
            self.reg.f |= 0x80; // Set Z flag
        }
        if carry {
            self.reg.f |= 0x10; // Set C flag
        }

        1
    }

    fn scf(&mut self) -> u8 {
        self.reg.f &= 0x90; // Clear N and H flags
        self.reg.f |= 0x10; // Set C flag

        1
    }

    fn cpl(&mut self) -> u8 {
        self.reg.a = !self.reg.a;
        self.reg.f |= 0x60; // Set N and H flags

        1
    }

    fn ccf(&mut self) -> u8 {
        self.reg.f &= 0x90; // Clear N and H flags
        self.reg.f ^= 0x10; // Toggle C flag

        1
    }
}
