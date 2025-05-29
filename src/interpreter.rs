use crate::Chip8;

use rand::Rng;

impl Chip8 {
    /// Executes a single instruction using the interpreter mode.
    pub fn interpreter(&mut self) {
        let opcode: u16 = ((self.memory[self.PC as usize] as u16) << 8) | (self.memory[self.PC as usize + 1] as u16);
        println!("opcode {:04X} at {:#X}", opcode, self.PC);
        self.PC += 2;

        match (opcode >> 12) & 0xF {
            0x0 => match opcode {
                0x00E0 => self.clear_screen(),
                0x00EE => {
                    if self.SP > 0 {
                        self.SP -= 1;
                        self.PC = self.stack[self.SP];
                    } else {
                        println!("-- Stack underflow (RET 0x00EE)");
                    }
                },
                _ => println!("Unknown opcode {}", opcode),
            },
            0x1 => self.PC = opcode & 0x0FFF,
            0x2 => {
                if self.SP < 15 {
                    self.stack[self.SP] = self.PC;
                    self.SP += 1;
                    self.PC = opcode & 0x0FFF;
                } else {
                    println!("-- Stack overflow (CALL 0x2nnn)");
                }
            },
            0x3 => {
                let x: usize = ((opcode >> 8) & 0xF) as usize;
                let kk: u8 = (opcode & 0xFF) as u8;
                if self.V[x] == kk {
                    self.PC += 2;
                }
            },
            0x4 => {
                let x: usize = ((opcode >> 8) & 0xF) as usize;
                let kk: u8 = (opcode & 0xFF) as u8;
                if self.V[x] != kk {
                    self.PC += 2;
                }
            },
            0x5 => {
                let x: usize = ((opcode >> 8) & 0xF) as usize;
                let y: usize = ((opcode >> 4) & 0xF) as usize;
                if self.V[x] == self.V[y] {
                    self.PC += 2;
                }
            },
            0x6 => {
                let x: usize = ((opcode >> 8) & 0xF) as usize;
                let kk: u8 = (opcode & 0xFF) as u8;
                self.V[x] = kk;
            },
            0x7 => {
                let x: usize = ((opcode >> 8) & 0xF) as usize;
                let kk: u8 = (opcode & 0xFF) as u8;
                self.V[x] += kk;
            },
            0x8 => {
                match opcode & 0xF00F {
                    0x8000 => {
                        let x: usize = ((opcode >> 8) & 0xF) as usize;
                        let y: usize = ((opcode >> 4) & 0xF) as usize;
                        self.V[x] = self.V[y];
                    },
                    0x8001 => {
                        let x: usize = ((opcode >> 8) & 0xF) as usize;
                        let y: usize = ((opcode >> 4) & 0xF) as usize;
                        self.V[x] |= self.V[y];
                    },
                    0x8002 => {
                        let x: usize = ((opcode >> 8) & 0xF) as usize;
                        let y: usize = ((opcode >> 4) & 0xF) as usize;
                        self.V[x] &= self.V[y];
                    },
                    0x8003 => {
                        let x: usize = ((opcode >> 8) & 0xF) as usize;
                        let y: usize = ((opcode >> 4) & 0xF) as usize;
                        self.V[x] ^= self.V[y];
                    },
                    0x8004 => {
                        let x: usize = ((opcode >> 8) & 0xF) as usize;
                        let y: usize = ((opcode >> 4) & 0xF) as usize;
                        if self.V[x] as u16 + self.V[y] as u16 > 255 {
                            self.V[15] = 1;
                        }
                        self.V[x] += self.V[y];
                    },
                    0x8005 => {
                        let x: usize = ((opcode >> 8) & 0xF) as usize;
                        let y: usize = ((opcode >> 4) & 0xF) as usize;
                        if self.V[x] > self.V[y] {
                            self.V[15] = 1;
                        } else {
                            self.V[15] = 0;
                        }
                        self.V[x] -= self.V[y];
                    },
                    0x8006 => {
                        let x: usize = ((opcode >> 8) & 0xF) as usize;
                        // let y: usize = ((opcode >> 4) & 0xF) as usize;
                        self.V[15] = self.V[x] & 1;
                        self.V[x] >>= 1;
                    },
                    0x8007 => {
                        let x: usize = ((opcode >> 8) & 0xF) as usize;
                        let y: usize = ((opcode >> 4) & 0xF) as usize;
                        if self.V[y] > self.V[x] {
                            self.V[15] = 1;
                        } else {
                            self.V[15] = 0;
                        }
                        self.V[x] = self.V[y] - self.V[x];
                    },
                    0x800E => {
                        let x: usize = ((opcode >> 8) & 0xF) as usize;
                        // let y: usize = ((opcode >> 4) & 0xF) as usize;
                        self.V[15] = (self.V[x] & 0x80) >> 7;
                        self.V[x] <<= 1;
                    },
                    _ => println!("Unknown opcode {}", opcode),
                }
            },
            0x9 => {
                let x: usize = ((opcode >> 8) & 0xF) as usize;
                let y: usize = ((opcode >> 4) & 0xF) as usize;
                if self.V[x] != self.V[y] {
                    self.PC += 2;
                }
            },
            0xA => self.I = opcode & 0x0FFF,
            0xB => self.PC = (opcode & 0x0FFF) + self.V[0] as u16,
            0xC => {
                let x: usize = ((opcode >> 8) & 0xF) as usize;
                let kk: u8 = (opcode & 0xFF) as u8;
                self.V[x] = rand::thread_rng().gen_range(0, 256) as u16 as u8 & kk;
            },
            0xD => {
                let x: usize = ((opcode >> 8) & 0xF) as usize;
                let y: usize = ((opcode >> 4) & 0xF) as usize;
                let n: u8 = (opcode & 0xFF) as u8;
                self.draw(x, y, n);
            },
            0xE => match opcode & 0xF0FF {
                0xE09E => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    if self.keys[self.V[x] as usize] {
                        self.PC += 2
                    }
                },
                0xE0A1 => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    if !self.keys[self.V[x] as usize] {
                        self.PC += 2
                    }
                },
                _ => println!("Unknown opcode {}", opcode),
            },
            0xF => match opcode & 0xF0FF {
                0xF007 => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    self.V[x] = self.delay;
                },
                0xF00A => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    self.last_key = 255;
                    while self.last_key > 15 {}
                    self.V[x] = self.last_key;
                },
                0xF015 => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    self.delay = self.V[x];
                },
                0xF018 => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    self.sound = self.V[x];
                },
                0xF01E => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    self.I += self.V[x] as u16;
                },
                0xF029 => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    self.I = self.V[x] as u16 * 5;
                },
                0xF033 => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    self.memory[self.I as usize] = self.V[x] / 100;
                    self.memory[(self.I + 1) as usize] = (self.V[x] - self.memory[self.I as usize] * 100) / 10;
                    self.memory[(self.I + 2) as usize] = (self.V[x] - self.memory[self.I as usize] * 100) - self.memory[(self.I + 1) as usize] * 10;
                },
                0xF055 => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    for i in 0..=x {
                        self.memory[(self.I as usize + i) as usize] = self.V[i];
                    }
                },
                0xF065 => {
                    let x: usize = ((opcode >> 8) & 0xF) as usize;
                    for i in 0..=x {
                        self.V[i] = self.memory[(self.I as usize + i) as usize];
                    }
                },
                _ => println!("Unknown opcode {}", opcode),
            },
            _ => println!("Unknown opcode {}", opcode),
        };

        self.dec_timer_ms += self.clock_delay;
        while self.dec_timer_ms >= 1000.0 / 60.0 {
            if self.delay > 0 {
                self.delay -= 1;
            }

            if self.sound > 0 {
                // TODO: play sound
                self.sound -= 1;
            }
            self.dec_timer_ms -= 1000.0 / 60.0;
        }

        std::thread::sleep(std::time::Duration::from_secs_f64(self.clock_delay));
    }
}
