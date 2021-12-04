use risp8::Chip8;

use winit::event::*;
use winit::window::WindowBuilder;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::dpi::PhysicalSize;

use pixels::*;

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;

const BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];
const WHITE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

fn main() {
    let freq = 1000u64;
    let chip8 = Chip8::new("ROM/PONG.ch8", freq as usize)
        .unwrap_or_else(|e| {
            println!("Failed to open ROM: {}", e);
            std::process::exit(1);
        });
    let chip8_thread = Arc::new(Mutex::new(chip8));
    let chip8_gui = Arc::clone(&chip8_thread);

    let run_thread = Arc::new(Mutex::new(true));
    let run_gui = Arc::clone(&run_thread);

    let _thread_join = thread::spawn(move || {
        while *run_thread.lock().unwrap() {
            chip8_thread.lock().unwrap().jit();
            // chip8_thread.lock().unwrap().interpreter();
            thread::sleep(Duration::from_micros(1_000_000 / freq));
        }
    });

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("risp8")
        .with_inner_size(PhysicalSize::<u32>::new(640, 320))
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
                *run_gui.lock().unwrap() = false;
                *flow = ControlFlow::Exit;
            },
            Event::MainEventsCleared => {
                draw(&chip8_gui.lock().unwrap().screen, pixels.get_frame());
                pixels.render().unwrap();
                thread::sleep(Duration::from_micros(16666));
            },
            Event::DeviceEvent { event: DeviceEvent::Key(key), .. } => {
                handle_keyboard(&mut chip8_gui.lock().unwrap(), &key);
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
