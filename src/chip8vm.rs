extern crate rand;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::num::Wrapping;

static char_mem: [u8; 0x50] = [
	0xF0, 0x90, 0x90, 0x90, 0xF0,
	0x20, 0x60, 0x20, 0x20, 0x70,
	0xF0, 0x10, 0xF0, 0x80, 0xF0,
	0xF0, 0x10, 0xF0, 0x10, 0xF0,
	0x90, 0x90, 0xF0, 0x10, 0x10,
	0xF0, 0x80, 0xF0, 0x10, 0xF0,
	0xF0, 0x80, 0xF0, 0x90, 0xF0,
	0xF0, 0x10, 0x20, 0x40, 0x40,
	0xF0, 0x90, 0xF0, 0x90, 0xF0,
	0xF0, 0x90, 0xF0, 0x10, 0xF0,
	0xF0, 0x90, 0xF0, 0x90, 0x90,
	0xE0, 0x90, 0xE0, 0x90, 0xE0,
	0xF0, 0x80, 0x80, 0x80, 0xF0,
	0xE0, 0x90, 0x90, 0x90, 0xE0,
	0xF0, 0x80, 0xF0, 0x80, 0xF0,
	0xF0, 0x80, 0xF0, 0x80, 0x80
];


pub struct Chip8VM {
	ram: [u8; 0x1000],
	pub vram: [bool; 128 * 64],
	v: [u8; 0x10],
	stack: [u16; 0x10],
	pub keys: [bool; 0x10],
	i: u16,
	dt: u8,
	st: u8,
	pc: u16,
	sp: i8,
	extended_mode: bool,
}

impl Chip8VM {
	pub fn new() -> Chip8VM {
		let mut vm = Chip8VM {
			ram: [0; 0x1000],
			vram: [false; 128 * 64],
			v: [0; 0x10],
			stack: [0; 0x10],
			keys: [false; 0x10],
			i: 0,
			dt: 0,
			st: 0,
			pc: 0x0200,
			sp: 0,
			extended_mode: false,
		};
		for i in 0..0x50 {
			vm.ram[i] = char_mem[i];
		}
		vm
	}

	pub fn load_rom(&mut self, path: &str) {
		let abspath = fs::canonicalize(path).expect("Invalid path.");
		let mut file = File::open(abspath.to_str().expect("Invalid path.")).unwrap();
		let mut buf = [0u8; 0xE00];
		let bytes_read = file.read(&mut buf).unwrap();
		for i in 0..bytes_read {
			self.ram[0x200 + i] = buf[i];
		}
	}

	pub fn do_frame(&mut self, cycles: u32) {
		for i in 0..cycles {
			self.do_cycle();
		}
		if self.dt > 0 {
			self.dt -= 1;
		}
		if self.st > 0 {
			self.st -= 1;
		}
	}

	fn do_cycle(&mut self) {
		let opcode: u16 = self.ram[self.pc as usize] as u16 * 0x100 + self.ram[(self.pc + 1) as usize % 0x1000] as u16;
		let x = ((opcode & 0x0F00) >> 8) as usize;
		let y = ((opcode & 0x00F0) >> 4) as usize;
		let byte = (opcode & 0x00FF) as u8;
		let nnn = opcode & 0x0FFF;
		let n = (opcode & 0x000F) as u8;
		//println!("PC: {:03X} opcode: {:04X}", self.pc, opcode);
		match opcode & 0xF000 {
			0x0000 => {
				if opcode & 0x00F0 == 0x00C0 { // SCD nibble

				} else if opcode != 0x0000 { // not NOP
					match opcode {
						0x00E0 => { // CLS
							for i in 0..(128 * 64) {
								self.vram[i] = false;
							}
						}
						0x00EE => { // RET
							if self.sp >= 0 {
								self.sp -= 1;
								self.pc = self.stack[self.sp as usize] - 2;
							}
						}
						0x00FB => { // SCR

						}
						0x00FC => { // SCL

						}
						0x00FD => { // EXIT
							self.pc -= 2;
						}
						0x00FE => { // LOW
							self.extended_mode = false;
						}
						0x00FF => { // HIGH
							self.extended_mode = true;
						}
						_ => {
							unknown(opcode);
						}
					}
				}
			}
			0x1000 => { // JP addr
				self.pc = nnn - 2;
			}
			0x2000 => { // CALL addr
				if self.sp < 15 {
					self.stack[self.sp as usize] = self.pc + 2;
					self.sp += 1;
					self.pc = nnn - 2;
				} else {
					println!("Stack pointer is too high!");
				}
			}
			0x3000 => { // SE Vx, byte
				if self.v[x] == byte {
					self.pc += 2;
				}
			}
			0x4000 => { // SNE Vx, byte
				if self.v[x] != byte {
					self.pc += 2;
				}
			}
			0x5000 => {
				if opcode & 0xF00F == 0x5000 { // SE Vx, Vy
					if self.v[x] == self.v[y] {
						self.pc += 2;
					}
				} else {
					unknown(opcode);
				}
			}
			0x6000 => { // LD Vx, byte
				self.v[x] = byte;
			}
			0x7000 => { // ADD Vx, byte
				self.v[x] = self.v[x].wrapping_add(byte);
			}
			0x8000 => { // LD Vx, Vy
				match opcode & 0xF00F {
					0x8000 => { // LD Vx, Vy
						self.v[x] = self.v[y];
					}
					0x8001 => { // OR Vx, Vy
						self.v[x] |= self.v[y];
					}
					0x8002 => { // AND Vx, Vy
						self.v[x] &= self.v[y];
					}
					0x8003 => { // XOR Vx, Vy
						self.v[x] ^= self.v[y];
					}
					0x8004 => { // ADD Vx, Vy
						self.v[0xF] = if (self.v[x] as u16 + self.v[y] as u16) > 0xFF { 1 } else { 0 };
						self.v[x] = self.v[x].wrapping_add(self.v[y]);
					}
					0x8005 => { // SUB Vx, Vy
						self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 }; 
						self.v[x] = self.v[x].wrapping_sub(self.v[y]);
					}
					0x8006 => { // SHR Vx
						self.v[0xF] = if (self.v[x] & 1) != 0 { 1 } else { 0 };
						self.v[x] = self.v[x].wrapping_shr(1);
					}
					0x8007 => { // SUBN Vx, Vy
						self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 }; 
						self.v[x] = self.v[y].wrapping_sub(self.v[x]);
					}
					0x800E => { // SHL Vx
						self.v[0xF] = if self.v[x] & 0x80 != 0 { 1 } else { 0 };
						self.v[x] = self.v[x].wrapping_shl(1);
					}
					_ => {
						unknown(opcode);
					}
				}
			}
			0x9000 => {
				if opcode & 0xF00F == 0x9000 { // SNE Vx, Vy
					if self.v[x] != self.v[y] {
						self.pc += 2;
					}
				} else {
					unknown(opcode);
				}
			}
			0xA000 => { // LD I, addr
				self.i = nnn;
			}
			0xB000 => { // JP V0, addr
				self.pc = self.v[0] as u16 + nnn - 2;
			}
			0xC000 => { // RND Vx, byte
				self.v[x] = rand::random::<u8>() & byte;
			}
			0xD000 => { // DRW Vx, Vy, nibble
				if !self.extended_mode {
					let x: u8 = self.v[x] % 64;
					let y: u8 = self.v[y] % 32;
					let w: u8 = if x <= 56 { 8 } else { 64 - x };
					let h: u8 = if y <= 32 - n { n } else { 32 - n };
					self.v[0xF] = 0;
					for j in 0..h {
						for i in 0..w {
							let pix: bool = if self.ram[self.i as usize + j as usize] & ((128 >> i) as u8) > 0 { true } else { false };
							if pix {
								let addr: usize = ((j + y) as u16 * 256 + (i + x) as u16 * 2) as usize;
								if addr < 8192 {
									if self.vram[addr] {
										self.v[0xF] = 1;
									}
									self.vram[addr] = !self.vram[addr];
									self.vram[addr + 1] = !self.vram[addr + 1];
									self.vram[addr + 128] = !self.vram[addr + 128];
									self.vram[addr + 129] = !self.vram[addr + 129];
								}
							}
						}
					}
				}
			}
			0xE000 => {
				match opcode & 0xF0FF {
					0xE09E => { // SKP Vx
						if self.keys[self.v[x] as usize] {
							self.pc += 2;
						}
					}
					0xE0A1 => { // SKNP Vx
						if !self.keys[self.v[x] as usize] {
							self.pc += 2;
						}
					}
					_ => {
						unknown(opcode);
					}
				}
			}
			0xF000 => {
				match opcode & 0xF0FF {
					0xF007 => { // LD Vx, DT
						self.v[x] = self.dt;
					}
					0xF00A => { // LD Vx, K
						let mut key_press = false;
						for i in 0..0xF {
							if self.keys[i] {
								key_press = true;
								self.v[x] = i as u8;
								break;
							}
						}
						if !key_press {
							self.pc -= 2;
						}
					}
					0xF015 => { // LD DT, Vx
						self.dt = self.v[x];
					}
					0xF018 => { // LD ST, Vx
						self.st = self.v[x];
					}
					0xF01E => { // ADD I, Vx
						self.i += self.v[x] as u16;
						self.i %= 0x1000;
					}
					0xF029 => { // LD F, Vx
						self.i = self.v[x] as u16 * 5;
					}
					0xF033 => { // LD B, Vx
						self.ram[self.i as usize] = self.v[x] / 100;
						self.ram[(self.i + 1) as usize] = self.v[x] / 10;
						self.ram[(self.i + 2) as usize] = self.v[x] % 10;
					}
					0xF055 => { // LD [I], Vx
						for i in 0..(x + 1) {
							self.ram[self.i as usize + i] = self.v[i];
						}
					}
					0xF065 => { // LD Vx, [i]
						for i in 0..(x + 1) {
							self.v[i] = self.ram[self.i as usize + i];
						}
					}
					_ => {
						unknown(opcode);
					}
				}
			}
			_ => {
				unknown(opcode); 
			}
		}
		self.pc += 2;
		self.pc %= 0x1000;
	}

	pub fn register_dump(&self) {
		for i in 0..0x10 {
			print!("V{:01X}: {:02X} {}", i, self.v[i], if (i + 1) % 4 == 0 { "\n" } else { "" });
		}
		print!("Stack: ");
		for i in 0..self.sp {
			print!("{:03X} ", self.stack[i as usize]);
		}
		println!("");
		println!("I : {:03X}", self.i);
		println!("PC: {:03X}", self.pc);
		println!("SP: {:02X}", self.sp);
		println!("DT: {:02X}", self.dt);
		println!("ST: {:02X}", self.st);
	}
}

fn unknown(opcode: u16)
{
	println!("Unknown opcode: {:04X}", opcode);
}