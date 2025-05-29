#![allow(non_snake_case)]

use std::fs::File;
use std::io::Read;
use rand::Rng;

pub struct Chip8 {
	SP: usize,
	PC: u16,
	I: u16,
	stack: [u16; 16],
	V: [u8; 16],
	memory: [u8; 4096],
	delay: u8,
	sound: u8,
	screen: [bool; 2048],
	keys: [bool; 16],
	
	last_key: u8,
	clockDelay: f64,
	decDelay: f64,
	instructionCount: usize,
	pub rom_opened: bool,
}

impl Chip8 {
	pub fn new(rom: &str, freq: usize) -> Result<Chip8, String> {
		let mut core = Chip8 {
			SP: 0,
			PC: 512,
			I: 0,
			stack: [0; 16],
			V: [0; 16],
			memory: [0; 4096],
			delay: 0,
			sound: 0,
			screen: [false; 2048],
			keys: [false; 16],
			
			last_key: 255,
			clockDelay: 1.0 / freq as f64,
			decDelay: freq as f64 / 60.0,
			instructionCount: 0,
			rom_opened: false,
		};
		core.load_font();
		core.load_rom(rom)?;
		Ok(core)
	}
	
	pub fn close_rom(self) {}
	
	fn load_rom(&mut self, filename: &str) -> Result<usize, String> {
		let mut input = match File::open(filename) {
			Ok(f) => f,
			Err(e) => return Err(format!("Could not open ROM: {}", e)),
		};
		
		match input.read(&mut self.memory[512..4096]) {
			Ok(size) => {
				self.rom_opened = true;
				println!("Successfully opened ROM \"{}\" (size: {} bytes)", filename, size);
				Ok(size)
			},
			Err(e) => Err(format!("Could not read from ROM: {}", e)),
		}
	}
	
	fn load_font(&mut self) {
		self.memory[0..80].copy_from_slice(&[
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
		]);
	}
	
	fn clear_screen(&mut self) {
		self.screen = [false; 2048];
	}
	
	pub fn set_key(&mut self, key: usize, pressed: bool) {
		self.keys[key] = pressed;
		self.last_key = key as u8;
	}
	
	fn draw(&mut self, x: usize, y: usize, n: u8) {
		for j in 0..n {
			let line: u8 = self.memory[(self.I + j as u16) as usize];
			
			for i in 0..8u8 {
				if line & (0x80 >> i) != 0 {
					let index: usize = ((self.V[y] + j) as usize * 64 + (self.V[x] + i) as usize) % 2048;
					
					if self.screen[index] {
						self.screen[index] = false;
						self.V[15] = 1;
					}
					else {
						self.screen[index] = true;
						self.V[15] = 0;
					}
				}
			}
		}
	}
	
	pub fn interpreter(&mut self) {
		let opcode: u16 = ((self.memory[self.PC as usize] as u16) << 8) | (self.memory[self.PC as usize + 1] as u16);
		self.PC += 2;
		
		match (opcode >> 12) & 0xF {
			0x0 => {
				match opcode {
					0x00E0 => self.clear_screen(),
					0x00EE => {
						if self.SP > 0 {
							self.SP -= 1;
							self.PC = self.stack[self.SP];
						}
						else {
							println!("-- Stack underflow (RET 0x00EE)");
						}
					},
					_ => println!("Unknown opcode {}", opcode),
				}
			},
			0x1 => self.PC = opcode & 0x0FFF,
			0x2 => {
				if self.SP < 15 {
					self.stack[self.SP] = self.PC;
					self.SP += 1;
					self.PC = opcode & 0x0FFF;
				}
				else {
					println!("-- Stack overflow (CALL 0x2nnn)");
				}
			},
			0x3 => {
				let x: usize = ((opcode >> 8) & 0xF) as usize;
				let kk: u8 = (opcode & 0xFF) as u8;
				if self.V[x] == kk { self.PC += 2; }
			},
			0x4 => {
				let x: usize = ((opcode >> 8) & 0xF) as usize;
				let kk: u8 = (opcode & 0xFF) as u8;
				if self.V[x] != kk { self.PC += 2; }
			},
			0x5 => {
				let x: usize = ((opcode >> 8) & 0xF) as usize;
				let y: usize = ((opcode >> 4) & 0xF) as usize;
				if self.V[x] == self.V[y] { self.PC += 2; }
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
						}
						else {
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
						}
						else {
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
				if self.V[x] != self.V[y] { self.PC += 2; }
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
			0xE => {
				match opcode & 0xF0FF {
					0xE09E => {
						let x: usize = ((opcode >> 8) & 0xF) as usize;
						if self.keys[self.V[x] as usize] { self.PC += 2 }
					},
					0xE0A1 => {
						let x: usize = ((opcode >> 8) & 0xF) as usize;
						if !self.keys[self.V[x] as usize] { self.PC += 2 }
					},
					_ => println!("Unknown opcode {}", opcode),
				}
			},
			0xF => {
				match opcode & 0xF0FF {
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
						self.memory[(self.I+1) as usize] = (self.V[x] - self.memory[self.I as usize] * 100) / 10;
						self.memory[(self.I+2) as usize] = (self.V[x] - self.memory[self.I as usize] * 100) - self.memory[(self.I+1) as usize] * 10;
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
				}
			},
			_ => println!("Unknown opcode {}", opcode),
		};
		
		self.instructionCount += 1;
		if self.instructionCount as f64 >= self.decDelay {
			if self.delay > 0 {
				self.delay -= 1;
			}

			if self.sound > 0 {
				// TODO: play sound
				self.sound -= 1;
			}
			self.instructionCount = 0;
		}

		std::thread::sleep(std::time::Duration::from_secs_f64(self.clockDelay));
	}
}
