use core::panic;
use std::env;

mod chip_8;
use chip_8::Chip8;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("No command line argument present");
    }

    let mut chip8 = Chip8::initialize();
    chip8.load_fontset();
    chip8.load_game(&args[1]);

    chip8.cycle();
}
