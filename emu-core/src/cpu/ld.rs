use std::panic;

use crate::memory::MemoryBus;
use crate::cpu::cpu::Cpu;

use crate::cpu::registers::*;

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

        0xF8 => { // LD HL, SP+e8
            // Flags are calculated by treating the offset as unsigned
            // and adding it to the lower byte of SP (same as ADD SP,e8)
            let offset_byte = cpu.read_byte();
            let sp_lower = (cpu.reg.sp & 0x00FF) as u8;

            // Calculate flags using unsigned addition on lower byte
            cpu.reg.clear_flags();
            cpu.reg.set_flag(CpuFlag::H, add8_needs_half_carry(sp_lower, offset_byte));
            cpu.reg.set_flag(CpuFlag::C, add8_needs_carry(sp_lower, offset_byte));

            // Perform the actual 16-bit operation with signed offset
            let signed_offset = offset_byte as i8 as i16;
            let result = (cpu.reg.sp as i16).wrapping_add(signed_offset) as u16;
            cpu.reg.set_hl(result);
            cpu.mmu.tick_internal();

            return 3;
        }
        0xF9 => {
            cpu.reg.sp = cpu.reg.hl();
            // SP is special and takes an extra cycle to load from HL (no direct path)
            cpu.mmu.tick_internal();
            return 2; // extra cycle for 16-bit transfer
        }

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
            return 2; // read from (HL), then increment happens in same cycles
        },
        0x3A => {
            cpu.reg.a = cpu.mmu.read_byte(cpu.reg.hl());
            cpu.reg.set_hl(cpu.reg.hl() - 1);
            return 2; // read from (HL), then decrement happens in same cycles
        },

        // loading mem(0xFF00 + 8-bit constant) in A (LDH A,(a8))
        0xF0 => {
            let cst = cpu.read_byte();
            cpu.reg.a = cpu.mmu.read_byte(0xFF00 + cst as u16);
            return 3; // additional tick for reading the constant
        },

        // loading mem(0xFF00 + C) in A (LD A,(C))
        0xF2 => cpu.reg.a = cpu.mmu.read_byte(0xFF00 + cpu.reg.c as u16),

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

        0xE0 => { // LDH (a8),A
            let cst = cpu.read_byte();
            cpu.mmu.write_byte(0xFF00 + cst as u16, cpu.reg.a);
            return 3;
        },
        0xE2 => cpu.mmu.write_byte(0xFF00 + cpu.reg.c as u16, cpu.reg.a), // LD (C),A
        0xEA => {
            let cst = cpu.read_word();
            cpu.mmu.write_byte(cst, cpu.reg.a);
            return 4;
        },
        0x08 => {
            let cst: u16 = cpu.read_word();
            cpu.mmu.write_byte(cst, (cpu.reg.sp & 0xFF) as u8);
            cpu.mmu.write_byte(cst+1, (cpu.reg.sp >> 8) as u8);
            return 5;
        }
        
        _ => panic!("Not a register to memory instruction; 0x{:02X}", opcode),
    }
    2
}

pub fn ld_cst_to_mem<M: MemoryBus>(cpu: &mut Cpu<M>) -> u8 {
    let cst = cpu.read_byte();
    cpu.mmu.write_byte(cpu.reg.hl(), cst);
    3
}
