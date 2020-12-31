#[macro_use]
extern crate bitflags;

mod alu;
mod bios;
mod board;
mod cart;
mod cpu;
mod gpu;
mod mmu;

fn main() {
    let mut cartridge = cart::Cartridge::default();
    cartridge.open("/Users/emiliano/Downloads/Tetris/Tetris.gb");

    println!("Title: {}", cartridge.title());
    println!("Color?: {}", cartridge.is_color());
    println!("Super?: {}", cartridge.is_super());
    println!("Japanese?: {}", cartridge.is_japanese());
    println!("Checksum: {:02x}", cartridge.checksum());
    println!("Complement: {:02x}", cartridge.complement_check());
    println!("Type Code: {:02x}", cartridge.cart_type());
    println!("Lincense Code: {:02x?}", cartridge.lincense_code());
    println!("ROM Size Code: {:02x}", cartridge.rom_size_code());
    println!("RAM Size Code: {:02x}", cartridge.ram_size_code());
    println!("START Point: {:02x?}", cartridge.entry_point());

    for rst in (0x00u16..=0x60u16).step_by(8) {
        println!("RESET {:02x?}: {:02x?}", rst, cartridge.rom_slice(rst, 8u16));
    }
}
