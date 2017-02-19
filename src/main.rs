extern crate sdl2;

mod chip8vm;
use chip8vm::Chip8VM;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

use std::time::Duration;
use std::env;

static key_map: [u32; 0x10] = [
	120, 49, 50, 51, 113, 119, 101, 97, 115, 100, 122, 99, 52, 114, 102, 118
];

fn main() {
	let args: Vec<_> = env::args().collect();
	if args.len() < 2 {
		println!("super-rs8 <path/to/rom>");
		return;
	}

	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let window = video_subsystem.window("super-rs8", 640, 320).position_centered().resizable().build().unwrap();
	let mut renderer = window.renderer().present_vsync().build().unwrap();
	let mut timer = sdl_context.timer().unwrap();
	let mut event_pump = sdl_context.event_pump().unwrap();

	let mut vm = Chip8VM::new();
	vm.load_rom(&args[1]);

	let mut running = true;
	let mut clock_speed: u32 = 600;

	let mut last_frame: u32 = 0;
	let mut rest_clocks: f32 = 0f32;
	loop {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
					return
				}
				Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
					running = !running;
				}
				Event::KeyDown {keycode: Some(Keycode::Return), ..} => {
					vm = Chip8VM::new();
					vm.load_rom(&args[1]);
				}
				Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
					if clock_speed > 100 {
						clock_speed -= 100;
					}
					println!("Clock speed: {}hz", clock_speed);
				}
				Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
					clock_speed += 100;
					println!("Clock speed: {}hz", clock_speed);
				}
				Event::KeyDown {keycode: Some(Keycode::Delete), ..} => {
					println!("Register dump:");
					vm.register_dump();
				}
				Event::KeyDown {keycode, ..} => {
					let key = keycode.unwrap() as u32;
					for i in 0..0x10 {
						if key == key_map[i] {
							vm.keys[i] = true;
						}
					}
				}
				Event::KeyUp {keycode, ..} => {
					let key = keycode.unwrap() as u32;
					for i in 0..0x10 {
						if key == key_map[i] {
							vm.keys[i] = false;
						}
					}
				}
				_ => { }
			}
		}
		let frame: u32 = ((timer.ticks() as f32) / 1000.0 * 60.0) as u32;
		if frame > last_frame {
			last_frame = frame;
			if running {
				let clocks_per_frame: f32 = clock_speed as f32 / 60f32;
				let clocks: f32 = rest_clocks + clocks_per_frame;
				rest_clocks = clocks % 1f32;
				vm.do_frame(clocks as u32);
			}
		}

		let mut width: u32 = 0;
		let mut height: u32 = 0;
		{
			let window = renderer.window_mut().unwrap();
			let size = window.size();
			width = size.0;
			height = size.1;
		}
		draw(&vm, &mut renderer, width, height);
		renderer.present();	
	}
}

fn screen_pos(x: u32, y:u32, swidth: u32, sheight: u32) -> (u32, u32) {
	let scale: f32 = if swidth >= sheight * 2 { sheight as f32 / 64f32 } else { swidth as f32 / 128f32 };
	let x_offset: f32 = if swidth >= sheight * 2 { (swidth as f32 - scale * 128f32) / 2f32 } else { 0f32 };
	let y_offset: f32 = if swidth >= sheight * 2 { 0f32 } else { (sheight as f32 - scale * 64f32) / 2f32 };
	((scale * (x as f32) + x_offset) as u32, (scale * (y as f32) + y_offset) as u32)
}

fn screen_rect(x: u32, y:u32, swidth: u32, sheight: u32) -> Rect {
	let pos1 = screen_pos(x, y, swidth, sheight);
	let pos2 = screen_pos(x + 1, y + 1, swidth, sheight);
	Rect::new(pos1.0 as i32, pos1.1 as i32, pos2.0 - pos1.0, pos2.1 - pos1.1)
}

fn draw(vm: &Chip8VM, renderer: &mut Renderer, swidth: u32, sheight: u32) {
	renderer.set_draw_color(Color::RGB(0, 0, 0));
	renderer.clear();

	renderer.set_draw_color(Color::RGB(255, 255, 255));
	for i in 0..128 {
		for j in 0..64 {
			if vm.vram[j * 128 + i] {
				renderer.fill_rect(screen_rect(i as u32, j as u32, swidth, sheight));
			}
		}
	}
}