const RAM_SIZE: usize = 0x1000;
const STACK_SIZE: usize = 16;

const V_REG_COUNT: usize = 16;

const ROM_LOAD_ADDR: usize = 0x200;

const FONT_DATA: &[u8] = &[
    0xF0, 0x90, 0x90, 0x90, 0xF0,  // 0
    0x20, 0x60, 0x20, 0x20, 0x70,  // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,  // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,  // 3
    0x90, 0x90, 0xF0, 0x10, 0x10,  // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,  // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,  // 6
    0xF0, 0x10, 0x20, 0x40, 0x40,  // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,  // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,  // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90,  // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,  // B
    0xF0, 0x80, 0x80, 0x80, 0xF0,  // C
    0xE0, 0x90, 0x90, 0x90, 0xE0,  // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,  // E
    0xF0, 0x80, 0xF0, 0x80, 0x80,  // F
];

pub struct Chip8 {
    mem: [u8; RAM_SIZE],       // 4KB RAM
    stack: [u16; STACK_SIZE],  // stack (for storing return addresses)
    reg: Registers,
}

#[derive(Default)]
struct Registers {
    pc: u16,               // program counter
    v: [u8; V_REG_COUNT],  // general purpose
    i: u16,                // index register
    sp: u8,                // stack pointer
    dt: u8,                // delay timer
    st: u8,                // sound timer
}

impl Chip8 {
    pub fn new() -> Self {
        let mut mem = [0; RAM_SIZE];

        for (addr, value) in FONT_DATA.iter().enumerate() {
            mem[addr] = *value;
        }

        Chip8 {
            mem,
            stack: [0; STACK_SIZE],
            reg: Registers::default(),
        }
    }

    pub fn load(&mut self, rom: &[u8]) {
        for (offset, value) in rom.iter().enumerate() {
            let addr = (ROM_LOAD_ADDR + offset).clamp(0, RAM_SIZE - 1);
            self.mem[addr] = *value;
        }

        self.reg.pc = ROM_LOAD_ADDR as u16;
    }

    pub fn step(&mut self, input: Option<u8>, output: &mut [bool; 64 * 32]) {
        unimplemented!()
    }
}
