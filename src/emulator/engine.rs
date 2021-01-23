use sdl2::audio::AudioQueue;
use sdl2::render::Texture;

use crate::emulator::bios::DMG_BIOS;
use crate::emulator::cpu::alu::Alu;
use crate::emulator::cpu::asm::*;
use crate::emulator::cpu::regs::Regs;
use crate::emulator::cpu::interrupts::Interrupts;
use crate::emulator::cpu::Processor;
use crate::emulator::cartridge::Cartridge;
use crate::emulator::ppu::Ppu;
use crate::emulator::ppu::SCREEN_BUFFER_WIDTH;
use crate::emulator::joypad::Joypad;
use crate::emulator::mmu::Memory;
use crate::emulator::serial::Serial;
use crate::emulator::sound::Sounder;
use crate::emulator::timer::Timer;

pub const TICKS_PER_SECOND: u64 = 4_194_304;
pub const TICKS_PER_FRAME:  u64 = TICKS_PER_SECOND / 60;

pub struct Engine {
    // Arithmetic Logic Unit
    alu: Alu,

    // Registers
    regs: Regs,

    // next program counter position
    next_pc: u16,

    // Master Interruption Enable
    interrupt_enable: bool,

    // Next Master Interruption Enabled State
    // - auxiliar flag to emulate EI/DI change after execute next instruction
    next_interrupt_enable: bool,

    // BIOS Enabled
    bios_enable: bool,

    // BIOS Program
    // - $0000..=$0100
    bios: Box<[u8; 0x100]>,

    // Cartrige Loader
    // - $0000..=$7FFFF (ROM)
    // - $A000..=$BFFFF (RAM)
    cartridge: Box<Cartridge>,

    // Random Access Memory
    // - $C000..=$DFFF (Internal RAM)
    // - $E000..=$FDFF (Echo of Internal RAM)
    // - $FF80..=$FFFE (Zero Page)
    ram: Box<[u8; 0x2000 + 127]>,

    // PPU
    // - $8000..=$9FFF (Video RAM)
    // - $FE00..=$FE9F (Object Attribute Memory)
    // - $FF40..=$FF4B (Hardware IO)
    ppu: Box<Ppu>,

    // Joypad
    // - $FF00 (Hardware IO)
    joypad: Box<Joypad>,

    // Serial
    // - $FF01..=$FF02 (Hardware IO)
    serial: Box<Serial>,

    // Sounder
    sounder: Box<Sounder>,

    // Timer
    // - $FF04..=$FF07 (Hardware IO)
    timer: Box<Timer>,

    // Interruption Enable Register (IE)
    // - $FFFF (Hardware IO)
    interruptions_enabled: Interrupts,

    // Interruption Flag (IF)
    // - $FF0F (Hardware IO)
    interruptions_requested: Interrupts,
}

impl Engine {
    pub fn open_rom_file(&mut self, filename: &str) {
        self.cartridge.open(filename);
    }

    pub fn blit_frame_to_texture(&mut self, texture: &mut Texture) {
        texture.update(None, self.ppu.frame_buffer(), SCREEN_BUFFER_WIDTH).unwrap();
    }

    pub fn enqueue_audio_samples(&mut self, channels: &mut [AudioQueue<i8>; 4]) {
        self.sounder.enqueue_audio_samples(channels);
    }

    pub fn process_event(&mut self, event: &sdl2::event::Event) {
        self.joypad.process_event(event);
    }

    pub fn run_next_step(&mut self) -> u64 {
        let ticks = self.fetch_decode_execute_store_cycle();

        self.serial.step(ticks);
        if self.serial.transfering_completion_interruption_requested() {
            self.interruptions_requested.set_serial_transfer_complete();
        }

        self.timer.step(ticks);
        if self.timer.overflow_interrupt_requested() {
            self.interruptions_requested.set_timer_overflow();
        }

        self.ppu.step(ticks);
        if self.ppu.lcdc_status_interrupt_requested() {
            self.interruptions_requested.set_lcdc_status();
        }
        if self.ppu.vertical_blank_interrupt_requested() {
            self.interruptions_requested.set_vertical_blank();
        }

        if self.joypad.interruption_requested() {
            self.joypad.reset_interruption_requested();
            self.interruptions_requested.set_high_to_low_pin10_to_pin_13();
        }

        ticks
    }

    pub fn run_next_frame(&mut self, ticks_counter: u64) -> u64 {
        let mut ticks_counter = ticks_counter;
        while ticks_counter < TICKS_PER_FRAME {
            ticks_counter += self.run_next_step();
        }
        ticks_counter - TICKS_PER_FRAME
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            alu: Alu::default(),
            regs: Regs::default(),
            next_pc: 0,

            interrupt_enable: true,
            next_interrupt_enable: true,

            bios_enable: true,
            bios: Box::new(DMG_BIOS),
            ram: Box::new([0; 0x2000 + 127]),
            cartridge: Box::new(Cartridge::default()),
            ppu: Box::new(Ppu::default()),
            joypad: Box::new(Joypad::default()),
            serial: Box::new(Serial::default()),
            sounder: Box::new(Sounder::default()),
            timer: Box::new(Timer::default()),
            interruptions_enabled: Interrupts::default(),
            interruptions_requested: Interrupts::default(),
        }
    }
}

impl Memory for Engine {
    fn read(&self, addr: u16) -> u8 {
        if self.bios_enable && addr < 0x100 {
            return self.bios[addr as usize];
        }

        if addr < 0x8000 {        // 0x0000..=0x7FFF (Cartridge ROM)
            self.cartridge.read_rom(addr)
        } else if addr < 0xA000 { // 0x8000..=0x9FFF (Video RAM)
            self.ppu.read_video_ram(addr - 0x8000)
        } else if addr < 0xC000 { // 0xA000..=0xBFFF (Cartridge RAM)
            self.cartridge.read_ram(addr - 0xA000)
        } else if addr < 0xE000 { // 0xC000..=0xDFFF (Internal RAM)
            self.ram[(addr - 0xC000) as usize]
        } else if addr < 0xFE00 { // 0xE000..=0xFDFF (Echo RAM)
            self.ram[(addr - 0xE000) as usize]
        } else if addr < 0xFEA0 { // 0xFE00..=0xFE9F (OAM)
            self.ppu.read_object_attribute_ram(addr - 0xFE00)
        } else if addr < 0xFF00 { // 0xFEA0..=0xFEFF (Unusable)
            0
        } else if addr < 0xFF80 { // 0xFF00..=0xFF7F (Hardware IO)
            match addr {
                // CPU
                0xFF0F => self.interruptions_requested.into(),

                // Joypad
                0xFF00 => self.joypad.get_p1(),

                // Serial
                0xFF01 => self.serial.data(),
                0xFF02 => self.serial.control(),

                // Timer
                0xFF04 => self.timer.divider(),
                0xFF05 => self.timer.counter(),
                0xFF06 => self.timer.modulo(),
                0xFF07 => self.timer.control(),

                // Sounder
                0xFF10 => self.sounder.channel1_r0(),
                0xFF11 => self.sounder.channel1_r1(),
                0xFF12 => self.sounder.channel1_r2(),
                0xFF13 => self.sounder.channel1_r3(),
                0xFF14 => self.sounder.channel1_r4(),

                0xFF16 => self.sounder.channel2_r1(),
                0xFF17 => self.sounder.channel2_r2(),
                0xFF18 => self.sounder.channel2_r3(),
                0xFF19 => self.sounder.channel2_r4(),

                0xFF1A => self.sounder.channel3_r0(),
                0xFF1B => self.sounder.channel3_r1(),
                0xFF1C => self.sounder.channel3_r2(),
                0xFF1D => self.sounder.channel3_r3(),
                0xFF1E => self.sounder.channel3_r4(),

                0xFF20 => self.sounder.channel4_r1(),
                0xFF21 => self.sounder.channel4_r2(),
                0xFF22 => self.sounder.channel4_r3(),
                0xFF23 => self.sounder.channel4_r4(),

                0xFF24 => self.sounder.master_r0(),
                0xFF25 => self.sounder.master_r1(),
                0xFF26 => self.sounder.master_r2(),

                0xFF30 => self.sounder.channel3_sample(0x0),
                0xFF31 => self.sounder.channel3_sample(0x1),
                0xFF32 => self.sounder.channel3_sample(0x2),
                0xFF33 => self.sounder.channel3_sample(0x3),
                0xFF34 => self.sounder.channel3_sample(0x4),
                0xFF35 => self.sounder.channel3_sample(0x5),
                0xFF36 => self.sounder.channel3_sample(0x6),
                0xFF37 => self.sounder.channel3_sample(0x7),
                0xFF38 => self.sounder.channel3_sample(0x8),
                0xFF39 => self.sounder.channel3_sample(0x9),
                0xFF3A => self.sounder.channel3_sample(0xA),
                0xFF3B => self.sounder.channel3_sample(0xB),
                0xFF3C => self.sounder.channel3_sample(0xC),
                0xFF3D => self.sounder.channel3_sample(0xD),
                0xFF3E => self.sounder.channel3_sample(0xE),
                0xFF3F => self.sounder.channel3_sample(0xF),

                // PPU
                0xFF40 => self.ppu.lcdc(),
                0xFF41 => self.ppu.stat(),
                0xFF42 => self.ppu.scroll_y(),
                0xFF43 => self.ppu.scroll_x(),
                0xFF44 => self.ppu.scanline(),
                0xFF45 => self.ppu.scanline_compare(),
                0xFF47 => self.ppu.background_palette(),
                0xFF48 => self.ppu.object_palette_0(),
                0xFF49 => self.ppu.object_palette_1(),
                0xFF4A => self.ppu.window_y(),
                0xFF4B => self.ppu.window_x(),
                _ => 0
            }
        } else if addr < 0xFFFF { // 0xFF80..=0xFFFE (Zero Page)
            self.ram[0x2000 + (addr - 0xFF80u16) as usize]
        } else {
            self.interruptions_enabled.into()
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr < 0x8000 {        // 0x0000..=0x7FFF (Cartridge ROM)
            // read-only
        } else if addr < 0xA000 { // 0x8000..=0x9FFF (Video RAM)
            self.ppu.write_video_ram(addr - 0x8000, data)
        } else if addr < 0xC000 { // 0xA000..=0xBFFF (Cartridge RAM)
            self.cartridge.write_ram(addr - 0xA000, data)
        } else if addr < 0xE000 { // 0xC000..=0xDFFF (Internal RAM)
            self.ram[(addr - 0xC000) as usize] = data
        } else if addr < 0xE000 { // 0xE000..=0xFDFF (Echo RAM)
            self.ram[(addr - 0xE000) as usize] = data
        } else if addr < 0xFEA0 { // 0xFE00..=0xFE9F (OAM)
            self.ppu.write_object_attribute_ram(addr - 0xFE00, data)
        } else if addr < 0xFF00 { // 0xFEA0..=0xFEFF (Unusable)
            // read-only
        } else if addr < 0xFF80 { // 0xFF00..=0xFF7F (Hardware IO)
            match addr {
                // CPU
                0xFF0F => self.interruptions_requested = data.into(),

                // JOYPAD
                0xFF00 => { self.joypad.set_p1(data) }

                // SERIAL
                0xFF01 => { self.serial.set_data(data) }
                0xFF02 => { self.serial.set_control(data) }

                // TIMER
                0xFF04 => self.timer.reset_divider(),
                0xFF05 => self.timer.set_counter(data),
                0xFF06 => self.timer.set_modulo(data),
                0xFF07 => self.timer.set_control(data),

                // Sounder
                0xFF10 => self.sounder.set_channel1_r0(data),
                0xFF11 => self.sounder.set_channel1_r1(data),
                0xFF12 => self.sounder.set_channel1_r2(data),
                0xFF13 => self.sounder.set_channel1_r3(data),
                0xFF14 => self.sounder.set_channel1_r4(data),

                0xFF16 => self.sounder.set_channel2_r1(data),
                0xFF17 => self.sounder.set_channel2_r2(data),
                0xFF18 => self.sounder.set_channel2_r3(data),
                0xFF19 => self.sounder.set_channel2_r4(data),

                0xFF1A => self.sounder.set_channel3_r0(data),
                0xFF1B => self.sounder.set_channel3_r1(data),
                0xFF1C => self.sounder.set_channel3_r2(data),
                0xFF1D => self.sounder.set_channel3_r3(data),
                0xFF1E => self.sounder.set_channel3_r4(data),

                0xFF20 => self.sounder.set_channel4_r1(data),
                0xFF21 => self.sounder.set_channel4_r2(data),
                0xFF22 => self.sounder.set_channel4_r3(data),
                0xFF23 => self.sounder.set_channel4_r4(data),

                0xFF24 => self.sounder.set_master_r0(data),
                0xFF25 => self.sounder.set_master_r1(data),
                0xFF26 => self.sounder.set_master_r2(data),

                0xFF30 => self.sounder.set_channel3_sample(0x0, data),
                0xFF31 => self.sounder.set_channel3_sample(0x1, data),
                0xFF32 => self.sounder.set_channel3_sample(0x2, data),
                0xFF33 => self.sounder.set_channel3_sample(0x3, data),
                0xFF34 => self.sounder.set_channel3_sample(0x4, data),
                0xFF35 => self.sounder.set_channel3_sample(0x5, data),
                0xFF36 => self.sounder.set_channel3_sample(0x6, data),
                0xFF37 => self.sounder.set_channel3_sample(0x7, data),
                0xFF38 => self.sounder.set_channel3_sample(0x8, data),
                0xFF39 => self.sounder.set_channel3_sample(0x9, data),
                0xFF3A => self.sounder.set_channel3_sample(0xA, data),
                0xFF3B => self.sounder.set_channel3_sample(0xB, data),
                0xFF3C => self.sounder.set_channel3_sample(0xC, data),
                0xFF3D => self.sounder.set_channel3_sample(0xD, data),
                0xFF3E => self.sounder.set_channel3_sample(0xE, data),
                0xFF3F => self.sounder.set_channel3_sample(0xF, data),

                // PPU
                0xFF40 => self.ppu.set_lcdc(data),
                0xFF41 => self.ppu.set_stat(data),
                0xFF42 => self.ppu.set_scroll_y(data),
                0xFF43 => self.ppu.set_scroll_x(data),
                0xFF45 => self.ppu.set_scanline_compare(data),
                0xFF47 => self.ppu.set_background_palette(data),
                0xFF48 => self.ppu.set_object_palette_0(data),
                0xFF49 => self.ppu.set_object_palette_1(data),
                0xFF4A => self.ppu.set_window_y(data),
                0xFF4B => self.ppu.set_window_x(data),

                // DMA
                0xFF46 => {
                    if data <= 0xF1 {
                        println!("DMA ${:02x}00", data);

                        let addr = u16::from_be_bytes([data, 0x00]);
                        let mut oam: [u8; 160] = [0; 160];
                        for i in 0..160 {
                            oam[i] = self.read(addr + i as u16)
                        }

                        self.ppu.populate_object_attribute_ram(&oam);
                    }
                }

                // TURN OFF BIOS
                0xFF50 => {
                    println!("Disabled Bios");
                    self.bios_enable = false;
                }

                _ => { }
            }
        } else if addr < 0xFFFF { // 0xFF80..=0xFFFE (Zero Page)
            self.ram[0x2000 + (addr - 0xFF80) as usize] = data
        } else {
            self.interruptions_enabled = data.into()
        }
    }
}

impl Processor for Engine {
    fn interrupt_service_routine(&mut self) -> bool {
        let interrupt_enable = self.interrupt_enable;
        self.interrupt_enable = self.next_interrupt_enable;

        if !interrupt_enable {
            return false;
        }

        let i = self.interruptions_enabled & self.interruptions_requested;

        if i.vertical_blank() {
            self.subroutine_call(0x40);
        } else if i.lcdc_status() {
            self.subroutine_call(0x48);
        } else if i.timer_overflow() {
            self.subroutine_call(0x50);
        } else if i.serial_transfer_complete() {
            self.subroutine_call(0x58);
        } else if i.high_to_low_pin10_to_pin_13() {
            self.subroutine_call(0x60);
        } else {
            return false;
        }

        self.interrupt_enable = false;
        return true;
    }

    fn fetch_decode_execute_store_cycle(&mut self) -> u64 {
        if self.interrupt_service_routine() {
            return 4
        }

        let pc = self.regs.pc();

        // Fetch
        let opcode = self.read(pc);
        let immediate8: u8 = self.read(pc + 1);
        let immediate16: u16 = u16::from_le_bytes([immediate8, self.read(pc + 2)]);
        self.next_pc = pc + instruction_size(opcode);

        // Decode => Execute => Store
        match opcode {
            0x00 => {
                // NOP
            }
            0x01 => {
                // LD BC, $0000
                self.regs.set_bc(immediate16);
            }
            0x02 => {
                // LD (BC), A
                self.write(self.regs.bc(), self.regs.a());
            },
            0x03 => {
                // INC BC
                self.regs.set_bc(self.alu.inc16(self.regs.bc()));
            }
            0x04 => {
                // INC B
                self.regs.set_b(self.alu.inc(self.regs.b()));
            }
            0x05 => {
                // DEC B
                self.regs.set_b(self.alu.dec(self.regs.b()));
            }
            0x06 => {
                // LD B, $00
                self.regs.set_b(immediate8);
            }
            0x07 => {
                // RLCA
                self.regs.set_a(self.alu.rlc(self.regs.a()));
            }
            0x08 => {
                // LD ($0000),SP
                let le_bytes = self.regs.sp().to_le_bytes();
                self.write(immediate16, le_bytes[0]);
                self.write(immediate16 + 1, le_bytes[1]);
            }
            0x09 => {
                // ADD HL, BC
                self.regs.set_hl(self.alu.add16(self.regs.hl(), self.regs.bc()));
            }
            0x0A => {
                // LD A, (BC)
                let data = self.read(self.regs.bc());
                self.regs.set_a(data);
            }
            0x0B => {
                // DEC BC
                self.regs.set_bc(self.alu.dec16(self.regs.bc()));
            }
            0x0C => {
                // INC C
                self.regs.set_c(self.alu.inc(self.regs.c()));
            }
            0x0D => {
                // DEC C
                self.regs.set_c(self.alu.dec(self.regs.c()));
            }
            0x0E => {
                // LD C, $00
                self.regs.set_c(immediate8);
            }
            0x0F => {
                // RRCA
                self.regs.set_a(self.alu.rrc(self.regs.a()));
            }
            0x10 => {
                // STOP 0
            }
            0x11 => {
                // LD DE, $0000
                self.regs.set_de(immediate16);
            }
            0x12 => {
                // LD (DE), A
                self.write(self.regs.de(), self.regs.a());
            }
            0x13 => {
                // INC DE
                self.regs.set_de(self.alu.inc16(self.regs.de()));
            }
            0x14 => {
                // INC D
                self.regs.set_d(self.alu.inc(self.regs.d()));
            }
            0x15 => {
                // DEC D
                self.regs.set_d(self.alu.dec(self.regs.d()));
            }
            0x16 => {
                // LD D, $00
                self.regs.set_d(immediate8);
            }
            0x17 => {
                // RLA
                self.regs.set_a(self.alu.rl(self.regs.a()));
            }
            0x18 => {
                // JR $00
                self.jump_relative(immediate8);
            }
            0x19 => {
                // ADD HL, DE
                self.regs.set_hl(self.alu.add16(self.regs.hl(), self.regs.de()));
            }
            0x1A => {
                // LD A, (DE)
                let data = self.read(self.regs.de());
                self.regs.set_a(data);
            }
            0x1B => {
                // DEC DE
                self.regs.set_de(self.alu.dec16(self.regs.de()));
            }
            0x1C => {
                // INC E
                self.regs.set_e(self.alu.inc(self.regs.e()));
            }
            0x1D => {
                // DEC E
                self.regs.set_e(self.alu.dec(self.regs.e()));
            }
            0x1E => {
                // LD E, $00
                self.regs.set_e(immediate8);
            }
            0x1F => {
                // RRA
                self.regs.set_a(self.alu.rr(self.regs.a()));
            }
            0x20 => {
                // JR NZ $00
                self.jump_relative_if(immediate8, !self.alu.flags.zero());
            }
            0x21 => {
                // LD HL, $0000
                self.regs.set_hl(immediate16);
            }
            0x22 => {
                // LDI (HL), A
                let addr = self.regs.hl();
                self.write(addr, self.regs.a());
                self.regs.set_hl(addr.wrapping_add(1));
            }
            0x23 => {
                // INC HL
                self.regs.set_hl(self.alu.inc16(self.regs.hl()));
            }
            0x24 => {
                // INC H
                self.regs.set_h(self.alu.inc(self.regs.h()));
            }
            0x25 => {
                // DEC H
                self.regs.set_h(self.alu.dec(self.regs.h()));
            }
            0x26 => {
                // LD H, $00
                self.regs.set_h(immediate8);
            }
            0x27 => {
                // DAA
                self.regs.set_a(self.alu.daa(self.regs.a()));
            }
            0x28 => {
                // JR Z $00
                self.jump_relative_if(immediate8, self.alu.flags.zero());
            }
            0x29 => {
                // ADD HL, HL
                self.regs.set_hl(self.alu.add16(self.regs.hl(), self.regs.hl()));
            }
            0x2A => {
                // LDI A, (HL)
                let addr = self.regs.hl();
                let data = self.read(addr);
                self.regs.set_a(data);
                self.regs.set_hl(addr.wrapping_add(1));
            }
            0x2B => {
                // DEC HL
                self.regs.set_hl(self.alu.dec16(self.regs.hl()));
            }
            0x2C => {
                // INC L
                self.regs.set_l(self.alu.inc(self.regs.l()));
            }
            0x2D => {
                // DEC L
                self.regs.set_l(self.alu.dec(self.regs.l()));
            }
            0x2E => {
                // LD L, $00
                self.regs.set_l(immediate8);
            }
            0x2F => {
                // CPL
                self.regs.set_a(self.alu.complement(self.regs.a()))
            }
            0x30 => {
                // JR NC $00
                self.jump_relative_if(immediate8, !self.alu.flags.carry());
            }
            0x31 => {
                // LD SP, $0000
                self.regs.set_sp(immediate16);
            }
            0x32 => {
                // LDD (HL), A
                let addr = self.regs.hl();
                self.write(addr, self.regs.a());
                self.regs.set_hl(addr.wrapping_sub(1));
            }
            0x33 => {
                // INC SP
                self.regs.set_sp(self.regs.sp().wrapping_add(1));
            }
            0x34 => {
                // INC (HL)
                let addr = self.regs.hl();
                let data = self.read(addr);
                let data = self.alu.inc(data);
                self.write(addr, data);
            }
            0x35 => {
                // DEC (HL)
                let addr = self.regs.hl();
                let data = self.read(addr);
                let data = self.alu.dec(data);
                self.write(addr, data);
            }
            0x36 => {
                // LD (HL), $00
                self.write(self.regs.hl(), immediate8);
            }
            0x37 => {
                // SCF
                self.alu.flags.set_carry();
            }
            0x38 => {
                // JR C $00
                self.jump_relative_if(immediate8, self.alu.flags.carry());
            }
            0x39 => {
                // ADD HL, SP
                self.regs.set_hl(self.alu.add16(self.regs.hl(), self.regs.sp()));
            }
            0x3A => {
                // LDD A, (HL)
                let addr = self.regs.hl();
                let data = self.read(addr);
                self.regs.set_a(data);
                self.regs.set_hl(addr.wrapping_sub(1));
            }
            0x3B => {
                // DEC SP
                self.regs.set_sp(self.regs.sp().wrapping_sub(1));
            }
            0x3C => {
                // INC A
                self.regs.set_a(self.alu.inc(self.regs.a()));
            }
            0x3D => {
                // DEC A
                self.regs.set_a(self.alu.dec(self.regs.a()));
            }
            0x3E => {
                // LD A, $00
                self.regs.set_a(immediate8);
            }
            0x3F => {
                // CCF
                self.alu.flags.reset_carry();
            }
            0x40 => {
                // LD B, B
            }
            0x41 => {
                // LD B, C
                self.regs.set_b(self.regs.c());
            }
            0x42 => {
                // LD B, D
                self.regs.set_b(self.regs.d());
            }
            0x43 => {
                // LD B, E
                self.regs.set_b(self.regs.e());
            }
            0x44 => {
                // LD B, H
                self.regs.set_b(self.regs.h());
            }
            0x45 => {
                // LD B, L
                self.regs.set_b(self.regs.l());
            }
            0x46 => {
                // LD B, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_b(data);
            }
            0x47 => {
                // LD B, A
                self.regs.set_b(self.regs.a());
            }
            0x48 => {
                // LD C, B
                self.regs.set_c(self.regs.b());
            }
            0x49 => {
                // LD C, C
            }
            0x4A => {
                // LD C, D
                self.regs.set_c(self.regs.d());
            }
            0x4B => {
                // LD C, E
                self.regs.set_c(self.regs.e());
            }
            0x4C => {
                // LD C, H
                self.regs.set_c(self.regs.h());
            }
            0x4D => {
                // LD C, L
                self.regs.set_c(self.regs.l());
            }
            0x4E => {
                // LD C, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_c(data);
            }
            0x4F => {
                // LD C, A
                self.regs.set_c(self.regs.a());
            }
            0x50 => {
                // LD D, B
                self.regs.set_d(self.regs.b());
            }
            0x51 => {
                // LD D, C
                self.regs.set_d(self.regs.c());
            }
            0x52 => {
                // LD D, D
            }
            0x53 => {
                // LD D, E
                self.regs.set_d(self.regs.e());
            }
            0x54 => {
                // LD D, H
                self.regs.set_d(self.regs.h());
            }
            0x55 => {
                // LD D, L
                self.regs.set_d(self.regs.l());
            }
            0x56 => {
                // LD D, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_d(data);
            }
            0x57 => {
                // LD D, A
                self.regs.set_d(self.regs.a());
            }
            0x58 => {
                // LD E, B
                self.regs.set_e(self.regs.b());
            }
            0x59 => {
                // LD E, C
                self.regs.set_e(self.regs.c());
            }
            0x5A => {
                // LD E, D
                self.regs.set_e(self.regs.d());
            }
            0x5B => {
                // LD E, E
            }
            0x5C => {
                // LD E, H
                self.regs.set_e(self.regs.h());
            }
            0x5D => {
                // LD E, L
                self.regs.set_e(self.regs.l());
            }
            0x5E => {
                // LD E, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_e(data);
            }
            0x5F => {
                // LD E, A
                self.regs.set_e(self.regs.a());
            }
            0x60 => {
                // LD H, B
                self.regs.set_h(self.regs.b());
            }
            0x61 => {
                // LD H, C
                self.regs.set_h(self.regs.c());
            }
            0x62 => {
                // LD H, D
                self.regs.set_h(self.regs.d());
            }
            0x63 => {
                // LD H, E
                self.regs.set_h(self.regs.e());
            }
            0x64 => {
                // LD H, H
            }
            0x65 => {
                // LD H, L
                self.regs.set_h(self.regs.l());
            }
            0x66 => {
                // LD H, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_h(data);
            }
            0x67 => {
                // LD H, A
                self.regs.set_h(self.regs.a());
            }
            0x68 => {
                // LD L, B
                self.regs.set_l(self.regs.b());
            }
            0x69 => {
                // LD L, C
                self.regs.set_l(self.regs.c());
            }
            0x6A => {
                // LD L, D
                self.regs.set_l(self.regs.d());
            }
            0x6B => {
                // LD L, E
                self.regs.set_l(self.regs.e());
            }
            0x6C => {
                // LD L, H
                self.regs.set_l(self.regs.h());
            }
            0x6D => {
                // LD L, L
            }
            0x6E => {
                // LD L, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_l(data);
            }
            0x6F => {
                // LD L, A
                self.regs.set_l(self.regs.a());
            }
            0x70 => {
                // LD (HL), B
                self.write(self.regs.hl(), self.regs.b());
            }
            0x71 => {
                // LD (HL), C
                self.write(self.regs.hl(), self.regs.c());
            }
            0x72 => {
                // LD (HL), D
                self.write(self.regs.hl(), self.regs.d());
            }
            0x73 => {
                // LD (HL), E
                self.write(self.regs.hl(), self.regs.e());
            }
            0x74 => {
                // LD (HL), H
                self.write(self.regs.hl(), self.regs.h());
            }
            0x75 => {
                // LD (HL), L
                self.write(self.regs.hl(), self.regs.l());
            }
            0x76 => {
                // LD (HL), (HL)
            }
            0x77 => {
                // LD (HL), A
                self.write(self.regs.hl(), self.regs.a());
            }
            0x78 => {
                // LD A, B
                self.regs.set_a(self.regs.b());
            }
            0x79 => {
                // LD A, C
                self.regs.set_a(self.regs.c());
            }
            0x7A => {
                // LD A, D
                self.regs.set_a(self.regs.d());
            }
            0x7B => {
                // LD A, E
                self.regs.set_a(self.regs.e());
            }
            0x7C => {
                // LD A, H
                self.regs.set_a(self.regs.h());
            }
            0x7D => {
                // LD A, L
                self.regs.set_a(self.regs.l());
            }
            0x7E => {
                // LD A, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_a(data);
            }
            0x7F => {
                // LD A, A
            }
            0x80 => {
                // ADD A, B
                self.regs.set_a(self.regs.b());
            }
            0x81 => {
                // ADD A, C
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.c()));
            }
            0x82 => {
                // ADD A, D
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.d()));
            }
            0x83 => {
                // ADD A, E
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.e()));
            }
            0x84 => {
                // ADD A, H
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.h()));
            }
            0x85 => {
                // ADD A, L
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.l()));
            }
            0x86 => {
                // ADD A, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_a(self.alu.add(self.regs.a(), data));
            }
            0x87 => {
                // ADD A, A
                self.regs.set_a(self.alu.add(self.regs.a(), self.regs.a()));
            }
            0x88 => {
                // ADC A, B
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.b()));
            }
            0x89 => {
                // ADC A, C
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.c()));
            }
            0x8A => {
                // ADC A, D
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.d()));
            }
            0x8B => {
                // ADC A, E
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.e()));
            }
            0x8C => {
                // ADC A, H
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.h()));
            }
            0x8D => {
                // ADC A, L
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.l()));
            }
            0x8E => {
                // ADC A, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_a(self.alu.adc(self.regs.a(), data));
            }
            0x8F => {
                // ADC A, A
                self.regs.set_a(self.alu.adc(self.regs.a(), self.regs.a()));
            }
            0x90 => {
                // SUB A, B
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.b()));
            }
            0x91 => {
                // SUB A, C
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.c()));
            }
            0x92 => {
                // SUB A, D
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.d()));
            }
            0x93 => {
                // SUB A, E
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.e()));
            }
            0x94 => {
                // SUB A, H
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.h()));
            }
            0x95 => {
                // SUB A, L
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.l()));
            }
            0x96 => {
                // SUB A, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_a(self.alu.sub(self.regs.a(), data));
            }
            0x97 => {
                // SUB A, A
                self.regs.set_a(self.alu.sub(self.regs.a(), self.regs.a()));
            }
            0x98 => {
                // SBC A, B
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.b()));
            }
            0x99 => {
                // SBC A, C
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.c()));
            }
            0x9A => {
                // SBC A, D
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.d()));
            }
            0x9B => {
                // SBC A, E
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.e()));
            }
            0x9C => {
                // SBC A, H
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.h()));
            }
            0x9D => {
                // SBC A, L
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.l()));
            }
            0x9E => {
                // SBC A, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_a(self.alu.sbc(self.regs.a(), data));
            }
            0x9F => {
                // SBC A, A
                self.regs.set_a(self.alu.sbc(self.regs.a(), self.regs.a()));
            }
            0xA0 => {
                // AND A, B
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.b()));
            }
            0xA1 => {
                // AND A, C
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.c()));
            }
            0xA2 => {
                // AND A, D
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.d()));
            }
            0xA3 => {
                // AND A, E
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.e()));
            }
            0xA4 => {
                // AND A, H
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.h()));
            }
            0xA5 => {
                // AND A, L
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.l()));
            }
            0xA6 => {
                // AND A, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_a(self.alu.and(self.regs.a(), data));
            }
            0xA7 => {
                // AND A, A
                self.regs.set_a(self.alu.and(self.regs.a(), self.regs.a()));
            }
            0xA8 => {
                // XOR A, B
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.b()));
            }
            0xA9 => {
                // XOR A, C
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.c()));
            }
            0xAA => {
                // XOR A, D
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.d()));
            }
            0xAB => {
                // XOR A, E
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.e()));
            }
            0xAC => {
                // XOR A, H
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.h()));
            }
            0xAD => {
                // XOR A, L
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.l()));
            }
            0xAE => {
                // XOR A, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_a(self.alu.xor(self.regs.a(), data));
            }
            0xAF => {
                // XOR A, A
                self.regs.set_a(self.alu.xor(self.regs.a(), self.regs.a()));
            }
            0xB0 => {
                // OR A, B
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.b()));
            }
            0xB1 => {
                // OR A, C
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.c()));
            }
            0xB2 => {
                // OR A, D
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.d()));
            }
            0xB3 => {
                // OR A, E
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.e()));
            }
            0xB4 => {
                // OR A, H
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.h()));
            }
            0xB5 => {
                // OR A, L
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.l()));
            }
            0xB6 => {
                // OR A, (HL)
                let data = self.read(self.regs.hl());
                self.regs.set_a(self.alu.or(self.regs.a(), data));
            }
            0xB7 => {
                // OR A, A
                self.regs.set_a(self.alu.or(self.regs.a(), self.regs.a()));
            }
            0xB8 => {
                // CP A, B
                self.alu.compare(self.regs.a(), self.regs.b());
            }
            0xB9 => {
                // CP A, C
                self.alu.compare(self.regs.a(), self.regs.c());
            }
            0xBA => {
                // CP A, D
                self.alu.compare(self.regs.a(), self.regs.d());
            }
            0xBB => {
                // CP A, E
                self.alu.compare(self.regs.a(), self.regs.e());
            }
            0xBC => {
                // CP A, H
                self.alu.compare(self.regs.a(), self.regs.h());
            }
            0xBD => {
                // CP A, L
                self.alu.compare(self.regs.a(), self.regs.l());
            }
            0xBE => {
                // CP A, (HL)
                let data = self.read(self.regs.hl());
                self.alu.compare(self.regs.a(), data);
            }
            0xBF => {
                // CP A, A
                self.alu.compare(self.regs.a(), self.regs.a());
            }
            0xC0 => {
                // RET NZ
                self.subroutine_return_if(!self.alu.flags.zero());
            }
            0xC1 => {
                // POP BC
                let bc = self.stack_pop();
                self.regs.set_bc(bc);
            }
            0xC2 => {
                // JP NZ $0000
                self.jump_absolute_if(immediate16, !self.alu.flags.zero());
            }
            0xC3 => {
                // JP $0000
                self.jump_absolute(immediate16);
            }
            0xC4 => {
                // CALL NZ $0000
                self.subroutine_call_if(immediate16, !self.alu.flags.zero());
            }
            0xC5 => {
                // PUSH BC
                self.stack_push(self.regs.bc());
            }
            0xC6 => {
                // ADD A, $00
                self.regs.set_a(self.alu.add(self.regs.a(), immediate8));
            }
            0xC7 => {
                // RST $00
                self.subroutine_call(0x00);
            }
            0xC8 => {
                // RET Z
                self.subroutine_return_if(self.alu.flags.zero());
            }
            0xC9 => {
                // RET
                self.subroutine_return();
            }
            0xCA => {
                // JP Z $0000
                self.jump_absolute_if(immediate16, self.alu.flags.zero());
            }
            0xCB => {
                // PREFIX CB (Logic Instruction Extension)

                let arg = match immediate8 & 0x7 {
                    0x0 => self.regs.b(),
                    0x1 => self.regs.c(),
                    0x2 => self.regs.d(),
                    0x3 => self.regs.e(),
                    0x4 => self.regs.h(),
                    0x5 => self.regs.l(),
                    0x6 => self.read(self.regs.hl()),
                    0x7 => self.regs.a(),
                    _ => panic!()
                };
        
                let mut ret = arg;
        
                match immediate8 {
                    0x00..=0x07 => {
                        // RLC
                        ret = self.alu.rlc(arg);
                    }
                    0x08..=0x0F => {
                        // RRC
                        ret = self.alu.rrc(arg);
                    }
                    0x10..=0x17 => {
                        // RL
                        ret = self.alu.rl(arg);
                    }
                    0x18..=0x1F => {
                        // RR
                        ret = self.alu.rr(arg);
                    }
                    0x20..=0x27 => {
                        // SLA
                        ret = self.alu.sla(arg);
                    }
                    0x28..=0x2F => {
                        // SRA
                        ret = self.alu.sra(arg);
                    }
                    0x30..=0x37 => {
                        // SWAP
                        ret = self.alu.nibble_swap(arg);
                    }
                    0x38..=0x3F => {
                        // SRL
                        ret = self.alu.srl(arg);
                    }
                    0x40..=0x7F => {
                        // BIT
                        let bit_index = (immediate8 - 0x40) / 8;
                        self.alu.test_bit(arg, bit_index);
                    }
                    0x80..=0xBF => {
                        // RES
                        let bit_index = (immediate8 - 0x80) / 8;
                        ret = self.alu.reset_bit(arg, bit_index);
                    }
                    0xC0..=0xFF => {
                        // SET
                        let bit_index = (immediate8 - 0xC0) / 8;
                        ret = self.alu.set_bit(arg, bit_index);
                    }
                }
        
                if ret != arg {
                    match immediate8 & 0x7 {
                        0x0 => self.regs.set_a(ret),
                        0x1 => self.regs.set_c(ret),
                        0x2 => self.regs.set_d(ret),
                        0x3 => self.regs.set_e(ret),
                        0x4 => self.regs.set_h(ret),
                        0x5 => self.regs.set_l(ret),
                        0x6 => { self.write(self.regs.hl(), ret); }
                        0x7 => self.regs.set_a(ret),
                        _ => panic!()
                    }
                }
            }
            0xCC => {
                // CALL Z $0000
                self.subroutine_call_if(immediate16, self.alu.flags.zero())
            }
            0xCD => {
                // CALL $0000
                self.subroutine_call(immediate16)
            }
            0xCE => {
                // ADC A, $00
                self.regs.set_a(self.alu.adc(self.regs.a(), immediate8));
            }
            0xCF => {
                // RST $08
                self.subroutine_call(0x08);
            }
            0xD0 => {
                // RET NC
                self.subroutine_return_if(!self.alu.flags.carry());
            }
            0xD1 => {
                // POP DE
                let de = self.stack_pop();
                self.regs.set_de(de);
            }
            0xD2 => {
                // JP NC $0000
                self.jump_absolute_if(immediate16, !self.alu.flags.carry());
            }
            0xD3 => {
                // [D3] - INVALID
            }
            0xD4 => {
                // CALL NC $0000
                self.subroutine_call_if(immediate16, !self.alu.flags.carry())
            }
            0xD5 => {
                // PUSH DE
                self.stack_push(self.regs.de());
            }
            0xD6 => {
                // SUB A, $00
                self.regs.set_a(self.alu.sub(self.regs.a(), immediate8));
            }
            0xD7 => {
                // RST $10
                self.subroutine_call(0x10);
            }
            0xD8 => {
                // RET C
                self.subroutine_return_if(self.alu.flags.carry());
            }
            0xD9 => {
                // RETI
                self.subroutine_return();
                self.interrupt_enable = true;
            }
            0xDA => {
                // JP C $0000
                self.jump_absolute_if(immediate16, self.alu.flags.carry());
            }
            0xDB => {
                // [DB] - INVALID
            }
            0xDC => {
                // CALL C $0000
                self.subroutine_call_if(immediate16, self.alu.flags.carry())
            }
            0xDD => {
                // [DD] - INVALID
            }
            0xDE => {
                // SBC A, $00
                self.regs.set_a(self.alu.sbc(self.regs.a(), immediate8));
            }
            0xDF => {
                // RST $18
                self.subroutine_call(0x18);
            }
            0xE0 => {
                // LDH ($00), A
                let addr: u16 = 0xff00u16 | immediate8 as u16;
                let data = self.regs.a();
                self.write(addr, data);
            }
            0xE1 => {
                // POP HL
                let hl = self.stack_pop();
                self.regs.set_hl(hl);
            }
            0xE2 => {
                // LDH (C), A
                let addr = 0xff00u16 | self.regs.c() as u16;
                let data = self.regs.a();
                self.write(addr, data);
            }
            0xE3 => {
                // [E3] - INVALID
            }
            0xE4 => {
                // [E4] - INVALID
            }
            0xE5 => {
                // PUSH HL
                self.stack_push(self.regs.hl());
            }
            0xE6 => {
                // AND $00
                self.regs.set_a(self.alu.and(self.regs.a(), immediate8));
            }
            0xE7 => {
                // RST $20
                self.subroutine_call(0x20);
            }
            0xE8 => {
                // ADD SP, $00
                self.regs.set_sp(self.alu.add16(self.regs.sp(), ((immediate8 as i8) as i16) as u16))
            }
            0xE9 => {
                // JP HL
                self.jump_absolute(self.regs.hl());
            }
            0xEA => {
                // LD ($0000), A
                self.write(immediate16, self.regs.a());
            }
            0xEB => {
                // [EB] - INVALID
            }
            0xEC => {
                // [EC] - INVALID
            }
            0xED => {
                // [ED] - INVALID
            }
            0xEE => {
                // XOR $00
                self.regs.set_a(self.alu.xor(self.regs.a(), immediate8));
            }
            0xEF => {
                // RST $28
                self.subroutine_call(0x28);
            }
            0xF0 => {
                // LDH A, ($00)
                let addr: u16 = 0xff00u16 | immediate8 as u16;
                let data = self.read(addr);
                self.regs.set_a(data);
            }
            0xF1 => {
                // POP AF
                let af = self.stack_pop();
                self.regs.set_af(af);
                self.alu.flags = self.regs.f().into();
            }
            0xF2 => {
                // LD A, ($FF00+C)
                let addr = 0xff00u16 | self.regs.c() as u16;
                let data = self.read(addr);
                self.regs.set_a(data);
            }
            0xF3 => {
                // DI
                self.next_interrupt_enable = false;
            }
            0xF4 => {
                // [F4] - INVALID
            }
            0xF5 => {
                // PUSH AF
                self.regs.set_f(self.alu.flags.into());
                self.stack_push(self.regs.af());
            }
            0xF6 => {
                // OR $00
                self.regs.set_a(self.alu.or(self.regs.a(), immediate8));
            }
            0xF7 => {
                // RST $30
                self.subroutine_call(0x30);
            }
            0xF8 => {
                // LD HL,SP+$00
                self.regs.set_hl(self.regs.sp().wrapping_add((immediate8 as i8) as u16));
            }
            0xF9 => {
                // LD SP, HL
                self.regs.set_sp(self.regs.hl());
            }
            0xFA => {
                // LD A, ($0000)
                let data = self.read(immediate16);
                self.regs.set_a(data);
            }
            0xFB => {
                // EI
                self.next_interrupt_enable = true;
            }
            0xFC => {
                // [FC] - INVALID;
            }
            0xFD => {
                // [FD] - INVALID;
            }
            0xFE => {
                // CP $00
                self.alu.compare(self.regs.a(), immediate8);
            }
            0xFF => {
                // RST $38
                self.subroutine_call(0x38);
            }
        }

        self.regs.set_pc(self.next_pc);
        instruction_ticks(opcode)
    }

    fn jump_absolute(&mut self, target: u16) {
        self.next_pc = target;
    }

    fn jump_absolute_if(&mut self, target: u16, cond: bool) {
        if cond {
            self.next_pc = target;
        }
    }

    fn jump_relative(&mut self, offset: u8) {
        self.next_pc = self.next_pc.wrapping_add((offset as i8) as u16)
    }

    fn jump_relative_if(&mut self, offset: u8, cond: bool) {
        if cond {
            self.next_pc = self.next_pc.wrapping_add((offset as i8) as u16)
        }
    }

    fn subroutine_call(&mut self, target: u16) {
        self.stack_push(self.next_pc);
        self.next_pc = target;
    }

    fn subroutine_call_if(&mut self, target: u16, cond: bool) {
        if cond {
            self.stack_push(self.next_pc);
            self.next_pc = target;
        }
    }

    fn subroutine_return(&mut self) {
        self.next_pc = self.stack_pop();
    }

    fn subroutine_return_if(&mut self, cond: bool) {
        if cond {
            self.next_pc = self.stack_pop();
        }
    }

    fn stack_push(&mut self, data: u16) {
        let be_bytes = data.to_be_bytes();

        let stack_pointer = self.regs.sp();

        self.write(stack_pointer, be_bytes[0]);
        self.write(stack_pointer - 1, be_bytes[1]);

        self.regs.set_sp(stack_pointer - 2);
    }

    fn stack_pop(&mut self) -> u16 {
        let stack_pointer = self.regs.sp();

        let lsb = self.read(stack_pointer + 1);
        let msb = self.read(stack_pointer + 2);

        self.regs.set_sp(stack_pointer + 2);

        u16::from_be_bytes([msb, lsb])
    }
}
