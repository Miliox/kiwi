pub mod lcd_control;
pub mod lcd_control_status;
pub mod palette;
pub mod sprite;

use lcd_control::LcdControl;
use lcd_control_status::LcdControlStatus;
use lcd_control_status::LcdControlMode;
use palette::Palette;
use sprite::Sprite;

use crate::types::*;

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
    lcdc_status_interrupt_requested: bool,
    vertical_blank_interrupt_requested: bool,

    back_buffer_index: usize,
    front_buffer_index: usize,
    frame_buffer: [Box<[u8; SCREEN_BUFFER_SIZE]>; 2],

    object_attribute_ram: Box<[Sprite; 40]>,
    video_ram: Box<[u8; 0x2000]>,
}

impl Default for Gpu {
    fn default() -> Self {
        let mut blank_frame: [u8; SCREEN_BUFFER_SIZE] = [0; SCREEN_BUFFER_SIZE];
        for index in 0..blank_frame.len() {
            match index % 4 {
                0 => blank_frame[index] = SHADE_0.a, // A
                1 => blank_frame[index] = SHADE_0.r, // R
                2 => blank_frame[index] = SHADE_0.g, // G
                3 => blank_frame[index] = SHADE_0.b, // B
                _ => panic!(),
            }
        }

        Self {
            lcdc: LcdControl::default(),
            stat: LcdControlStatus::default(),

            scanline: 0,
            scanline_compare: 0,

            scroll_y: 0,
            scroll_x: 0,

            window_x: 0,
            window_y: 0,

            background_palette: Palette::default(),
            object_palette_0: Palette::default(),
            object_palette_1: Palette::default(),

            ticks: 0,
            lcdc_status_interrupt_requested: false,
            vertical_blank_interrupt_requested: false,

            back_buffer_index: 0,
            front_buffer_index: 1,
            frame_buffer: [Box::new(blank_frame), Box::new(blank_frame)],

            object_attribute_ram: Box::new([Sprite::default(); 40]),
            video_ram: Box::new([0; 0x2000]),
        }
    }
}

#[allow(dead_code)]
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
        self.stat.set_scanline_coincidence(self.scanline == self.scanline_compare);
    }

    pub fn mode(&self) -> LcdControlMode {
        self.stat.mode()
    }

    pub fn set_mode(&mut self, mode: LcdControlMode) {
        self.stat.set_mode(mode);

        if mode == LcdControlMode::ScanningOAM && self.stat.contains(LcdControlStatus::MODE_OAM_INTERRUPT_ENABLE) {
            self.lcdc_status_interrupt_requested = true;
        }

        if mode == LcdControlMode::VerticalBlank && self.stat.contains(LcdControlStatus::MODE_V_BLANK_INTERRUPT_ENABLE) {
            self.lcdc_status_interrupt_requested = true;
        }

        if mode == LcdControlMode::HorizontalBlank && self.stat.contains(LcdControlStatus::MODE_H_BLANK_INTERRUPT_ENABLE) {
            self.lcdc_status_interrupt_requested = true;
        }

        self.vertical_blank_interrupt_requested = mode == LcdControlMode::VerticalBlank;
    }

    pub fn scanline(&self) -> u8 {
        self.scanline
    }

    fn increment_scanline(&mut self) {
        self.scanline += 1;
        self.stat.set_scanline_coincidence(self.scanline == self.scanline_compare);

        if self.stat.contains(LcdControlStatus::LINE_Y_COINCIDENCE_INTERRUPT_ENABLE) {
            self.lcdc_status_interrupt_requested = true;
        }
    }

    fn reset_scanline(&mut self) {
        self.scanline = 0;
        self.stat.set_scanline_coincidence(self.scanline == self.scanline_compare);

        if self.stat.contains(LcdControlStatus::LINE_Y_COINCIDENCE_INTERRUPT_ENABLE) {
            self.lcdc_status_interrupt_requested = true;
        }
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

    pub fn lcdc_status_interrupt_requested(&self) -> bool {
        self.lcdc_status_interrupt_requested
    }

    pub fn vertical_blank_interrupt_requested(&self) -> bool {
        self.vertical_blank_interrupt_requested
    }

    pub fn read_video_ram(&self, addr: u16) -> u8 {
        self.video_ram[addr as usize]
    }

    pub fn write_video_ram(&mut self, addr: u16, data: u8) {
        self.video_ram[addr as usize] = data;
    }

    pub fn frame_buffer(&self) -> &[u8; SCREEN_BUFFER_SIZE] {
        &self.frame_buffer[self.front_buffer_index]
    }

    pub fn read_object_attribute_ram(&self, addr: u16) -> u8 {
        let sprite_index = addr as usize / 4;
        let sprite_field = addr % 4;
        match sprite_field {
            0 => self.object_attribute_ram[sprite_index].y(),
            1 => self.object_attribute_ram[sprite_index].x(),
            2 => self.object_attribute_ram[sprite_index].tile(),
            3 => self.object_attribute_ram[sprite_index].flags(),
            _ => panic!()
        }
    }

    pub fn write_object_attribute_ram(&mut self, addr: u16, data: u8) {
        let sprite_index = addr as usize / 4;
        let sprite_field = addr % 4;
        match sprite_field {
            0 => self.object_attribute_ram[sprite_index].set_y(data),
            1 => self.object_attribute_ram[sprite_index].set_x(data),
            2 => self.object_attribute_ram[sprite_index].set_tile(data),
            3 => self.object_attribute_ram[sprite_index].set_flags(data),
            _ => panic!()
        }
    }

    pub fn populate_object_attribute_ram(&mut self, data: &[u8; 160]) {
        for sprite_index in 0..40 {
            let beg = sprite_index * 4;
            self.object_attribute_ram[sprite_index] = [data[beg+0],
                                                       data[beg+1],
                                                       data[beg+2],
                                                       data[beg+3]].into();
        }
    }

    pub fn step(&mut self, ticks: u64) {
        self.ticks += ticks;
        self.lcdc_status_interrupt_requested = false;
        self.vertical_blank_interrupt_requested = false;

        match self.mode() {
            LcdControlMode::HorizontalBlank => {
                if self.ticks >= 204 {
                    self.ticks -= 204;
                    self.increment_scanline();
    
                    if self.scanline >= 143 {
                        self.set_mode(LcdControlMode::VerticalBlank);
                    } else {
                        self.set_mode(LcdControlMode::ScanningOAM);
                    }
    
                }
            }
            LcdControlMode::VerticalBlank => {
                if self.ticks >= 456 {
                    self.ticks -= 456;
                    self.increment_scanline();
    
                    if self.scanline > 153 {
                        self.set_mode(LcdControlMode::ScanningOAM);
                        self.reset_scanline();
                    }
                }
            }
            LcdControlMode::ScanningOAM => {
                if self.ticks >= 80 {
                    self.ticks -= 80;
                    self.set_mode(LcdControlMode::Transfering);
                }
            }
            LcdControlMode::Transfering => {
                if self.ticks >= 172 {
                    self.ticks -= 172;
                    self.set_mode(LcdControlMode::HorizontalBlank);
                    // TODO: Render line

                    // Swap frame buffers
                    let (front, back) = (self.back_buffer_index, self.front_buffer_index);
                    self.front_buffer_index = front;
                    self.back_buffer_index  = back;
                }
            }
        }
    }
}
