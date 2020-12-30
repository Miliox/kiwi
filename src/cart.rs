use crate::mmu::Mmu;

#[allow(dead_code)]
#[derive(Default)]
pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
}

#[allow(dead_code)]
impl Cartridge {
    pub fn open(&mut self, filename: &str) {
        self.rom = std::fs::read(filename).unwrap();
    }

    pub fn title(&self) -> &str {
        std::str::from_utf8(&self.rom[0x134..=0x142]).unwrap()
    }

    pub fn is_color(&self) -> bool {
        self.rom[0x0143] == 0x80
    }

    pub fn is_super(&self) -> bool {
        self.rom[0x0146] == 0x03
    }

    pub fn is_japanese(&self) -> bool {
        self.rom[0x014a] == 0x00
    }

    pub fn entry_point(&self) -> &[u8] {
        &self.rom[0x100..=0x103]
    }

    pub fn lincense_code(&self) -> u16 {
        let old_license = self.rom[0x144];
        if old_license != 0x33 {
            old_license as u16
        } else {
            ((self.rom[0x144] as u16) << 8) | (self.rom[0x145] as u16)
        }
    }

    pub fn cart_type(&self) -> u8 {
        self.rom[0x147]
    }

    pub fn rom_size_code(&self) -> u8 {
        self.rom[0x148]
    }

    pub fn ram_size_code(&self) -> u8 {
        self.rom[0x149]
    }

    pub fn rom_slice(&self, addr: u16, size: u16) -> &[u8] {
        &self.rom[addr as usize..((addr + size) as usize)]
    }

    pub fn complement_check(&self) -> u8 {
        self.rom[0x14d]
    }

    pub fn checksum(&self) -> u16 {
        ((self.rom[0x14e] as u16) << 8) | (self.rom[0x14f] as u16)
    }
}

impl Mmu for Cartridge {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0..=0x8000 => self.rom[addr as usize],
            _ => 0
        }
    }

    fn write_byte(&mut self, _addr: u16, _data: u8) -> bool {
        false
    }

    fn read_word(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) | ((self.read_byte(addr + 1) as u16) << 8)
    }

    fn write_word(&mut self, _addr: u16, _data: u16) -> bool {
        false
    }
}