use crate::{Chip8, State};
use crate::opcode::Opcode;

use rand::Rng;

impl Chip8 {
    /// Executes a single instruction using the interpreter.
    pub(super) fn interpreter(&mut self) {
        let opcode = Opcode((self.state.memory[self.state.PC as usize] as u16) << 8 | self.state.memory[self.state.PC as usize + 1] as u16);
        // #[cfg(debug_assertions)] println!("opcode {opcode:04X} at {:#X}", self.state.PC);
        self.state.PC += 2;

        match opcode.0 >> 12 & 0xF {
            0x0 => match opcode.0 {
                0x00E0 => self.state.execute_00E0(opcode),
                0x00EE => self.state.execute_00EE(opcode),
                _ => panic!("Unknown opcode {opcode:04X}"),
            },
            0x1 => self.state.execute_1nnn(opcode),
            0x2 => self.state.execute_2nnn(opcode),
            0x3 => self.state.execute_3xkk(opcode),
            0x4 => self.state.execute_4xkk(opcode),
            0x5 => self.state.execute_5xy0(opcode),
            0x6 => self.state.execute_6xkk(opcode),
            0x7 => self.state.execute_7xkk(opcode),
            0x8 => {
                match opcode.0 & 0xF00F {
                    0x8000 => self.state.execute_8xy0(opcode),
                    0x8001 => self.state.execute_8xy1(opcode),
                    0x8002 => self.state.execute_8xy2(opcode),
                    0x8003 => self.state.execute_8xy3(opcode),
                    0x8004 => self.state.execute_8xy4(opcode),
                    0x8005 => self.state.execute_8xy5(opcode),
                    0x8006 => self.state.execute_8xy6(opcode),
                    0x8007 => self.state.execute_8xy7(opcode),
                    0x800E => self.state.execute_8xyE(opcode),
                    _ => panic!("Unknown opcode {opcode:04X}"),
                }
            },
            0x9 => self.state.execute_9xy0(opcode),
            0xA => self.state.execute_Annn(opcode),
            0xB => self.state.execute_Bnnn(opcode),
            0xC => self.state.execute_Cxkk(opcode),
            0xD => self.state.execute_Dxyn(opcode),
            0xE => match opcode.0 & 0xF0FF {
                0xE09E => self.state.execute_Ex9E(opcode),
                0xE0A1 => self.state.execute_ExA1(opcode),
                _ => panic!("Unknown opcode {opcode:04X}"),
            },
            0xF => match opcode.0 & 0xF0FF {
                0xF007 => self.state.execute_Fx07(opcode),
                0xF00A => self.state.execute_Fx0A(opcode),
                0xF015 => self.state.execute_Fx15(opcode),
                0xF018 => self.state.execute_Fx18(opcode),
                0xF01E => self.state.execute_Fx1E(opcode),
                0xF029 => self.state.execute_Fx29(opcode),
                0xF033 => self.state.execute_Fx33(opcode),
                0xF055 => self.state.execute_Fx55(opcode),
                0xF065 => self.state.execute_Fx65(opcode),
                _ => panic!("Unknown opcode {opcode:04X}"),
            },
            _ => panic!("Unknown opcode {opcode:04X}"),
        };

        self.handle_timers();
    }
}

/// The execution methods returns 1 if the cached interpreter should be interrupted,
/// > 1 to request a cache invalidation of [beg, end) with the beg in the high order word,
/// 0 if everything is good to continue.
#[allow(non_snake_case)]
impl State {
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
}
