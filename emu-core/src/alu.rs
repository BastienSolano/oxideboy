use crate::memory::MemoryBus;
use crate::cpu::Cpu;
use crate::registers::*;

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