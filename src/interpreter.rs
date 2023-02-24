use crate::{Chip8, State};
use crate::opcode::Opcode;

use rand::Rng;

impl Chip8 {
    /// Executes a single instruction using the interpreter.
    pub(super) fn interpreter(&mut self) {
        let opcode = Opcode((self.state.memory[self.state.PC as usize] as u16) << 8 | self.state.memory[self.state.PC as usize + 1] as u16);
        // #[cfg(debug_assertions)] println!("opcode {opcode:04X} at {:#X}", self.state.PC);
        self.state.PC += 2;

        (State::ILUT[opcode.0 as usize])(&mut self.state, opcode);

        self.handle_timers();
    }
}

/// The execution methods returns 1 if the cached interpreter should be interrupted,
/// > 1 to request a cache invalidation of [beg, end) with the beg in the high order word,
/// 0 if everything is good to continue.
#[allow(non_snake_case)]
impl State {
    pub(super) const ILUT: [fn(&mut State, Opcode) -> u32; 1 << 16] = generate_decoder();

    pub(super) fn execute_00E0(&mut self, _: Opcode) -> u32 {
        self.clear_screen();
        0
    }

    pub(super) fn execute_00EE(&mut self, _: Opcode) -> u32 {
        if self.SP > 0 {
            self.SP -= 1;
            self.PC = self.stack[self.SP];
        } else {
            println!("Stack underflow (RET 0x00EE)");
        }
        1
    }

    pub(super) fn execute_1nnn(&mut self, opcode: Opcode) -> u32 {
        self.PC = opcode.nnn();
        1
    }

    pub(super) fn execute_2nnn(&mut self, opcode: Opcode) -> u32 {
        if self.SP < 0xF {
            self.stack[self.SP] = self.PC;
            self.SP += 1;
            self.PC = opcode.nnn();
        } else {
            println!("Stack overflow (CALL 0x2nnn)");
        }
        1
    }

    pub(super) fn execute_3xkk(&mut self, opcode: Opcode) -> u32 {
        let (x, kk) = opcode.xkk();
        if self.V[x] == kk {
            self.PC += 2;
            1
        } else {
            0
        }
    }

    pub(super) fn execute_4xkk(&mut self, opcode: Opcode) -> u32 {
        let (x, kk) = opcode.xkk();
        if self.V[x] != kk {
            self.PC += 2;
            1
        } else {
            0
        }
    }

    pub(super) fn execute_5xy0(&mut self, opcode: Opcode) -> u32 {
        let (x, y) = opcode.xy();
        if self.V[x] == self.V[y] {
            self.PC += 2;
            1
        } else {
            0
        }
    }

    pub(super) fn execute_6xkk(&mut self, opcode: Opcode) -> u32 {
        let (x, kk) = opcode.xkk();
        self.V[x] = kk;
        0
    }

    pub(super) fn execute_7xkk(&mut self, opcode: Opcode) -> u32 {
        let (x, kk) = opcode.xkk();
        self.V[x] = self.V[x].wrapping_add(kk);
        0
    }

    pub(super) fn execute_8xy0(&mut self, opcode: Opcode) -> u32 {
        let (x, y) = opcode.xy();
        self.V[x] = self.V[y];
        0
    }

    pub(super) fn execute_8xy1(&mut self, opcode: Opcode) -> u32 {
        let (x, y) = opcode.xy();
        self.V[x] |= self.V[y];
        0
    }

    pub(super) fn execute_8xy2(&mut self, opcode: Opcode) -> u32 {
        let (x, y) = opcode.xy();
        self.V[x] &= self.V[y];
        0
    }

    pub(super) fn execute_8xy3(&mut self, opcode: Opcode) -> u32 {
        let (x, y) = opcode.xy();
        self.V[x] ^= self.V[y];
        0
    }

    pub(super) fn execute_8xy4(&mut self, opcode: Opcode) -> u32 {
        let (x, y) = opcode.xy();
        let (res, c) = self.V[x].overflowing_add(self.V[y]);
        self.V[x] = res;
        self.V[0xF] = c as u8;
        0
    }

    pub(super) fn execute_8xy5(&mut self, opcode: Opcode) -> u32 {
        let (x, y) = opcode.xy();
        let (res, b) = self.V[x].overflowing_sub(self.V[y]);
        self.V[x] = res;
        self.V[0xF] = (!b) as u8;
        0
    }

    pub(super) fn execute_8xy6(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        // let y = opcode.y();
        let c = self.V[x] & 1;
        self.V[x] >>= 1;
        self.V[0xF] = c;
        0
    }

    pub(super) fn execute_8xy7(&mut self, opcode: Opcode) -> u32 {
        let (x, y) = opcode.xy();
        let (res, b) = self.V[y].overflowing_sub(self.V[x]);
        self.V[x] = res;
        self.V[0xF] = (!b) as u8;
        0
    }

    pub(super) fn execute_8xyE(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        // let y = opcode.y();
        let c = self.V[x] >> 7 & 1;
        self.V[x] <<= 1;
        self.V[0xF] = c;
        0
    }

    pub(super) fn execute_9xy0(&mut self, opcode: Opcode) -> u32 {
        let (x, y) = opcode.xy();
        if self.V[x] != self.V[y] {
            self.PC += 2;
            1
        } else {
            0
        }
    }

    pub(super) fn execute_Annn(&mut self, opcode: Opcode) -> u32 {
        self.I = opcode.nnn();
        0
    }

    pub(super) fn execute_Bnnn(&mut self, opcode: Opcode) -> u32 {
        self.PC = opcode.nnn() + self.V[0] as u16;
        1
    }

    pub(super) fn execute_Cxkk(&mut self, opcode: Opcode) -> u32 {
        let (x, kk) = opcode.xkk();
        self.V[x] = rand::thread_rng().gen_range(0, 256) as u8 & kk;
        0
    }

    pub(super) fn execute_Dxyn(&mut self, opcode: Opcode) -> u32 {
        let (x, y) = opcode.xy();
        let n = opcode.n();
        self.draw(x, y, n);
        0
    }

    pub(super) fn execute_Ex9E(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        if self.keys[self.V[x] as usize] {
            self.PC += 2;
            1
        } else {
            0
        }
    }

    pub(super) fn execute_ExA1(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        if !self.keys[self.V[x] as usize] {
            self.PC += 2;
            1
        } else {
            0
        }
    }

    pub(super) fn execute_Fx07(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        self.V[x] = self.delay;
        0
    }

    pub(super) fn execute_Fx0A(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        if !self.wait_key(x) {
            // If it is still waiting for a key, decrement PC to make it loop over this instruction.
            self.PC -= 2;
            1
        } else {
            0
        }
    }

    pub(super) fn execute_Fx15(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        self.delay = self.V[x];
        0
    }

    pub(super) fn execute_Fx18(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        self.sound = self.V[x];
        0
    }

    pub(super) fn execute_Fx1E(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        self.I += self.V[x] as u16;
        0
    }

    pub(super) fn execute_Fx29(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        self.I = self.V[x] as u16 * 5;
        0
    }

    pub(super) fn execute_Fx33(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        self.memory[self.I as usize] = self.V[x] / 100;
        self.memory[self.I as usize + 1] = (self.V[x] / 10) % 10;
        self.memory[self.I as usize + 2] = self.V[x] % 10;
        0
    }

    pub(super) fn execute_Fx55(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        for i in 0..=x {
            self.memory[self.I as usize + i] = self.V[i];
        }
        (self.I as u32) << 16 | self.I as u32 + x as u32
    }

    pub(super) fn execute_Fx65(&mut self, opcode: Opcode) -> u32 {
        let x = opcode.x();
        for i in 0..=x {
            self.V[i] = self.memory[self.I as usize + i];
        }
        0
    }

    fn execute_invalid(&mut self, opcode: Opcode) -> u32 {
        panic!("invalid opcode {:04X} at {:#X}", opcode, self.PC - 2);
    }
}

const fn generate_decoder() -> [fn(&mut State, Opcode) -> u32; 1 << 16] {
    let mut lut: [fn(&mut State, Opcode) -> u32; 1 << 16] = [State::execute_invalid; 1 << 16];

    let mut i = 0;
    while i < INSTRUCTION_FORMATS.len() {
        let (format, execute) = INSTRUCTION_FORMATS[i];

        generate_opcodes(format.as_bytes(), execute, &mut lut);

        i += 1;
    }

    lut
}

/// Send `format.as_bytes()` as the `format` parameter (slice of u8 charactere values).
const fn generate_opcodes(format: &[u8], execute: fn(&mut State, Opcode) -> u32, lut: &mut [fn(&mut State, Opcode) -> u32; 1 << 16]) {
    let mut ok = true;

    let mut i = 0;
    while i < format.len() {
        if format[i] > 'F' as u8 {
            ok = false;
            let mut fmt = slice_to_array(format);

            let mut j = 0;
            while j < 16 {
                let c = if j > 9 { j + 0x37 } else { j + 0x30 }; // u8 to ascii that doesn't crash the const evaluator.
                fmt[i] = c;
                generate_opcodes(&fmt, execute, lut);
                j += 1;
            }

            break;
        }

        i += 1;
    }

    if ok {
        let index = slice_to_usize(format);
        lut[index] = execute;
    }
}

const fn slice_to_usize(bytes: &[u8]) -> usize {
    let b0 = (bytes[0] as char).to_digit(16).unwrap() as usize;
    let b1 = (bytes[1] as char).to_digit(16).unwrap() as usize;
    let b2 = (bytes[2] as char).to_digit(16).unwrap() as usize;
    let b3 = (bytes[3] as char).to_digit(16).unwrap() as usize;

    b0 << 12 | b1 << 8 | b2 << 4 | b3
}

const fn slice_to_array(bytes: &[u8]) -> [u8; 4] {
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}

const INSTRUCTION_FORMATS: [(&str, fn(&mut State, Opcode) -> u32); 34] = [
    ("00E0", State::execute_00E0),
    ("00EE", State::execute_00EE),
    ("1nnn", State::execute_1nnn),
    ("2nnn", State::execute_2nnn),
    ("3xkk", State::execute_3xkk),
    ("4xkk", State::execute_4xkk),
    ("5xy0", State::execute_5xy0),
    ("6xkk", State::execute_6xkk),
    ("7xkk", State::execute_7xkk),
    ("8xy0", State::execute_8xy0),
    ("8xy1", State::execute_8xy1),
    ("8xy2", State::execute_8xy2),
    ("8xy3", State::execute_8xy3),
    ("8xy4", State::execute_8xy4),
    ("8xy5", State::execute_8xy5),
    ("8xy6", State::execute_8xy6),
    ("8xy7", State::execute_8xy7),
    ("8xyE", State::execute_8xyE),
    ("9xy0", State::execute_9xy0),
    ("Annn", State::execute_Annn),
    ("Bnnn", State::execute_Bnnn),
    ("Cxkk", State::execute_Cxkk),
    ("Dxyn", State::execute_Dxyn),
    ("Ex9E", State::execute_Ex9E),
    ("ExA1", State::execute_ExA1),
    ("Fx07", State::execute_Fx07),
    ("Fx0A", State::execute_Fx0A),
    ("Fx15", State::execute_Fx15),
    ("Fx18", State::execute_Fx18),
    ("Fx1E", State::execute_Fx1E),
    ("Fx29", State::execute_Fx29),
    ("Fx33", State::execute_Fx33),
    ("Fx55", State::execute_Fx55),
    ("Fx65", State::execute_Fx65),
];
