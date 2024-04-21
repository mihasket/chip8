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

fn main() {
    println!("Hello, world!");
}
