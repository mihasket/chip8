use std::{fs::File, io::Read, usize};
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

pub const GAME_WIDTH: usize = 64;
pub const GAME_HEIGHT: usize = 32;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Chip8 {
    opcode: u16,
    memory: [u8; 4096],
    cpu_register_v: [u8; 16],
    screen: [bool; GAME_WIDTH * GAME_HEIGHT],
    register_index: u16,
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack_pointer: u16,
    stack: [u16; 16],
    keys: [bool; 16],
}

impl Chip8 {
    pub fn initialize() -> Chip8 {
        Chip8 {
            opcode: 0,
            memory: [0; 4096],
            screen: [false; GAME_WIDTH * GAME_HEIGHT],
            cpu_register_v: [0; 16],
            register_index: 0, 
            pc: 512, 
            delay_timer: 0,
            sound_timer: 0, 
            stack_pointer: 0,
            stack: [0; 16],
            keys: [false; 16]
        }
    }

    fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    pub fn load_fontset(&mut self) {
        let mut i = 0;
        while i < 80 {
            self.memory[i] = CHIP8_FONTSET[i];
            i += 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, i: usize, pressed: bool) {
        self.keys[i] = pressed;
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

    pub fn cycle_timers(&mut self) {
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

    pub fn cycle(&mut self) {
        self.opcode = 
            (self.memory[self.pc as usize] as u16) << 8 |
            self.memory[(self.pc + 1) as usize] as u16;

        self.pc += 2;
        println!("opcode: {:#06x}", self.opcode);

        match self.opcode & 0xF000 {
            // This is for 0x00e0 and 0x000e
            // Figure out a way to write this match more cleanly
            0x0000 => {
                match self.opcode & 0x000F {
                    // 0x00e0
                    0x0000 => {
                        // Clear the screen
                        self.screen = [false; GAME_WIDTH * GAME_HEIGHT];
                    },
                    // 0x000e
                    0x000E => {
                        // Return from subroutine
                        // Subroutine is the same as jump, but expects to return
                        let return_address = self.pop();
                        self.pc = return_address;
                    },
                    _ => {
                        println!("No such opcode: {:#x}", self.opcode);
                        return;
                    }
                }
            },
            // Set program counter to location NNN
            0x1000 => {
                self.pc = self.opcode & 0x0FFF;
            },
            // Increment stack pointer, put current PC on top of stack. PC is set to NNN
            0x2000 => {
                self.push(self.pc);
                self.pc = self.opcode & 0x0FFF;
            },
            // 3xkk
            // Compares register Vx to kk, if equal => pc += 2
            0x3000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let kk = (self.opcode & 0x00FF) as u8;
                
                if self.cpu_register_v[x] == kk {
                    self.pc += 2;
                }
            },
            // 4xkk
            // Compares register Vx to kk, if NOT equal => pc += 2
            0x4000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let kk = (self.opcode & 0x00FF) as u8;

                if self.cpu_register_v[x] != kk {
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

                self.cpu_register_v[x] = kk;
            },
            // 7xkk
            // Sets register Vx = Vx + kk
            0x7000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let kk = (self.opcode & 0x00FF) as u8;

                // self.cpu_register_v[x] += kk;
                self.cpu_register_v[x] = self.cpu_register_v[x].wrapping_add(kk);
            },
            // 8xyz
            0x8000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let y = ((self.opcode & 0x00F0) >> 4) as usize;

                match self.opcode & 0x000F {
                    // Set Vx = Vx OR Vy
                    0x0001 => {
                        self.cpu_register_v[x] = self.cpu_register_v[x] | self.cpu_register_v[y];
                    },
                    // Set Vx = Vx AND Vy.
                    0x0002 => {
                        self.cpu_register_v[x] = self.cpu_register_v[x] & self.cpu_register_v[y];
                    },
                    // Set Vx = Vx XOR Vy.
                    0x0003 => {
                        self.cpu_register_v[x] = self.cpu_register_v[x] ^ self.cpu_register_v[y];
                    },
                    // Set Vx = Vx + Vy, set VF = carry.
                    0x0004 => {
                        let (new_vx, carry) = self.cpu_register_v[x].overflowing_add(self.cpu_register_v[y]);
                        let new_vf = if carry { 1 } else { 0 };
                        self.cpu_register_v[x] = new_vx;
                        self.cpu_register_v[0xF] = new_vf;
                    },
                    // Set Vx = Vx - Vy, set VF = NOT borrow.
                    0x0005 => {
                        let (new_vx, carry) = self.cpu_register_v[x].overflowing_sub(self.cpu_register_v[y]);
                        let new_vf = if carry { 1 } else { 0 };
                        self.cpu_register_v[x] = new_vx;
                        self.cpu_register_v[0xF] = new_vf;
                    },
                    // Set Vx = Vx SHR 1.
                    0x0006 => {
                        // Chip 8 is big endian
                        self.cpu_register_v[0xF] = self.cpu_register_v[x] & 1;
                        self.cpu_register_v[x] >>= 1;
                    },
                    // Set Vx = Vy - Vx, set VF = NOT borrow.
                    0x0007 => {
                        let (new_vx, borrow) = self.cpu_register_v[y].overflowing_sub(self.cpu_register_v[x]);
                        let new_vf = if borrow { 0 } else { 1 };

                        self.cpu_register_v[x] = new_vx;
                        self.cpu_register_v[0xF] = new_vf;
                    },
                    // Set Vx = Vx SHL 1.
                    0x000E => {
                        // Chip 8 is big endian
                        self.cpu_register_v[0xF] = (self.cpu_register_v[x] >> 7) & 1;
                        self.cpu_register_v[x] <<= 1;
                    },
                    _ => {
                        println!("No such opcode: {:#x}", self.opcode);
                        return;
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
            },
            // BNNN: Jump to location nnn + V0.
            0xB000 => {
                self.pc = (self.cpu_register_v[0] as u16) + (self.opcode & 0x0FFF);
            },
            // Cxkk: Set Vx = random byte & kk.
            // Might not work
            0xC000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let kk = self.opcode & 0x00FF;
                let random_number = rand::thread_rng().gen_range(0..=255);
                self.cpu_register_v[x] = (random_number & kk) as u8;
            },
            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
            // Dxyn
            0xD000 => {
                let digit2 = ((self.opcode & 0x0F00) >> 8) as usize;
                let digit3 = ((self.opcode & 0x00F0) >> 4) as usize;

                let x_coord = self.cpu_register_v[digit2] as u16;
                let y_coord = self.cpu_register_v[digit3] as u16;

                let n_bytes = self.opcode & 0x000F;
                let mut flipped = false;

                for i in 0..n_bytes {
                    let addr = self.register_index + i as u16;
                    let pixels = self.memory[addr as usize];

                    // 8 bits long
                    for j in 0..8 {
                        if (pixels & (0b1000_0000 >> j)) != 0 {
                            let x = (x_coord + j) as usize % GAME_WIDTH;
                            let y = (y_coord + i) as usize % GAME_HEIGHT;

                            let idx = x + GAME_WIDTH * y;

                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.cpu_register_v[0xF] = 1;
                }
                else {
                    self.cpu_register_v[0xF] = 0;
                }
            },
            0xE000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;

                match self.opcode & 0x00FF {
                    // Skip next instruction if key with the value of Vx is pressed.
                    // Checks the keyboard, and if the key corresponding to the value of Vx 
                    // is currently in the down position, PC is increased by 2.
                    0x009E => {
                        let key = self.keys[self.cpu_register_v[x] as usize];

                        if key {
                            self.pc += 2;
                        }
                    },
                    // ExA1 - SKNP Vx
                    // Skip next instruction if key with the value of Vx is not pressed.
                    // Checks the keyboard, and if the key corresponding to the value of Vx 
                    // is currently in the up position, PC is increased by 2.
                    0x00A1 => {
                        let key = self.keys[self.cpu_register_v[x] as usize];

                        if !key {
                            self.pc += 2;
                        }
                    },
                    _ => {
                        println!("No such opcode: {:#x}", self.opcode);
                    }
                }
            }
            // FxZZ
            0xF000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;

                match self.opcode & 0x00FF {
                    // Set Vx = delay timer value.
                    0x0007 => {
                        self.cpu_register_v[x] = self.delay_timer; 
                    },
                    // Fx0A - LD Vx, K
                    // Wait for a key press, store the value of the key in Vx.
                    // All execution stops until a key is pressed, then the value of that key is stored in Vx.
                    0x000A => {
                        let mut pressed = false;

                        for i in 0..self.keys.len() {
                            if self.keys[i] {
                                self.cpu_register_v[x] = i as u8;
                                pressed = true;
                                break;
                            }
                        }

                        if !pressed {
                            self.pc += 2;
                        }
                    },
                    // Set delay timer = Vx
                    0x0015 => {
                        self.delay_timer = self.cpu_register_v[x]; 
                    },
                    // Set sound timer = Vx
                    0x0018 => {
                        self.sound_timer = self.cpu_register_v[x]; 
                    },
                    // Set I = I + Vx
                    0x001E => {
                        self.register_index += self.cpu_register_v[x] as u16;
                    },
                    // Set I = location of sprite for digit Vx.
                    0x0029 => {
                        let font_sprite = self.cpu_register_v[x] as u16;
                        self.register_index = font_sprite * 5;
                    },
                    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                    0x0033 => {
                        // The interpreter takes the decimal value of Vx,
                        // places the hundreds digit in memory at location in I, 
                        // the tens digit at location I+1, 
                        // and the ones digit at location I+2.

                        // Casting might not work!
                        let decimal = self.cpu_register_v[x] as u16;
                        self.memory[self.register_index as usize] = (decimal / 100) as u8;
                        self.memory[(self.register_index + 1) as usize] = ((decimal / 10) % 10) as u8;
                        self.memory[(self.register_index + 2) as usize] = ((decimal / 100) % 10) as u8;
                    },
                    // Store registers V0 through Vx in memory starting at location I.
                    0x0055 => {
                        // The interpreter copies the values of registers V0 through Vx into memory,
                        // starting at the address in I.

                        // Might need to check if program counter is out of range
                        // also cast
                        for i in 0..=x {
                            self.memory[(self.register_index as usize) + i] = self.cpu_register_v[i];
                        }
                    },
                    // Read registers V0 through Vx from memory starting at location I.
                    0x0065 => {
                        for i in 0..=x {
                            self.cpu_register_v[i] = self.memory[(self.register_index as usize) + i];
                        }
                    },
                    _ => {
                        println!("No such opcode: {:#x}", self.opcode);
                        return;
                    }
                }
            },
            _ => {
                println!("No such opcode: {:#x}", self.opcode);
                return;
            }
        }
    }
}
