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
    cycles: Vec<MemoryCycle>, // we can parse cycles later if needed
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
        // Clear any cycles recorded during setup
        cpu.mmu.clear_cycles();

        cpu
    }
}

#[test]
fn run_all_tests() {
    let paths = std::fs::read_dir(TESTS_PATH).expect("Failed to read tests directory");
    for entry in paths {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            run_test_file(path.to_str().unwrap());
        }
    }
}

fn run_test_file(file_path: &str) {
    // Read the test file
    let file_content = std::fs::read_to_string(file_path)
        .expect(&format!("Failed to read test file: {}", file_path));

    // Parse the JSON content into a vector of CpuTest
    let tests: Vec<CpuTest> = serde_json::from_str(&file_content)
        .expect(&format!("Failed to parse JSON in file: {}", file_path));

    // Run each test
    for test in &tests {
        run_single_test(test);
    }
}

fn run_single_test(test: &CpuTest) {
    // Initialize MockMemory and CPU
    let mmu = MockMemory::default();
    let mut cpu = test.initial.into_cpu(mmu);

    // Execute the CPU for the number of cycles specified
    for _ in &test.cycles {
        cpu.tick();
    }

    // Compare final CPU state
    let final_state = CpuState::from_cpu(&cpu);
    assert_eq!(final_state, test.final_state, "Test '{}' failed", test.name);

    // Compare memory cycles
    let recorded_cycles = cpu.mmu.get_cycles();
    assert_eq!(recorded_cycles, test.cycles, "Memory cycles do not match for test '{}'", test.name);
}