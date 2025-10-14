use std::{cell::RefCell, collections::HashMap};
use emu_core::memory::MemoryBus;
use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MemoryCycleType {
    Read,
    Write,
}

// Tuple struct matching JSON format: [address, value, "read"/"write"]
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryCycle(u16, u8, MemoryCycleType);

pub struct MockMemory {
    data: HashMap<u16, u8>, // for unit tests, only stores relevant (addr, value)
    cycles: RefCell<Vec<MemoryCycle>>, // for integration tests, stores all memory accesses
}

impl MockMemory {
    pub fn dump_mem(&self) -> Vec<(u16, u8)> {
        let mut result: Vec<(u16, u8)> = self.data.iter()
            .map(|(&addr, &value)| (addr, value))
            .collect();
        result.sort_by_key(|(addr, _)| *addr);
        result
    }

    pub fn get_cycles(&self) -> Vec<MemoryCycle> {
         self.cycles.borrow().clone()
    }

    pub fn clear_cycles(&self) {
        self.cycles.borrow_mut().clear();
    }
}

impl Default for MockMemory {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
            cycles: RefCell::new(Vec::new()),
        }
    }
}

impl MemoryBus for MockMemory {
    fn read_byte(&self, addr: u16) -> u8 {
        let val = *self.data.get(&addr).expect(&format!("Address 0x{:04X} not found in MockMemory", addr));
        self.cycles.borrow_mut().push(MemoryCycle(addr, val, MemoryCycleType::Read));
        val
    }

    fn read_word(&self, addr: u16) -> u16 {
        let first = *self.data.get(&addr).expect(&format!("Address 0x{:04X} not found in MockMemory", addr));
        let second = *self.data.get(&(addr+1)).expect(&format!("Address 0x{:04X} not found in MockMemory", addr+1));

        self.cycles.borrow_mut().push(MemoryCycle(addr, first, MemoryCycleType::Read));
        self.cycles.borrow_mut().push(MemoryCycle(addr+1, second, MemoryCycleType::Read));

        (first as u16) << 8 | second as u16
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        self.cycles.borrow_mut().push(MemoryCycle(addr, val, MemoryCycleType::Write));
        self.data.insert(addr, val);
    }

    fn write_word(&mut self, addr: u16, val: u16) {
        let high = (val >> 8) as u8;
        let low = (val & 0xFF) as u8;

        self.cycles.borrow_mut().push(MemoryCycle(addr, high, MemoryCycleType::Write));
        self.cycles.borrow_mut().push(MemoryCycle(addr+1, low, MemoryCycleType::Write));

        self.data.insert(addr, high);
        self.data.insert(addr+1, low);
    }

    fn tick(&mut self) {
        // No-op for now
    }
}
