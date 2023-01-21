pub struct Chip8 {
    mem: [u8; 4096],   // 4KB RAM
    stack: [u16; 16],  // stack (for storing return addresses)
    reg: Registers,
}

#[derive(Default)]
struct Registers {
    pc: u16,      // program counter
    v: [u8; 16],  // general purpose
    i: u16,       // index register
    sp: u8,       // stack pointer
    dt: u8,       // delay timer
    st: u8,       // sound timer
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            mem: [0; 4096],
            stack: [0; 16],
            reg: Registers::default(),
        }
    }

    pub fn step(&mut self, input: Option<u8>, output: &mut [bool; 64 * 32]) {
        unimplemented!()
    }
}
