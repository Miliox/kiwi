#[macro_use]
extern crate bitflags;
extern crate sdl2;

mod emulator;

use emulator::Emulator;
use emulator::ppu::SCREEN_PIXEL_WIDTH;
use emulator::ppu::SCREEN_PIXEL_HEIGHT;
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioQueue};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;
use std::time::{Instant, Duration};

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

struct SquareWaveChannel {
    amplitude: i8,
    phase_inc: f32,
    phase: f32,
}

impl AudioCallback for SquareWaveChannel {
    type Channel = i8;

    fn callback(&mut self, samples: &mut [i8]) {
        let length = samples.len() / 2;
        for i in 0..length {
            let sample = self.amplitude * if self.phase < 0.5 { 1 } else { -1 };

            self.phase += self.phase_inc;
            self.phase %= 1.0;

            samples[i * 2]     = sample; // left
            samples[i * 2 + 1] = sample; // right
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(2),
        samples: Some(2048),
    };

    /*
    let channel1 = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        let freq = 100.0;
        let phase_inc = freq / spec.freq as f32;
        println!("SquareWaveChannel {} {}", freq, phase_inc);
        SquareWaveChannel {
            amplitude: 16,
            phase_inc: freq / spec.freq as f32,
            phase: 0.0,
        }
    }).unwrap();
    channel1.resume();
    */

    let mut channel1: AudioQueue<i8> = audio_subsystem.open_queue(None, &desired_spec).unwrap();
    let mut channel1_wave = SquareWaveChannel {
        amplitude: 16,
        phase_inc: 100.0 / channel1.spec().freq as f32,
        phase: 0.0,
    };
    let mut buffer: [i8; 8192] = [0; 8192];
    channel1.resume();

    let video_subsystem = sdl_context.video().unwrap();

    let scale = 4;
    let width = (SCREEN_PIXEL_WIDTH * scale) as u32;
    let height = (SCREEN_PIXEL_HEIGHT * scale) as u32;

    let window = video_subsystem.window("Kiwi", width, height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture(
        Some(PixelFormatEnum::ARGB32),
        TextureAccess::Static,
        SCREEN_PIXEL_WIDTH as u32,
        SCREEN_PIXEL_HEIGHT as u32).unwrap();

    let mut emulator = Emulator::default();
    emulator.open_rom_file("/Users/emiliano/Downloads/Tetris/Tetris.gb");

    let mut frame_begin_timestamp = Instant::now();
    let mut frame_overslept_duration = Duration::from_nanos(0);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut frame_counter: u64 = 0;

    'gameloop: loop {
        for event in event_pump.poll_iter() {
            emulator.process_event(&event);
            match event {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } | Event::Quit {..} => break 'gameloop,
                _ => {}
            }
        }

        emulator.run_next_frame();
        emulator.blit_frame_to_texture(&mut texture);
        frame_counter += 1;

        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        if channel1.size() < buffer.len() as u32 {
            channel1_wave.callback(&mut buffer);
            channel1.queue(&buffer);
        }

        let frame_complete_timestamp = Instant::now();
        let frame_busy_duration = frame_complete_timestamp - frame_begin_timestamp;

        match FRAME_DURATION.checked_sub(frame_busy_duration + frame_overslept_duration) {
            Some(frame_wait_duration) => {
                std::thread::sleep(frame_wait_duration);
                frame_begin_timestamp = Instant::now();
                frame_overslept_duration = (frame_begin_timestamp - frame_complete_timestamp) - frame_wait_duration;
            }
            None => {
                println!("Frame overrun {:?} {:?} {:?}", frame_counter, frame_busy_duration, frame_overslept_duration);
                frame_begin_timestamp = frame_complete_timestamp;
                frame_overslept_duration = Duration::from_nanos(0);
            }
        }
    }
}
