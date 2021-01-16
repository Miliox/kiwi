#[macro_use]
extern crate bitflags;
extern crate sdl2;

mod types;
mod ticks;

mod cpu;

mod bios;
mod cart;
mod gpu;
mod joypad;
mod mmu;

mod timer;
mod emulator;

use emulator::Emulator;
use joypad::JoypadKeys;
use std::time::{Instant, Duration};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

const BUTTON_A: Keycode = Keycode::Space;
const BUTTON_B: Keycode = Keycode::LShift;
const BUTTON_UP: Keycode = Keycode::Up;
const BUTTON_DOWN: Keycode = Keycode::Down;
const BUTTON_LEFT: Keycode = Keycode::Left;
const BUTTON_RIGHT: Keycode = Keycode::Right;
const BUTTON_START: Keycode = Keycode::Return;
const BUTTON_SELECT: Keycode = Keycode::Backspace;

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
            match event {
                Event::KeyDown { keycode: Some(BUTTON_UP) , .. } => {
                    emulator.press_joypad_key(JoypadKeys::UP);
                }
                Event::KeyUp { keycode: Some(BUTTON_UP) , .. } => {
                    emulator.release_joypad_key(JoypadKeys::UP);
                }
                Event::KeyDown { keycode: Some(BUTTON_DOWN) , .. } => {
                    emulator.press_joypad_key(JoypadKeys::DOWN);
                }
                Event::KeyUp { keycode: Some(BUTTON_DOWN) , .. } => {
                    emulator.release_joypad_key(JoypadKeys::DOWN);
                }
                Event::KeyDown { keycode: Some(BUTTON_LEFT) , .. } => {
                    emulator.press_joypad_key(JoypadKeys::LEFT);
                }
                Event::KeyUp { keycode: Some(BUTTON_LEFT) , .. } => {
                    emulator.release_joypad_key(JoypadKeys::LEFT);
                }
                Event::KeyDown { keycode: Some(BUTTON_RIGHT) , .. } => {
                    emulator.press_joypad_key(JoypadKeys::RIGHT);
                }
                Event::KeyUp { keycode: Some(BUTTON_RIGHT) , .. } => {
                    emulator.release_joypad_key(JoypadKeys::RIGHT);
                }
                Event::KeyDown { keycode: Some(BUTTON_START) , .. } => {
                    emulator.press_joypad_key(JoypadKeys::START);
                }
                Event::KeyUp { keycode: Some(BUTTON_START) , .. } => {
                    emulator.release_joypad_key(JoypadKeys::START);
                }
                Event::KeyDown { keycode: Some(BUTTON_SELECT) , .. } => {
                    emulator.press_joypad_key(JoypadKeys::SELECT);
                }
                Event::KeyUp { keycode: Some(BUTTON_SELECT) , .. } => {
                    emulator.release_joypad_key(JoypadKeys::SELECT);
                }
                Event::KeyDown { keycode: Some(BUTTON_A) , .. } => {
                    emulator.press_joypad_key(JoypadKeys::A);
                }
                Event::KeyUp { keycode: Some(BUTTON_A) , .. } => {
                    emulator.release_joypad_key(JoypadKeys::A);
                }
                Event::KeyDown { keycode: Some(BUTTON_B) , .. } => {
                    emulator.press_joypad_key(JoypadKeys::B);
                }
                Event::KeyUp { keycode: Some(BUTTON_B) , .. } => {
                    emulator.release_joypad_key(JoypadKeys::B);
                }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } | Event::Quit {..} => break 'gameloop,
                _ => {}
            }
        }

        emulator.step();
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
