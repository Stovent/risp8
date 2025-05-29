use risp8::{Chip8, ExecutionMethod, Risp8Answer, Risp8Command};

use kanal::{Sender, Receiver};

use pixels::{Pixels, SurfaceTexture};

use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, Event, RawKeyEvent, StartCause, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;

use std::thread;
use std::time::{Duration, Instant};

const BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];
const WHITE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

/// Copies the chip8 screen to a RGBA buffer.
fn chip8_screen_to_rgba(screen: &[[bool; 64]; 32], rgba: &mut [u8]) {
    for (i, pixel) in rgba.chunks_exact_mut(4).enumerate() {
        let y = i / 64;
        let x = i % 64;
        pixel.copy_from_slice(if screen[y][x] {
            &WHITE
        } else {
            &BLACK
        });
    }
}

/// The context used to run the app.
struct ExecutionContext {
    pub send: Sender<Risp8Command>,
    pub recv: Receiver<Risp8Answer>,
    pub is_playing: bool,
    pub execution_method: ExecutionMethod,

    pub update_window: bool,
}

impl ExecutionContext {
    fn generate_window_title(&self) -> String {
        let playing = if self.is_playing { "Running" } else { "Paused" };
        let exec = match self.execution_method {
            ExecutionMethod::Interpreter => "Interpreter",
            ExecutionMethod::CachedInterpreter => "Cached interpreter",
            ExecutionMethod::CachedInterpreter2 => "Cached interpreter 2",
            ExecutionMethod::CachedInterpreter3 => "Cached interpreter 3",
            ExecutionMethod::Jit => "Jit",
        };

        format!("{playing} - {exec} - risp8")
    }

    fn handle_keyboard(&mut self, key: &RawKeyEvent) {
        let k = key.physical_key;
        let pressed = key.state == ElementState::Pressed;
        // println!("{k:?} {pressed}");
        match k {
            // Chip8 key
            KeyCode::KeyV   | KeyCode::Numpad0 => { self.send.send(Risp8Command::SetKey(0x0, pressed)).unwrap() },
            KeyCode::Digit3 | KeyCode::Numpad7 => { self.send.send(Risp8Command::SetKey(0x1, pressed)).unwrap() },
            KeyCode::Digit4 | KeyCode::Numpad8 => { self.send.send(Risp8Command::SetKey(0x2, pressed)).unwrap() },
            KeyCode::Digit5 | KeyCode::Numpad9 => { self.send.send(Risp8Command::SetKey(0x3, pressed)).unwrap() },
            KeyCode::KeyE   | KeyCode::Numpad4 => { self.send.send(Risp8Command::SetKey(0x4, pressed)).unwrap() },
            KeyCode::KeyR   | KeyCode::Numpad5 => { self.send.send(Risp8Command::SetKey(0x5, pressed)).unwrap() },
            KeyCode::KeyT   | KeyCode::Numpad6 => { self.send.send(Risp8Command::SetKey(0x6, pressed)).unwrap() },
            KeyCode::KeyD   | KeyCode::Numpad1 => { self.send.send(Risp8Command::SetKey(0x7, pressed)).unwrap() },
            KeyCode::KeyF   | KeyCode::Numpad2 => { self.send.send(Risp8Command::SetKey(0x8, pressed)).unwrap() },
            KeyCode::KeyG   | KeyCode::Numpad3 => { self.send.send(Risp8Command::SetKey(0x9, pressed)).unwrap() },
            KeyCode::KeyC   | KeyCode::NumpadDivide =>   { self.send.send(Risp8Command::SetKey(0xA, pressed)).unwrap() },
            KeyCode::KeyB   | KeyCode::NumpadMultiply => { self.send.send(Risp8Command::SetKey(0xB, pressed)).unwrap() },
            KeyCode::Digit6 | KeyCode::NumpadSubtract => { self.send.send(Risp8Command::SetKey(0xC, pressed)).unwrap() },
            KeyCode::KeyY   | KeyCode::NumpadAdd =>      { self.send.send(Risp8Command::SetKey(0xD, pressed)).unwrap() },
            KeyCode::KeyH   | KeyCode::NumpadEnter =>    { self.send.send(Risp8Command::SetKey(0xE, pressed)).unwrap() },
            KeyCode::KeyN   | KeyCode::NumpadDecimal =>  { self.send.send(Risp8Command::SetKey(0xF, pressed)).unwrap() },
            // Control
            KeyCode::KeyI => {
                self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Interpreter)).unwrap();
                self.execution_method = ExecutionMethod::Interpreter;
                self.update_window = true;
            },
            KeyCode::KeyK => {
                self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter)).unwrap();
                self.execution_method = ExecutionMethod::CachedInterpreter;
                self.update_window = true;
            },
            KeyCode::KeyL => {
                self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter2)).unwrap();
                self.execution_method = ExecutionMethod::CachedInterpreter2;
                self.update_window = true;
            },
            KeyCode::KeyM => {
                self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter3)).unwrap();
                self.execution_method = ExecutionMethod::CachedInterpreter3;
                self.update_window = true;
            },
            KeyCode::KeyJ => {
                self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Jit)).unwrap();
                self.execution_method = ExecutionMethod::Jit;
                self.update_window = true;
            },
            KeyCode::KeyS  => if pressed { self.send.send(Risp8Command::SingleStep).unwrap() },
            KeyCode::KeyP => {
                if pressed {
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

fn print_usage_and_exit(exec: &str) -> ! {
    println!("Usage: {exec} <ROM>");
    std::process::exit(1);
}

fn main() {
    let mut args = std::env::args();
    let exec = args.next().unwrap();
    if args.len() != 1 {
        print_usage_and_exit(&exec);
    }

    let romfile = args.next().unwrap();
    let (mut chip8, chip8_in, chip8_out) = Chip8::new(&romfile)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

    let event_loop = EventLoop::new().unwrap();
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
        is_playing: false,
        execution_method: ExecutionMethod::Interpreter,

        update_window: true, // To set the window title at the first event loop.
    };

    thread::spawn(move || {
        chip8.run();
    });

    event_loop.run(move |event, _, flow| {
        while !ctx.recv.is_empty() {
            let Ok(answer) = ctx.recv.recv() else {
                flow.set_exit();
                return;
            };

            match answer {
                Risp8Answer::Screen(screen) => {
                    chip8_screen_to_rgba(&screen, pixels.frame_mut());
                    window.request_redraw();
                },
                _ => (), // TODO: sound.
            }
        }

        if ctx.update_window {
            window.set_title(&ctx.generate_window_title());
            ctx.update_window = false;
        }

        match event {
            Event::NewEvents(cause) => {
                match cause {
                    StartCause::ResumeTimeReached { .. } => {
                        if ctx.is_playing && ctx.send.send(Risp8Command::GetScreen).is_err() {
                            flow.set_exit();
                            return;
                        }
                    },
                    _ => (),
                }
            },
            Event::WindowEvent { event: evt, .. } => {
                match evt {
                    WindowEvent::CloseRequested => {
                        ctx.send.send(Risp8Command::Exit).unwrap();
                        flow.set_exit();
                        return;
                    },
                    WindowEvent::Resized(size) => {
                        let _ = pixels.resize_surface(size.width, size.height);
                    },
                    _ => (),
                }
            },
            Event::DeviceEvent { event: DeviceEvent::Key(key), .. } => {
                ctx.handle_keyboard(&key);
            },
            Event::RedrawRequested(_) => {
                window.pre_present_notify();
                pixels.render().unwrap();
            },
            Event::AboutToWait => if ctx.is_playing {
                flow.set_wait_until(Instant::now() + Duration::from_millis(16));
            } else {
                flow.set_wait();
            },
            _ => (),
        }
    }).unwrap();
}
