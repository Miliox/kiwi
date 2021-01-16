use crate::cart::Cartridge;
use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::joypad::JoypadKeys;
use crate::timer::Timer;
use crate::ticks::TickConsumer;
use crate::types::MutRc;
use crate::create_mut_rc;

#[allow(dead_code)]
pub struct Emulator {
    cpu: MutRc<Cpu>,
    mmu: MutRc<Mmu>,
    cart: MutRc<Cartridge>,
    timer: MutRc<Timer>,
    clock: u64,
}

#[allow(dead_code)]
impl Emulator {
    pub fn new() -> Self {
        let cpu = create_mut_rc!(Cpu::default());

        let cart = create_mut_rc!(Cartridge::default());

        let timer = create_mut_rc!(Timer::default());

        let mmu  = create_mut_rc!(
            Mmu::new(cart.borrow().rom.clone(),
                     cart.borrow().ram.clone(),
                     cpu.clone(),
                     timer.clone()));

        cpu.borrow_mut().mmu = Some(mmu.clone());

        Emulator {
            cpu: cpu,
            mmu: mmu,
            cart: cart,
            timer: timer,
            clock: 0,
        }
    }

    pub fn open(&mut self, gb_rom_filename: &str) {
        self.cart.borrow_mut().open(gb_rom_filename);
    }

    pub fn frame(&mut self) {

    }

    pub fn step(&mut self) {
        let ticks = self.cpu.borrow_mut().cycle();
        self.timer.borrow_mut().sync(ticks);
        self.clock += ticks;
    }

    pub fn print_cartridge_info(&self) {
        let cart = self.cart.borrow();

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

    pub fn press_joypad_key(&mut self, keys: JoypadKeys) {
        self.mmu.borrow_mut().press_joypad_key(keys);
    } 

    pub fn release_joypad_key(&mut self, keys: JoypadKeys) {
        self.mmu.borrow_mut().release_joypad_key(keys);
    } 
}

impl Default for Emulator {
    fn default() -> Self { Self::new() }
}
