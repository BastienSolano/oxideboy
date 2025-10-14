use core::num;
use std::{cell::RefCell, collections::HashMap};
use emu_core::memory::MemoryBus;
use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryCycleType {
    Read,
    Write,
}

// Tuple struct matching JSON format: [address, value, "read"/"write"] or null
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MemoryCycle {
    BusActivity(u16, u8, MemoryCycleType),
    Null
}

pub struct MockMemory {
    data: HashMap<u16, u8>, // for unit tests, only stores relevant (addr, value)
    cycles: RefCell<Vec<MemoryCycle>>, // for integration tests, stores all memory accesses
    had_bus_activity: RefCell<bool>, // tracks if current cycle had any bus activity
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
        *self.had_bus_activity.borrow_mut() = false;
    }
}

impl Default for MockMemory {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
            cycles: RefCell::new(Vec::new()),
            had_bus_activity: RefCell::new(false),
        }
    }
}

impl MemoryBus for MockMemory {
    fn read_byte(&self, addr: u16) -> u8 {
        let val = *self.data.get(&addr).expect(&format!("Address 0x{:04X} not found in MockMemory", addr));
        self.cycles.borrow_mut().push(MemoryCycle::BusActivity(addr, val, MemoryCycleType::Read));
        *self.had_bus_activity.borrow_mut() = true;
        val
    }

    fn read_word(&self, addr: u16) -> u16 {
        let first = *self.data.get(&addr).expect(&format!("Address 0x{:04X} not found in MockMemory", addr));
        let second = *self.data.get(&(addr+1)).expect(&format!("Address 0x{:04X} not found in MockMemory", addr+1));

        self.cycles.borrow_mut().push(MemoryCycle::BusActivity(addr, first, MemoryCycleType::Read));
        self.cycles.borrow_mut().push(MemoryCycle::BusActivity(addr+1, second, MemoryCycleType::Read));
        *self.had_bus_activity.borrow_mut() = true;

        (first as u16) << 8 | second as u16
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        self.cycles.borrow_mut().push(MemoryCycle::BusActivity(addr, val, MemoryCycleType::Write));
        *self.had_bus_activity.borrow_mut() = true;
        self.data.insert(addr, val);
    }

    fn write_word(&mut self, addr: u16, val: u16) {
        let high = (val >> 8) as u8;
        let low = (val & 0xFF) as u8;

        self.cycles.borrow_mut().push(MemoryCycle::BusActivity(addr, high, MemoryCycleType::Write));
        self.cycles.borrow_mut().push(MemoryCycle::BusActivity(addr+1, low, MemoryCycleType::Write));
        *self.had_bus_activity.borrow_mut() = true;

        self.data.insert(addr, high);
        self.data.insert(addr+1, low);
    }

    fn tick(&mut self, num_cycles: u8) {
        // TODO: what if the null cycles shouldn't be added at the end of the cycle?
        // e.g. if there is a read, then some internal operations, then a write

        // The number of null cycles = total_cycles - 1 (prefetch) - number_of_accesses_already_done
        let recorded_cycles = self.cycles.borrow().len();
        let expected_total = num_cycles as usize;

        // Add null cycles for any remaining internal operations
        // (before the final prefetch which will be added by read_byte)
        let nulls_needed = expected_total.saturating_sub(recorded_cycles + 1);
        for _ in 0..nulls_needed {
            self.cycles.borrow_mut().push(MemoryCycle::Null);
        }

        *self.had_bus_activity.borrow_mut() = false;
    }
}
