use crate::Chip8;
use crate::utils::*;

use dynasmrt::{dynasm, DynasmApi, x64::Assembler};

#[derive(Debug)]
pub enum Interrupts {
    UseInterpreter = 1,
}

impl Interrupts {
    pub fn make(int: Interrupts, arg: u16) -> u32 {
        (int as u32) << 16 | arg as u32
    }
}

impl From<u32> for Interrupts {
    fn from(i: u32) -> Self {
        match i {
            1 => Self::UseInterpreter,
            _ => panic!(),
        }
    }
}

impl Chip8 {
    /// Executes indefinitely using the JIT compiler.
    pub fn jit(&mut self) {
        // loop {
            if let Some(cache) = self.caches.get(self.PC) {
                let ret = cache.run();
                match Interrupts::from(ret >> 16) {
                    Interrupts::UseInterpreter => {
                        self.PC = ret as u16;
                        self.interpreter();
                    },
                }
            } else {
                let pc = self.PC;
                self.compile_block();
                self.PC = pc;
            }
        // }
    }

    fn compile_block(&mut self) {
        let pc = self.PC;
        let mut asm = Assembler::new().expect("Failed to create new assembler");

        'outer: loop {
            let opcode: u16 = ((self.memory[self.PC as usize] as u16) << 8) | (self.memory[self.PC as usize + 1] as u16);

            log(format!("Compiling opcode {:#04X} at {:#X}", opcode, self.PC));
            self.PC += 2;
            match (opcode >> 12) & 0xF {
                0x0 => {
                    match opcode {
                        0x00E0 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0x00EE => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        _ => println!("Unknown opcode {}", opcode),
                    }
                },
                0x1 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                        ; ret
                    );
                    break 'outer;
                },
                0x2 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                        ; ret
                    );
                    break 'outer;
                },
                    0x3 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                        ; ret
                    );
                    break 'outer;
                },
                0x4 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                        ; ret
                    );
                    break 'outer;
                },
                0x5 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                        ; ret
                    );
                    break 'outer;
                },
                0x6 => {
                    let x = ((opcode >> 8) & 0xF) as usize;
                    let kk = (opcode & 0xFF) as i8;
                    let addr = self.V.address(x as isize) as i64;
                    println!("addr {}: {:X}", x, addr);
                    dynasm!(asm
                        ; .arch x64
                        ; mov rbx, QWORD addr
                        ; mov BYTE [rbx], kk
                    );
                },
                0x7 => {
                    let x = ((opcode >> 8) & 0xF) as usize;
                    let kk = (opcode & 0xFF) as i8;
                    let addr = self.V.address(x as isize) as i64;
                    dynasm!(asm
                        ; .arch x64
                        ; mov rbx, QWORD addr
                        ; add BYTE [rbx], kk
                    );
                },
                0x8 => {
                    match opcode & 0xF00F {
                        0x8000 => {
                            let x = ((opcode >> 8) & 0xF) as usize;
                            let y = ((opcode >> 4) & 0xF) as usize;
                            dynasm!(asm
                                ; .arch x64
                                ; mov al, BYTE [self.V.address(y as isize) as i32]
                                ; mov BYTE [self.V.address(x as isize) as i32], al
                            );
                        },
                        0x8001 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0x8002 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0x8003 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0x8004 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0x8005 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0x8006 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0x8007 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0x800E => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        _ => println!("Unknown opcode {}", opcode),
                    }
                },
                0x9 => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                        ; ret
                    );
                    break 'outer;
                },
                0xA => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                        ; ret
                    );
                    break 'outer;
                },
                0xB => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                        ; ret
                    );
                    break 'outer;
                },
                0xC => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                        ; ret
                    );
                    break 'outer;
                },
                0xD => {
                    dynasm!(asm
                        ; .arch x64
                        ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                        ; ret
                    );
                    break 'outer;
                },
                0xE => {
                    match opcode & 0xF0FF {
                        0xE09E => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0xE0A1 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        _ => println!("Unknown opcode {}", opcode),
                    }
                },
                0xF => {
                    match opcode & 0xF0FF {
                        0xF007 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0xF00A => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0xF015 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0xF018 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0xF01E => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0xF029 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0xF033 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0xF055 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        0xF065 => {
                            dynasm!(asm
                                ; .arch x64
                                ; mov eax, DWORD Interrupts::make(Interrupts::UseInterpreter, self.PC - 2) as i32
                                ; ret
                            );
                            break 'outer;
                        },
                        _ => println!("Unknown opcode {}", opcode),
                    }
                },
                _ => println!("Unknown opcode {}", opcode),
            };
        }

        self.caches.create(pc, asm.finalize().unwrap());

        // self.timer += self.clock_delay;
        // while self.timer >= 1000.0 / 60.0 {
        //     if self.delay > 0 {
        //         self.delay -= 1;
        //     }

        //     if self.sound > 0 {
        //         // TODO: play sound
        //         self.sound -= 1;
        //     }
        //     self.timer -= 1000.0 / 60.0;
        // }
    }
}
