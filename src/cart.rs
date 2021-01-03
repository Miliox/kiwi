use crate::mmu::FlatMemory;
use std::cell::RefCell;
use std::rc::Rc;

#[allow(dead_code)]
pub struct Cartridge {
    pub rom: Rc<RefCell<FlatMemory>>,
    pub ram: Rc<RefCell<FlatMemory>>,
}

#[allow(dead_code)]
impl Cartridge {
    pub fn new() -> Self {
        Self {
            rom: Rc::new(RefCell::new(FlatMemory { readable: true, writable: false, memory: Vec::new() })),
            ram: Rc::new(RefCell::new(FlatMemory { readable: true, writable: true,  memory: Vec::new() })),
        }
    }

    pub fn open(&mut self, filename: &str) {
        self.rom.borrow_mut().memory = std::fs::read(filename).unwrap();
    }

    pub fn title(&self) -> String {
        std::str::from_utf8(&self.rom.borrow().memory[0x134..=0x142]).unwrap().to_string()
    }

    pub fn is_color(&self) -> bool {
        self.rom.borrow().memory[0x0143] == 0x80
    }

    pub fn is_super(&self) -> bool {
        self.rom.borrow().memory[0x0146] == 0x03
    }

    pub fn is_japanese(&self) -> bool {
        self.rom.borrow().memory[0x014a] == 0x00
    }

    pub fn entry_point(&self) -> Vec<u8> {
        self.rom.borrow().memory[0x100..=0x103].to_vec()
    }

    pub fn lincense_code(&self) -> u16 {
        let old_license = self.rom.borrow_mut().memory[0x144];
        if old_license != 0x33 {
            old_license as u16
        } else {
            u16::from_be_bytes([
                self.rom.borrow_mut().memory[0x144],
                self.rom.borrow_mut().memory[0x145]])
        }
    }

    pub fn cart_type(&self) -> u8 {
        self.rom.borrow_mut().memory[0x147]
    }

    pub fn rom_size_code(&self) -> u8 {
        self.rom.borrow_mut().memory[0x148]
    }

    pub fn ram_size_code(&self) -> u8 {
        self.rom.borrow_mut().memory[0x149]
    }

    pub fn rom_slice(&self, addr: u16, size: u16) -> Vec<u8> {
        self.rom.borrow().memory[addr as usize..((addr + size) as usize)].to_vec()
    }

    pub fn complement_check(&self) -> u8 {
        self.rom.borrow_mut().memory[0x14d]
    }

    pub fn checksum(&self) -> u16 {
        let rom = self.rom.borrow();
        u16::from_be_bytes([
            rom.memory[0x14e],
            rom.memory[0x14f]])
    }
}

impl Default for Cartridge {
    fn default() -> Self { Self::new() }
}