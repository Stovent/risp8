use crate::cache::*;
use crate::Chip8;
use crate::utils::*;

use rand::Rng;

pub struct Jit {
    chip8: Chip8,
	resume: u16,
	caches: Caches,
}

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

impl Jit {
    pub fn new(rom: &str, freq: usize) -> Result<Jit, String> {
        log_str("New JIT");
        let chip8 = Chip8::new(rom, freq)?;
        Ok(Self {
            chip8: chip8,
			resume: 0x200,
            caches: Caches::new(),
        })
    }

    pub fn run(&mut self) {
		loop {
			if let Some(cache) = self.caches.get(self.chip8.PC) {
				let ret = cache.execute();
				match Interrupts::from(ret >> 16) {
					Interrupts::UseInterpreter => {
						self.chip8.PC = ret as u16;
						self.chip8.interpreter();
					},
				}
			} else {
				let pc = self.chip8.PC;
				self.compile_block();
				self.chip8.PC = pc;
			}
		}
    }

	fn compile_block(&mut self) {
        let cache = self.caches.get_or_create(self.chip8.PC);

		'outer: loop {
			let opcode: u16 = ((self.chip8.memory[self.chip8.PC as usize] as u16) << 8) | (self.chip8.memory[self.chip8.PC as usize + 1] as u16);

			log(format!("Compiling opcode {:#04X} at {:#X}", opcode, self.chip8.PC));
			self.chip8.PC += 2;
			match (opcode >> 12) & 0xF {
				0x0 => {
					match opcode {
						0x00E0 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0x00EE => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						_ => println!("Unknown opcode {}", opcode),
					}
				},
				0x1 => {
					cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
					break 'outer;
				},
				0x2 => {
					cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
					break 'outer;
				},
				0x3 => {
					cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
					break 'outer;
				},
				0x4 => {
					cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
					break 'outer;
				},
				0x5 => {
					cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
					break 'outer;
				},
				0x6 => {
					let x = ((opcode >> 8) & 0xF) as usize;
					let kk = (opcode & 0xFF) as u8;
					cache.mov_mem_imm8(arr8_to_u32(&self.chip8.V, x), kk);
				},
				0x7 => {
					let x = ((opcode >> 8) & 0xF) as usize;
					let kk = (opcode & 0xFF) as u8;
					cache.add_mem_imm8(arr8_to_u32(&self.chip8.V, x), kk);
				},
				0x8 => {
					match opcode & 0xF00F {
						0x8000 => {
							let x = ((opcode >> 8) & 0xF) as usize;
							let y = ((opcode >> 4) & 0xF) as usize;
							cache.mov_eax_mem(arr8_to_u32(&self.chip8.V, x));
							cache.mov_mem_eax(arr8_to_u32(&self.chip8.V, y));
						},
						0x8001 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0x8002 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0x8003 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0x8004 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0x8005 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0x8006 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0x8007 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0x800E => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						_ => println!("Unknown opcode {}", opcode),
					}
				},
				0x9 => {
					cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
					break 'outer;
				},
				0xA => {
					cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
					break 'outer;
				},
				0xB => {
					cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
					break 'outer;
				},
				0xC => {
					cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
					break 'outer;
				},
				0xD => {
					cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
					break 'outer;
				},
				0xE => {
					match opcode & 0xF0FF {
						0xE09E => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0xE0A1 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						_ => println!("Unknown opcode {}", opcode),
					}
				},
				0xF => {
					match opcode & 0xF0FF {
						0xF007 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0xF00A => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0xF015 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0xF018 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0xF01E => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0xF029 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0xF033 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0xF055 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						0xF065 => {
							cache.ret(Interrupts::make(Interrupts::UseInterpreter, self.chip8.PC - 2));
							break 'outer;
						},
						_ => println!("Unknown opcode {}", opcode),
					}
				},
				_ => println!("Unknown opcode {}", opcode),
			};
		}

		self.chip8.dec_timer_ms += self.chip8.clock_delay;
		while self.chip8.dec_timer_ms >= 1000.0 / 60.0 {
			if self.chip8.delay > 0 {
				self.chip8.delay -= 1;
			}

			if self.chip8.sound > 0 {
				// TODO: play sound
				self.chip8.sound -= 1;
			}
			self.chip8.dec_timer_ms -= 1000.0 / 60.0;
		}
	}
}
