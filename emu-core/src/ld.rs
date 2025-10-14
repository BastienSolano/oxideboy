use std::panic;

use crate::memory::MemoryBus;
use crate::cpu::Cpu;

pub fn ld_reg_to_reg<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x40 => (), // Nothing to do (LD B B)
        0x41 => cpu.reg.b = cpu.reg.c,
        0x42 => cpu.reg.b = cpu.reg.d,
        0x43 => cpu.reg.b = cpu.reg.e,
        0x44 => cpu.reg.b = cpu.reg.h,
        0x45 => cpu.reg.b = cpu.reg.l,
        0x47 => cpu.reg.b = cpu.reg.a,
        0x48 => cpu.reg.c = cpu.reg.b,
        0x49 => (), // Nothing to do (LD C C)
        0x4A => cpu.reg.c = cpu.reg.d,
        0x4B => cpu.reg.c = cpu.reg.e,
        0x4C => cpu.reg.c = cpu.reg.h,
        0x4D => cpu.reg.c = cpu.reg.l,
        0x4F => cpu.reg.c = cpu.reg.a,
        
        0x50 => cpu.reg.d = cpu.reg.b,
        0x51 => cpu.reg.d = cpu.reg.c,
        0x52 => (),
        0x53 => cpu.reg.d = cpu.reg.e,
        0x54 => cpu.reg.d = cpu.reg.h,
        0x55 => cpu.reg.d = cpu.reg.l,
        0x57 => cpu.reg.d = cpu.reg.a,
        0x58 => cpu.reg.e = cpu.reg.b,
        0x59 => cpu.reg.e = cpu.reg.c,
        0x5A => cpu.reg.e = cpu.reg.d,
        0x5B => (), // Nothing to do (LD E E)
        0x5C => cpu.reg.e = cpu.reg.h,
        0x5D => cpu.reg.e = cpu.reg.l,
        0x5F => cpu.reg.e = cpu.reg.a,

        0x60 => cpu.reg.h = cpu.reg.b,
        0x61 => cpu.reg.h = cpu.reg.c,
        0x62 => cpu.reg.h = cpu.reg.d,
        0x63 => cpu.reg.h = cpu.reg.e,
        0x64 => (), // Nothing to do (LD H H)
        0x65 => cpu.reg.h = cpu.reg.l,
        0x67 => cpu.reg.h = cpu.reg.a,
        0x68 => cpu.reg.l = cpu.reg.b,
        0x69 => cpu.reg.l = cpu.reg.c,
        0x6A => cpu.reg.l = cpu.reg.d,
        0x6B => cpu.reg.l = cpu.reg.e,
        0x6C => cpu.reg.l = cpu.reg.h,
        0x6D => (), // Nothing to do (LD L L)
        0x6F => cpu.reg.l = cpu.reg.a,

        0x78 => cpu.reg.a = cpu.reg.b,
        0x79 => cpu.reg.a = cpu.reg.c,
        0x7A => cpu.reg.a = cpu.reg.d,
        0x7B => cpu.reg.a = cpu.reg.e,
        0x7C => cpu.reg.a = cpu.reg.h,
        0x7D => cpu.reg.a = cpu.reg.l,
        0x7F => (), // Nothing to do (LD A A)

        _ => panic!("Not a register to register load instruction: 0x{:02X}", opcode),
    }
    1
}

pub fn ld_cst_to_reg<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
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

pub fn ld_cst16_to_reg<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    let constant: u16 = cpu.read_word();
    match opcode {
        0x01 => cpu.reg.set_bc(constant),
        0x11 => cpu.reg.set_de(constant),
        0x21 => cpu.reg.set_hl(constant),
        0x31 => cpu.reg.sp = constant,
        _ => panic!("Not a constant to 16-bit register load instruction: 0x{:02X}", opcode),
    }
    3
}

pub fn ld_mem_to_reg<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        // load (hl) to all registers
        0x46 => cpu.reg.b = cpu.mmu.read_byte(cpu.reg.hl()),
        0x56 => cpu.reg.d = cpu.mmu.read_byte(cpu.reg.hl()),
        0x66 => cpu.reg.h = cpu.mmu.read_byte(cpu.reg.hl()),
        0x4E => cpu.reg.c = cpu.mmu.read_byte(cpu.reg.hl()),
        0x5E => cpu.reg.e = cpu.mmu.read_byte(cpu.reg.hl()),
        0x6E => cpu.reg.l = cpu.mmu.read_byte(cpu.reg.hl()),
        0x7E => cpu.reg.a = cpu.mmu.read_byte(cpu.reg.hl()),

        // loading mem(16-bit registers) in A
        0x0A => cpu.reg.a = cpu.mmu.read_byte(cpu.reg.bc()),
        0x1A => cpu.reg.a = cpu.mmu.read_byte(cpu.reg.de()),
        0x2A => {
            cpu.reg.a = cpu.mmu.read_byte(cpu.reg.hl());
            cpu.reg.set_hl(cpu.reg.hl() + 1);
            return 3; // additional cycle for incrementing HL
        },
        0x3A => {
            cpu.reg.a = cpu.mmu.read_byte(cpu.reg.hl());
            cpu.reg.set_hl(cpu.reg.hl() - 1);
            return 3; // additional cycle for decrementing HL
        },

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

pub fn ld_reg_to_mem<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
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

pub fn ld_cst_to_mem<M: MemoryBus>(cpu: &mut Cpu<M>) -> u8 {
    let cst = cpu.read_byte();
    cpu.mmu.write_byte(cpu.reg.hl(), cst);
    3
}
