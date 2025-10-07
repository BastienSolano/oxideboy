use std::panic;

use crate::memory::MemoryBus;
    use crate::registers::Registers;
use crate::cpu::Cpu;

pub fn ld_reg_to_reg(registers: &mut Registers, opcode: u8) -> u8 {
    match opcode {
        0x40 => (), // Nothing to do (LD B B)
        0x41 => registers.b = registers.c,
        0x42 => registers.b = registers.d,
        0x43 => registers.b = registers.e,
        0x44 => registers.b = registers.h,
        0x45 => registers.b = registers.l,
        0x47 => registers.b = registers.a,
        0x48 => registers.c = registers.b,
        0x49 => (), // Nothing to do (LD C C)
        0x4A => registers.c = registers.d,
        0x4B => registers.c = registers.e,
        0x4C => registers.c = registers.h,
        0x4D => registers.c = registers.l,
        0x4F => registers.c = registers.a,
        
        0x50 => registers.d = registers.b,
        0x51 => registers.d = registers.c,
        0x52 => (),
        0x53 => registers.d = registers.e,
        0x54 => registers.d = registers.h,
        0x55 => registers.d = registers.l,
        0x57 => registers.d = registers.a,
        0x58 => registers.e = registers.b,
        0x59 => registers.e = registers.c,
        0x5A => registers.e = registers.d,
        0x5B => (), // Nothing to do (LD E E)
        0x5C => registers.e = registers.h,
        0x5D => registers.e = registers.l,
        0x5F => registers.e = registers.a,

        0x60 => registers.h = registers.b,
        0x61 => registers.h = registers.c,
        0x62 => registers.h = registers.d,
        0x63 => registers.h = registers.e,
        0x64 => (), // Nothing to do (LD H H)
        0x65 => registers.h = registers.l,
        0x67 => registers.h = registers.a,
        0x68 => registers.l = registers.b,
        0x69 => registers.l = registers.c,
        0x6A => registers.l = registers.d,
        0x6B => registers.l = registers.e,
        0x6C => registers.l = registers.h,
        0x6D => (), // Nothing to do (LD L L)
        0x6F => registers.l = registers.a,

        0x78 => registers.a = registers.b,
        0x79 => registers.a = registers.c,
        0x7A => registers.a = registers.d,
        0x7B => registers.a = registers.e,
        0x7C => registers.a = registers.h,
        0x7D => registers.a = registers.l,
        0x7F => (), // Nothing to do (LD A A)

        _ => panic!("Not a register to register load instruction: 0x{:02X}", opcode),
    }
    1
}

pub fn ld_cst_to_reg(cpu: &mut Cpu, opcode: u8) -> u8 {
    let constant = cpu.read_byte();
    match opcode {
        0x06 => cpu.reg.b = constant,
        0x16 => cpu.reg.d = constant,
        0x26 => cpu.reg.h = constant,
        0x0E => cpu.reg.c = constant,
        0x1E => cpu.reg.e = constant,
        0x2E => cpu.reg.l = constant,
        0x3E => cpu.reg.a = constant,
        _ => panic!("Not a constant to register load instruction: 0x{:02X}", opcode),
    }
    2
}

pub fn ld_mem_to_reg(cpu: &mut Cpu, opcode: u8) -> u8 {
    match opcode {
        // load (hl) to all registers
        0x46 => cpu.reg.b = cpu.mmu.read_byte(cpu.reg.hl()),
        0x56 => cpu.reg.d = cpu.mmu.read_byte(cpu.reg.hl()),
        0x66 => cpu.reg.h = cpu.mmu.read_byte(cpu.reg.hl()),
        0x4E => cpu.reg.c = cpu.mmu.read_byte(cpu.reg.hl()),
        0x5E => cpu.reg.e = cpu.mmu.read_byte(cpu.reg.hl()),
        0x6E => cpu.reg.l = cpu.mmu.read_byte(cpu.reg.hl()),
        0x7E => cpu.reg.a = cpu.mmu.read_byte(cpu.reg.hl()),
        
        // loading mem(8-bit constant) in A
        0xF0 => {
            let cst = cpu.read_byte();
            cpu.reg.a = cpu.mmu.read_byte(cst as u16);
            return 3; // additional tick for reading the constant
        },

        // loading mem(C) in A
        0xF2 => cpu.reg.a = cpu.mmu.read_byte(cpu.reg.c as u16),

        // loading mem(16-bit constant) in A
        0xFA => {
            let cst = cpu.read_word();
            cpu.reg.a = cpu.mmu.read_byte(cst);
            return 4; // extra two ticks to read the 16-bit constant
        },
        _ => panic!("Not a memory to register instruction: 0x{:02X}", opcode),
    }
    2
}

pub fn ld_reg_to_mem(cpu: &mut Cpu, opcode: u8) -> u8 {
    match opcode {
        // load registers into memory(hl)
        0x70 => cpu.mmu.write_byte(cpu.reg.hl(), cpu.reg.b),
        0x71 => cpu.mmu.write_byte(cpu.reg.hl(), cpu.reg.c),
        0x72 => cpu.mmu.write_byte(cpu.reg.hl(), cpu.reg.d),
        0x73 => cpu.mmu.write_byte(cpu.reg.hl(), cpu.reg.e),
        0x74 => cpu.mmu.write_byte(cpu.reg.hl(), cpu.reg.h),
        0x75 => cpu.mmu.write_byte(cpu.reg.hl(), cpu.reg.l),
        0x77 => cpu.mmu.write_byte(cpu.reg.hl(), cpu.reg.a),

        0x02 => cpu.mmu.write_byte(cpu.reg.bc(), cpu.reg.a),
        0x12 => cpu.mmu.write_byte(cpu.reg.de(), cpu.reg.a),
        0x22 => {
            cpu.mmu.write_byte(cpu.reg.hl(), cpu.reg.a);
            cpu.reg.set_hl(cpu.reg.hl().wrapping_add(1));
        },
        0x32 => {
            cpu.mmu.write_byte(cpu.reg.hl(), cpu.reg.a);
            cpu.reg.set_hl(cpu.reg.hl().wrapping_sub(1));
        },

        0xE0 => {
            let cst = cpu.read_byte();
            cpu.mmu.write_byte(cst as u16, cpu.reg.a);
            return 3;
        },
        0xE2 => cpu.mmu.write_byte(cpu.reg.c as u16, cpu.reg.a),
        0xEA => {
            let cst = cpu.read_word();
            cpu.mmu.write_byte(cst, cpu.reg.a);
            return 4;
        },
        
        _ => panic!("Not a register to memory instruction; 0x{:02X}", opcode),
    }
    2
}

pub fn ld_cst_to_mem(cpu: &mut Cpu) -> u8 {
    let cst = cpu.read_byte();
    cpu.mmu.write_byte(cpu.reg.hl(), cst);
    3
}
