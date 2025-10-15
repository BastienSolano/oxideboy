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
            let cst = cpu.read_byte() as i8 as i16; // casting to a signed value using 2's complement

            if cst >= 0 {
                let sp_lower = (cpu.reg.sp & 0x00FF) as u8;
                let cst_lower = (cst as u16 & 0x00FF) as u8;
                cpu.reg.set_flag(CpuFlag::H, add8_needs_half_carry(sp_lower, cst_lower));
                cpu.reg.set_flag(CpuFlag::C, add8_needs_carry(sp_lower, cst_lower));
                cpu.reg.sp = cpu.reg.sp.wrapping_add(cst as u16);
            } else {
                let sp_lower = (cpu.reg.sp & 0x00FF) as u8;
                let cst_lower = ((-cst) as u16 & 0x00FF) as u8;
                cpu.reg.set_flag(CpuFlag::H, sub8_needs_half_carry(sp_lower, cst_lower));
                cpu.reg.set_flag(CpuFlag::C, sub8_needs_carry(sp_lower, cst_lower));
                cpu.reg.sp = cpu.reg.sp.wrapping_sub((-cst) as u16);
            }

            cpu.reg.set_flag(CpuFlag::Z, false);
            cpu.reg.set_flag(CpuFlag::N, false);
            return 4;
        },
        _ => panic!("Not yet implemented ADD instruction: 0x{:02X}", opcode),
    }
}