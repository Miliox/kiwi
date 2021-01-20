pub trait Memory {
    // Read a single byte from memory
    fn read(&self, addr: u16) -> u8;

    // Write a single byte to memory
    fn write(&mut self, addr: u16, data: u8);
}