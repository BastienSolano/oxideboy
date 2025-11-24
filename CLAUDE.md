# OxideBoy - GameBoy DMG Emulator in Rust

## Project Overview

OxideBoy is an implementation of the original Game Boy (DMG - Dot Matrix Game) emulator written in Rust. The project aims to accurately emulate the Sharp LR35902 CPU and other Game Boy hardware components.

### Current Goal
The immediate focus is on achieving cycle-accurate CPU emulation by passing Blargg's CPU instruction test suite. The CPU instruction set is largely complete, and work is underway on interrupt handling and timing accuracy to pass test 11 of the cpu_instrs suite.

## Project Structure

```
oxideboy/
├── emu-core/           # Core emulation library
│   ├── src/
│   │   ├── cpu/        # CPU module
│   │   │   ├── cpu.rs       # CPU struct and main execution logic
│   │   │   ├── registers.rs # CPU registers and flag operations
│   │   │   ├── ld.rs        # LD instruction implementations
│   │   │   ├── alu.rs       # ALU operations (ADD, SUB, INC, DEC, etc.)
│   │   │   ├── jumps.rs     # Jump and call instructions
│   │   │   ├── stack.rs     # Stack operations (PUSH, POP)
│   │   │   └── mod.rs       # CPU module exports
│   │   ├── memory.rs   # Memory Management Unit (MMU)
│   │   └── lib.rs      # Library root
│   └── tests/          # Integration tests
│       ├── cpu_tests.rs
│       ├── blargg_tests.rs  # Blargg's test ROM integration
│       └── data/       # Test data (submodule)
```

## Core Components

### CPU ([emu-core/src/cpu/cpu.rs](emu-core/src/cpu/cpu.rs))

The CPU struct represents the Sharp LR35902 processor:
- `reg: Registers` - CPU registers (A, B, C, D, E, H, L, F, SP, PC)
- `halted: bool` - CPU halt state
- `ime: bool` - Interrupt Master Enable flag
- `setei: u32` - EI instruction delay counter
- `setdi: u32` - DI instruction delay counter
- `prefetched: u8` - Prefetched instruction byte for pipelining
- `mmu: M` - Memory Management Unit (generic over MemoryBus trait)

Key methods:
- `new(mmu)` - Create CPU with default (zero) state
- `boot_rom_initialized(mmu)` - Create CPU with post-boot ROM state
- `tick()` - Execute one instruction cycle
- `doctor_log_state()` - Format CPU state for Gameboy Doctor debugging

### Registers ([emu-core/src/cpu/registers.rs](emu-core/src/cpu/registers.rs))

8-bit registers: A, B, C, D, E, H, L, F (flags)
16-bit registers: SP (stack pointer), PC (program counter)

Register pairs can be accessed as 16-bit values:
- AF, BC, DE, HL (via methods like `bc()`, `set_bc()`, etc.)

CPU Flags (in F register):
- Z (Zero): bit 7
- N (Subtract): bit 6
- H (Half Carry): bit 5
- C (Carry): bit 4

Initial register values follow the DMG boot sequence (post-bootrom state).

### Memory ([emu-core/src/memory.rs](emu-core/src/memory.rs))

The MMU provides the `MemoryBus` trait for memory operations:
- `read_byte(addr: u16) -> u8`
- `write_byte(addr: u16, val: u8)`
- `read_word(addr: u16) -> u16` (little-endian)
- `write_word(addr: u16, val: u16)` (little-endian)
- `tick(num_cycles: u8)` - Advance emulation time
- `tick_internal()` - Record internal CPU operations (1 M-cycle)

The `Mmu` struct implements the full Game Boy memory map:
- ROM banks (0x0000-0x7FFF) with basic MBC1 support
- Working RAM (0xC000-0xDFFF, 8KB)
- Echo RAM (0xE000-0xFDFF)
- High RAM (0xFF80-0xFFFE, 127 bytes)
- I/O registers (partial implementation):
  - 0xFF01/0xFF02: Serial transfer (SB/SC)
  - 0xFF0F: Interrupt Flag (IF)
  - 0xFF44: LY register (dummy value)
  - 0xFFFF: Interrupt Enable (IE)
- Serial output capture for test ROM debugging

### Instructions

Instructions are organized by category in separate modules:

#### LD Instructions ([emu-core/src/cpu/ld.rs](emu-core/src/cpu/ld.rs))
Load and move operations between registers, memory, and immediate values.

#### ALU Operations ([emu-core/src/cpu/alu.rs](emu-core/src/cpu/alu.rs))
Arithmetic and logic unit operations (~1200 lines):
- ADD, ADC, SUB, SBC (8-bit and 16-bit variants)
- AND, OR, XOR, CP (compare)
- INC, DEC (8-bit and 16-bit)
- Bit rotation and shifts (RLC, RRC, RL, RR, SLA, SRA, SRL, SWAP)
- CB-prefixed instructions (bit operations: BIT, SET, RES)
- DAA (Decimal Adjust Accumulator)
- CPL, CCF, SCF

#### Jump Instructions ([emu-core/src/cpu/jumps.rs](emu-core/src/cpu/jumps.rs))
Control flow operations:
- JR (relative jumps, conditional and unconditional)
- JP (absolute jumps, including JP HL)
- CALL and RET (with conditional variants)
- RST (restart vectors)
- RETI (return from interrupt)

#### Stack Operations ([emu-core/src/cpu/stack.rs](emu-core/src/cpu/stack.rs))
- PUSH (BC, DE, HL, AF)
- POP (BC, DE, HL, AF)

All instruction functions return M-cycle count for cycle-accurate emulation.

## Game Boy DMG Specifications

### CPU
- **Processor**: Sharp LR35902 (similar to Z80)
- **Clock Speed**: 4.194304 MHz (~1.05 MHz effective)
- **Instruction Set**: Mix of 8080 and Z80 instructions (not fully compatible)

### Memory Map
```
0000-3FFF   16KB ROM Bank 00 (fixed)
4000-7FFF   16KB ROM Bank 01-NN (switchable via MBC)
8000-9FFF   8KB Video RAM (VRAM)
A000-BFFF   8KB External RAM (cartridge)
C000-CFFF   4KB Work RAM Bank 0 (WRAM)
D000-DFFF   4KB Work RAM Bank 1 (WRAM)
E000-FDFF   Echo RAM (mirror of C000-DDFF)
FE00-FE9F   Sprite Attribute Table (OAM)
FEA0-FEFF   Not Usable
FF00-FF7F   I/O Registers
FF80-FFFE   High RAM (HRAM)
FFFF        Interrupt Enable Register
```

### Display
- **Resolution**: 160x144 pixels
- **Refresh Rate**: ~59.73 Hz
- **Colors**: 4 shades of gray (2-bit color depth)
- **Sprites**: 40 sprites max, 10 per scanline
- **Background/Window**: 256x256 pixel tilemap

### Timing
- **Machine Cycle (M-cycle)**: 4 clock cycles
- **Instruction timing**: Measured in M-cycles
- **Frame**: 70224 clock cycles (17556 M-cycles)

## Development Status

### Implemented
- [x] CPU structure with generic MemoryBus trait
- [x] Register system with 8-bit and 16-bit access
- [x] CPU flags (Z, N, H, C)
- [x] Full Game Boy memory map
- [x] Basic MBC1 support (ROM banking)
- [x] All LD instruction variants
- [x] Complete ALU instruction set (ADD, SUB, AND, OR, XOR, CP, INC, DEC, etc.)
- [x] CB-prefixed instructions (bit operations, shifts, rotates)
- [x] Jump and call instructions (JP, JR, CALL, RET, RST, RETI)
- [x] Stack operations (PUSH, POP)
- [x] Interrupt architecture (IME, IE, IF registers)
- [x] Initial register states (default and post-boot)
- [x] Instruction prefetching/pipelining
- [x] Cycle-accurate timing with tick_internal()
- [x] Gameboy Doctor logging format
- [x] Serial output capture for test ROMs

### In Progress / TODO
- [ ] Complete interrupt handling implementation (service routine calls)
- [ ] HALT instruction and HALT bug
- [ ] STOP instruction
- [ ] Full MBC support (MBC2, MBC3, MBC5)
- [ ] External RAM (cartridge save RAM)
- [ ] PPU (Picture Processing Unit) - graphics rendering
- [ ] Timer implementation
- [ ] Joypad input
- [ ] Audio Processing Unit (APU)
- [ ] Full serial communication
- [ ] Save state support

## Testing

The project uses multiple test approaches:

### Unit Tests
- Test runner in [emu-core/tests/cpu_tests.rs](emu-core/tests/cpu_tests.rs)
- JSON test format for CPU instruction validation
- Test data located in [emu-core/tests/data](emu-core/tests/data) (git submodule)

### Integration Tests
- Blargg's test ROMs in [emu-core/tests/blargg_tests.rs](emu-core/tests/blargg_tests.rs)
- Full CPU instruction test suite (cpu_instrs.gb)
- Individual instruction tests with Gameboy Doctor logging
- Serial output capture for test results

Available Blargg test suites (in `emu-core/tests/data/blarggs/`):
- `cpu_instrs/` - CPU instruction tests (in progress)
- `halt_bug.gb` - HALT bug test (not yet passing)
- `instr_timing/` - Instruction timing tests
- `interrupt_time/` - Interrupt timing tests
- `mem_timing/` and `mem_timing-2/` - Memory timing tests
- Audio and OAM bug tests (not applicable yet)

Run tests with:
```bash
cd emu-core
cargo test
```

## Building

```bash
cd emu-core
cargo build
cargo build --release  # for optimized build
```

## Resources

### Essential Documentation
- [Pan Docs](https://gbdev.io/pandocs/) - Complete Game Boy technical reference
- [Game Boy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf) - Instruction set reference
- [Game Boy Opcode Table](https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)

### Test ROMs
- [Blargg's Test ROMs](https://github.com/retrio/gb-test-roms)
- [Mooneye GB Test Suite](https://github.com/Gekkio/mooneye-test-suite)
- [SM83 Test Data](https://github.com/SingleStepTests/sm83) - JSON-based instruction tests

### Community
- [GBDev](https://gbdev.io/) - Game Boy development community
- [r/EmuDev](https://reddit.com/r/EmuDev) - Emulator development subreddit

## Architecture Notes

### Cycle Accuracy
Each instruction function returns the number of M-cycles consumed. This enables cycle-accurate emulation, which is crucial for proper timing of graphics, audio, and hardware synchronization.

### Memory Access
All memory access goes through the MemoryBus trait, allowing for easy implementation of memory-mapped I/O, banking, and other hardware features without modifying instruction implementations.

### Instruction Organization
Instructions are organized by category in separate modules:
- `ld.rs` - Load and move operations
- `alu.rs` - Arithmetic and logic operations (includes CB-prefixed instructions)
- `jumps.rs` - Jump, call, and return instructions
- `stack.rs` - Stack operations (PUSH/POP)

Each module contains the instruction implementations with proper cycle counting and flag updates.

## Known Issues / Limitations

1. **Incomplete I/O register implementation**: Most I/O registers (0xFF00-0xFF7F) return 0xFF. Only serial transfer (0xFF01/0xFF02), interrupt flags (0xFF0F), LY (0xFF44), and interrupt enable (0xFFFF) are partially implemented.

2. **No VRAM/OAM**: Video RAM (0x8000-0x9FFF) and Object Attribute Memory (0xFE00-0xFE9F) are not implemented - reads return 0xFF.

3. **Incomplete MBC support**: Only basic MBC1 ROM banking is implemented. No external RAM support, and advanced MBC features are missing.

4. **Interrupt handling incomplete**: Interrupt architecture is present (IME, IE, IF registers) but the actual interrupt service routine mechanism is not fully implemented.

5. **No PPU/Timer/Audio**: Graphics, timing, and audio subsystems are not yet implemented.

## Development Workflow

### Testing Strategy
The emulator is developed using a test-driven approach:
1. Run Blargg's test ROMs to identify failing instructions
2. Use Gameboy Doctor logs to compare CPU state against reference emulator
3. Debug and fix instructions one at a time
4. Verify fixes with JSON-based unit tests

### Debugging Tools
- **Gameboy Doctor format**: CPU state logging in format: `A:XX F:XX B:XX C:XX D:XX E:XX H:XX L:XX SP:XXXX PC:XXXX PCMEM:XX,XX,XX,XX`
- **Serial output capture**: Test ROMs output results via serial port (0xFF01/0xFF02)
- **Cycle counting**: All instructions return accurate M-cycle counts for timing verification

## Contributing Guidelines

When implementing new features:
1. Follow existing code structure and naming conventions
2. Return cycle counts from instruction implementations
3. Add test cases for new instructions
4. Refer to Pan Docs for accurate timing and behavior
5. Use bit manipulation helpers for flag operations
6. Document any deviations from hardware behavior
7. Ensure cycle-accurate timing by using `mmu.tick_internal()` for internal operations

## Quick Reference

### Key Code Locations
- **CPU execution loop**: [emu-core/src/cpu/cpu.rs](emu-core/src/cpu/cpu.rs) - `tick()` method
- **Instruction dispatch**: [emu-core/src/cpu/cpu.rs](emu-core/src/cpu/cpu.rs) - opcode matching
- **Memory interface**: [emu-core/src/memory.rs](emu-core/src/memory.rs) - `MemoryBus` trait and `Mmu` struct
- **Register helpers**: [emu-core/src/cpu/registers.rs](emu-core/src/cpu/registers.rs) - flag getters/setters, 16-bit register access

### Dependencies
- `serde` and `serde_json` - For JSON test data parsing
- `paste` (dev) - Macro support for test generation

### Project Layout
This is a Cargo workspace with a single core library (`emu-core`). Future additions may include:
- Frontend/UI crate
- Debugger crate
- ROM analysis tools

## License

[To be determined]
