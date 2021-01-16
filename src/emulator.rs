use crate::create_mut_rc;
use crate::types::*;

use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::joypad::Joypad;
use crate::timer::Timer;

use sdl2::event::Event;
use sdl2::keyboard::*;

const BUTTON_A: Keycode = Keycode::Space;
const BUTTON_B: Keycode = Keycode::LShift;
const BUTTON_UP: Keycode = Keycode::Up;
const BUTTON_DOWN: Keycode = Keycode::Down;
const BUTTON_LEFT: Keycode = Keycode::Left;
const BUTTON_RIGHT: Keycode = Keycode::Right;
const BUTTON_START: Keycode = Keycode::Return;
const BUTTON_SELECT: Keycode = Keycode::Backspace;

pub struct Emulator {
    cartridge: MutRc<Cartridge>,
    clock: u64,
    cpu: MutRc<Cpu>,
    joypad: MutRc<Joypad>,
    mmu: MutRc<Mmu>,
    timer: MutRc<Timer>,
}

impl Emulator {
    pub fn new() -> Self {
        let cpu = create_mut_rc!(Cpu::default());

        let cartridge = create_mut_rc!(Cartridge::default());

        let joypad = create_mut_rc!(Joypad::default());

        let timer = create_mut_rc!(Timer::default());

        let mmu = create_mut_rc!(Mmu::new(
            cartridge.clone(),
            cpu.clone(),
            joypad.clone(),
            timer.clone(),
        ));

        cpu.borrow_mut().mmu = Some(mmu.clone());

        Emulator {
            clock: 0,
            cartridge: cartridge,
            cpu: cpu,
            joypad: joypad,
            mmu: mmu,
            timer: timer,
        }
    }

    pub fn open(&mut self, gb_rom_filename: &str) {
        self.cartridge.borrow_mut().open(gb_rom_filename);
    }

    pub fn frame(&mut self) {
        loop {
            self.step();
            if self.clock >= TICKS_PER_FRAME {
                self.clock -= TICKS_PER_FRAME;
                break;
            }
        }
    }

    pub fn step(&mut self) {
        let ticks = self.cpu.borrow_mut().step();
        self.timer.borrow_mut().step(ticks);

        if self.timer.borrow().interrupt() {
            self.cpu.borrow_mut().set_timer_overflow_interrupt_triggered();
        }

        self.clock += ticks;
    }

    pub fn process_event(&mut self, event: &sdl2::event::Event) {
        match event {
            Event::KeyDown { keycode: Some(BUTTON_A), ..} => { self.joypad.borrow_mut().press_a(); }
            Event::KeyDown { keycode: Some(BUTTON_B), ..} => { self.joypad.borrow_mut().press_b(); }
            Event::KeyDown { keycode: Some(BUTTON_UP), .. } => { self.joypad.borrow_mut().press_up(); }
            Event::KeyDown { keycode: Some(BUTTON_DOWN), .. } => { self.joypad.borrow_mut().press_down(); }
            Event::KeyDown { keycode: Some(BUTTON_LEFT), .. } => { self.joypad.borrow_mut().press_left(); }
            Event::KeyDown { keycode: Some(BUTTON_RIGHT), .. } => { self.joypad.borrow_mut().press_right(); }
            Event::KeyDown { keycode: Some(BUTTON_START), ..} => { self.joypad.borrow_mut().press_start(); }
            Event::KeyDown { keycode: Some(BUTTON_SELECT), ..} => { self.joypad.borrow_mut().press_select(); }

            Event::KeyUp { keycode: Some(BUTTON_A), ..} => { self.joypad.borrow_mut().release_a(); }
            Event::KeyUp { keycode: Some(BUTTON_B), ..} => { self.joypad.borrow_mut().release_b(); }
            Event::KeyUp { keycode: Some(BUTTON_UP), .. } => { self.joypad.borrow_mut().release_up(); }
            Event::KeyUp { keycode: Some(BUTTON_DOWN), .. } => { self.joypad.borrow_mut().release_down(); }
            Event::KeyUp { keycode: Some(BUTTON_LEFT), .. } => { self.joypad.borrow_mut().release_left(); }
            Event::KeyUp { keycode: Some(BUTTON_RIGHT), .. } => { self.joypad.borrow_mut().release_right(); }
            Event::KeyUp { keycode: Some(BUTTON_START), ..} => { self.joypad.borrow_mut().release_start(); }
            Event::KeyUp { keycode: Some(BUTTON_SELECT), ..} => { self.joypad.borrow_mut().release_select(); }

            _ => { }
        }
    }

    pub fn print_cartridge_info(&self) {
        let mmu = self.mmu.borrow();
        let cart = mmu.cartridge.borrow();

        println!("Title: {}", cart.title());
        println!("Color?: {}", cart.is_color());
        println!("Super?: {}", cart.is_super());
        println!("Japanese?: {}", cart.is_japanese());
        println!("Checksum: {:02x}", cart.checksum());
        println!("Complement: {:02x}", cart.complement_check());
        println!("Type Code: {:02x}", cart.cart_type());
        println!("Lincense Code: {:02x?}", cart.lincense_code());
        println!("ROM Size Code: {:02x}", cart.rom_size_code());
        println!("RAM Size Code: {:02x}", cart.ram_size_code());
        println!("START Point: {:02x?}", cart.entry_point());

        for rst in (0x00u16..=0x60u16).step_by(8) {
            println!("RESET {:02x?}: {:02x?}", rst, cart.rom_slice(rst, 8u16));
        }
    }
}

impl Default for Emulator {
    fn default() -> Self { Self::new() }
}
