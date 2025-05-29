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
    pub numpad_keyboard: bool,

    pub update_window: bool,
}

impl ExecutionContext {
    fn chip8_to_pixels(&self, pixels: &mut [u8]) {
        for (i, pixel) in pixels.chunks_exact_mut(4).enumerate() {
            let y = i / 64;
            let x = i % 64;
            pixel.copy_from_slice(if self.screen[y][x] {
                &WHITE
            } else {
                &BLACK
            });
        }
    }

    fn handle_keyboard(&mut self, key: &KeyboardInput) {
        if self.numpad_keyboard {
            self.keymap_numpad(key);
        } else {
            self.keymap_keyboard(key);
        }
    }

    /// Keymap on the keyboard.
    fn keymap_keyboard(&mut self, key: &KeyboardInput) {
        let k = key.virtual_keycode;
        // println!("{:#X} {k:?}", key.scancode);
        if k.is_some() {
            match key.virtual_keycode.unwrap() {
                VirtualKeyCode::V    => { self.send.send(Risp8Command::SetKey(0x0, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Key3 => { self.send.send(Risp8Command::SetKey(0x1, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Key4 => { self.send.send(Risp8Command::SetKey(0x2, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Key5 => { self.send.send(Risp8Command::SetKey(0x3, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::E    => { self.send.send(Risp8Command::SetKey(0x4, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::R    => { self.send.send(Risp8Command::SetKey(0x5, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::T    => { self.send.send(Risp8Command::SetKey(0x6, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::D    => { self.send.send(Risp8Command::SetKey(0x7, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::F    => { self.send.send(Risp8Command::SetKey(0x8, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::G    => { self.send.send(Risp8Command::SetKey(0x9, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::C    => { self.send.send(Risp8Command::SetKey(0xA, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::B    => { self.send.send(Risp8Command::SetKey(0xB, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Key6 => { self.send.send(Risp8Command::SetKey(0xC, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Y    => { self.send.send(Risp8Command::SetKey(0xD, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::H    => { self.send.send(Risp8Command::SetKey(0xE, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::N    => { self.send.send(Risp8Command::SetKey(0xF, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::I   => {
                    self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Interpreter)).unwrap();
                    self.execution_method = ExecutionMethod::Interpreter;
                    self.update_window = true;
                },
                VirtualKeyCode::K => {
                    self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter)).unwrap();
                    self.execution_method = ExecutionMethod::CachedInterpreter;
                    self.update_window = true;
                },
                VirtualKeyCode::L => {
                    self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter2)).unwrap();
                    self.execution_method = ExecutionMethod::CachedInterpreter2;
                    self.update_window = true;
                },
                VirtualKeyCode::M => {
                    self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter3)).unwrap();
                    self.execution_method = ExecutionMethod::CachedInterpreter3;
                    self.update_window = true;
                },
                VirtualKeyCode::J => {
                    self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Jit)).unwrap();
                    self.execution_method = ExecutionMethod::Jit;
                    self.update_window = true;
                },
                VirtualKeyCode::S  => if key.state == ElementState::Pressed { self.send.send(Risp8Command::SingleStep).unwrap() },
                VirtualKeyCode::P => {
                    if key.state == ElementState::Pressed {
                        if self.is_playing {
                            self.send.send(Risp8Command::Pause).unwrap();
                            self.is_playing = false;
                        } else {
                            self.send.send(Risp8Command::Play).unwrap();
                            self.is_playing = true;
                        }
                        self.update_window = true;
                    }
                },
                _ => (),
            }
        }
    }

    /// Keymap on the numpad.
    fn keymap_numpad(&mut self, key: &KeyboardInput) {
        let k = key.virtual_keycode;
        // println!("{:#X} {k:?}", key.scancode);
        if k.is_some() {
            match key.virtual_keycode.unwrap() {
                VirtualKeyCode::Numpad0 => { self.send.send(Risp8Command::SetKey(0x0, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Numpad7 => { self.send.send(Risp8Command::SetKey(0x1, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Numpad8 => { self.send.send(Risp8Command::SetKey(0x2, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Numpad9 => { self.send.send(Risp8Command::SetKey(0x3, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Numpad4 => { self.send.send(Risp8Command::SetKey(0x4, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Numpad5 => { self.send.send(Risp8Command::SetKey(0x5, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Numpad6 => { self.send.send(Risp8Command::SetKey(0x6, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Numpad1 => { self.send.send(Risp8Command::SetKey(0x7, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Numpad2 => { self.send.send(Risp8Command::SetKey(0x8, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Numpad3 => { self.send.send(Risp8Command::SetKey(0x9, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::NumpadDivide   => { self.send.send(Risp8Command::SetKey(0xA, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::NumpadMultiply => { self.send.send(Risp8Command::SetKey(0xB, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::NumpadSubtract => { self.send.send(Risp8Command::SetKey(0xC, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::NumpadAdd      => { self.send.send(Risp8Command::SetKey(0xD, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::NumpadEnter    => { self.send.send(Risp8Command::SetKey(0xE, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::Return         => { self.send.send(Risp8Command::SetKey(0xE, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::NumpadDecimal  => { self.send.send(Risp8Command::SetKey(0xF, key.state == ElementState::Pressed)).unwrap() },
                VirtualKeyCode::I => {
                    self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Interpreter)).unwrap();
                    self.execution_method = ExecutionMethod::Interpreter;
                    self.update_window = true;
                },
                VirtualKeyCode::K => {
                    self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter)).unwrap();
                    self.execution_method = ExecutionMethod::CachedInterpreter;
                    self.update_window = true;
                },
                VirtualKeyCode::L => {
                    self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter2)).unwrap();
                    self.execution_method = ExecutionMethod::CachedInterpreter2;
                    self.update_window = true;
                },
                VirtualKeyCode::M => {
                    self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter3)).unwrap();
                    self.execution_method = ExecutionMethod::CachedInterpreter3;
                    self.update_window = true;
                },
                VirtualKeyCode::J => {
                    self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Jit)).unwrap();
                    self.execution_method = ExecutionMethod::Jit;
                    self.update_window = true;
                },
                VirtualKeyCode::S  => if key.state == ElementState::Pressed { self.send.send(Risp8Command::SingleStep).unwrap() },
                VirtualKeyCode::P => {
                    if key.state == ElementState::Pressed {
                        if self.is_playing {
                            self.send.send(Risp8Command::Pause).unwrap();
                            self.is_playing = false;
                        } else {
                            self.send.send(Risp8Command::Play).unwrap();
                            self.is_playing = true;
                        }
                        self.update_window = true;
                    }
                },
                _ => (),
            }
        }
    }
}

fn print_usage_and_exit(exec: &str) -> ! {
    println!("Usage: {exec} [--keymap-numpad] <ROM>");
    std::process::exit(1);
}

fn main() {
    let mut args = std::env::args();
    let exec = args.next().unwrap();
    if args.len() == 0 || args.len() > 2 {
        print_usage_and_exit(&exec);
    }

    let mut numpad_keyboard = false;
    if args.len() == 2 {
        let keymap = args.next().unwrap();
        if keymap != "--keymap-numpad" {
            println!("Unrecognized argument `{keymap}`");
            print_usage_and_exit(&exec);
        }

        numpad_keyboard = true;
    }

    let romfile = args.next().unwrap();
    let (mut chip8, chip8_in, chip8_out) = Chip8::new(&romfile)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

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
        numpad_keyboard,

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
                ExecutionMethod::CachedInterpreter => "Cached interpreter",
                ExecutionMethod::CachedInterpreter2 => "Cached interpreter 2",
                ExecutionMethod::CachedInterpreter3 => "Cached interpreter 3",
                ExecutionMethod::Jit => "Jit",
            };

            window.set_title(&format!("{playing} - {exec} - risp8"));
            ctx.update_window = false;
        }

        match event {
            Event::WindowEvent { event: evt, .. } => {
                match evt {
                    WindowEvent::CloseRequested => {
                        ctx.send.send(Risp8Command::Exit).unwrap();
                        *flow = ControlFlow::Exit;
                    },
                    WindowEvent::Resized(size) => {
                        let _ = pixels.resize_surface(size.width, size.height);
                    },
                    _ => (),
                }
            },
            Event::MainEventsCleared => {
                if ctx.send.send(Risp8Command::GetScreen).is_err() {
                    *flow = ControlFlow::Exit;
                }
                ctx.chip8_to_pixels(pixels.frame_mut());
                pixels.render().unwrap();
            },
            Event::DeviceEvent { event: DeviceEvent::Key(key), .. } => {
                ctx.handle_keyboard(&key);
            },
            _ => (),
        }
    });
}
