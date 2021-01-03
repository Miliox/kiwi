use crate::cart::Cartridge;
use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::mmu::NullMemory;
use std::cell::RefCell;
use std::rc::Rc;

#[allow(dead_code)]
pub struct MainBoard {
    cpu: Rc<RefCell<Cpu>>,
    mmu: Rc<RefCell<Mmu>>,
    cart: Rc<RefCell<Cartridge>>,
    hwio: Rc<RefCell<NullMemory>>,
    clock: u64,
}

#[allow(dead_code)]
impl MainBoard {
    pub fn new() -> Self {
        let cpu = Rc::new(RefCell::new(Cpu::default()));

        let cart = Rc::new(RefCell::new(Cartridge::default()));

        let hwio = Rc::new(RefCell::new(NullMemory::default()));

        let mmu  = Rc::new(RefCell::new(
            Mmu::new(cart.borrow().rom.clone(),
                     cart.borrow().ram.clone(),
                     hwio.clone())));

        cpu.borrow_mut().mmu = Some(mmu.clone());

        Self {
            cpu: cpu,
            mmu: mmu,
            cart: cart,
            clock: 0,
            hwio: hwio,
        }
    }

    pub fn open(&mut self, gb_rom_filename: &str) {
        self.cart.borrow_mut().open(gb_rom_filename);
    }

    pub fn step(&mut self) {
        self.clock += self.cpu.borrow_mut().cycle();
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
}

impl Default for MainBoard {
    fn default() -> Self { Self::new() }
}
