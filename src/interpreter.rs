use crate::Chip8;
use crate::opcode::Opcode;

use rand::Rng;

impl Chip8 {
    /// Executes a single instruction using the interpreter.
    pub fn interpreter(&mut self) {
        let opcode = Opcode((self.memory[self.PC as usize] as u16) << 8 | self.memory[self.PC as usize + 1] as u16);
        // #[cfg(debug_assertions)] println!("opcode {:04X} at {:#X}", opcode, self.PC);
        self.PC += 2;

        match opcode.0 >> 12 & 0xF {
            0x0 => match opcode.0 {
                0x00E0 => self.clear_screen(),
                0x00EE => {
                    if self.SP > 0 {
                        self.SP -= 1;
                        self.PC = self.stack[self.SP];
                    } else {
                        println!("Stack underflow (RET 0x00EE)");
                    }
                },
                _ => panic!("Unknown opcode {:04X}", opcode),
            },
            0x1 => self.PC = opcode.nnn(),
            0x2 => {
                if self.SP < 0xF {
                    self.stack[self.SP] = self.PC;
                    self.SP += 1;
                    self.PC = opcode.nnn();
                } else {
                    println!("Stack overflow (CALL 0x2nnn)");
                }
            },
            0x3 => {
                let (x, kk) = opcode.xkk();
                if self.V[x] == kk {
                    self.PC += 2;
                }
            },
            0x4 => {
                let (x, kk) = opcode.xkk();
                if self.V[x] != kk {
                    self.PC += 2;
                }
            },
            0x5 => {
                let (x, y) = opcode.xy();
                if self.V[x] == self.V[y] {
                    self.PC += 2;
                }
            },
            0x6 => {
                let (x, kk) = opcode.xkk();
                self.V[x] = kk;
            },
            0x7 => {
                let (x, kk) = opcode.xkk();
                self.V[x] = self.V[x].wrapping_add(kk);
            },
            0x8 => {
                match opcode.0 & 0xF00F {
                    0x8000 => {
                        let (x, y) = opcode.xy();
                        self.V[x] = self.V[y];
                    },
                    0x8001 => {
                        let (x, y) = opcode.xy();
                        self.V[x] |= self.V[y];
                    },
                    0x8002 => {
                        let (x, y) = opcode.xy();
                        self.V[x] &= self.V[y];
                    },
                    0x8003 => {
                        let (x, y) = opcode.xy();
                        self.V[x] ^= self.V[y];
                    },
                    0x8004 => {
                        let (x, y) = opcode.xy();
                        let (res, c) = self.V[x].overflowing_add(self.V[y]);
                        self.V[x] = res;
                        self.V[0xF] = c as u8;
                    },
                    0x8005 => {
                        let (x, y) = opcode.xy();
                        let (res, b) = self.V[x].overflowing_sub(self.V[y]);
                        self.V[x] = res;
                        self.V[0xF] = (!b) as u8;
                    },
                    0x8006 => {
                        let x = opcode.x();
                        // let y = opcode.y();
                        let c = self.V[x] & 1;
                        self.V[x] >>= 1;
                        self.V[0xF] = c;
                    },
                    0x8007 => {
                        let (x, y) = opcode.xy();
                        let (res, b) = self.V[y].overflowing_sub(self.V[x]);
                        self.V[x] = res;
                        self.V[0xF] = (!b) as u8;
                    },
                    0x800E => {
                        let x = opcode.x();
                        // let y = opcode.y();
                        let c = self.V[x] >> 7 & 1;
                        self.V[x] <<= 1;
                        self.V[0xF] = c;
                    },
                    _ => panic!("Unknown opcode {:04X}", opcode),
                }
            },
            0x9 => {
                let (x, y) = opcode.xy();
                if self.V[x] != self.V[y] {
                    self.PC += 2;
                }
            },
            0xA => self.I = opcode.nnn(),
            0xB => self.PC = opcode.nnn() + self.V[0] as u16,
            0xC => {
                let (x, kk) = opcode.xkk();
                self.V[x] = rand::thread_rng().gen_range(0, 256) as u8 & kk;
            },
            0xD => {
                let (x, y) = opcode.xy();
                let n = opcode.n();
                self.draw(x, y, n);
            },
            0xE => match opcode.0 & 0xF0FF {
                0xE09E => {
                    let x = opcode.x();
                    if self.keys[self.V[x] as usize] {
                        self.PC += 2;
                    }
                },
                0xE0A1 => {
                    let x = opcode.x();
                    if !self.keys[self.V[x] as usize] {
                        self.PC += 2;
                    }
                },
                _ => panic!("Unknown opcode {:04X}", opcode),
            },
            0xF => match opcode.0 & 0xF0FF {
                0xF007 => {
                    let x = opcode.x();
                    self.V[x] = self.delay;
                },
                0xF00A => {
                    let x = opcode.x();
                    self.wait_key(x);
                },
                0xF015 => {
                    let x = opcode.x();
                    self.delay = self.V[x];
                },
                0xF018 => {
                    let x = opcode.x();
                    self.sound = self.V[x];
                },
                0xF01E => {
                    let x = opcode.x();
                    self.I += self.V[x] as u16;
                },
                0xF029 => {
                    let x = opcode.x();
                    self.I = self.V[x] as u16 * 5;
                },
                0xF033 => {
                    let x = opcode.x();
                    self.memory[self.I as usize] = self.V[x] / 100;
                    self.memory[self.I as usize + 1] = (self.V[x] / 10) % 10;
                    self.memory[self.I as usize + 2] = self.V[x] % 10;
                },
                0xF055 => {
                    let x = opcode.x();
                    for i in 0..=x {
                        self.memory[self.I as usize + i] = self.V[i];
                    }
                },
                0xF065 => {
                    let x = opcode.x();
                    for i in 0..=x {
                        self.V[i] = self.memory[self.I as usize + i];
                    }
                },
                _ => panic!("Unknown opcode {:04X}", opcode),
            },
            _ => panic!("Unknown opcode {:04X}", opcode),
        };

        self.handle_timers();
    }
}
