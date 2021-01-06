#[macro_use]
extern crate bitflags;

mod ticks;

mod cpu;

mod bios;
mod cart;
mod gpu;
mod mmu;

mod timer;
mod mainboard;

use mainboard::MainBoard;

fn main() {
    let mut board = MainBoard::default();
    board.open("/Users/emiliano/Downloads/Tetris/Tetris.gb");
    board.print_cartridge_info();
    for _ in 0..100 {
        board.step();
    }
}
