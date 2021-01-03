#[macro_use]
extern crate bitflags;

mod flags;
mod alu8;
mod alu16;
mod bios;
mod cart;
mod cpu;
mod gpu;
mod mmu;
mod mainboard;

use mainboard::MainBoard;

fn main() {
    let mut board = MainBoard::default();
    board.open("/Users/emiliano/Downloads/Tetris/Tetris.gb");
    board.print_cartridge_info();
    board.step();
    board.step();
    board.step();
    board.step();
    board.step();
}
