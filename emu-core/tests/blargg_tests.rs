use std::fs;

#[test]
fn blarggs_cpu_instr() {
    let rom = fs::read("tests/data/blarggs/cpu_instrs/cpu_instrs.gb").expect("Failed to read ROM");
    let mut mmu = emu_core::memory::Mmu::new(rom);
    let mut cpu = emu_core::cpu::cpu::Cpu::new(mmu);

    for _ in 0..100_000_000 {
        cpu.tick();
    }

    let output = cpu.mmu.get_serial_output();
    println!("Serial output:\n{}", output);
}