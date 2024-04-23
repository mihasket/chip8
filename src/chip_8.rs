use std::{fs::File, io::Read};
use rand::Rng;

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

#[derive(Debug)]
#[allow(dead_code)]
pub struct Chip8 {
    opcode: u16,
    memory: [u8; 4096],
    cpu_register_v: [u8; 16],
    carry_register_vf: u8,
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
    pub fn initialize() -> Chip8 {
        Chip8 {
            opcode: 0,
            memory: [0; 4096],
            cpu_register_v: [0; 16],
            carry_register_vf: 0,
            register_index: 0, 
            pc: 512, 
            gfx: [0; 64 * 32], 
            delay_timer: 0,
            sound_timer: 0, 
            stack: Vec::new(),
            key: [0; 16]
        }
    }

    pub fn load_fontset(&mut self) {
        let mut i = 0;
        while i < 80 {
            self.memory[i] = CHIP8_FONTSET[i];
            i += 1;
        }
    }

    pub fn load_game(&mut self, file_name: &str) {
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

    pub fn cycle(&mut self) {
        self.opcode = 
            (self.memory[self.pc as usize] as u16) << 8 |
            self.memory[(self.pc + 1) as usize] as u16;

        self.opcode = 0x8bF6;
        println!("opcode: {:#06x}", self.opcode);

        match self.opcode & 0xF000 {
            // This is for 0x00e0 and 0x000e
            // Figure out a way to write this match more cleanly
            0x0000 => {
                match self.opcode & 0x000F {
                    // 0x00e0
                    0x0000 => {
                        // Clear the screen
                        println!("Clear the screen");
                    },
                    // 0x000e
                    0x000E => {
                        // Return from subroutine
                        println!("Return from subroutine");
                    },
                    _ => {
                        println!("No such opcode: {:#x}", self.opcode);
                    }
                }
            },
            // Set program counter to location NNN
            0x1000 => {
                self.pc = self.opcode & 0x0FFF;
            },
            // Increment stack pointer, put current PC on top of stack. PC is set to NNN
            0x2000 => {
                self.stack.push(self.pc);
                self.pc = self.opcode & 0x0FFF;
            },
            // 3xkk
            // Compares register Vx to kk, if equal => pc += 2
            0x3000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let kk = self.opcode & 0x00FF;
                
                if u16::from(self.cpu_register_v[x]) == kk {
                    self.pc += 2;
                }
            },
            // 4xkk
            // Compares register Vx to kk, if NOT equal => pc += 2
            0x4000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let kk = self.opcode & 0x00FF;

                if u16::from(self.cpu_register_v[x]) != kk {
                    self.pc += 2;
                }
            },
            // 5xy0
            // Compares register Vx with Vy, if equal => pc += 2
            0x5000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let y = (self.opcode & 0x00F0 >> 4) as usize;

                if self.cpu_register_v[x] == self.cpu_register_v[y] {
                    self.pc += 2;
                }
            },
            // 6xkk
            // Sets register Vx to kk
            0x6000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let kk = (self.opcode & 0x00FF) as u8;

                // The cast might not work
                self.cpu_register_v[x] = kk;
                self.pc += 2;
            },
            // 7xkk
            // Sets register Vx = Vx + kk
            0x7000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let kk = (self.opcode & 0x00FF) as u8;

                // The cast might not work
                self.cpu_register_v[x] += kk;
                self.pc += 2;
            },
            // 8xyz
            0x8000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let y = ((self.opcode & 0x00F0) >> 4) as usize;

                match self.opcode & 0x000F {
                    // Set Vx = Vx OR Vy
                    0x0001 => {
                        self.cpu_register_v[x] = self.cpu_register_v[x] | self.cpu_register_v[y];
                        self.pc += 2;
                    },
                    // Set Vx = Vx AND Vy.
                    0x0002 => {
                        self.cpu_register_v[x] = self.cpu_register_v[x] & self.cpu_register_v[y];
                        self.pc += 2;
                    },
                    // Set Vx = Vx XOR Vy.
                    0x0003 => {
                        self.cpu_register_v[x] = self.cpu_register_v[x] ^ self.cpu_register_v[y];
                        self.pc += 2;
                    },
                    // Set Vx = Vx + Vy, set VF = carry.
                    0x0004 => {
                        if self.cpu_register_v[x] + self.cpu_register_v[y] > 0 {
                            self.carry_register_vf = 1;
                        }
                        else {
                            self.carry_register_vf = 0;
                        }

                        self.cpu_register_v[x] = self.cpu_register_v[x] + self.cpu_register_v[y];
                        self.pc += 2;
                    },
                    // Set Vx = Vx - Vy, set VF = NOT borrow.
                    0x0005 => {
                        if self.cpu_register_v[x] > self.cpu_register_v[y] {
                            self.carry_register_vf = 1;
                        }
                        else {
                            self.carry_register_vf = 0;
                        }

                        self.cpu_register_v[x] = self.cpu_register_v[x] - self.cpu_register_v[y];
                        self.pc += 2;
                    },
                    // Set Vx = Vx SHR 1.
                    0x0006 => {
                        // Chip 8 is big endian
                        if self.cpu_register_v[x] & 1 == 1 {
                            self.carry_register_vf = 1;
                        }
                        else {
                            self.carry_register_vf = 0;
                        }

                        self.cpu_register_v[x] >>= 1;
                        self.pc += 2;
                    },
                    // Set Vx = Vy - Vx, set VF = NOT borrow.
                    0x0007 => {
                        if self.cpu_register_v[y] > self.cpu_register_v[x] {
                            self.carry_register_vf = 1;
                        }
                        else {
                            self.carry_register_vf = 0;
                        }

                        self.cpu_register_v[x] = self.cpu_register_v[y] - self.cpu_register_v[x];
                        self.pc += 2;
                    },
                    // Set Vx = Vx SHL 1.
                    0x000E => {
                        // Chip 8 is big endian
                        if self.cpu_register_v[x] & 0b10000000 == 1 {
                            self.carry_register_vf = 1;
                        }
                        else {
                            self.carry_register_vf = 0;
                        }

                        self.cpu_register_v[x] <<= 1;
                        self.pc += 2;
                    },
                    _ => {
                        todo!()
                    }
                }
            },
            // 9xy0 - SNE Vx, Vy
            0x9000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let y = ((self.opcode & 0x00F0) >> 4) as usize;

                if self.cpu_register_v[x] != self.cpu_register_v[y] {
                    self.pc += 2;
                }
            }
            // ANNN: Register Index = NNN
            0xA000 => {
                self.register_index = self.opcode & 0x0FFF;
                self.pc += 2;
            },
            // BNNN: Jump to location nnn + V0.
            0xB000 => {
                self.pc = (self.opcode & 0x0FFF) + (self.cpu_register_v[0] as u16);
            },
            // Cxkk: Set Vx = random byte & kk.
            0xC000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let kk = self.opcode & 0x00FF;
                let random_number = rand::thread_rng().gen_range(0..=255);
                self.cpu_register_v[x] = (random_number & kk) as u8;
            },
            _ => {
                println!("No such opcode: {:#x}", self.opcode);
            }
        }

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }

            self.sound_timer -= 1;
        }
    }
}
