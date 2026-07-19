mod cpu;
extern crate sdl2;

use cpu::Cpu;
use std::{env, thread, time::Duration, time::Instant};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

// Stuff to add:
//  Accumulators for cycle accuracy
//  Load ROMS from GUI
// CTRL + F for setting cpu frequency
// CTRL + L for load rom
// 

const INTEGER_SCALE: u32 = 8;


// Keycode match
fn map_key(key: Keycode) -> Option<usize> {
    match key {
        Keycode::X => Some(0x0),
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),

        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::A => Some(0x7),

        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::Z => Some(0xA),
        Keycode::C => Some(0xB),

        Keycode::Num4 => Some(0xC),
        Keycode::R => Some(0xD),
        Keycode::F => Some(0xE),
        Keycode::V => Some(0xF),

        _ => None,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: chipd-rust <rom_path>");
        return;
    }

    let rom_path = &args[1];

    let mut cpu = Cpu::new();

    let cpu_hz = 500.0;
    let cpu_dt = Duration::from_secs_f64(1.0 / cpu_hz);
    let timer_dt = Duration::from_secs_f64(1.0 / 60.0);

    let mut last = Instant::now();
    let mut cpu_accum = Duration::ZERO;
    let mut timer_accum = Duration::ZERO;
    let target_time = Duration::from_micros(1000); // 1 kHz

    cpu.load_rom(rom_path)
        .expect("Failed to load ROM");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Chip'd", 64 * INTEGER_SCALE, 32 * INTEGER_SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        // timing
        let now = Instant::now();
        let elapsed = now - last;
        last = now;

        cpu_accum += elapsed;
        timer_accum += elapsed;

        // get input
        for event in event_pump.poll_iter() {
            match event {
                // Quit key down event
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }

                // Control key down events
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    keymod,
                    repeat: false,
                    ..
                } if keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) => {
                    println!("Reset");
                    cpu.reset();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    keymod,
                    repeat: false,
                    ..
                } if keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) => {
                    cpu.paused = !cpu.paused;
                    println!("Paused = {}", cpu.paused);
                }

                // Chip 8 emulator key down event
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    if let Some(chip8_key) = map_key(key) {
                        cpu.press_key(chip8_key);
                    }
                }

                // Chip 8 emulator key up event
                Event::KeyUp {
                    keycode: Some(key),
                    ..
                } => {
                    if let Some(chip8_key) = map_key(key) {
                        cpu.release_key(chip8_key);
                    }
                }
                
                _ => {}
            }
        }

        //emulate cpu cycle and calculate timing 
        while cpu_accum >= cpu_dt {
            cpu.emu_cycle();
            cpu_accum -= cpu_dt;
        }

        //update timers at 60hz
        while timer_accum >= timer_dt {
            cpu.update_timers();
            timer_accum -= timer_dt;
        }

        // render the screen if there's an update
        if cpu.screen_update {
            draw_screen(&cpu, &mut canvas, INTEGER_SCALE);
            cpu.screen_update = false;
        }

        // sleep for the remaining time
        if elapsed < target_time {
            thread::sleep(target_time - elapsed);
        }
    }
}

fn draw_screen(cpu: &Cpu, canvas: &mut Canvas<Window>, scale: u32) {

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));

    for y in 0..32 {
        for x in 0..64 {
            let pixel = cpu.display[y * 64 + x];

            if pixel {
                canvas.fill_rect(
                    Rect::new(
                        (x as i32) * scale as i32,
                        (y as i32) * scale as i32,
                        scale,
                        scale,
                    )
                ).unwrap();
            }
        }
    }

    canvas.present();
}