#[macro_use]
extern crate bitflags;
extern crate sdl2;

mod types;
mod cpu;

mod bios;
mod cartridge;
mod gpu;
mod joypad;
mod mmu;
mod timer;
mod serial;
mod emulator;

use emulator::Emulator;
use std::time::{Instant, Duration};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Kiwi", 4*160, 4*144)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut emulator = Emulator::default();
    emulator.open("/Users/emiliano/Downloads/Tetris/Tetris.gb");
    emulator.print_cartridge_info();

    let mut frame_begin_timestamp = Instant::now();
    let mut frame_overslept_duration = Duration::from_nanos(0);

    'gameloop: loop {
        for event in event_pump.poll_iter() {
            emulator.process_event(&event);
            match event {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } | Event::Quit {..} => break 'gameloop,
                _ => {}
            }
        }

        emulator.frame();
        let frame_complete_timestamp = Instant::now();
        let frame_busy_duration = frame_complete_timestamp - frame_begin_timestamp;

        match FRAME_DURATION.checked_sub(frame_busy_duration + frame_overslept_duration) {
            Some(frame_wait_duration) => {
                std::thread::sleep(frame_wait_duration);
                frame_begin_timestamp = Instant::now();
                frame_overslept_duration = (frame_begin_timestamp - frame_complete_timestamp) - frame_wait_duration;
            }
            None => {
                frame_begin_timestamp = frame_complete_timestamp;
                frame_overslept_duration = Duration::from_nanos(0);
            }
        }
    }
}
