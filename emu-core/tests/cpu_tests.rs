mod common;

use serde::{Serialize, Deserialize};
use emu_core::cpu::Cpu;
use emu_core::memory::MemoryBus;

use crate::common::MockMemory;
use crate::common::MemoryCycle;

static TESTS_PATH: &str = "tests/data/v2";

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct CpuState {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
    ram: Vec<(u16, u8)>,
}

#[derive(Debug, Deserialize)]
struct CpuTest {
    name: String,
    initial: CpuState,
    #[serde(rename = "final")]
    final_state: CpuState,
    cycles: Vec<MemoryCycle>,
}

// Conversion functions between Cpu and CpuState
impl CpuState {
    /// Create a CpuState from a Cpu instance
    fn from_cpu(cpu: &Cpu<MockMemory>) -> Self {
        Self {
            a: cpu.reg.a,
            b: cpu.reg.b,
            c: cpu.reg.c,
            d: cpu.reg.d,
            e: cpu.reg.e,
            f: cpu.reg.f,
            h: cpu.reg.h,
            l: cpu.reg.l,
            pc: cpu.reg.pc,
            sp: cpu.reg.sp,
            ram: cpu.mmu.dump_mem(),
        }
    }

    /// Create a Cpu from a CpuState
    fn into_cpu(&self, mmu: MockMemory) -> Cpu<MockMemory> {
        let mut cpu = Cpu::new(mmu);
        cpu.reg.a = self.a;
        cpu.reg.b = self.b;
        cpu.reg.c = self.c;
        cpu.reg.d = self.d;
        cpu.reg.e = self.e;
        cpu.reg.f = self.f & 0xF0; // lower nibble of F is always 0
        cpu.reg.h = self.h;
        cpu.reg.l = self.l;
        cpu.reg.pc = self.pc;
        cpu.reg.sp = self.sp;

        // Write RAM values
        for (addr, val) in &self.ram {
            cpu.mmu.write_byte(*addr, *val);
        }

        // Loading the prefetched opcode
        cpu.prefetched = cpu.mmu.read_byte(cpu.reg.pc-1);

        // Clear any cycles recorded during setup
        cpu.mmu.clear_cycles();

        cpu
    }
}

// List of implemented opcodes (as hex strings matching filenames)
const IMPLEMENTED_OPCODES: &[&str] = &[
    "00", // NOP
    "01", // LD BC, nn
    "02", // LD (BC), A
    "06", // LD B, n
    "0a", // LD A, (BC)
    "0e", // LD C, n
    "12", // LD (DE), A
    "16", // LD D, n
    "1a", // LD A, (DE)
    "1e", // LD E, n
    "22", // LD (HL+), A
    "26", // LD H, n
    "2a", // LD A, (HL+)
    "2e", // LD L, n
    "32", // LD (HL-), A
    "36", // LD (HL), n
    "3a", // LD A, (HL-)
    "3e", // LD A, n
    "40", "41", "42", "43", "44", "45", "46", "47", // LD B, r / LD B, (HL)
    "48", "49", "4a", "4b", "4c", "4d", "4e", "4f", // LD C, r / LD C, (HL)
    "50", "51", "52", "53", "54", "55", "56", "57", // LD D, r / LD D, (HL)
    "58", "59", "5a", "5b", "5c", "5d", "5e", "5f", // LD E, r / LD E, (HL)
    "60", "61", "62", "63", "64", "65", "66", "67", // LD H, r / LD H, (HL)
    "68", "69", "6a", "6b", "6c", "6d", "6e", "6f", // LD L, r / LD L, (HL)
    "70", "71", "72", "73", "74", "75", "77",       // LD (HL), r (skip 0x76 HALT)
    "78", "79", "7a", "7b", "7c", "7d", "7e", "7f", // LD A, r / LD A, (HL)
    "ea", // LD (nn), A
    "fa", // LD A, (nn)
];

#[test]
fn test_implemented_opcodes() {
    let mut total_passed = 0;
    let mut total_tests = 0;

    for opcode in IMPLEMENTED_OPCODES {
        let file_path = format!("{}/{}.json", TESTS_PATH, opcode);

        if !std::path::Path::new(&file_path).exists() {
            println!("⚠️  Skipping {}: file not found", opcode);
            continue;
        }

        let (passed, total) = run_test_file(&file_path, opcode);
        total_passed += passed;
        total_tests += total;
    }

    println!("\n========================================");
    println!("Total: {} / {} tests passed", total_passed, total_tests);
    println!("========================================");

    if total_passed < total_tests {
        panic!("{} tests failed", total_tests - total_passed);
    }
}

fn run_test_file(file_path: &str, opcode: &str) -> (usize, usize) {
    // Read the test file
    let file_content = std::fs::read_to_string(file_path)
        .expect(&format!("Failed to read test file: {}", file_path));

    // Parse the JSON content into a vector of CpuTest
    let tests: Vec<CpuTest> = serde_json::from_str(&file_content)
        .expect(&format!("Failed to parse JSON in file: {}", file_path));

    println!("\n[Opcode 0x{}] Running {} tests...", opcode.to_uppercase(), tests.len());

    let mut passed = 0;
    let mut failed_tests = Vec::new();

    // Run each test
    for test in &tests {
        match run_single_test(test) {
            Ok(_) => passed += 1,
            Err(e) => {
                failed_tests.push((test.name.clone(), e));
            }
        }
    }

    let total = tests.len();

    if failed_tests.is_empty() {
        println!("  ✅ All {} tests passed!", total);
    } else {
        println!("  ⚠️  {} / {} tests passed", passed, total);
        for (name, error) in &failed_tests {
            println!("    ❌ {}: {}", name, error);
        }
    }

    (passed, total)
}

fn run_single_test(test: &CpuTest) -> Result<(), String> {
    // Initialize MockMemory and CPU
    let mmu = MockMemory::default();
    let mut cpu = test.initial.into_cpu(mmu);

    // Run a single CPU tick
    cpu.tick();

    // Compare final CPU state
    let final_state = CpuState::from_cpu(&cpu);
    if final_state != test.final_state {
        return Err(format!(
            "CPU state mismatch:\n      Expected: {:?}\n      Got:      {:?}",
            test.final_state, final_state
        ));
    }

    // Compare memory cycles
    let recorded_cycles = cpu.mmu.get_cycles();
    if recorded_cycles != test.cycles {
        return Err(format!(
            "Memory cycles mismatch:\n      Expected: {:?}\n      Got:      {:?}",
            test.cycles, recorded_cycles
        ));
    }

    Ok(())
}