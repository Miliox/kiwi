pub trait Mmu {
    fn read_byte(&self, addr: u16) -> u8;

    fn write_byte(&mut self, addr: u16, data: u8) -> bool;

    fn read_word(&self, addr: u16) -> u16;

    fn write_word(&mut self, addr: u16, data: u16) -> bool;
}