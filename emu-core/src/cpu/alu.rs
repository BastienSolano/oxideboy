use core::panic;

use crate::memory::MemoryBus;
use crate::cpu::cpu::Cpu;
use crate::cpu::registers::*;

macro_rules! incr_8bit_reg {
    ($cpu:expr, $myreg:ident) => {{
        // first check if half-carry
        $cpu.reg.set_flag(CpuFlag::H, add8_needs_half_carry($cpu.reg.$myreg, 1));

        $cpu.reg.$myreg = $cpu.reg.$myreg.wrapping_add(1);

        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$myreg == 0);
        $cpu.reg.set_flag(CpuFlag::N, false);

        return 1;
    }};
}

macro_rules! incr_16bit_reg {
    ($cpu:expr, $myreg:ident, $mysetreg:ident) => {{
        $cpu.mmu.tick_internal();
        $cpu.reg.$mysetreg($cpu.reg.$myreg().wrapping_add(1));
        return 2;
    }};
}

macro_rules! decr_8bit_reg {
    ($cpu:expr, $myreg:ident) => {{
        // first check if half-carry
        $cpu.reg.set_flag(CpuFlag::H, sub8_needs_half_carry($cpu.reg.$myreg, 1));

        $cpu.reg.$myreg = $cpu.reg.$myreg.wrapping_sub(1);

        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$myreg == 0);
        $cpu.reg.set_flag(CpuFlag::N, true);

        return 1;
    }};
}

macro_rules! decr_16bit_reg {
    ($cpu:expr, $myreg:ident, $mysetreg:ident) => {{
        $cpu.mmu.tick_internal();
        $cpu.reg.$mysetreg($cpu.reg.$myreg().wrapping_sub(1));
        return 2;
    }};
}

pub fn incr<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x04 => { incr_8bit_reg!(cpu, b) },
        0x14 => { incr_8bit_reg!(cpu, d) },
        0x24 => { incr_8bit_reg!(cpu, h) },
        0x0C => { incr_8bit_reg!(cpu, c) },
        0x1C => { incr_8bit_reg!(cpu, e) },
        0x2C => { incr_8bit_reg!(cpu, l) },
        0x3C => { incr_8bit_reg!(cpu, a) },
        0x03 => { incr_16bit_reg!(cpu, bc, set_bc) },
        0x13 => { incr_16bit_reg!(cpu, de, set_de) },
        0x23 => { incr_16bit_reg!(cpu, hl, set_hl) },
        0x33 => { incr_16bit_reg!(cpu, sp, set_sp) },
        0x34 => {
            // increment memory pointed by HL
            let addr = cpu.reg.hl();
            let val = cpu.mmu.read_byte(addr);
            // first check if half-carry
            cpu.reg.set_flag(CpuFlag::H, add8_needs_half_carry(val, 1));
            let newval = val.wrapping_add(1);
            cpu.mmu.write_byte(addr, newval);
            cpu.reg.set_flag(CpuFlag::Z, newval == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            return 3;
        },
        _ => panic!("Not a register incr instruction: 0x{:02X}", opcode),
    }
}

pub fn decr<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x05 => { decr_8bit_reg!(cpu, b) },
        0x15 => { decr_8bit_reg!(cpu, d) },
        0x25 => { decr_8bit_reg!(cpu, h) },
        0x0D => { decr_8bit_reg!(cpu, c) },
        0x1D => { decr_8bit_reg!(cpu, e) },
        0x2D => { decr_8bit_reg!(cpu, l) },
        0x3D => { decr_8bit_reg!(cpu, a) },
        0x0B => { decr_16bit_reg!(cpu, bc, set_bc) },
        0x1B => { decr_16bit_reg!(cpu, de, set_de) },
        0x2B => { decr_16bit_reg!(cpu, hl, set_hl) },
        0x3B => { decr_16bit_reg!(cpu, sp, set_sp) },
        0x35 => {
            // decrement memory pointed by HL
            let addr = cpu.reg.hl();
            let val = cpu.mmu.read_byte(addr);
            // first check if half-carry
            cpu.reg.set_flag(CpuFlag::H, sub8_needs_half_carry(val, 1));
            let newval = val.wrapping_sub(1);
            cpu.mmu.write_byte(addr, newval);
            cpu.reg.set_flag(CpuFlag::Z, newval == 0);
            cpu.reg.set_flag(CpuFlag::N, true);
            return 3;
        },
        _ => panic!("Not a register decr instruction: 0x{:02X}", opcode),
    }
}

macro_rules! add_reg8_reg8 {
    ($cpu:expr, $dst_reg:ident, $src_reg:ident) => {{
        $cpu.reg.set_flag(CpuFlag::H, add8_needs_half_carry($cpu.reg.$dst_reg, $cpu.reg.$src_reg));
        $cpu.reg.set_flag(CpuFlag::C, add8_needs_carry($cpu.reg.$dst_reg, $cpu.reg.$src_reg));

        $cpu.reg.$dst_reg = $cpu.reg.$dst_reg.wrapping_add($cpu.reg.$src_reg);

        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$dst_reg == 0);
        $cpu.reg.set_flag(CpuFlag::N, false);

        return 1;
    }};
}

macro_rules! add_reg16_reg16 {
    ($cpu:expr, $dst_reg:ident, $set_dst_reg:ident, $src_reg:ident) => {{
        $cpu.mmu.tick_internal();
        $cpu.reg.set_flag(CpuFlag::H, add16_needs_half_carry($cpu.reg.$dst_reg(), $cpu.reg.$src_reg()));
        $cpu.reg.set_flag(CpuFlag::C, add16_needs_carry($cpu.reg.$dst_reg(), $cpu.reg.$src_reg()));

        $cpu.reg.$set_dst_reg($cpu.reg.$dst_reg().wrapping_add($cpu.reg.$src_reg()));

        $cpu.reg.set_flag(CpuFlag::N, false);

        return 2;
    }};
}

pub fn add<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x80 => { add_reg8_reg8!(cpu, a, b) },
        0x81 => { add_reg8_reg8!(cpu, a, c) },
        0x82 => { add_reg8_reg8!(cpu, a, d) },
        0x83 => { add_reg8_reg8!(cpu, a, e) },
        0x84 => { add_reg8_reg8!(cpu, a, h) },
        0x85 => { add_reg8_reg8!(cpu, a, l) },
        0x87 => { add_reg8_reg8!(cpu, a, a) },
        0x09 => { add_reg16_reg16!(cpu, hl, set_hl, bc) },
        0x19 => { add_reg16_reg16!(cpu, hl, set_hl, de) },
        0x29 => { add_reg16_reg16!(cpu, hl, set_hl, hl) },
        0x39 => { add_reg16_reg16!(cpu, hl, set_hl, sp) },
        0x86 => {
            // ADD A, (HL)
            let addr = cpu.reg.hl();
            let val = cpu.mmu.read_byte(addr);
            cpu.reg.set_flag(CpuFlag::H, add8_needs_half_carry(cpu.reg.a, val));
            cpu.reg.set_flag(CpuFlag::C, add8_needs_carry(cpu.reg.a, val));
            cpu.reg.a = cpu.reg.a.wrapping_add(val);
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            return 2;
        },
        0xc6 => {
            let cst = cpu.read_byte();
            cpu.reg.set_flag(CpuFlag::H, add8_needs_half_carry(cpu.reg.a, cst));
            cpu.reg.set_flag(CpuFlag::C, add8_needs_carry(cpu.reg.a, cst));
            cpu.reg.a = cpu.reg.a.wrapping_add(cst);
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            return 2;
        },
        0xe8 => {
            // ADD SP, s8
            // Flags are ALWAYS calculated by treating the offset as unsigned
            // and adding it to the lower byte of SP, regardless of sign
            let offset_byte = cpu.read_byte();
            let sp_lower = (cpu.reg.sp & 0x00FF) as u8;

            // Calculate flags using unsigned addition on lower byte
            cpu.reg.set_flag(CpuFlag::H, add8_needs_half_carry(sp_lower, offset_byte));
            cpu.reg.set_flag(CpuFlag::C, add8_needs_carry(sp_lower, offset_byte));

            // Perform the actual 16-bit operation with signed offset
            let signed_offset = offset_byte as i8 as i16;
            cpu.mmu.tick_internal();
            cpu.mmu.tick_internal(); // two internal ticks for 16-bit operation

            cpu.reg.sp = (cpu.reg.sp as i16).wrapping_add(signed_offset) as u16;

            cpu.reg.set_flag(CpuFlag::Z, false);
            cpu.reg.set_flag(CpuFlag::N, false);
            return 4;
        },
        _ => panic!("Not ADD instruction: 0x{:02X}", opcode),
    }
}

macro_rules! sub_a_reg8 {
    ($cpu:expr, $src_reg:ident) => {{
        $cpu.reg.set_flag(CpuFlag::H, sub8_needs_half_carry($cpu.reg.a, $cpu.reg.$src_reg));
        $cpu.reg.set_flag(CpuFlag::C, sub8_needs_carry($cpu.reg.a, $cpu.reg.$src_reg));

        $cpu.reg.a = $cpu.reg.a.wrapping_sub($cpu.reg.$src_reg);

        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.a == 0);
        $cpu.reg.set_flag(CpuFlag::N, true);

        return 1;
    }};
}

pub fn sub<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x90 => { sub_a_reg8!(cpu, b) },
        0x91 => { sub_a_reg8!(cpu, c) },
        0x92 => { sub_a_reg8!(cpu, d) },
        0x93 => { sub_a_reg8!(cpu, e) },
        0x94 => { sub_a_reg8!(cpu, h) },
        0x95 => { sub_a_reg8!(cpu, l) },
        0x97 => { sub_a_reg8!(cpu, a) },
        0x96 => {
            // ADD A, (HL)
            let addr = cpu.reg.hl();
            let val = cpu.mmu.read_byte(addr);
            cpu.reg.set_flag(CpuFlag::H, sub8_needs_half_carry(cpu.reg.a, val));
            cpu.reg.set_flag(CpuFlag::C, sub8_needs_carry(cpu.reg.a, val));
            cpu.reg.a = cpu.reg.a.wrapping_sub(val);
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, true);
            return 2;
        },
        0xd6 => {
            let cst = cpu.read_byte();
            cpu.reg.set_flag(CpuFlag::H, sub8_needs_half_carry(cpu.reg.a, cst));
            cpu.reg.set_flag(CpuFlag::C, sub8_needs_carry(cpu.reg.a, cst));
            cpu.reg.a = cpu.reg.a.wrapping_sub(cst);
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N,true);
            return 2;
        },
        _ => panic!("Not a SUB instruction: 0x{:02X}", opcode),
    }
}

macro_rules! add_carry_a_reg8 {
    ($cpu:expr, $reg:ident) => {{
        let carry = if $cpu.reg.get_flag(CpuFlag::C) { 1 } else { 0 };

        $cpu.reg.set_flag(CpuFlag::H, adc_needs_half_carry($cpu.reg.a, $cpu.reg.$reg, carry));
        $cpu.reg.set_flag(CpuFlag::C, adc_needs_carry($cpu.reg.a, $cpu.reg.$reg, carry));

        $cpu.reg.a = $cpu.reg.a.wrapping_add($cpu.reg.$reg).wrapping_add(carry);

        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.a == 0);
        $cpu.reg.set_flag(CpuFlag::N, false);

        return 1;
    }};
}

pub fn adc<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x88 => { add_carry_a_reg8!(cpu, b) },
        0x89 => { add_carry_a_reg8!(cpu, c) },
        0x8A => { add_carry_a_reg8!(cpu, d) },
        0x8B => { add_carry_a_reg8!(cpu, e) },
        0x8C => { add_carry_a_reg8!(cpu, h) },
        0x8D => { add_carry_a_reg8!(cpu, l) },
        0x8F => { add_carry_a_reg8!(cpu, a) },
        0x8E => {
            // ADC A, (HL)
            let addr = cpu.reg.hl();
            let val = cpu.mmu.read_byte(addr);
            let carry = if cpu.reg.get_flag(CpuFlag::C) { 1 } else { 0 };
            cpu.reg.set_flag(CpuFlag::H, adc_needs_half_carry(cpu.reg.a, val, carry));
            cpu.reg.set_flag(CpuFlag::C, adc_needs_carry(cpu.reg.a, val, carry));
            cpu.reg.a = cpu.reg.a.wrapping_add(val).wrapping_add(carry);
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            return 2;
        },
        0xCE => {
            let cst = cpu.read_byte();
            let carry = if cpu.reg.get_flag(CpuFlag::C) { 1 } else { 0 };
            cpu.reg.set_flag(CpuFlag::H, adc_needs_half_carry(cpu.reg.a, cst, carry));
            cpu.reg.set_flag(CpuFlag::C, adc_needs_carry(cpu.reg.a, cst, carry));
            cpu.reg.a = cpu.reg.a.wrapping_add(cst).wrapping_add(carry);
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            return 2;
        },
        _ => panic!("Not an ADC instruction: 0x{:02X}", opcode),
    }
}

macro_rules! sub_carry_a_reg8 {
    ($cpu:expr, $reg:ident) => {{
        let carry = if $cpu.reg.get_flag(CpuFlag::C) { 1 } else { 0 };

        $cpu.reg.set_flag(CpuFlag::H, sbc_needs_half_carry($cpu.reg.a, $cpu.reg.$reg, carry));
        $cpu.reg.set_flag(CpuFlag::C, sbc_needs_carry($cpu.reg.a, $cpu.reg.$reg, carry));

        $cpu.reg.a = $cpu.reg.a.wrapping_sub($cpu.reg.$reg).wrapping_sub(carry);

        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.a == 0);
        $cpu.reg.set_flag(CpuFlag::N, true);

        return 1;
    }};
}

pub fn sbc<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x98 => { sub_carry_a_reg8!(cpu, b) },
        0x99 => { sub_carry_a_reg8!(cpu, c) },
        0x9A => { sub_carry_a_reg8!(cpu, d) },
        0x9B => { sub_carry_a_reg8!(cpu, e) },
        0x9C => { sub_carry_a_reg8!(cpu, h) },
        0x9D => { sub_carry_a_reg8!(cpu, l) },
        0x9F => { sub_carry_a_reg8!(cpu, a) },
        0x9E => {
            // SBC A, (HL)
            let addr = cpu.reg.hl();
            let val = cpu.mmu.read_byte(addr);
            let carry = if cpu.reg.get_flag(CpuFlag::C) { 1 } else { 0 };
            cpu.reg.set_flag(CpuFlag::H, sbc_needs_half_carry(cpu.reg.a, val, carry));
            cpu.reg.set_flag(CpuFlag::C, sbc_needs_carry(cpu.reg.a, val, carry));
            cpu.reg.a = cpu.reg.a.wrapping_sub(val).wrapping_sub(carry);
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, true);
            return 2;
        },
        0xDE => {
            // SBC A, d8
            let cst = cpu.read_byte();
            let carry = if cpu.reg.get_flag(CpuFlag::C) { 1 } else { 0 };
            cpu.reg.set_flag(CpuFlag::H, sbc_needs_half_carry(cpu.reg.a, cst, carry));
            cpu.reg.set_flag(CpuFlag::C, sbc_needs_carry(cpu.reg.a, cst, carry));
            cpu.reg.a = cpu.reg.a.wrapping_sub(cst).wrapping_sub(carry);
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, true);
            return 2;
        },
        _ => panic!("Not a SBC instruction: 0x{:02X}", opcode),
    }
}

macro_rules! and_a_reg8 {
    ($cpu:expr, $reg:ident) => {{
        $cpu.reg.a &= $cpu.reg.$reg;
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.a == 0);
        $cpu.reg.set_flag(CpuFlag::N, false);
        $cpu.reg.set_flag(CpuFlag::H, true);
        $cpu.reg.set_flag(CpuFlag::C, false);
        return 1;
    }};
}

pub fn and<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0xA0 => { and_a_reg8!(cpu, b) },
        0xA1 => { and_a_reg8!(cpu, c) },
        0xA2 => { and_a_reg8!(cpu, d) },
        0xA3 => { and_a_reg8!(cpu, e) },
        0xA4 => { and_a_reg8!(cpu, h) },
        0xA5 => { and_a_reg8!(cpu, l) },
        0xA7 => { and_a_reg8!(cpu, a) },
        0xA6 => {
            // AND A, (HL)
            let addr = cpu.reg.hl();
            let val = cpu.mmu.read_byte(addr);
            cpu.reg.a &= val;
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            cpu.reg.set_flag(CpuFlag::H, true);
            cpu.reg.set_flag(CpuFlag::C, false);
            return 2;
        },
        0xE6 => {
            let cst = cpu.read_byte();
            cpu.reg.a &= cst;
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            cpu.reg.set_flag(CpuFlag::H, true);
            cpu.reg.set_flag(CpuFlag::C, false);
            return 2;
        },
        _ => panic!("Not ADD instruction: 0x{:02X}", opcode),
    }
}

macro_rules! or_a_reg8 {
    ($cpu:expr, $reg:ident) => {{
        $cpu.reg.a |= $cpu.reg.$reg;
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.a == 0);
        $cpu.reg.set_flag(CpuFlag::N, false);
        $cpu.reg.set_flag(CpuFlag::H, false);
        $cpu.reg.set_flag(CpuFlag::C, false);
        return 1;
    }};
}

pub fn or<M:MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0xB0 => { or_a_reg8!(cpu, b) },
        0xB1 => { or_a_reg8!(cpu, c) },
        0xB2 => { or_a_reg8!(cpu, d) },
        0xB3 => { or_a_reg8!(cpu, e) },
        0xB4 => { or_a_reg8!(cpu, h) },
        0xB5 => { or_a_reg8!(cpu, l) },
        0xB7 => { or_a_reg8!(cpu, a) },
        0xB6 => {
            // OR A, (HL)
            let addr = cpu.reg.hl();
            let val = cpu.mmu.read_byte(addr);
            cpu.reg.a |= val;
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            cpu.reg.set_flag(CpuFlag::H, false);
            cpu.reg.set_flag(CpuFlag::C, false);
            return 2;
        },
        0xF6 => {
            let cst = cpu.read_byte();
            cpu.reg.a |= cst;
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            cpu.reg.set_flag(CpuFlag::H, false);
            cpu.reg.set_flag(CpuFlag::C, false);
            return 2;
        },
        _ => panic!("Not OR instruction: 0x{:02X}", opcode),
    }
}

macro_rules! xor_a_reg8 {
    ($cpu:expr, $reg:ident) => {{
        $cpu.reg.a ^= $cpu.reg.$reg;
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.a == 0);
        $cpu.reg.set_flag(CpuFlag::N, false);
        $cpu.reg.set_flag(CpuFlag::H, false);
        $cpu.reg.set_flag(CpuFlag::C, false);
        return 1;
    }};
}

pub fn xor<M:MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0xA8 => { xor_a_reg8!(cpu, b) },
        0xA9 => { xor_a_reg8!(cpu, c) },
        0xAA => { xor_a_reg8!(cpu, d) },
        0xAB => { xor_a_reg8!(cpu, e) },
        0xAC => { xor_a_reg8!(cpu, h) },
        0xAD => { xor_a_reg8!(cpu, l) },
        0xAF => { xor_a_reg8!(cpu, a) },
        0xAE => {
            // XOR A, (HL)
            let addr = cpu.reg.hl();
            let val = cpu.mmu.read_byte(addr);
            cpu.reg.a ^= val;
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            cpu.reg.set_flag(CpuFlag::H, false);
            cpu.reg.set_flag(CpuFlag::C, false);
            return 2;
        },
        0xEE => {
            let cst = cpu.read_byte();
            cpu.reg.a ^= cst;
            cpu.reg.set_flag(CpuFlag::Z, cpu.reg.a == 0);
            cpu.reg.set_flag(CpuFlag::N, false);
            cpu.reg.set_flag(CpuFlag::H, false);
            cpu.reg.set_flag(CpuFlag::C, false);
            return 2;
        },
        _ => panic!("Not OR instruction: 0x{:02X}", opcode),
    }
}

macro_rules!  cp_a_reg8 {
    ($cpu:expr, $reg:ident) => {{
        $cpu.reg.set_flag(CpuFlag::H, sub8_needs_half_carry($cpu.reg.a, $cpu.reg.$reg));
        $cpu.reg.set_flag(CpuFlag::C, sub8_needs_carry($cpu.reg.a, $cpu.reg.$reg));

        let result = $cpu.reg.a.wrapping_sub($cpu.reg.$reg);

        $cpu.reg.set_flag(CpuFlag::Z, result == 0);
        $cpu.reg.set_flag(CpuFlag::N, true);

        return 1;
    }};
}

pub fn cp<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0xB8 => { cp_a_reg8!(cpu, b) },
        0xB9 => { cp_a_reg8!(cpu, c) },
        0xBA => { cp_a_reg8!(cpu, d) },
        0xBB => { cp_a_reg8!(cpu, e) },
        0xBC => { cp_a_reg8!(cpu, h) },
        0xBD => { cp_a_reg8!(cpu, l) },
        0xBF => { cp_a_reg8!(cpu, a) },
        0xBE => {
            let addr = cpu.reg.hl();
            let val = cpu.mmu.read_byte(addr);

            cpu.reg.set_flag(CpuFlag::H, sub8_needs_half_carry(cpu.reg.a, val));
            cpu.reg.set_flag(CpuFlag::C, sub8_needs_carry(cpu.reg.a, val));

            let result = cpu.reg.a.wrapping_sub(val);

            cpu.reg.set_flag(CpuFlag::Z, result == 0);
            cpu.reg.set_flag(CpuFlag::N, true);

            return 2;
        },
        0xFE => {
            let cst = cpu.read_byte();

            cpu.reg.set_flag(CpuFlag::H, sub8_needs_half_carry(cpu.reg.a, cst));
            cpu.reg.set_flag(CpuFlag::C, sub8_needs_carry(cpu.reg.a, cst));

            let result = cpu.reg.a.wrapping_sub(cst);

            cpu.reg.set_flag(CpuFlag::Z, result == 0);
            cpu.reg.set_flag(CpuFlag::N, true);

            return 2;
        },
        _ => panic!("Not a CP instruction: 0x{:02X}", opcode),
    }
}

pub fn rla<M: MemoryBus>(cpu: &mut Cpu<M>) -> u8 {
    let carry: u8 = if cpu.reg.get_flag(CpuFlag::C) { 1} else { 0 };
    cpu.reg.clear_flags();
    cpu.reg.set_flag(CpuFlag::C, cpu.reg.a & 0x80 > 0); // C flag <- bit 7 of A
    cpu.reg.a = cpu.reg.a << 1;
    cpu.reg.a = ( cpu.reg.a & 0xFE ) | carry;
    2
}

pub fn rra<M: MemoryBus>(cpu: &mut Cpu<M>) -> u8 {
    let carry: u8 = if cpu.reg.get_flag(CpuFlag::C) { 1} else { 0 };
    cpu.reg.clear_flags();
    cpu.reg.set_flag(CpuFlag::C, cpu.reg.a & 1 > 0); // C flag <- bit 7 of A
    cpu.reg.a = cpu.reg.a >> 1;
    cpu.reg.a = ( cpu.reg.a & 0x7F ) | ( carry << 7 );
    2
}

pub fn rlca<M: MemoryBus>(cpu: &mut Cpu<M>) -> u8 {
    cpu.reg.clear_flags();
    let bit7 = (cpu.reg.a & 0x80) >> 7;
    cpu.reg.a = cpu.reg.a << 1;
    cpu.reg.a = ( cpu.reg.a & 0xFE ) | bit7;
    cpu.reg.set_flag(CpuFlag::C, bit7 == 1);
    2
}

pub fn rrca<M: MemoryBus>(cpu: &mut Cpu<M>) -> u8 {
    cpu.reg.clear_flags();
    let bit0 = cpu.reg.a & 1;
    cpu.reg.a = cpu.reg.a >> 1;
    cpu.reg.a = ( cpu.reg.a & 0x7F ) | ( bit0 << 7 );
    cpu.reg.set_flag(CpuFlag::C, bit0 == 1);
    2
}

macro_rules! rlc_reg8 {
    ($cpu:expr, $reg:ident) => {{
        $cpu.reg.clear_flags();
        let bit7 = ($cpu.reg.$reg & 0x80) >> 7;
        $cpu.reg.set_flag(CpuFlag::C, bit7 != 0);
        $cpu.reg.$reg <<= 1;
        $cpu.reg.$reg |= bit7; 
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$reg == 0);
    }};
}

pub fn rlc<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x00 => rlc_reg8!(cpu, b),
        0x01 => rlc_reg8!(cpu, c),
        0x02 => rlc_reg8!(cpu, d),
        0x03 => rlc_reg8!(cpu, e),
        0x04 => rlc_reg8!(cpu, h),
        0x05 => rlc_reg8!(cpu, l),
        0x07 => rlc_reg8!(cpu, a),
        0x06 => {
            let mut cst = cpu.mmu.read_byte(cpu.reg.hl());

            cpu.reg.clear_flags();

            let bit7 = (cst & 0x80) >> 7;
            cpu.reg.set_flag(CpuFlag::C, bit7 != 0);
            cst <<= 1;
            cst |= bit7; 
            cpu.reg.set_flag(CpuFlag::Z, cst == 0);


            cpu.mmu.write_byte(cpu.reg.hl(), cst);

            return 4;
        },
        _ => panic!("Not a RLC instruction: 0x{:02X}", opcode),
    }
    2
}

macro_rules! rrc_reg8 {
    ($cpu:expr, $reg:ident) => {{
        $cpu.reg.clear_flags();
        let bit0 = $cpu.reg.$reg & 1;
        $cpu.reg.set_flag(CpuFlag::C, bit0 != 0);
        $cpu.reg.$reg >>= 1;
        $cpu.reg.$reg |= bit0 << 7; 
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$reg == 0);
    }};
}

pub fn rrc<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x08 => rrc_reg8!(cpu, b),
        0x09 => rrc_reg8!(cpu, c),
        0x0A => rrc_reg8!(cpu, d),
        0x0B => rrc_reg8!(cpu, e),
        0x0C => rrc_reg8!(cpu, h),
        0x0D => rrc_reg8!(cpu, l),
        0x0F => rrc_reg8!(cpu, a),
        0x0E => {
            let mut cst = cpu.mmu.read_byte(cpu.reg.hl());

            cpu.reg.clear_flags();

            let bit0 = cst & 1;
            cpu.reg.set_flag(CpuFlag::C, bit0 != 0);
            cst >>= 1;
            cst |= bit0 << 7; 
            cpu.reg.set_flag(CpuFlag::Z, cst == 0);


            cpu.mmu.write_byte(cpu.reg.hl(), cst);

            return 4;
        },
        _ => panic!("Not a RRC instruction: 0x{:02X}", opcode),
    }
    2
}

macro_rules! rl_reg8 {
    ($cpu:expr, $reg:ident) => {{
        let carry = if $cpu.reg.get_flag(CpuFlag::C) { 1 } else { 0 };
        let bit7 = ($cpu.reg.$reg & 0x80) >> 7;
        $cpu.reg.clear_flags();
        $cpu.reg.set_flag(CpuFlag::C, bit7 != 0);
        $cpu.reg.$reg <<= 1;
        $cpu.reg.$reg |= carry;
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$reg == 0);
    }};
}

pub fn rl<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x10 => rl_reg8!(cpu, b),
        0x11 => rl_reg8!(cpu, c),
        0x12 => rl_reg8!(cpu, d),
        0x13 => rl_reg8!(cpu, e),
        0x14 => rl_reg8!(cpu, h),
        0x15 => rl_reg8!(cpu, l),
        0x17 => rl_reg8!(cpu, a),
        0x16 => {
            let mut cst = cpu.mmu.read_byte(cpu.reg.hl());

            let bit7 = (cst & 0x80) >> 7;
            let carry = if cpu.reg.get_flag(CpuFlag::C) { 1 } else { 0 };

            cpu.reg.clear_flags();

            cpu.reg.set_flag(CpuFlag::C, bit7 != 0);
            cst <<= 1;
            cst |= carry; 
            cpu.reg.set_flag(CpuFlag::Z, cst == 0);


            cpu.mmu.write_byte(cpu.reg.hl(), cst);

            return 4;
        },
        _ => panic!("Not a RL instruction: 0x{:02X}", opcode),
    }
    2
}

macro_rules! rr_reg8 {
    ($cpu:expr, $reg:ident) => {{
        let carry = if $cpu.reg.get_flag(CpuFlag::C) { 1 } else { 0 };
        let bit0 = $cpu.reg.$reg & 1;
        $cpu.reg.clear_flags();
        $cpu.reg.set_flag(CpuFlag::C, bit0 != 0);
        $cpu.reg.$reg >>= 1;
        $cpu.reg.$reg |= carry << 7;
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$reg == 0);
    }};
}

pub fn rr<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x18 => rr_reg8!(cpu, b),
        0x19 => rr_reg8!(cpu, c),
        0x1A => rr_reg8!(cpu, d),
        0x1B => rr_reg8!(cpu, e),
        0x1C => rr_reg8!(cpu, h),
        0x1D => rr_reg8!(cpu, l),
        0x1F => rr_reg8!(cpu, a),
        0x1E => {
            let mut cst = cpu.mmu.read_byte(cpu.reg.hl());

            let bit0 = cst & 1;
            let carry = if cpu.reg.get_flag(CpuFlag::C) { 1 } else { 0 };

            cpu.reg.clear_flags();
            cpu.reg.set_flag(CpuFlag::C, bit0 != 0);
            cst >>= 1;
            cst |= carry << 7; 
            cpu.reg.set_flag(CpuFlag::Z, cst == 0);


            cpu.mmu.write_byte(cpu.reg.hl(), cst);

            return 4;
        },
        _ => panic!("Not a RRC instruction: 0xCB{:02X}", opcode),
    }
    2
}

macro_rules! sla_reg8 {
    ($cpu:expr, $reg:ident) => {{
        let bit7 = ($cpu.reg.$reg & 0x80) >> 7;

        $cpu.reg.$reg <<= 1;
        $cpu.reg.$reg &= 0xFE;

        $cpu.reg.clear_flags();
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$reg == 0);
        $cpu.reg.set_flag(CpuFlag::C, bit7 == 1);

        2
    }};
}

pub fn sla<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x20 => sla_reg8!(cpu, b),
        0x21 => sla_reg8!(cpu, c),
        0x22 => sla_reg8!(cpu, d),
        0x23 => sla_reg8!(cpu, e),
        0x24 => sla_reg8!(cpu, h),
        0x25 => sla_reg8!(cpu, l),
        0x27 => sla_reg8!(cpu, a),
        0x26 => {
            let mut val = cpu.mmu.read_byte(cpu.reg.hl());

            let bit7 = (val & 0x80) >> 7;

            val <<= 1;
            val &= 0xFE;

            cpu.reg.clear_flags();
            cpu.reg.set_flag(CpuFlag::Z, val == 0);
            cpu.reg.set_flag(CpuFlag::C, bit7 == 1);

            cpu.mmu.write_byte(cpu.reg.hl(), val);

            4
        },
        _ => panic!("Not a SLA instrction: 0xCB{:02X}", opcode),
    }
}

macro_rules! sra_reg8 {
    ($cpu:expr, $reg:ident) => {{
        let bit0 = $cpu.reg.$reg & 1;
        let bit7 = ($cpu.reg.$reg & 0x80) >> 7;

        $cpu.reg.$reg >>= 1;
        $cpu.reg.$reg = ( $cpu.reg.$reg & 0x7F ) | ( bit7 << 7 );

        $cpu.reg.clear_flags();
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$reg == 0);
        $cpu.reg.set_flag(CpuFlag::C, bit0 == 1);

        2
    }};
}

pub fn sra<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x28 => sra_reg8!(cpu, b),
        0x29 => sra_reg8!(cpu, c),
        0x2A => sra_reg8!(cpu, d),
        0x2B => sra_reg8!(cpu, e),
        0x2C => sra_reg8!(cpu, h),
        0x2D => sra_reg8!(cpu, l),
        0x2F => sra_reg8!(cpu, a),
        0x2E => {
            let mut val = cpu.mmu.read_byte(cpu.reg.hl());

            let bit0 = val & 1;
            let bit7 = (val & 0x80) >> 7;

            val >>= 1;
            val = ( val & 0x7F ) | ( bit7 << 7 );

            cpu.reg.clear_flags();
            cpu.reg.set_flag(CpuFlag::Z, val == 0);
            cpu.reg.set_flag(CpuFlag::C, bit0 == 1);

            cpu.mmu.write_byte(cpu.reg.hl(), val);

            4
        },
        _ => panic!("Not a SRA instrction: 0xCB{:02X}", opcode),
    }
}

macro_rules! swap_reg8 {
    ($cpu:expr, $reg:ident) => {{
        $cpu.reg.clear_flags();
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$reg == 0);

        let high = ($cpu.reg.$reg & 0xF0) >> 4;
        $cpu.reg.$reg <<= 4;
        $cpu.reg.$reg |= high;

        2
    }};
}

pub fn swap<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x30 => swap_reg8!(cpu, b),
        0x31 => swap_reg8!(cpu, c),
        0x32 => swap_reg8!(cpu, d),
        0x33 => swap_reg8!(cpu, e),
        0x34 => swap_reg8!(cpu, h),
        0x35 => swap_reg8!(cpu, l),
        0x37 => swap_reg8!(cpu, a),
        0x36 => {
            let mut val = cpu.mmu.read_byte(cpu.reg.hl());

            cpu.reg.clear_flags();
            cpu.reg.set_flag(CpuFlag::Z, val == 0);

            let high = (val & 0xF0) >> 4;
            val <<= 4;
            val |= high;

            cpu.mmu.write_byte(cpu.reg.hl(), val);

            4
        },
        _ => panic!("Not a SWAP instruciton 0xCB{:02X}", opcode),
    }
}

macro_rules! srl_reg8 {
    ($cpu:expr, $reg:ident) => {{
        let bit0 = $cpu.reg.$reg & 1;

        $cpu.reg.$reg >>= 1;
        $cpu.reg.$reg = $cpu.reg.$reg & 0x7F;

        $cpu.reg.clear_flags();
        $cpu.reg.set_flag(CpuFlag::Z, $cpu.reg.$reg == 0);
        $cpu.reg.set_flag(CpuFlag::C, bit0 == 1);

        2
    }};
}

pub fn srl<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x38 => srl_reg8!(cpu, b),
        0x39 => srl_reg8!(cpu, c),
        0x3A => srl_reg8!(cpu, d),
        0x3B => srl_reg8!(cpu, e),
        0x3C => srl_reg8!(cpu, h),
        0x3D => srl_reg8!(cpu, l),
        0x3F => srl_reg8!(cpu, a),
        0x3E => {
            let mut val = cpu.mmu.read_byte(cpu.reg.hl());

            let bit0 = val & 1;

            val >>= 1;
            val = val & 0x7F;

            cpu.reg.clear_flags();
            cpu.reg.set_flag(CpuFlag::Z, val == 0);
            cpu.reg.set_flag(CpuFlag::C, bit0 == 1);

            cpu.mmu.write_byte(cpu.reg.hl(), val);

            4
        },
        _ => panic!("Not a SRL instruction: 0xCB{:02X}", opcode),
    }
}

macro_rules! bit_reg8 {
    ($cpu:expr, $reg:ident, $bit:expr) => {{
        $cpu.reg.set_flag(CpuFlag::N, false);
        $cpu.reg.set_flag(CpuFlag::H, true);

        let considered_bit = $cpu.reg.$reg & (1 << $bit);
        $cpu.reg.set_flag(CpuFlag::Z, considered_bit == 0);

        2
    }};
}

macro_rules! bit_hl {
    ($cpu:expr, $bit:expr) => {{
        let val = $cpu.mmu.read_byte($cpu.reg.hl());

        $cpu.reg.set_flag(CpuFlag::N, false);
        $cpu.reg.set_flag(CpuFlag::H, true);

        let val = val & (1 << $bit);
        $cpu.reg.set_flag(CpuFlag::Z, val == 0);

        3
    }};
}

pub fn bit<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x40 => bit_reg8!(cpu, b, 0),
        0x41 => bit_reg8!(cpu, c, 0),
        0x42 => bit_reg8!(cpu, d, 0),
        0x43 => bit_reg8!(cpu, e, 0),
        0x44 => bit_reg8!(cpu, h, 0),
        0x45 => bit_reg8!(cpu, l, 0),
        0x46 => bit_hl!(cpu, 0),
        0x47 => bit_reg8!(cpu, a, 0),

        0x48 => bit_reg8!(cpu, b, 1),
        0x49 => bit_reg8!(cpu, c, 1),
        0x4A => bit_reg8!(cpu, d, 1),
        0x4B => bit_reg8!(cpu, e, 1),
        0x4C => bit_reg8!(cpu, h, 1),
        0x4D => bit_reg8!(cpu, l, 1),
        0x4E => bit_hl!(cpu, 1),
        0x4F => bit_reg8!(cpu, a, 1),

        0x50 => bit_reg8!(cpu, b, 2),
        0x51 => bit_reg8!(cpu, c, 2),
        0x52 => bit_reg8!(cpu, d, 2),
        0x53 => bit_reg8!(cpu, e, 2),
        0x54 => bit_reg8!(cpu, h, 2),
        0x55 => bit_reg8!(cpu, l, 2),
        0x56 => bit_hl!(cpu, 2),
        0x57 => bit_reg8!(cpu, a, 2),

        0x58 => bit_reg8!(cpu, b, 3),
        0x59 => bit_reg8!(cpu, c, 3),
        0x5A => bit_reg8!(cpu, d, 3),
        0x5B => bit_reg8!(cpu, e, 3),
        0x5C => bit_reg8!(cpu, h, 3),
        0x5D => bit_reg8!(cpu, l, 3),
        0x5E => bit_hl!(cpu, 3),
        0x5F => bit_reg8!(cpu, a, 3),

        0x60 => bit_reg8!(cpu, b, 4),
        0x61 => bit_reg8!(cpu, c, 4),
        0x62 => bit_reg8!(cpu, d, 4),
        0x63 => bit_reg8!(cpu, e, 4),
        0x64 => bit_reg8!(cpu, h, 4),
        0x65 => bit_reg8!(cpu, l, 4),
        0x66 => bit_hl!(cpu, 4),
        0x67 => bit_reg8!(cpu, a, 4),

        0x68 => bit_reg8!(cpu, b, 5),
        0x69 => bit_reg8!(cpu, c, 5),
        0x6A => bit_reg8!(cpu, d, 5),
        0x6B => bit_reg8!(cpu, e, 5),
        0x6C => bit_reg8!(cpu, h, 5),
        0x6D => bit_reg8!(cpu, l, 5),
        0x6E => bit_hl!(cpu, 5),
        0x6F => bit_reg8!(cpu, a, 5),

        0x70 => bit_reg8!(cpu, b, 6),
        0x71 => bit_reg8!(cpu, c, 6),
        0x72 => bit_reg8!(cpu, d, 6),
        0x73 => bit_reg8!(cpu, e, 6),
        0x74 => bit_reg8!(cpu, h, 6),
        0x75 => bit_reg8!(cpu, l, 6),
        0x76 => bit_hl!(cpu, 6),
        0x77 => bit_reg8!(cpu, a, 6),

        0x78 => bit_reg8!(cpu, b, 7),
        0x79 => bit_reg8!(cpu, c, 7),
        0x7A => bit_reg8!(cpu, d, 7),
        0x7B => bit_reg8!(cpu, e, 7),
        0x7C => bit_reg8!(cpu, h, 7),
        0x7D => bit_reg8!(cpu, l, 7),
        0x7E => bit_hl!(cpu, 7),
        0x7F => bit_reg8!(cpu, a, 7),

        _ => panic!("Not a BIT instruction 0xCB{:02X}", opcode),
    }
}


macro_rules! res_reg8 {
    ($cpu:expr, $reg:ident, $bit:expr) => {{
        let mask = 0xFF ^ (1 << $bit);
        $cpu.reg.$reg &= mask;

        2
    }};
}

macro_rules! res_hl {
    ($cpu:expr, $bit:expr) => {{
        let mut val = $cpu.mmu.read_byte($cpu.reg.hl());

        let mask = 0xFF ^ (1 << $bit);
        val &= mask;

        $cpu.mmu.write_byte($cpu.reg.hl(), val);

        3
    }};
}

pub fn res<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x80 => res_reg8!(cpu, b, 0),
        0x81 => res_reg8!(cpu, c, 0),
        0x82 => res_reg8!(cpu, d, 0),
        0x83 => res_reg8!(cpu, e, 0),
        0x84 => res_reg8!(cpu, h, 0),
        0x85 => res_reg8!(cpu, l, 0),
        0x86 => res_hl!(cpu, 0),
        0x87 => res_reg8!(cpu, a, 0),

        0x88 => res_reg8!(cpu, b, 1),
        0x89 => res_reg8!(cpu, c, 1),
        0x8A => res_reg8!(cpu, d, 1),
        0x8B => res_reg8!(cpu, e, 1),
        0x8C => res_reg8!(cpu, h, 1),
        0x8D => res_reg8!(cpu, l, 1),
        0x8E => res_hl!(cpu, 1),
        0x8F => res_reg8!(cpu, a, 1),

        0x90 => res_reg8!(cpu, b, 2),
        0x91 => res_reg8!(cpu, c, 2),
        0x92 => res_reg8!(cpu, d, 2),
        0x93 => res_reg8!(cpu, e, 2),
        0x94 => res_reg8!(cpu, h, 2),
        0x95 => res_reg8!(cpu, l, 2),
        0x96 => res_hl!(cpu, 2),
        0x97 => res_reg8!(cpu, a, 2),

        0x98 => res_reg8!(cpu, b, 3),
        0x99 => res_reg8!(cpu, c, 3),
        0x9A => res_reg8!(cpu, d, 3),
        0x9B => res_reg8!(cpu, e, 3),
        0x9C => res_reg8!(cpu, h, 3),
        0x9D => res_reg8!(cpu, l, 3),
        0x9E => res_hl!(cpu, 3),
        0x9F => res_reg8!(cpu, a, 3),

        0xA0 => res_reg8!(cpu, b, 4),
        0xA1 => res_reg8!(cpu, c, 4),
        0xA2 => res_reg8!(cpu, d, 4),
        0xA3 => res_reg8!(cpu, e, 4),
        0xA4 => res_reg8!(cpu, h, 4),
        0xA5 => res_reg8!(cpu, l, 4),
        0xA6 => res_hl!(cpu, 4),
        0xA7 => res_reg8!(cpu, a, 4),

        0xA8 => res_reg8!(cpu, b, 5),
        0xA9 => res_reg8!(cpu, c, 5),
        0xAA => res_reg8!(cpu, d, 5),
        0xAB => res_reg8!(cpu, e, 5),
        0xAC => res_reg8!(cpu, h, 5),
        0xAD => res_reg8!(cpu, l, 5),
        0xAE => res_hl!(cpu, 5),
        0xAF => res_reg8!(cpu, a, 5),

        0xB0 => res_reg8!(cpu, b, 6),
        0xB1 => res_reg8!(cpu, c, 6),
        0xB2 => res_reg8!(cpu, d, 6),
        0xB3 => res_reg8!(cpu, e, 6),
        0xB4 => res_reg8!(cpu, h, 6),
        0xB5 => res_reg8!(cpu, l, 6),
        0xB6 => res_hl!(cpu, 6),
        0xB7 => res_reg8!(cpu, a, 6),

        0xB8 => res_reg8!(cpu, b, 7),
        0xB9 => res_reg8!(cpu, c, 7),
        0xBA => res_reg8!(cpu, d, 7),
        0xBB => res_reg8!(cpu, e, 7),
        0xBC => res_reg8!(cpu, h, 7),
        0xBD => res_reg8!(cpu, l, 7),
        0xBE => res_hl!(cpu, 7),
        0xBF => res_reg8!(cpu, a, 7),

        _ => panic!("Not a RES instruction 0xCB{:02X}", opcode),
    }
}

macro_rules! set_reg8 {
    ($cpu:expr, $reg:ident, $bit:expr) => {{
        $cpu.reg.$reg |= 1 << $bit;

        2
    }};
}

macro_rules! set_hl {
    ($cpu:expr, $bit:expr) => {{
        let mut val = $cpu.mmu.read_byte($cpu.reg.hl());

        val |= 1 << $bit;

        $cpu.mmu.write_byte($cpu.reg.hl(), val);

        3
    }};
}

pub fn set<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0xC0 => set_reg8!(cpu, b, 0),
        0xC1 => set_reg8!(cpu, c, 0),
        0xC2 => set_reg8!(cpu, d, 0),
        0xC3 => set_reg8!(cpu, e, 0),
        0xC4 => set_reg8!(cpu, h, 0),
        0xC5 => set_reg8!(cpu, l, 0),
        0xC6 => set_hl!(cpu, 0),
        0xC7 => set_reg8!(cpu, a, 0),

        0xC8 => set_reg8!(cpu, b, 1),
        0xC9 => set_reg8!(cpu, c, 1),
        0xCA => set_reg8!(cpu, d, 1),
        0xCB => set_reg8!(cpu, e, 1),
        0xCC => set_reg8!(cpu, h, 1),
        0xCD => set_reg8!(cpu, l, 1),
        0xCE => set_hl!(cpu, 1),
        0xCF => set_reg8!(cpu, a, 1),

        0xD0 => set_reg8!(cpu, b, 2),
        0xD1 => set_reg8!(cpu, c, 2),
        0xD2 => set_reg8!(cpu, d, 2),
        0xD3 => set_reg8!(cpu, e, 2),
        0xD4 => set_reg8!(cpu, h, 2),
        0xD5 => set_reg8!(cpu, l, 2),
        0xD6 => set_hl!(cpu, 2),
        0xD7 => set_reg8!(cpu, a, 2),

        0xD8 => set_reg8!(cpu, b, 3),
        0xD9 => set_reg8!(cpu, c, 3),
        0xDA => set_reg8!(cpu, d, 3),
        0xDB => set_reg8!(cpu, e, 3),
        0xDC => set_reg8!(cpu, h, 3),
        0xDD => set_reg8!(cpu, l, 3),
        0xDE => set_hl!(cpu, 3),
        0xDF => set_reg8!(cpu, a, 3),

        0xE0 => set_reg8!(cpu, b, 4),
        0xE1 => set_reg8!(cpu, c, 4),
        0xE2 => set_reg8!(cpu, d, 4),
        0xE3 => set_reg8!(cpu, e, 4),
        0xE4 => set_reg8!(cpu, h, 4),
        0xE5 => set_reg8!(cpu, l, 4),
        0xE6 => set_hl!(cpu, 4),
        0xE7 => set_reg8!(cpu, a, 4),

        0xE8 => set_reg8!(cpu, b, 5),
        0xE9 => set_reg8!(cpu, c, 5),
        0xEA => set_reg8!(cpu, d, 5),
        0xEB => set_reg8!(cpu, e, 5),
        0xEC => set_reg8!(cpu, h, 5),
        0xED => set_reg8!(cpu, l, 5),
        0xEE => set_hl!(cpu, 5),
        0xEF => set_reg8!(cpu, a, 5),

        0xF0 => set_reg8!(cpu, b, 6),
        0xF1 => set_reg8!(cpu, c, 6),
        0xF2 => set_reg8!(cpu, d, 6),
        0xF3 => set_reg8!(cpu, e, 6),
        0xF4 => set_reg8!(cpu, h, 6),
        0xF5 => set_reg8!(cpu, l, 6),
        0xF6 => set_hl!(cpu, 6),
        0xF7 => set_reg8!(cpu, a, 6),

        0xF8 => set_reg8!(cpu, b, 7),
        0xF9 => set_reg8!(cpu, c, 7),
        0xFA => set_reg8!(cpu, d, 7),
        0xFB => set_reg8!(cpu, e, 7),
        0xFC => set_reg8!(cpu, h, 7),
        0xFD => set_reg8!(cpu, l, 7),
        0xFE => set_hl!(cpu, 7),
        0xFF => set_reg8!(cpu, a, 7),

        _ => panic!("Not a SET instruction 0xCB{:02X}", opcode),
    }
}