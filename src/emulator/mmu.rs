use crate::types::*;
use crate::emulator::*;

pub trait Memory {
    // Read a single byte from memory
    fn read(&self, addr: u16) -> u8;

    // Write a single byte to memory
    fn write(&mut self, addr: u16, data: u8);
}

pub struct Mmu {
    /// BIOS Enabled
    /// $0000..=$0100
    bios_enable: bool,

    /// Internal RAM
    /// - $C000..=$DFFF
    /// - $E000..=$FDFF (Echo)
    iram: Box<[u8; 0x2000]>,

    /// High RAM (Zero Page)
    /// - $FF80-$FFFE
    hram: Box<[u8; 127]>,

    pub cartridge: MutRc<Cartridge>,

    pub cpu: MutRc<Cpu>,

    pub gpu: MutRc<Gpu>,

    pub joypad: MutRc<Joypad>,

    pub serial: MutRc<Serial>,

    pub timer: MutRc<Timer>,
}

impl Mmu {
    pub fn new(cartridge: MutRc<Cartridge>, cpu: MutRc<Cpu>, gpu: MutRc<Gpu>, joypad: MutRc<Joypad>, serial: MutRc<Serial>, timer: MutRc<Timer>) -> Self {
        Self {
            bios_enable: true,
            iram: Box::new([0; 0x2000]),
            hram: Box::new([0; 127]),

            cartridge: cartridge,
            cpu: cpu,
            gpu: gpu,
            joypad: joypad,
            serial: serial,
            timer: timer,
        }
    }
}

impl Memory for Mmu {
    fn read(&self, addr: u16) -> u8 {
        if self.bios_enable && addr < 0x100 {
            return DMG_BIOS[addr as usize];
        }

        match addr {
            0x0000..=0x7FFF => self.cartridge.borrow().read_rom(addr),
            0x8000..=0x9FFF => self.gpu.borrow().read_video_ram(addr - 0x8000u16),
            0xC000..=0xDFFF => self.iram[(addr - 0xC000u16) as usize],
            0xE000..=0xFDFF => self.iram[(addr - 0xE000u16) as usize],
            0xFE00..=0xFE9F => self.gpu.borrow().read_object_attribute_ram(addr - 0xFE00u16),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80u16) as usize],

            // Cartridge
            0xA000..=0xBFFF => self.cartridge.borrow().read_ram(addr - 0xA000u16),

            // CPU
            0xFF0F => self.cpu.borrow().interruptions_requested(),
            0xFFFF => self.cpu.borrow().enabled_interrupts(),

            // Joypad
            0xFF00 => self.joypad.borrow().get_p1(),

            // Serial
            0xFF01 => self.serial.borrow().data(),
            0xFF02 => self.serial.borrow().control(),

            // Timer
            0xFF04 => self.timer.borrow().divider(),
            0xFF05 => self.timer.borrow().counter(),
            0xFF06 => self.timer.borrow().modulo(),
            0xFF07 => self.timer.borrow().control(),

            // GPU
            0xFF40 => self.gpu.borrow().lcdc(),
            0xFF41 => self.gpu.borrow().stat(),
            0xFF42 => self.gpu.borrow().scroll_y(),
            0xFF43 => self.gpu.borrow().scroll_x(),
            0xFF44 => self.gpu.borrow().scanline(),
            0xFF45 => self.gpu.borrow().scanline_compare(),
            0xFF47 => self.gpu.borrow().background_palette(),
            0xFF48 => self.gpu.borrow().object_palette_0(),
            0xFF49 => self.gpu.borrow().object_palette_1(),
            0xFF4A => self.gpu.borrow().window_y(),
            0xFF4B => self.gpu.borrow().window_x(),

            _ => 0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9FFF => self.gpu.borrow_mut().write_video_ram(addr - 0x8000, data),
            0xC000..=0xDFFF => self.iram[(addr - 0xC000) as usize] = data,
            0xE000..=0xFDFF => self.iram[(addr - 0xE000) as usize] = data,
            0xFE00..=0xFE9F => self.gpu.borrow_mut().write_object_attribute_ram(addr - 0xFE00, data),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = data,

            // Cartridge
            0xA000..=0xBFFF => self.cartridge.borrow_mut().write_ram(addr - 0xA000, data),

            // CPU
            0xFF0F => self.cpu.borrow_mut().set_interruptions_requested(data),
            0xFFFF => self.cpu.borrow_mut().set_enabled_interrupts(data),

            // JOYPAD
            0xFF00 => { self.joypad.borrow_mut().set_p1(data) }

            // SERIAL
            0xFF01 => { self.serial.borrow_mut().set_data(data) }
            0xFF02 => { self.serial.borrow_mut().set_control(data) }

            // TIMER
            0xFF04 => self.timer.borrow_mut().reset_divider(),
            0xFF05 => self.timer.borrow_mut().set_counter(data),
            0xFF06 => self.timer.borrow_mut().set_modulo(data),
            0xFF07 => self.timer.borrow_mut().set_control(data),

            // GPU
            0xFF40 => self.gpu.borrow_mut().set_lcdc(data),
            0xFF41 => self.gpu.borrow_mut().set_stat(data),
            0xFF42 => self.gpu.borrow_mut().set_scroll_y(data),
            0xFF43 => self.gpu.borrow_mut().set_scroll_x(data),
            0xFF45 => self.gpu.borrow_mut().set_scanline_compare(data),
            0xFF47 => self.gpu.borrow_mut().set_background_palette(data),
            0xFF48 => self.gpu.borrow_mut().set_object_palette_0(data),
            0xFF49 => self.gpu.borrow_mut().set_object_palette_1(data),
            0xFF4A => self.gpu.borrow_mut().set_window_y(data),
            0xFF4B => self.gpu.borrow_mut().set_window_x(data),

            // DMA
            0xFF46 => {
                if data <= 0xF1 {
                    println!("DMA ${:02x}00", data);

                    let addr = u16::from_be_bytes([data, 0x00]);
                    let mut oam: [u8; 160] = [0; 160];
                    for i in 0..160 {
                        oam[i] = self.read(addr + i as u16)
                    }
                    self.gpu.borrow_mut().populate_object_attribute_ram(&oam);
                }
            }

            // TURN OFF BIOS
            0xFF50 => {
                println!("Disabled Bios");
                self.bios_enable = false;
            }

            _ => { }
        }
    }
}