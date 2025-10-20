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

// Macro to generate individual test functions for each opcode
macro_rules! opcode_tests {
    ($($opcode:literal),* $(,)?) => {
        $(
            paste::paste! {
                #[test]
                fn [<test_opcode_ $opcode>]() {
                    let file_path = format!("{}/{}.json", TESTS_PATH, $opcode);

                    // Read the test file
                    let file_content = std::fs::read_to_string(&file_path)
                        .expect(&format!("Failed to read test file: {}", file_path));

                    // Parse the JSON content into a vector of CpuTest
                    let tests: Vec<CpuTest> = serde_json::from_str(&file_content)
                        .expect(&format!("Failed to parse JSON in file: {}", file_path));

                    // Run each test
                    for test in &tests {
                        run_single_test(test);
                    }
                }
            }
        )*
    };
}

// LD instructions
opcode_tests! {
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
    "e0", // LDH (a8), A
    "e2", // LD (C), A
    "ea", // LD (nn), A
    "f0", // LDH A, (a8)
    "f2", // LD A, (C)
    "fa", // LD A, (nn)
    "08", // LD (nn), SP
    "f8", // LD HL, SP+e8
    "f9", // LD SP, HL
}

// INC/DEC instructions
opcode_tests! {
    "04", "14", "24", "0c", "1c", "2c", "3c", // INC {B,D,H,C,E,L,A}
    "03", "13", "23", "33", // INC {BC,DE,HL,SP}
    "34", // INC (HL)
    "05", "15", "25", "0d", "1d", "2d", "3d", // DEC {B,D,H,C,E,L,A}
    "0b", "1b", "2b", "3b", // DEC {BC,DE,HL,SP}
    "35", // DEC (HL)   
}

// ADD/SUB instructions
opcode_tests! {
    "80", "81", "82", "83", "84", "85", "86", "87", // ADD A, r / ADD A, (HL)
    "c6", // ADD A, d8
    "e8", // ADD SP, s8
    "09", "19", "29", "39", // ADD HL, rr
    "90", "91", "92", "93", "94", "95", "96", "97", // SUB A, r / SUB A, (HL)
    "d6", // SUB A, d8
}

// ADC/SBC instructions
opcode_tests! {
    "88", "89", "8a", "8b", "8c", "8d", "8e", "8f", // ADC A, r / ADC A, (HL)
    "ce", // ADC A, d8
    "98", "99", "9a", "9b", "9c", "9d", "9e", "9f", // SBC A, r / SBC A, (HL)
    "de", // SBC A, d8
}

// AND/OR/XOR instructions
opcode_tests! {
    "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", // AND A, r / AND A, (HL)
    "e6", // AND A, d8
    "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", // OR A, r / OR A, (HL)
    "f6", // OR A, d8
    "a8", "a9", "aa", "ab", "ac", "ad", "ae", "af", // XOR A, r / XOR A, (HL)
    "ee", // XOR A, d8
}

// CP instructions
opcode_tests! {
    "b8", "b9", "ba", "bb", "bc", "bd", "be", "bf", // CP A, r / CP A, (HL)
    "fe", // CP A, d8
}

// Push/Pop instructions
// opcode_tests! {
//     "c1", "d1", "e1", "f1", // POP BC, DE, HL, AF
//     "c5", "d5", "e5", "f5", // PUSH BC, DE, HL, AF
// }

// Register value manipulation instructions
opcode_tests! {
    "27", // DAA
    "2f", // CPL
    "3f", // CCF
    "37", // SCF
}

// Jumps
opcode_tests! {
    // JR instructions
    "18", // JR r8
    "20", // JR NZ, r8
    "28", // JR Z, r8
    "30", // JR NC, r8
    "38", // JR C, r8
    // JP instructions
    "c2", // JP NZ, a16
    "c3", // JP a16
    "ca", // JP Z, a16
    "d2", // JP NC, a16
    "da", // JP C, a16
    "e9", // JP HL
}

// RL/RR instructions
opcode_tests! {
    "07", // RLCA
    "0f", // RRCA
    "17", // RLA
    "1f", // RRA
}

fn run_single_test(test: &CpuTest) {
    // Initialize MockMemory and CPU
    let mmu = MockMemory::default();
    let mut cpu = test.initial.into_cpu(mmu);

    // Run a single CPU tick
    cpu.tick();

    // Compare final CPU state
    let final_state = CpuState::from_cpu(&cpu);
    assert_eq!(final_state, test.final_state, "Cpu cycles do not match for test '{}'", test.name);

    // Compare memory cycles
    let recorded_cycles = cpu.mmu.get_cycles();
    assert_eq!(recorded_cycles, test.cycles, "Memory cycles do not match for test '{}'", test.name);
}