const RAM_SIZE: u16 = 0x1000;
const STACK_SIZE: u8 = 16;
const ROM_LOAD_ADDR: u16 = 0x200;
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
    mem: [u8; RAM_SIZE as usize],       // 4KB RAM
    stack: [u16; STACK_SIZE as usize],  // stack (for storing return addresses)
    pc: u16,      // program counter
    v: [u8; 16],  // general purposeisters
    i: u16,       // indexister
    sp: u8,       // stack pointer
    dt: u8,       // delay timer
    st: u8,       // sound timer
}

impl Chip8 {
    pub fn new() -> Self {
        let mut mem = [0; RAM_SIZE as usize];

        for (addr, value) in FONT_DATA.iter().enumerate() {
            mem[addr] = *value;
        }

        Chip8 {
            mem,
            stack: [0; STACK_SIZE as usize],
            pc: ROM_LOAD_ADDR,
            v: [0; 16],
            i: 0,
            sp: 0,
            dt: 0,
            st: 0,
        }
    }

    pub fn load(&mut self, rom: &[u8]) {
        for (offset, value) in rom.iter().enumerate() {
            self.write(ROM_LOAD_ADDR + offset as u16, *value);
        }

        self.pc = ROM_LOAD_ADDR as u16;
    }

    pub fn step(&mut self, input: &[bool; 16], output: &mut [bool; 64 * 32]) {
        let instr = self.next_instruction();

        match instr.opcode {
            0x00E0 => {  // CLS
                // TODO
            },
            0x00EE => {  // RET
                self.pc = self.pop();
            },
            _ => {},
        }

        match instr.c() {
            0x1 => {  // JMP NNN
                self.pc = instr.nnn();
            },
            0x2 => {  // CALL NNN
                self.push(self.pc);
                self.pc = instr.nnn();
            },
            0x3 => {  // SE VX, NN
                if self.v(instr.x()) == instr.nn() {
                    self.pc += 2;
                }
            },
            0x4 => {  // SNE VX, NN
                if self.v(instr.x()) != instr.nn() {
                    self.pc += 2;
                }
            },
            0x6 => {  // LD VX, NN
                *self.v_mut(instr.x()) = instr.nn();
            },
            0x7 => {  // ADD VX, NN
                *self.v_mut(instr.x()) += instr.nn();
            },
            0xA => {  // LD I, NNN
                self.i = instr.nnn();
            },
            0xB => {  // JMP V0, NNN
                self.pc = self.v(0) as u16 + instr.nnn();
            },
            0xC => {  // RND VX, NN
                let r = 25; // TODO: Generate random number 0..255.
                *self.v_mut(instr.x()) = r & instr.nn();
            },
            0xD => {  // DRW VX, VY, N
                // TODO
            },
            _ => {},
        }

        match (instr.c(), instr.n()) {
            (0x5, 0x0) => {  // SE VX, VY
                if self.v(instr.x()) == self.v(instr.y()) {
                    self.pc += 2;
                }
            },
            (0x8, 0x0) => {  // LD VX, VY
                *self.v_mut(instr.x()) = self.v(instr.y());
            },
            (0x8, 0x1) => {  // OR VX, VY
                *self.v_mut(instr.x()) |= self.v(instr.y());
            },
            (0x8, 0x2) => {  // AND VX, VY
                *self.v_mut(instr.x()) &= self.v(instr.y());
            },
            (0x8, 0x3) => {  // XOR VX, VY
                *self.v_mut(instr.x()) ^= self.v(instr.y());
            },
            (0x8, 0x4) => {  // ADD VX, VY
                let sum = self.v(instr.x()) as u16 + self.v(instr.y()) as u16;
                *self.v_mut(0xF) = (sum > 0xFF) as u8;  // carry flag
                *self.v_mut(instr.x()) = sum as u8;
            },
            (0x8, 0x5) => {  // SUB VX, VY
                *self.v_mut(0xF) = (self.v(instr.x()) > self.v(instr.y())) as u8;  // borrow flag
                *self.v_mut(instr.x()) -= self.v(instr.y());
            },
            (0x8, 0x6) => {  // SHR VX
                *self.v_mut(0xF) = self.v(instr.x()) & 0x01;  // least significant bit
                *self.v_mut(instr.x()) /= 2;
            },
            (0x8, 0x7) => {  // SUBN VX, VY
                *self.v_mut(0xF) = (self.v(instr.y()) > self.v(instr.x())) as u8;  // borrow flag
                *self.v_mut(instr.x()) = self.v(instr.y()) - self.v(instr.x())

            },
            (0x8, 0xE) => {  // SHL VX
                *self.v_mut(0xF) = self.v(instr.x()) & 0x80;  // most significant bit
                *self.v_mut(instr.x()) *= 2;
            },
            (0x9, 0x0) => {  // SNE VX, VY
                if self.v(instr.x()) != self.v(instr.y()) {
                    self.pc += 2;
                }
            },
            _ => {},
        }

        match (instr.c(), instr.nn()) {
            (0xE, 0x9E) => {  // SKP VX
                if input[self.v(instr.x()) as usize] {
                    self.pc += 2;
                }
            },
            (0xE, 0xA1) => {  // SKNP VX
                if !input[self.v(instr.x()) as usize] {
                    self.pc += 2;
                }
            },
            (0xF, 0x07) => {  // LD VX, DT
                *self.v_mut(instr.x()) = self.dt;
            },
            (0xF, 0x0A) => {  // LD VX, K
                // TODO
            },
            (0xF, 0x16) => {  // LD DT, VX
                self.dt = self.v(instr.x());
            },
            (0xF, 0x18) => {  // LD ST, VX
                self.st = self.v(instr.x());
            },
            (0xF, 0x1E) => {  // ADD I, VX
                self.i += self.v(instr.x()) as u16;
            },
            (0xF, 0x29) => {  // LD F, VX
                // TODO
            },
            (0xF, 0x33) => {  // LD B, VX
                // TODO
            },
            (0xF, 0x55) => {  // LD [I], VX
                for index in 0..0xF {
                    self.write(self.i + index, self.v(index as u8));
                }
            },
            (0xF, 0x65) => {  // LD VX, [I]
                for index in 0..0xF {
                    *self.v_mut(index as u8) = self.read(self.i + index);
                }
            },
            _ => {},
        }
    }

    fn next_instruction(&mut self) -> Instruction {
        let x = self.read(self.pc) as u16;
        let y = self.read(self.pc + 1) as u16;

        self.pc += 2;

        Instruction { opcode: (x << 8) + y }
    }

    fn read(&self, addr: u16) -> u8 {
        if addr < RAM_SIZE {
            return self.mem[addr as usize];
        }
        0
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr < RAM_SIZE {
            self.mem[addr as usize] = value;
        }
    }

    fn pop(&mut self) -> u16 {
        if self.sp == 0 {
            return 0;
        }
        self.sp -= 1;
        if self.sp >= STACK_SIZE {
            return 0;
        }
        self.stack[self.sp as usize]
    }

    fn push(&mut self, value: u16) {
        if self.sp < STACK_SIZE {
            self.stack[self.sp as usize] = value;
            self.sp += 1;
        }
    }

    fn v(&self, index: u8) -> u8 {
        self.v[index as usize]
    }

    fn v_mut(&mut self, index: u8) -> &mut u8 {
        &mut self.v[index as usize]
    }
}

struct Instruction {
    opcode: u16,
}

impl Instruction {
    fn c(&self) -> u8 {
        ((self.opcode >> 12) & 0xF) as u8
    }

    fn x(&self) -> u8 {
        ((self.opcode >> 8) & 0xF) as u8
    }

    fn y(&self) -> u8 {
        ((self.opcode >> 4) & 0xF) as u8
    }

    fn n(&self) -> u8 {
        (self.opcode & 0xF) as u8
    }

    fn nn(&self) -> u8 {
        (self.opcode & 0xFF) as u8
    }

    fn nnn(&self) -> u16 {
        self.opcode & 0xFFF
    }
}
