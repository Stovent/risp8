use risp8::{Chip8, ExecutionMethod, Risp8Answer, Risp8Command};

use kanal::{Sender, Receiver};

use pixels::{Pixels, SurfaceTexture};

use winit::event::*;
use winit::window::WindowBuilder;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::dpi::PhysicalSize;

use std::thread;

const BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];
const WHITE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

/// The context used to run the app.
struct ExecutionContext {
    pub send: Sender<Risp8Command>,
    pub recv: Receiver<Risp8Answer>,
    pub screen: [[bool; 64]; 32],
    pub is_playing: bool,
    pub execution_method: ExecutionMethod,

    pub update_window: bool,
}

fn main() {
    let mut args = std::env::args();
    let exec = args.next().unwrap();
    if args.len() != 1 {
        println!("Usage: {} <ROM>", exec);
        std::process::exit(1);
    }

    let romfile = args.next().unwrap();
    let (mut chip8, chip8_in, chip8_out) = Chip8::new(&romfile)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });
    println!("Successfully opened ROM \"{}\"", romfile);

    thread::spawn(move || {
        chip8.run();
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

    let mut ctx = ExecutionContext {
        send: chip8_in,
        recv: chip8_out,
        screen: [[false; 64]; 32],
        is_playing: false,
        execution_method: ExecutionMethod::Interpreter,

        update_window: true, // To set the window title at the first event loop.
    };

    event_loop.run(move |event, _, flow| {
        while !ctx.recv.is_empty() {
            let Ok(answer) = ctx.recv.recv() else {
                *flow = ControlFlow::Exit;
                return;
            };

            match answer {
                Risp8Answer::Screen(s) => ctx.screen = s,
                _ => (), // TODO: sound.
            }
        }

        if ctx.update_window {
            let playing = if ctx.is_playing { "Running" } else { "Paused" };
            let exec = match ctx.execution_method {
                ExecutionMethod::Interpreter => "Interpreter",
                ExecutionMethod::Jit => "Jit",
            };

            window.set_title(&format!("{playing} - {exec} - risp8"));
            ctx.update_window = false;
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                ctx.send.send(Risp8Command::Exit).unwrap();
                *flow = ControlFlow::Exit;
            },
            Event::MainEventsCleared => {
                if ctx.send.send(Risp8Command::GetScreen).is_err() {
                    *flow = ControlFlow::Exit;
                }
                chip8_to_pixels(&ctx.screen, pixels.get_frame_mut());
                pixels.render().unwrap();
            },
            Event::DeviceEvent { event: DeviceEvent::Key(key), .. } => {
                handle_keyboard(&key, &mut ctx);
            },
            _ => (),
        }
    });
}

fn handle_keyboard(key: &KeyboardInput, ctx: &mut ExecutionContext) {
    if key.virtual_keycode.is_some() {
        match key.virtual_keycode.unwrap() {
            VirtualKeyCode::Numpad0 => { ctx.send.send(Risp8Command::SetKey(0x0, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::Numpad7 => { ctx.send.send(Risp8Command::SetKey(0x1, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::Numpad8 => { ctx.send.send(Risp8Command::SetKey(0x2, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::Numpad9 => { ctx.send.send(Risp8Command::SetKey(0x3, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::Numpad4 => { ctx.send.send(Risp8Command::SetKey(0x4, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::Numpad5 => { ctx.send.send(Risp8Command::SetKey(0x5, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::Numpad6 => { ctx.send.send(Risp8Command::SetKey(0x6, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::Numpad1 => { ctx.send.send(Risp8Command::SetKey(0x7, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::Numpad2 => { ctx.send.send(Risp8Command::SetKey(0x8, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::Numpad3 => { ctx.send.send(Risp8Command::SetKey(0x9, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::NumpadDivide   => { ctx.send.send(Risp8Command::SetKey(0xA, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::NumpadMultiply => { ctx.send.send(Risp8Command::SetKey(0xB, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::NumpadSubtract => { ctx.send.send(Risp8Command::SetKey(0xC, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::NumpadAdd      => { ctx.send.send(Risp8Command::SetKey(0xD, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::NumpadEnter    => { ctx.send.send(Risp8Command::SetKey(0xE, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::Return         => { ctx.send.send(Risp8Command::SetKey(0xE, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::NumpadDecimal  => { ctx.send.send(Risp8Command::SetKey(0xF, key.state == ElementState::Pressed)).unwrap() },
            VirtualKeyCode::I => {
                ctx.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Interpreter)).unwrap();
                ctx.execution_method = ExecutionMethod::Interpreter;
                ctx.update_window = true;
            },
            VirtualKeyCode::J => {
                ctx.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Jit)).unwrap();
                ctx.execution_method = ExecutionMethod::Jit;
                ctx.update_window = true;
            },
            VirtualKeyCode::P => {
                if key.state == ElementState::Pressed {
                    if ctx.is_playing {
                        ctx.send.send(Risp8Command::Pause).unwrap();
                        ctx.is_playing = false;
                    } else {
                        ctx.send.send(Risp8Command::Play).unwrap();
                        ctx.is_playing = true;
                    }
                    ctx.update_window = true;
                }
            },
            _ => (),
        }
    }
}

fn chip8_to_pixels(screen: &[[bool; 64]; 32], pixels: &mut [u8]) {
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
