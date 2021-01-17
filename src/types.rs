use std::cell::RefCell;
use std::rc::Rc;

pub const TICKS_PER_SECOND: u64 = 4_194_304;
pub const TICKS_PER_FRAME: u64 = TICKS_PER_SECOND / 60;

pub const SCREEN_PIXEL_WIDTH:  usize = 160;
pub const SCREEN_PIXEL_HEIGHT: usize = 144;
pub const SCREEN_PIXEL_TOTAL: usize  = SCREEN_PIXEL_HEIGHT * SCREEN_PIXEL_WIDTH;

pub const BITS_PER_PIXEL: usize = 2;
pub const BITS_PER_BYTE:  usize = 8;
pub const SCREEN_BYTES_TOTAL: usize = SCREEN_PIXEL_TOTAL / (BITS_PER_BYTE / BITS_PER_PIXEL);

pub type MutRc<T> = Rc<RefCell<T>>;

// This macro creates a new mutable rc reference
#[macro_export]
macro_rules! create_mut_rc {
    ($data:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new($data))
    };
}
