use crate::bios::DMG_BIOS;
use crate::cart::Cartridge;
use crate::cpu::Cpu;
use crate::mmu::Mmu;

#[allow(dead_code)]
pub struct Board {
    bios: bool,
    cpu: Cpu,
    clock: u64,
    iram: [u8; 0x2000], // Internal RAM
    hram: [u8; 127],    // High RAM (Zero Page)
    oram: [u8; 160],    // Object (sprite) Attribute RAM
    vram: [u8; 0x2000], // Video RAM
    cart: Cartridge,
}

#[allow(dead_code)]
impl Default for Board {
    fn default() -> Self {
        Board {
            bios: false,
            cpu: Cpu::default(),
            clock: 0,
            iram: [0; 0x2000],
            hram: [0; 127],
            oram: [0; 160],
            vram: [0; 0x2000],
            cart: Cartridge::default(),
        }
    }
}

#[allow(dead_code)]
impl Mmu for Board {
    fn read_byte(&self, addr: u16) -> u8 {
        if self.bios && addr <= 0x100 {
            return DMG_BIOS[addr as usize];
        }

        match addr {
            0x0000..=0x7FFF => self.cart.read_byte(addr),
            0x8000..=0x9FFF => self.vram[(addr - 0x8000u16) as usize],
            0xA000..=0xBFFF => self.cart.read_byte(addr),
            0xC000..=0xDfff => self.iram[(addr - 0xC000u16) as usize],
            0xE000..=0xFDFF => self.iram[(addr - 0xE000u16) as usize],
            0xFE00..=0xFE9F => self.oram[(addr - 0xFE00u16) as usize],
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80u16) as usize],
            _ => 0
        }
    }

    fn write_byte(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0x8000..=0x9FFF =>  {
                self.vram[(addr - 0x8000u16) as usize] = data;
                true
            }
            0xA000..=0xBFFF => {
                self.write_byte(addr, data)
            }
            0xC000..=0xDFFF => {
                self.iram[(addr - 0xC000u16) as usize] = data;
                true
            }
            0xE000..=0xFDFF => {
                self.iram[(addr - 0xE000u16) as usize] = data;
                true
            }
            0xFE00..=0xFE9F => {
                self.oram[(addr - 0xFE00u16) as usize] = data;
                true
            }
            0xFF80..=0xFFFE => {
                self.hram[(addr - 0xFF80u16) as usize] = data;
                true
            }
            _ => false
        }
    }

    fn read_word(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) | ((self.read_byte(addr + 1) as u16) << 8)
    }

    fn write_word(&mut self, addr: u16, data: u16) -> bool {
        self.write_byte(addr, (data >> 8) as u8) && self.write_byte(addr + 1, (data >> 8) as u8)
    }
}