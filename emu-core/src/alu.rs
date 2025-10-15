use crate::memory::MemoryBus;
use crate::cpu::Cpu;
use crate::registers::*;

macro_rules! incr_8bit_reg {
    ($cpu:expr, $myreg:ident) => {{
        // first check if half-carry
        $cpu.reg.set_flag(crate::registers::CpuFlag::H, add8_needs_half_carry($cpu.reg.$myreg, 1));

        $cpu.reg.$myreg = $cpu.reg.$myreg.wrapping_add(1);

        $cpu.reg.set_flag(crate::registers::CpuFlag::Z, $cpu.reg.$myreg == 0);
        $cpu.reg.set_flag(crate::registers::CpuFlag::N, false);

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
        $cpu.reg.set_flag(crate::registers::CpuFlag::H, sub8_needs_half_carry($cpu.reg.$myreg, 1));

        $cpu.reg.$myreg = $cpu.reg.$myreg.wrapping_sub(1);

        $cpu.reg.set_flag(crate::registers::CpuFlag::Z, $cpu.reg.$myreg == 0);
        $cpu.reg.set_flag(crate::registers::CpuFlag::N, true);

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
            cpu.reg.set_flag(crate::registers::CpuFlag::H, add8_needs_half_carry(val, 1));
            let newval = val.wrapping_add(1);
            cpu.mmu.write_byte(addr, newval);
            cpu.reg.set_flag(crate::registers::CpuFlag::Z, newval == 0);
            cpu.reg.set_flag(crate::registers::CpuFlag::N, false);
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
            cpu.reg.set_flag(crate::registers::CpuFlag::H, sub8_needs_half_carry(val, 1));
            let newval = val.wrapping_sub(1);
            cpu.mmu.write_byte(addr, newval);
            cpu.reg.set_flag(crate::registers::CpuFlag::Z, newval == 0);
            cpu.reg.set_flag(crate::registers::CpuFlag::N, true);
            return 3;
        },
        _ => panic!("Not a register decr instruction: 0x{:02X}", opcode),
    }
}