mod chip8;

use chip8::{decompiler, emulator};


use clap::{App, Arg};

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
        println!("{}", decompiler::decompile_rom(input));
    } else {
        emulator::Chip8::new(input).run();
    }
}
