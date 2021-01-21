mod bios;
mod cartridge;
pub mod cpu;
pub mod engine;
pub mod ppu;
pub mod joypad;
pub mod mmu;
pub mod serial;
pub mod sound;
pub mod timer;

use engine::Engine;

use sdl2::render::Texture;

pub struct Emulator {
    clock: u64,
    engine: Box<Engine>,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            clock: 0,
            engine: Box::new(Engine::default()),
        }
    }

    pub fn blit_frame_to_texture(&mut self, texture: &mut Texture) {
        self.engine.blit_frame_to_texture(texture);
    }

    pub fn open_rom_file(&mut self, filename: &str) {
        self.engine.open_rom_file(filename);
    }

    pub fn process_event(&mut self, event: &sdl2::event::Event) {
        self.engine.process_event(event);
    }

    pub fn run_next_frame(&mut self) {
        self.clock = self.engine.run_next_frame(self.clock);
    }
}

impl Default for Emulator {
    fn default() -> Self { Self::new() }
}
