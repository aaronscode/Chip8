mod chip8;

use chip8::constants::{CELL_H, CELL_W, CHIP8_DISP_H, CHIP8_DISP_W, FPS};
use chip8::{decompiler, emulator};

use std::convert::TryInto;

use clap::{App, Arg};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

pub fn main() {
    let matches = App::new("Chip-8 Emulator")
        .about("A Chip-8 Emulator written in Rust")
        .arg(
            Arg::with_name("INPUT")
                .help("The input file. Either a ROM or source file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("compile")
                .help("Compile a ROM from an assembly source")
                .short("c"),
        )
        .arg(
            Arg::with_name("decompile")
                .conflicts_with("compile")
                .help("Decompile a ROM to assembly source")
                .short("d"),
        )
        .get_matches();
    let input = matches.value_of("INPUT").unwrap().to_string();
    if matches.is_present("compile") {
        println!("Compile flag");
        // TODO: call compilation method
    } else if matches.is_present("decompile") {
        println!("decompile flag");
        // TODO: Call decomp method
    } else {
        // TODO: refactor emulator
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "Chip-8 Emulator",
            CELL_W * (CHIP8_DISP_W),
            CELL_H * (CHIP8_DISP_H),
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut chip8 = emulator::Chip8::new(input);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        println!("Top of loop");
        chip8.tick();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        for x in 0..CHIP8_DISP_W {
            for y in 0..CHIP8_DISP_H {
                //println!("{}", vram[y as usize]);
                if chip8.get_vram_bit(x as usize, y as usize) {
                    canvas
                        .fill_rect(Rect::new(
                            (x * CELL_W).try_into().unwrap(),
                            (y * CELL_H).try_into().unwrap(),
                            CELL_W,
                            CELL_H,
                        ))
                        .unwrap();
                }
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => chip8.keydown(0b0000_0000_0000_0010),
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => chip8.keydown(0b0000_0000_0000_0100),
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => chip8.keydown(0b0000_0000_0000_1000),
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => chip8.keydown(0b0001_0000_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => chip8.keydown(0b0000_0000_0001_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => chip8.keydown(0b0000_0000_0010_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => chip8.keydown(0b0000_0000_0100_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => chip8.keydown(0b0010_0000_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => chip8.keydown(0b0000_0000_1000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => chip8.keydown(0b0000_0001_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => chip8.keydown(0b0000_0010_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => chip8.keydown(0b0100_0000_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => chip8.keydown(0b0000_0100_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => chip8.keydown(0b0000_0000_0000_0001),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => chip8.keydown(0b0000_1000_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    ..
                } => chip8.keydown(0b1000_0000_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::Num1),
                    ..
                } => chip8.keyup(0b0000_0000_0000_0010),
                Event::KeyUp {
                    keycode: Some(Keycode::Num2),
                    ..
                } => chip8.keyup(0b0000_0000_0000_0100),
                Event::KeyUp {
                    keycode: Some(Keycode::Num3),
                    ..
                } => chip8.keyup(0b0000_0000_0000_1100),
                Event::KeyUp {
                    keycode: Some(Keycode::Num4),
                    ..
                } => chip8.keyup(0b0001_0000_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::Q),
                    ..
                } => chip8.keyup(0b0000_0000_0001_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => chip8.keyup(0b0000_0000_0010_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::E),
                    ..
                } => chip8.keyup(0b0000_0000_0100_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::R),
                    ..
                } => chip8.keyup(0b0010_0000_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => chip8.keyup(0b0000_0000_1000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => chip8.keyup(0b0000_0001_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => chip8.keyup(0b0000_0010_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::F),
                    ..
                } => chip8.keyup(0b0100_0000_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::Z),
                    ..
                } => chip8.keyup(0b0000_0100_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::X),
                    ..
                } => chip8.keyup(0b0000_0000_0000_0001),
                Event::KeyUp {
                    keycode: Some(Keycode::C),
                    ..
                } => chip8.keyup(0b0000_1000_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::V),
                    ..
                } => chip8.keyup(0b1000_0000_0000_0000),
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
    }
}
