use rand::Rng;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const MEM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const NUM_V: usize = 16;
const START_ADDR: u16 = 0x200;
const NUM_KEYS: usize = 16;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub struct Emu {
    mem: [u8; MEM_SIZE],
    display: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    pc: u16,
    i_reg: u16,
    stack: Vec<u16>,
    d_timer: u8,
    s_timer: u8,
    v_reg: [u8; NUM_V],
    keys: [bool; NUM_KEYS],
    cosmac: bool,
}

impl Emu {
    pub fn new() -> Self {
        let mut emu_inst = Self {
            mem: [0; MEM_SIZE],
            display: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            pc: START_ADDR,
            i_reg: 0,
            stack: Vec::new(),
            d_timer: 0,
            s_timer: 0,
            v_reg: [0; NUM_V],
            keys: [false; NUM_KEYS],
            cosmac: false
        };

        emu_inst.mem[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        emu_inst
    }

    pub fn reset(&mut self) {
        self.mem = [0; MEM_SIZE];
        self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.pc = START_ADDR;
        self.i_reg = 0;
        self.stack = Vec::new();
        self.d_timer = 0;
        self.s_timer = 0;
        self.v_reg = [0; NUM_V];
        self.keys = [false; NUM_KEYS];
        self.cosmac = false;
        self.mem[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn set_cosmac(&mut self) {
        self.cosmac = true;
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        print!("{}:   ", self.pc);
        self.execute(op);
    }

    pub fn tick_timers(& mut self) {
        if self.d_timer > 0 {
            self.d_timer -= 1;
        }

        if self.s_timer > 0 {
            if self.s_timer == 1 {
                // BEEP
            }
            self.s_timer -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let hi_byte = self.mem[self.pc as usize] as u16;
        let lo_byte = self.mem[(self.pc + 1) as usize] as u16;
        let op = (hi_byte << 8) | lo_byte;

        self.pc += 2;

        op
    }

    fn execute(&mut self, op: u16) {
        let nibble1 = (op & 0xF000) >> 12;
        let nibble2 = (op & 0x0F00) >> 8;
        let nibble3 = (op & 0x00F0) >> 4;
        let nibble4 = op & 0x000F;

        match(nibble1, nibble2, nibble3, nibble4) {
            // 00E0: Clear screen
            (0, 0, 0xE, 0) => {
                self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
                println!("CLS");
            },

            // 00EE: Return from subroutine
            (0, 0, 0xE, 0xE) => {
                let addr = self.stack.pop();

                match addr {
                    Some(x) => {
                        self.pc = x;
                        println!("RET")
                    },
                    None => {
                        println!("The stack was unexpectedly empty.")
                    }
                }
            },

            // 1NNN: Jump 
            (1, _, _, _) => {
                let nnn = op & 0x0FFF;

                self.pc = nnn;
                
                println!("JP {}", nnn);
            },

            // 2NNN: Call subroutine
            (2, _, _, _) => {
                let nnn = op & 0x0FFF;

                self.stack.push(nnn);
                self.pc = nnn;

                println!("CALL {}", nnn);
            },

            // 3XNN: Skip if VX = NN
            (3, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (op & 0x00FF) as u8;

                if self.v_reg[x] == nn {
                    self.pc += 2;
                }

                println!("SE V{}, {}", x, nn);
            },

            // 4XNN: Skip if VX != NN
            (4, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (op & 0x00FF) as u8;

                if self.v_reg[x] != nn {
                    self.pc +=2;
                }

                println!("SNE V{}. {}", x, nn);
            }

            // 5XY0: Skip if VX = VY
            (5, _, _, 0) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }

                println!("SE V{}, V{}", x, y);
            }

            // 6XNN: Set
            (6, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (op & 0x00FF) as u8;

                self.v_reg[x] = nn;

                println!("LD V{}, {}", x, nn);
            },
            
            // 7XNN: Add
            (7, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (op & 0x00FF) as u8;

                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);

                println!("ADD V{}, {}", x, nn);
            },

            // 8XY0: Set
            (8, _, _, 0) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                self.v_reg[x] = self.v_reg[y];

                println!("LD V{}, V{}", x, y);
            } ,

            // 8XY1: Binary OR
            (8, _, _, 1) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                self.v_reg[x] |= self.v_reg[y];

                println!("OR V{}, V{}", x, y);
            },

            // 8XY2: Binary AND
            (8, _, _, 2) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                self.v_reg[x] &= self.v_reg[y];

                println!("AND V{}, V{}", x, y);
            },

            // 8XY3: Binary XOR
            (8, _, _, 3) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                self.v_reg[x] ^= self.v_reg[y];

                println!("XOR V{}, V{}", x, y);
            },

            // 8XY4: Add with carry
            (8, _, _, 4) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                let (sum, overflow) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                
                if overflow {
                    self.v_reg[0xF] = 1;
                }
                else {
                    self.v_reg[0xF] = 0;
                }

                self.v_reg[x] = sum;

                println!("ADD V{}, V{}", x, y);
            },

            // 8XY5: Subtract VY from VX
            (8, _, _, 5) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                let (diff, underflow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);

                if underflow {
                    self.v_reg[0xF] = 0;
                }
                else {
                    self.v_reg[0xF] = 1;
                }

                self.v_reg[x] = diff;

                println!("SUB V{}, V{}", x, y);
            },

            // 8XY6: Shift to right
            (8, _, _, 6) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                if self.cosmac { 
                    self.v_reg[x] = self.v_reg[y];
                }
                
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;

                println!("SHR V{}, V{}", x, y);
            },

            // 8XY7: Subtract VX from VY
            (8, _, _, 7) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                let (diff, underflow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);

                if underflow {
                    self.v_reg[0xF] = 0;
                }
                else {
                    self.v_reg[0xF] = 1;
                }

                self.v_reg[x] = diff;
                println!("SUBN V{}, V{}", x, y);
            },

            // 8XYE: Shift to left
            (8, _, _, 0xE) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                if self.cosmac {
                    self.v_reg[x] = self.v_reg[y];
                }

                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },

            // 9XY0: Skip if VX != VY
            (9, _, _, 0) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
                println!("SNE V{}, V{}", x, y);
            },

            // ANNN: Set index
            (0xA, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.i_reg = nnn;
                println!("LD I, {}", nnn);
            },

            // BNNN: Jump with offset
            (0xB, _, _, _) => {
                let mut nnn = op & 0x0FFF;
                let x = nibble2 as usize;

                if self.cosmac {
                    nnn += self.v_reg[0] as u16;
                }
                else {
                    let x = nibble2 as usize;
                    nnn += self.v_reg[x] as u16;
                }

                self.pc = nnn;
                
                if self.cosmac {
                    println!("JP V0, {}", nnn);
                }
                else {
                    println!("JP V{}, {}", x, nnn);
                }
            },

            // CXNN: Random
            (0xC, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (op & 0x00FF) as u16;
                
                let mut rng = rand::thread_rng();
                let rand_num: u16 = rng.gen();

                self.v_reg[x] = (nn & rand_num) as u8;

                println!("RND V{}, {}", x, nn);
            },

            // DXYN: Display
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[nibble2 as usize] as u16;
                let y_coord = self.v_reg[nibble3 as usize] as u16;

                let num_rows = nibble4;
                // Keep track if any pixels were flipped
                let mut flipped = false;
                // Iterate over each row of our sprite
                for y_line in 0..num_rows {
                    // Determine which memory address our row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.mem[addr as usize];
                    // Iterate over each column in our row
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            // Get our pixel's index for our 1D screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.display[idx];
                            self.display[idx] ^= true;
                        }
                    }
                }
                // Populate VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }

                println!("DRW V{}, V{}, {}", nibble2, nibble3, nibble4);
            },

            // EX9E: Skip if pressed
            (0xE, _, 9, 0xE) => {
                let x = nibble2 as usize;
                let index = self.v_reg[x] as usize;

                let key_pressed = self.keys[index];

                if key_pressed {
                    self.pc += 2;
                }

                println!("SKP V{}", x);
            },

            // EXA1: Skip if not pressed
            (0xE, _, 0xA, 1) => {
                let x = nibble2 as usize;
                let index = self.v_reg[x] as usize;

                let key_pressed = self.keys[index];

                if !key_pressed {
                    self.pc += 2;
                }

                println!("SKNP V{}", x);
            },

            // FX07: Set VX to delay timer value
            (0xF, _, 0, 7) => {
                let x = nibble2 as usize;

                self.v_reg[x] = self.d_timer;

                println!("LD V{}, DT", x);
            },

            // FX0A: Get key
            (0xF, _, 0, 0xA) => {
                let x = nibble2 as usize;

                let mut key_pressed = false;

                for i in 0..NUM_KEYS {
                    if self.keys[i] {
                        key_pressed = true;
                        self.v_reg[x] = i as u8;
                        break;
                    }
                }

                if !key_pressed {
                    self.pc -= 2;
                }
                else {
                    println!("LD V{}, K", x);
                }
            },

            // FX15: Set delay timer to VX
            (0xF, _, 1, 5) => {
                let x = nibble2 as usize;

                self.d_timer = self.v_reg[x];

                println!("LD DT ,V{}", x);
            }

            // FX18: Set sound timer to VX
            (0xF, _, 1, 8) => {
                let x = nibble2 as usize;
                self.s_timer = self.v_reg[x];

                println!("LD ST ,V{}", x);
            },

            // FX1E: Add to index
            (0xF, _, 1, 0xE) => {
                let x = nibble2 as usize;
                
                if self.cosmac{
                    self.i_reg += self.v_reg[x] as u16;
                }
                else {
                    self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
                }

                println!("ADD I, V{}", x);
            },

            // FX29: Font character
            (0xF, _, 2, 9) => {
                let x = nibble2 as usize;
                let hex_char = self.v_reg[x] & 0x0F;
                
                self.i_reg = (hex_char as u16) * 5;

                println!("LD F, V{}", x);
            },


            // FX33: Binary-coded decimal conversion
            (0xF, _, 3, 3) => {
                let x = nibble2 as usize;

                let value = self.v_reg[x] as f32;

                let hundreds = (value / 100.0).floor() as u8;
                let tens = ((value / 10.0) % 10.0).floor() as u8;
                let ones = (value % 10.0) as u8;

                self.mem[self.i_reg as usize] = hundreds;
                self.mem[(self.i_reg + 1) as usize] = tens;
                self.mem[(self.i_reg + 2) as usize] = ones;

                println!("LD B, V{}", x);
            },

            // FX55: Store memory
            (0xF, _, 5, 5) => {
                let x = nibble2 as usize;

                if self.cosmac {
                    for i in 0..(x + 1) {
                        self.mem[((self.i_reg as usize) + i) as usize] = self.v_reg[x];
                        self.i_reg += 1;
                    }
                }
                else {
                    for i in 0..(x + 1) {
                        self.mem[((self.i_reg as usize) + i) as usize] = self.v_reg[x];
                    }
                }

                println!("LD [I], V{}", x);
            },

            // FX65: Load memory
            (0xF, _, 6, 5) => {
                let x = nibble2 as usize;

                if self.cosmac {
                    for i in 0..(x + 1) {
                        self.v_reg[x] = self.mem[((self.i_reg as usize) + i) as usize]; 
                        self.i_reg += 1;
                    }
                }
                else {
                    for i in 0..(x + 1) {
                        self.v_reg[x] = self.mem[((self.i_reg as usize) + i) as usize]; 
                    }
                }

                println!("LD V{}, [I]", x);
            },

            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.display
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.mem[start..end].copy_from_slice(data);
    }

    pub fn keypress(&mut self, index: usize, pressed: bool) {
        self.keys[index] = pressed;
    }

}