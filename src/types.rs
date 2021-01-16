use std::cell::RefCell;
use std::rc::Rc;

pub const TICKS_PER_SECOND: u64 = 4_194_304;
pub const TICKS_PER_FRAME: u64 = TICKS_PER_SECOND / 60;

pub type MutRc<T> = Rc<RefCell<T>>;

pub trait Memory {
    // Read a single byte from memory
    fn read(&self, addr: u16) -> u8;

    // Write a single byte to memory
    fn write(&mut self, addr: u16, data: u8);
}

pub trait TickProducer {
    fn step(&mut self) -> u64;
}

pub trait TickConsumer {
    fn step(&mut self, ticks: u64);
}

// This macro creates a new mutable rc reference
#[macro_export]
macro_rules! create_mut_rc {
    ($data:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new($data))
    };
}