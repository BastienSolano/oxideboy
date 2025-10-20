use core::panic;

use crate::cpu::Cpu;
use crate::memory::MemoryBus;
use crate::registers::CpuFlag;

fn jr_conditional<M: MemoryBus>(cpu:&mut Cpu<M>, condition: bool) -> u8 {
    let steps = cpu.read_byte();
    if condition {
        cpu.reg.pc = cpu.reg.pc.wrapping_add(steps as i8 as u16);
        cpu.mmu.tick_internal();
        return 3;
    }
    2
}

pub fn jr<M: MemoryBus>(cpu:&mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0x28 => {
            // JR Z, r8
            let cond = cpu.reg.get_flag(CpuFlag::Z);
            jr_conditional(cpu, cond)
        },
        0x20 => {
            // JR NZ, r8
            let cond = !cpu.reg.get_flag(CpuFlag::Z);
            jr_conditional(cpu, cond)
        },
        0x38 => {
            // JR C, s8
            let cond = cpu.reg.get_flag(CpuFlag::C);
            jr_conditional(cpu, cond)
        },
        0x30 => {
            // JR NC, s8
            let cond = !cpu.reg.get_flag(CpuFlag::C);
            jr_conditional(cpu, cond)
        },
        0x18 => {
            // JR r8
            jr_conditional(cpu, true)
        },
        _ => panic!("Not a JR opcode: {:02X}", opcode),
    }
}

fn jp_conditional<M: MemoryBus>(cpu:&mut Cpu<M>, condition: bool) -> u8 {
    let addr = cpu.read_word();
    if condition {
        cpu.reg.pc = addr;
        cpu.mmu.tick_internal();
        return 4;
    }
    3
}

pub fn jp<M: MemoryBus>(cpu:&mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0xE9 => {
            cpu.reg.pc = cpu.reg.hl();
            1
        },
        0xC3 => {
            cpu.reg.pc = cpu.read_word();
            cpu.mmu.tick_internal();
            4
        },
        0xC2 => {
            // JP NZ, a16
            let cond = !cpu.reg.get_flag(CpuFlag::Z);
            jp_conditional(cpu, cond)
        },
        0xD2 => {
            // JP NC, a16
            let cond = !cpu.reg.get_flag(CpuFlag::C);
            jp_conditional(cpu, cond)
        },
        0xCA => {
            // JP Z, a16
            let cond = cpu.reg.get_flag(CpuFlag::Z);
            jp_conditional(cpu, cond)
        },
        0xDA => {
            // JP C, a16
            let cond = cpu.reg.get_flag(CpuFlag::C);
            jp_conditional(cpu, cond)
        }
        _ => panic!("Not a JP opcode: {:02X}", opcode),
    }
}