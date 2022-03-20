use risp8::Chip8;

use winit::event::*;
use winit::window::WindowBuilder;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::dpi::PhysicalSize;

use pixels::*;

use std::cell::UnsafeCell;
use std::sync::Arc;
use std::thread;

const BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];
const WHITE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

struct Obj<T>(UnsafeCell<T>);
unsafe impl<T> std::marker::Sync for Obj<T> {}

fn main() {
    let mut args = std::env::args();
    let exec = args.next().unwrap();
    if args.len() != 1 {
        println!("Usage: {} <ROM>", exec);
        std::process::exit(1);
    }

    let romfile = args.next().unwrap();
    let chip8 = Chip8::new(&romfile)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });
    println!("Successfully opened ROM \"{}\"", romfile);

    let chip8_thread = Arc::new(Obj(UnsafeCell::new(chip8)));
    let chip8_gui = Arc::clone(&chip8_thread);

    let run_thread = Arc::new(Obj(UnsafeCell::new(true)));
    let run_gui = Arc::clone(&run_thread);

    let _thread_join = thread::spawn(move || {
        unsafe {
            while *run_thread.0.get() {
                (*chip8_thread.0.get()).jit();
                // (*chip8_thread.0.get()).interpreter();
            }
        }
    });

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("risp8")
        .with_inner_size(PhysicalSize::new(640, 320))
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(64, 32, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                unsafe {
                    *run_gui.0.get() = false;
                    *flow = ControlFlow::Exit;
                }
            },
            Event::MainEventsCleared => {
                unsafe {
                    draw(&(*chip8_gui.0.get()).screen, pixels.get_frame());
                    pixels.render().unwrap();
                }
            },
            Event::DeviceEvent { event: DeviceEvent::Key(key), .. } => {
                unsafe {
                    handle_keyboard(chip8_gui.0.get().as_mut().unwrap(), &key);
                }
            },
            _ => (),
        }
    });
}

fn handle_keyboard(chip8: &mut Chip8, key: &KeyboardInput) {
    if key.virtual_keycode.is_some() {
        match key.virtual_keycode.unwrap() {
            VirtualKeyCode::Numpad0 => { chip8.set_key(0x0, key.state == ElementState::Pressed) },
            VirtualKeyCode::Numpad7 => { chip8.set_key(0x1, key.state == ElementState::Pressed) },
            VirtualKeyCode::Numpad8 => { chip8.set_key(0x2, key.state == ElementState::Pressed) },
            VirtualKeyCode::Numpad9 => { chip8.set_key(0x3, key.state == ElementState::Pressed) },
            VirtualKeyCode::Numpad4 => { chip8.set_key(0x4, key.state == ElementState::Pressed) },
            VirtualKeyCode::Numpad5 => { chip8.set_key(0x5, key.state == ElementState::Pressed) },
            VirtualKeyCode::Numpad6 => { chip8.set_key(0x6, key.state == ElementState::Pressed) },
            VirtualKeyCode::Numpad1 => { chip8.set_key(0x7, key.state == ElementState::Pressed) },
            VirtualKeyCode::Numpad2 => { chip8.set_key(0x8, key.state == ElementState::Pressed) },
            VirtualKeyCode::Numpad3 => { chip8.set_key(0x9, key.state == ElementState::Pressed) },
            VirtualKeyCode::NumpadDivide   => { chip8.set_key(0xA, key.state == ElementState::Pressed) },
            VirtualKeyCode::NumpadMultiply => { chip8.set_key(0xB, key.state == ElementState::Pressed) },
            VirtualKeyCode::NumpadSubtract => { chip8.set_key(0xC, key.state == ElementState::Pressed) },
            VirtualKeyCode::NumpadAdd      => { chip8.set_key(0xD, key.state == ElementState::Pressed) },
            VirtualKeyCode::NumpadEnter    => { chip8.set_key(0xE, key.state == ElementState::Pressed) },
            VirtualKeyCode::NumpadDecimal  => { chip8.set_key(0xF, key.state == ElementState::Pressed) },
            _ => (),
        }
    }
}

fn draw(screen: &[[bool; 64]; 32], pixels: &mut [u8]) {
    for (i, pixel) in pixels.chunks_exact_mut(4).enumerate() {
        let y = (i / 64) as usize;
        let x = (i % 64) as usize;
        pixel.copy_from_slice(if screen[y][x] {
            &WHITE
        } else {
            &BLACK
        });
    }
}
