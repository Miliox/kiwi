use crate::bios::DMG_BIOS;
use crate::cpu::Cpu;
use crate::joypad::JoypadKeys;
use crate::joypad::JoypadRegs;
use crate::timer::Timer;
use crate::types::MutRc;

pub trait Memory {
    // Read a single byte from memory
    fn read_byte(&self, addr: u16) -> u8;

    // Write a single byte to memory
    fn write_byte(&mut self, addr: u16, data: u8);
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

    fn write_byte(&mut self, addr: u16, data: u8) {
        let addr: usize = addr.into();
        if self.writable && addr < self.memory.len() {
            self.memory[addr] = data;
        }
    }
}

#[derive(Default)]
pub struct NullMemory {
}

impl Memory for NullMemory {
    fn read_byte(&self, _addr: u16) -> u8 { 0 }
    fn write_byte(&mut self, _addr: u16, _data: u8) { }
}

pub struct Mmu {
    /// BIOS Enabled
    /// $0000..=$0100
    bios_enable: bool,

    /// Cartridge ROM
    /// - $0000..=$7FFF
    crom: MutRc<FlatMemory>,

    /// Video RAM
    /// - $8000..=$9FFF
    vram: Box<[u8; 0x2000]>,

    /// Cartridge RAM
    /// - $A000..=$BFFF
    cram: MutRc<FlatMemory>,

    /// Internal RAM
    /// - $C000..=$DFFF
    /// - $E000..=$FDFF (Echo)
    iram: Box<[u8; 0x2000]>,

    /// Object (sprite) Attribute RAM
    /// - $FE00..=$FE9F
    oram: Box<[u8; 160]>,

    /// Hardware IO
    /// - $FF00-$FF7F, $FFFF
    cpu: MutRc<Cpu>,
    timer: MutRc<Timer>,
    joypad_regs: JoypadRegs,
    joypad_keys: JoypadKeys,

    /// High RAM (Zero Page)
    /// - $FF80-$FFFE
    hram: Box<[u8; 127]>,
}

impl Mmu {
    pub fn new(
            crom: MutRc<FlatMemory>,
            cram: MutRc<FlatMemory>,
            cpu:  MutRc<Cpu>,
            timer: MutRc<Timer>) -> Self {
        Self {
            bios_enable: true,
            crom: crom,
            vram: Box::new([0; 0x2000]),
            cram: cram,
            iram: Box::new([0; 0x2000]),
            oram: Box::new([0; 160]),
            cpu: cpu,
            timer: timer,
            joypad_regs: JoypadRegs::default(),
            joypad_keys: JoypadKeys::default(),
            hram: Box::new([0; 127]),
        }
    }

    pub fn press_joypad_key(&mut self, keys: JoypadKeys) {
        self.joypad_keys.insert(keys);
        self.joypad_regs.merge_keys(self.joypad_keys);
    }

    pub fn release_joypad_key(&mut self, keys: JoypadKeys) {
        self.joypad_keys.remove(keys);
        self.joypad_regs.merge_keys(self.joypad_keys);
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
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80u16) as usize],

            // CPU
            0xFF0F => self.cpu.borrow().triggered_interrupts(),
            0xFFFF => self.cpu.borrow().enabled_interrupts(),

            // Joypad
            0xFF00 => self.joypad_regs.bits(),

            // Timer
            0xFF04 => self.timer.borrow().divider(),
            0xFF05 => self.timer.borrow().counter(),
            0xFF06 => self.timer.borrow().modulo(),
            0xFF07 => self.timer.borrow().control(),

            _ => 0
        }
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = data,
            0xA000..=0xBFFF => self.write_byte(addr - 0xA000, data),
            0xC000..=0xDFFF => self.iram[(addr - 0xC000) as usize] = data,
            0xE000..=0xFDFF => self.iram[(addr - 0xE000) as usize] = data,
            0xFE00..=0xFE9F => self.oram[(addr - 0xFE00) as usize] = data,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = data,

            // CPU
            0xFF0F => self.cpu.borrow_mut().set_triggered_interrupts(data),
            0xFFFF => self.cpu.borrow_mut().set_enabled_interrupts(data),

            // JOYPAD
            0xFF00 => {
                self.joypad_regs = JoypadRegs::from_bits(data & 0xf0).unwrap();
                self.joypad_regs.merge_keys(self.joypad_keys);
            }

            // TIMER
            0xFF04 => self.timer.borrow_mut().reset_divider(),
            0xFF05 => self.timer.borrow_mut().set_counter(data),
            0xFF06 => self.timer.borrow_mut().set_modulo(data),
            0xFF07 => self.timer.borrow_mut().set_control(data),

            // DMA
            0xFF46 => {
                if data <= 0xF1 {
                    let src = u16::from_be_bytes([data, 0x00]);
                    for i in 0..160 {
                        self.oram[i] = self.read_byte(src + i as u16)
                    }
                }
            }

            _ => { }
        }
    }
}