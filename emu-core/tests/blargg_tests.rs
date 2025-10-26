use std::fs;
use std::io::Write;

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

#[test]
fn blarggs_cpu_instrs_2() {
    run_with_doctor_log("tests/data/blarggs/cpu_instrs/individual/02-interrupts.gb", "tests/data/blarggs/cpu_instrs/individual/02-interrupts.log");
}

fn run_with_doctor_log(rom_path: &str, log_path: &str) {
    let rom = fs::read(rom_path).expect("Failed to read ROM");
    let mut mmu = emu_core::memory::Mmu::new(rom);
    let mut cpu = emu_core::cpu::cpu::Cpu::boot_rom_initialized(mmu);

    // open the log file to write to it
    let mut log_file = fs::File::create(log_path).expect("Failed to create log file");

    for _ in 0..100_000_000 {
        cpu.tick();
        // Write CPU state to log file in Gameboy Doctor format
        let state = cpu.doctor_log_state();
        writeln!(log_file, "{}", state).expect("Failed to write to log file");
    }

    let output = cpu.mmu.get_serial_output();
}