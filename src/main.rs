use std::{fs::File, io::Read};

const CHIP8_FONTSET: [u8; 80] =
[
  0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
  0x20, 0x60, 0x20, 0x20, 0x70, // 1
  0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
  0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
  0x90, 0x90, 0xF0, 0x10, 0x10, // 4
  0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
  0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
  0xF0, 0x10, 0x20, 0x40, 0x40, // 7
  0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
  0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
  0xF0, 0x90, 0xF0, 0x90, 0x90, // A
  0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
  0xF0, 0x80, 0x80, 0x80, 0xF0, // C
  0xE0, 0x90, 0x90, 0x90, 0xE0, // D
  0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
  0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

struct Chip8 {
    opcode: u16,
    memory: [u8; 4096],
    cpu_register_v: [u8; 15],
    register_index: u16,
    pc: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    // Stack includes stack pointer
    stack: Vec<u16>,
    key: [u8; 16],
}

impl Chip8 {
    fn initialize() -> Chip8 {
        Chip8 {
            opcode: 0,
            memory: [0; 4096],
            cpu_register_v: [0; 15],
            register_index: 0, 
            pc: 512, 
            gfx: [0; 64 * 32], 
            delay_timer: 0,
            sound_timer: 0, 
            stack: Vec::new(),
            key: [0; 16]
        }
    }

    fn load_fontset(&mut self) {
        let mut i = 0;
        while i < 80 {
            self.memory[i] = CHIP8_FONTSET[i];
            i += 1;
        }
    }

    fn load_game(&mut self, file_name: &str) {
        let file_result = File::open(file_name);

        let mut file = match file_result {
            Ok(file) => file,
            Err(error) => panic!("Error opening file: {:?}", error)
        };

        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer);

        let mut i = 0;
        for byte in &buffer {
            // Start reading to memory at position 0x200 which is 512
            self.memory[i + 512] = *byte;
            i += 1;
        }
    }
}

fn main() {
    println!("Hello, world!");
}
