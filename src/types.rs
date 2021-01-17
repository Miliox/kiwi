use std::cell::RefCell;
use std::rc::Rc;
use sdl2::pixels::Color;

pub const TICKS_PER_SECOND: u64 = 4_194_304;
pub const TICKS_PER_FRAME: u64 = TICKS_PER_SECOND / 60;

pub const SCREEN_PIXEL_WIDTH:  usize = 160;
pub const SCREEN_PIXEL_HEIGHT: usize = 144;
pub const SCREEN_PIXEL_SIZE: usize  = SCREEN_PIXEL_HEIGHT * SCREEN_PIXEL_WIDTH;

pub const ARGB_BYTES_PER_PIXEL: usize = 4;
pub const SCREEN_BUFFER_SIZE: usize = SCREEN_PIXEL_SIZE * ARGB_BYTES_PER_PIXEL;
pub const SCREEN_BUFFER_WIDTH: usize = SCREEN_PIXEL_WIDTH * ARGB_BYTES_PER_PIXEL;

pub const SHADE_0: Color = Color::RGB(0x9B, 0xBC, 0x0F); // Light
pub const SHADE_1: Color = Color::RGB(0x8B, 0xAC, 0x0F); // Light Gray
pub const SHADE_2: Color = Color::RGB(0x30, 0x62, 0x30); // Dark Gray
pub const SHADE_3: Color = Color::RGB(0x0F, 0x38, 0x0F); // Dark
pub const SHADE: [Color; 4] = [SHADE_0, SHADE_1, SHADE_2, SHADE_3];

pub type MutRc<T> = Rc<RefCell<T>>;

// This macro creates a new mutable rc reference
#[macro_export]
macro_rules! create_mut_rc {
    ($data:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new($data))
    };
}
