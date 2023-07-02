use rand::Rng;

const RAM_SIZE: u16 = 0x1000;
const STACK_SIZE: u8 = 16;
const ROM_LOAD_ADDR: u16 = 0x200;
const GENERAL_REG_COUNT: usize = 16;
pub const INPUT_COUNT: usize = 16;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

const FONT_DATA: &[u8] = &[
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    mem: [u8; RAM_SIZE as usize],      // 4KB RAM
    stack: [u16; STACK_SIZE as usize], // stack (for storing return addresses)
    pc: u16,                           // program counter
    v: [u8; GENERAL_REG_COUNT],        // general purpose registers
    i: u16,                            // index register
    sp: u8,                            // stack pointer
    dt: u8,                            // delay timer
    st: u8,                            // sound timer
    waiting_for_input: Option<usize>, // wait for key press instruction - write key code to VX where X is the usize value
}

impl Chip8 {
    pub fn new() -> Self {
        let mut c8 = Chip8 {
            mem: [0; RAM_SIZE as usize],
            stack: [0; STACK_SIZE as usize],
            pc: ROM_LOAD_ADDR,
            v: [0; 16],
            i: 0,
            sp: 0,
            dt: 0,
            st: 0,
            waiting_for_input: None,
        };

        c8.write_sequence(0, FONT_DATA);

        c8
    }

    /*
     * Load a ROM into memory and set the PC register to the start address of the loaded code.
     */
    pub fn load(&mut self, rom: &[u8]) {
        self.write_sequence(ROM_LOAD_ADDR, rom);

        self.pc = ROM_LOAD_ADDR;
    }

    /*
     * Appropriately adjust the values of the delay timer and sound timer registers. This method should be called 60
     * times per second. This method returns whether or not the delayer timer register value is greater than 0 (i.e.,
     * if the buzzer should sound or not).
     */
    pub fn step_timers(&mut self) -> bool {
        if self.dt > 0 {
            self.dt -= 1;
        }

        let buzz = self.st > 0;

        if buzz {
            self.st -= 1;
        }

        buzz
    }

    /*
     * Run a CPU cycle (fetch instruction and execute). This method should be called at a rate of at least 500 times per
     * second.
     */
    pub fn step(
        &mut self,
        input: &[bool; INPUT_COUNT],
        output: &mut [[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
    ) {
        if let Some(x) = self.waiting_for_input {
            match input.iter().position(|down| *down) {
                Some(key) => {
                    self.v[x] = key as u8;
                    self.waiting_for_input = None;
                }
                None => return,
            }
        }

        let instr = self.next_instruction();

        match instr.opcode {
            // CLS
            0x00E0 => {
                for x in 0..DISPLAY_WIDTH {
                    output[x].fill(false);
                }
            }

            // RET
            0x00EE => self.pc = self.pop(),
            _ => {}
        }

        match instr.c() {
            // JMP NNN
            0x1 => self.pc = instr.nnn(),

            // CALL NNN
            0x2 => {
                self.push(self.pc);
                self.pc = instr.nnn();
            }

            // SE VX, NN
            0x3 => {
                if self.v[instr.x()] == instr.nn() {
                    self.pc += 2;
                }
            }

            // SNE VX, NN
            0x4 => {
                if self.v[instr.x()] != instr.nn() {
                    self.pc += 2;
                }
            }

            // LD VX, NN
            0x6 => self.v[instr.x()] = instr.nn(),

            // ADD VX, NN
            0x7 => self.v[instr.x()] = self.v[instr.x()].wrapping_add(instr.nn()),

            // LD I, NNN
            0xA => self.i = instr.nnn(),

            // JMP V0, NNN
            0xB => self.pc = self.v[0] as u16 + instr.nnn(),

            // RND VX, NN
            0xC => {
                let r = rand::thread_rng().gen_range(0..=255);
                self.v[instr.x()] = r & instr.nn();
            }

            // DRW VX, VY, N
            0xD => {
                let sprite_height = instr.n() as usize;

                let mut sprite = vec![vec![false; sprite_height]; 8];

                let mem_range = (self.i as usize)..(self.i as usize + instr.n() as usize);
                for (sy, byte) in self.mem[mem_range].iter().enumerate() {
                    for sx in 0..8 {
                        let bit = (byte >> (7 - sx)) & 1;
                        sprite[sx][sy] = bit != 0;
                    }
                }

                let vx = self.v[instr.x()] as usize % DISPLAY_WIDTH;
                let vy = self.v[instr.y()] as usize % DISPLAY_HEIGHT;

                self.v[0xF] = 0; // clear flag

                for sx in 0..8 {
                    let x = vx + sx;

                    if x == DISPLAY_WIDTH {
                        break;
                    }

                    for sy in 0..sprite_height {
                        let y = vy + sy;

                        if y == DISPLAY_HEIGHT {
                            break;
                        }

                        if sprite[sx][sy] {
                            if output[x][y] {
                                self.v[0xF] = 1; // collision
                            }
                            output[x][y] = !output[x][y];
                        }
                    }
                }
            }

            _ => {}
        }

        match (instr.c(), instr.n()) {
            // SE VX, VY
            (0x5, 0x0) => {
                if self.v[instr.x()] == self.v[instr.y()] {
                    self.pc += 2;
                }
            }

            // LD VX, VY
            (0x8, 0x0) => self.v[instr.x()] = self.v[instr.y()],

            // OR VX, VY
            (0x8, 0x1) => self.v[instr.x()] |= self.v[instr.y()],

            // AND VX, VY
            (0x8, 0x2) => self.v[instr.x()] &= self.v[instr.y()],

            // XOR VX, VY
            (0x8, 0x3) => self.v[instr.x()] ^= self.v[instr.y()],

            // ADD VX, VY
            (0x8, 0x4) => {
                let (sum, overflow) = self.v[instr.x()].overflowing_add(self.v[instr.y()]);
                self.v[0xF] = overflow as u8;
                self.v[instr.x()] = sum;
            }

            // SUB VX, VY
            (0x8, 0x5) => {
                self.v[0xF] = (self.v[instr.x()] >= self.v[instr.y()]) as u8;
                self.v[instr.x()] = self.v[instr.x()].wrapping_sub(self.v[instr.y()]);
            }

            // SHR VX
            (0x8, 0x6) => {
                self.v[0xF] = self.v[instr.x()] & 1; // least significant bit
                self.v[instr.x()] /= 2;
            }

            // SUBN VX, VY
            (0x8, 0x7) => {
                self.v[0xF] = (self.v[instr.y()] >= self.v[instr.x()]) as u8;
                self.v[instr.x()] = self.v[instr.y()].wrapping_sub(self.v[instr.x()]);
            }

            // SHL VX
            (0x8, 0xE) => {
                self.v[0xF] = (self.v[instr.x()] >> 7) & 1; // most significant bit
                self.v[instr.x()] *= 2;
            }

            // SNE VX, VY
            (0x9, 0x0) => {
                if self.v[instr.x()] != self.v[instr.y()] {
                    self.pc += 2;
                }
            }

            _ => {}
        }

        match (instr.c(), instr.nn()) {
            // SKP VX
            (0xE, 0x9E) => {
                if input[self.v[instr.x()] as usize] {
                    self.pc += 2;
                }
            }

            // SKNP VX
            (0xE, 0xA1) => {
                if !input[self.v[instr.x()] as usize] {
                    self.pc += 2;
                }
            }

            // LD VX, DT
            (0xF, 0x07) => self.v[instr.x()] = self.dt,

            // LD VX, K
            (0xF, 0x0A) => self.waiting_for_input = Some(instr.x()),

            // LD DT, VX
            (0xF, 0x15) => self.dt = self.v[instr.x()],

            // LD ST, VX
            (0xF, 0x18) => self.st = self.v[instr.x()],

            // ADD I, VX
            (0xF, 0x1E) => self.i = self.i.wrapping_add(self.v[instr.x()] as u16),

            // LD F, VX
            (0xF, 0x29) => self.i = self.v[instr.x()] as u16 * 5,

            // LD B, VX
            (0xF, 0x33) => {
                let vx = self.v[instr.x()];

                let hundreds = vx / 100;
                let tens = (vx - hundreds * 100) / 10;
                let ones = (vx - hundreds * 100) - tens * 10;

                self.write_sequence(self.i, &[hundreds, tens, ones]);
            }

            // LD [I], VX
            (0xF, 0x55) => {
                for index in 0..0xF {
                    self.write(self.i + index as u16, self.v[index]);
                }
            }

            // LD VX, [I]
            (0xF, 0x65) => {
                for index in 0..=instr.x() {
                    self.v[index] = self.read(self.i + index as u16);
                }
            }

            _ => {}
        }
    }

    fn next_instruction(&mut self) -> Instruction {
        let x = self.read(self.pc) as u16;
        let y = self.read(self.pc + 1) as u16;

        self.pc += 2;

        Instruction {
            opcode: (x << 8) + y,
        }
    }

    /*
     * Read a single 8-bit value from RAM. Will return 0 if the given address is out of bounds.
     */
    fn read(&self, addr: u16) -> u8 {
        if addr < RAM_SIZE {
            return self.mem[addr as usize];
        }
        0
    }

    /*
     * Write a single 8-bit value to RAM. Will do nothing if the given address is out of bounds.
     */
    fn write(&mut self, addr: u16, value: u8) {
        if addr < RAM_SIZE {
            self.mem[addr as usize] = value;
        }
    }

    /*
     * Write multiple values to RAM beginning from the given starting address.
     */
    fn write_sequence(&mut self, start_addr: u16, values: &[u8]) {
        for (offset, value) in values.iter().enumerate() {
            self.write(start_addr + offset as u16, *value);
        }
    }

    /*
     * Pop a value off the stack. Will return 0 if the stack pointer is out of bounds.
     */
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

    /*
     * Push a value onto the stack. Will do nothing if the stack is full.
     */
    fn push(&mut self, value: u16) {
        if self.sp < STACK_SIZE {
            self.stack[self.sp as usize] = value;
            self.sp += 1;
        }
    }
}

struct Instruction {
    opcode: u16,
}

impl Instruction {
    fn c(&self) -> u8 {
        ((self.opcode >> 12) & 0xF) as u8
    }

    fn x(&self) -> usize {
        ((self.opcode >> 8) & 0xF) as usize
    }

    fn y(&self) -> usize {
        ((self.opcode >> 4) & 0xF) as usize
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
