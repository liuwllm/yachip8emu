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
    v_reg: [u8; NUM_V]
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
            v_reg: [0; NUM_V]
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
    }

    pub fn tick(&mut self) {
        let op = self.fetch();

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
            }

            // 1NNN: Jump 
            (1, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.pc = nnn;
            }
            // 6XNN: Set
            (6, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (op & 0x00FF) as u8;

                self.v_reg[x] = nn;
            }

            // ANNN: Set index
            (0xA, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.i_reg = nnn;
            }

            // DXYN: Display
            (0xD, _, _, _) => {
                let x = nibble2 as usize;
                let y = nibble2 as usize;
                let n = nibble4;

                let x_coord = self.v_reg[x] as u16;
                let y_coord = self.v_reg[y] as u16;

                self.v_reg[0xF] = 0;

                // Iterate over rows
                for row in 0..n {
                    // Retrieve the byte (8 bits) for this row
                    let sprite_byte = self.mem[(self.i_reg + row as u16) as usize];

                    for col in 0..8 {
                        if sprite_byte & (0b1000_0000 >> col) != 0 {
                            let adjusted_x = (x_coord + col) as usize % SCREEN_WIDTH;
                            let adjusted_y = (y_coord + row) as usize % SCREEN_HEIGHT;

                            let flat_index = x + SCREEN_WIDTH * y;

                            if self.display[flat_index] != false {
                                self.display[flat_index] = false;
                                self.v_reg[0xF] = 1;
                            }
                            else {
                                self.display[flat_index] = true;
                            }
                        }
                    }
                }
            }

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