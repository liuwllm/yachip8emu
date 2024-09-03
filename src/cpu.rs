pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const MEM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const NUM_V: usize = 16;
const START_ADDR: u16 = 0x200;

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
        self.mem[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        self.cosmac = false;
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
                        panic!("The stack was unexpectedly empty.")
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

                self.v_reg[x] += nn;

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
            }

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
            }

            // 9XY0: Skip if VX != VY
            (9, _, _, 0) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
                println!("SNE V{}, V{}", x, y);
            }

            // ANNN: Set index
            (0xA, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.i_reg = nnn;
                println!("LD I, {}", nnn);
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

}