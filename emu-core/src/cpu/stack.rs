use crate::{cpu::cpu::Cpu, memory::MemoryBus};

macro_rules! pop_reg16 {
    ($cpu:expr, $hreg:ident, $lreg:ident) => {{
        // Actually emulating the two-steps procedure in case it is important
        // for timing purposes

        let lower = $cpu.mmu.read_byte($cpu.reg.sp);
        $cpu.reg.$lreg = lower;
        $cpu.reg.sp = $cpu.reg.sp.wrapping_add(1);

        let upper = $cpu.mmu.read_byte($cpu.reg.sp);
        $cpu.reg.$hreg = upper;
        $cpu.reg.sp = $cpu.reg.sp.wrapping_add(1);
    }};
}

pub fn pop<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0xC1 => pop_reg16!(cpu, b, c),
        0xD1 => pop_reg16!(cpu, d, e),
        0xE1 => pop_reg16!(cpu, h, l),
        0xF1 => {
            // When popping into AF, the lower nibble of F is always 0
            pop_reg16!(cpu, a, f);
            cpu.reg.f &= 0xF0;
        },
        _ => panic!("Not a POP instruction: 0x{:02X}", opcode),
    }
    3
}

macro_rules! push_reg16 {
    ($cpu:expr, $hreg:ident, $lreg:ident) => {{
        // Initial sp decrement
        $cpu.reg.sp = $cpu.reg.sp.wrapping_sub(1);
        $cpu.mmu.tick_internal();

        // this second decremeent does not take a cycle because it is done "in parallel"
        // via the IDU (Increment/Decrement Unit)
        // the whole thing can be written as LD [SP-], upper 8 bits of reg pair
        $cpu.mmu.write_byte($cpu.reg.sp, $cpu.reg.$hreg);
        $cpu.reg.sp = $cpu.reg.sp.wrapping_sub(1);

        $cpu.mmu.write_byte($cpu.reg.sp, $cpu.reg.$lreg);
    }};
}



pub fn push<M: MemoryBus>(cpu: &mut Cpu<M>, opcode: u8) -> u8 {
    match opcode {
        0xC5 => push_reg16!(cpu, b, c),
        0xD5 => push_reg16!(cpu, d, e),
        0xE5 => push_reg16!(cpu, h, l),
        0xF5 => push_reg16!(cpu, a, f),
        _ => panic!("Not a PUSH instruction: 0x{:02X}", opcode),
    }
    4
}