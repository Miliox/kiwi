pub mod lcd_control;
pub mod lcd_control_status;
pub mod palette;

use crate::types::*;
use lcd_control::LcdControl;
use lcd_control_status::LcdControlStatus;
use lcd_control_status::LcdControlMode;
use palette::Palette;

#[derive(Default)]
pub struct Gpu {
    lcdc: LcdControl,
    stat: LcdControlStatus,

    scanline: u8,
    scanline_compare: u8,

    scroll_y: u8,
    scroll_x: u8,

    window_y: u8,
    window_x: u8,

    background_palette: Palette,

    object_palette_0: Palette,
    object_palette_1: Palette,

    ticks: u64,
}

impl Gpu {
    pub fn lcdc(&self) -> u8 {
        self.lcdc.into()
    }

    pub fn set_lcdc(&mut self, lcdc: u8) {
        self.lcdc = LcdControl::from(lcdc);
    }

    pub fn stat(&self) -> u8 {
        self.stat.into()
    }

    pub fn set_stat(&mut self, stat: u8) {
        self.stat = (LcdControlStatus::from(stat) & !LcdControlStatus::READ_ONLY_MASK)  | (self.stat & LcdControlStatus::READ_ONLY_MASK);
    }

    pub fn scanline(&self) -> u8 {
        self.scanline
    }

    fn increment_scanline(&mut self) {
        self.scanline += 1;
        self.stat.set_scanline_coincidence(self.scanline == self.scanline_compare);
    }

    fn reset_scanline(&mut self) {
        self.scanline = 0;
        self.stat.set_scanline_coincidence(self.scanline == self.scanline_compare);
    }

    pub fn scanline_compare(&self) -> u8 {
        self.scanline_compare
    }

    pub fn set_scanline_compare(&mut self, lyc: u8) {
        self.scanline_compare = lyc;
    }

    pub fn background_palette(&self) -> u8 {
        self.background_palette.into()
    }

    pub fn set_background_palette(&mut self, bgp: u8) {
        self.background_palette = bgp.into();
    }

    pub fn object_palette_0(&self) -> u8 {
        self.object_palette_0.into()
    }

    pub fn set_object_palette_0(&mut self, obp0: u8) {
        self.object_palette_0 = obp0.into();
    }

    pub fn object_palette_1(&self) -> u8 {
        self.object_palette_1.into()
    }

    pub fn set_object_palette_1(&mut self, obp1: u8) {
        self.object_palette_1 = obp1.into();
    }

    pub fn scroll_x(&self) -> u8 {
        self.scroll_x
    }

    pub fn set_scroll_x(&mut self, scroll_x: u8) {
        self.scroll_x = scroll_x;
    }

    pub fn scroll_y(&self) -> u8 {
        self.scroll_y
    }

    pub fn set_scroll_y(&mut self, scroll_y: u8) {
        self.scroll_y = scroll_y;
    }

    pub fn window_x(&self) -> u8 {
        self.window_x
    }

    pub fn set_window_x(&mut self, window_x: u8) {
        self.window_x = window_x;
    }

    pub fn window_y(&self) -> u8 {
        self.window_y
    }

    pub fn set_window_y(&mut self, window_y: u8) {
        self.window_y = window_y;
    }
}

impl TickConsumer for Gpu {
    fn step(&mut self, ticks: u64) {
        self.ticks += ticks;

        match self.stat.mode() {
            LcdControlMode::HorizontalBlank => {
                if self.ticks >= 204 {
                    self.ticks -= 204;
                    self.increment_scanline();
    
                    if self.scanline >= 143 {
                        self.stat.set_mode(LcdControlMode::VerticalBlank);
                    } else {
                        self.stat.set_mode(LcdControlMode::ScanningOAM);
                    }
    
                }
            }
            LcdControlMode::VerticalBlank => {
                if self.ticks >= 456 {
                    self.ticks -= 456;
                    self.increment_scanline();
    
                    if self.scanline > 153 {
                        self.stat.set_mode(LcdControlMode::ScanningOAM);
                        self.reset_scanline();
                    }
                }
            }
            LcdControlMode::ScanningOAM => {
                if self.ticks >= 80 {
                    self.ticks -= 80;
                    self.stat.set_mode(LcdControlMode::Transfering);
                }
            }
            LcdControlMode::Transfering => {
                if self.ticks >= 172 {
                    self.ticks -= 172;
                    self.stat.set_mode(LcdControlMode::HorizontalBlank);
                    // TODO: Render line
                }
            }
        }
    }
}