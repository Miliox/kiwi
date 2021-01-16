use std::cell::RefCell;
use std::rc::Rc;

pub const TICKS_PER_SECOND: u64 = 4_194_304;
pub const TICKS_PER_FRAME: u64 = TICKS_PER_SECOND / 60;

pub type MutRc<T> = Rc<RefCell<T>>;

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
