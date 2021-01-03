use crate::bios::DMG_BIOS;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Memory {
    // Read a single byte from memory
    fn read_byte(&self, addr: u16) -> u8;

    // Write a single byte to memory
    fn write_byte(&mut self, addr: u16, data: u8) -> bool;

    // Read a word from memory
    fn read_word(&self, addr: u16) -> u16;

    // Write a word into memory
    fn write_word(&mut self, addr: u16, data: u16) -> bool;
}

pub struct FlatMemory {
    pub readable: bool,
    pub writable: bool,
    pub memory: Vec<u8>,
}

impl Memory for FlatMemory {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr: usize = addr.into();
        match self.readable && addr < self.memory.len() {
            true => self.memory[addr],
            false => 0
        }
    }

    fn write_byte(&mut self, addr: u16, data: u8) -> bool {
        let addr: usize = addr.into();
        match self.writable && addr < self.memory.len() {
            true => {
                self.memory[addr] = data;
                true
            }
            false => false
        }
    }

    fn read_word(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read_byte(addr), self.read_byte(addr + 1)])
    }

    fn write_word(&mut self, addr: u16, data: u16) -> bool {
        let le_bytes = data.to_le_bytes();
        self.write_byte(addr, le_bytes[0]) && self.write_byte(addr + 1, le_bytes[1])
    }
}

#[derive(Default)]
pub struct NullMemory {
}

impl Memory for NullMemory {
    fn read_byte(&self, _addr: u16) -> u8 { 0 }
    fn write_byte(&mut self, _addr: u16, _data: u8) -> bool { false }
    fn read_word(&self, _addr: u16) -> u16 { 0 }
    fn write_word(&mut self, _addr: u16, _data: u16) -> bool { false }
}

pub struct Mmu {
    /// BIOS Enabled
    /// $0000..=$0100
    bios_enable: bool,

    /// Cartridge ROM
    /// - $0000..=$7FFF
    crom: Rc<RefCell<FlatMemory>>,

    /// Video RAM
    /// - $8000..=$9FFF
    vram: Box<[u8; 0x2000]>,

    /// Cartridge RAM
    /// - $A000..=$BFFF
    cram: Rc<RefCell<FlatMemory>>,

    /// Internal RAM
    /// - $C000..=$DFFF
    /// - $E000..=$FDFF (Echo)
    iram: Box<[u8; 0x2000]>,

    /// Object (sprite) Attribute RAM
    /// - $FE00..=$FE9F
    oram: Box<[u8; 160]>,

    /// Hardware IO
    /// - $FF00-$FF7F, $FFFF
    hwio: Rc<RefCell<NullMemory>>,

    /// High RAM (Zero Page)
    /// - $FF80-$FFFE
    hram: Box<[u8; 127]>,
}

impl Mmu {
    pub fn new(
            crom: Rc<RefCell<FlatMemory>>,
            cram: Rc<RefCell<FlatMemory>>,
            hwio: Rc<RefCell<NullMemory>>) -> Self {
        Self {
            bios_enable: true,
            crom: crom,
            vram: Box::new([0; 0x2000]),
            cram: cram,
            iram: Box::new([0; 0x2000]),
            oram: Box::new([0; 160]),
            hwio: hwio,
            hram: Box::new([0; 127]),
        }
    }
}

impl Memory for Mmu {
    fn read_byte(&self, addr: u16) -> u8 {
        if self.bios_enable && addr <= 0x100 {
            return DMG_BIOS[addr as usize];
        }

        match addr {
            0x0000..=0x7FFF => self.crom.borrow().read_byte(addr),
            0x8000..=0x9FFF => self.vram[(addr - 0x8000u16) as usize],
            0xA000..=0xBFFF => self.cram.borrow().read_byte(addr - 0xA000u16),
            0xC000..=0xDFFF => self.iram[(addr - 0xC000u16) as usize],
            0xE000..=0xFDFF => self.iram[(addr - 0xE000u16) as usize],
            0xFE00..=0xFE9F => self.oram[(addr - 0xFE00u16) as usize],
            0xFF00..=0xFF7F => self.hwio.borrow().read_byte(addr - 0xFF00u16),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80u16) as usize],
            0xFFFF => self.hwio.borrow().read_byte(128),
            _ => 0
        }
    }

    fn write_byte(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0x8000..=0x9FFF =>  {
                self.vram[(addr - 0x8000) as usize] = data;
            }
            0xA000..=0xBFFF => {
                self.write_byte(addr - 0xA000, data);
            }
            0xC000..=0xDFFF => {
                self.iram[(addr - 0xC000) as usize] = data;
            }
            0xE000..=0xFDFF => {
                self.iram[(addr - 0xE000) as usize] = data;
            }
            0xFE00..=0xFE9F => {
                self.oram[(addr - 0xFE00) as usize] = data;
            }
            0xFF00..=0xFF7F => {
                // self.hwio.read_byte(addr - 0xFF00);
            }
            0xFF80..=0xFFFE => {
                self.hram[(addr - 0xFF80) as usize] = data;
            }
            0xFFFF => {
                self.hwio.borrow_mut().write_byte(128, data);
            }
            _ => return false
        }
        return true
    }

    fn read_word(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read_byte(addr), self.read_byte(addr + 1)])
    }

    fn write_word(&mut self, addr: u16, data: u16) -> bool {
        let le_bytes = data.to_le_bytes();
        self.write_byte(addr, le_bytes[0]) && self.write_byte(addr + 1, le_bytes[1])
    }
}