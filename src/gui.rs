use risp8::{Chip8, ExecutionMethod, Risp8Answer, Risp8Command};

use kanal::{Sender, Receiver};

use pixels::{Pixels, SurfaceTexture};

use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, Event, RawKeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

use std::thread;
use std::time::Duration;

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
struct GuiContext {
    pub send: Sender<Risp8Command>,
    pub recv: Receiver<Risp8Answer>,
    pub is_playing: bool,
    pub execution_method: ExecutionMethod,

    pub update_window: bool,
}

impl GuiContext {
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
        let PhysicalKey::Code(k) = key.physical_key else {
            return;
        };

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

pub fn gui_main(mut chip8: Chip8, chip8_in: Sender<Risp8Command>, chip8_out: Receiver<Risp8Answer>) {
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

    let mut ctx = GuiContext {
        send: chip8_in,
        recv: chip8_out,
        is_playing: false,
        execution_method: ExecutionMethod::Interpreter,

        update_window: true, // To set the window title at the first event loop.
    };

    thread::spawn(move || {
        chip8.run();
    });

    event_loop.run(move |event, target| {
        while !ctx.recv.is_empty() {
            let Ok(answer) = ctx.recv.recv() else {
                target.exit();
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
                            target.exit();
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
                        target.exit();
                        return;
                    },
                    WindowEvent::Resized(size) => {
                        let _ = pixels.resize_surface(size.width, size.height);
                    },
                    WindowEvent::RedrawRequested => {
                        window.pre_present_notify();
                        pixels.render().unwrap();
                    },
                    _ => (),
                }
            },
            Event::DeviceEvent { event: DeviceEvent::Key(key), .. } => {
                ctx.handle_keyboard(&key);
            },
            Event::AboutToWait => if ctx.is_playing {
                target.set_control_flow(ControlFlow::wait_duration(Duration::from_millis(16)));
            },
            _ => (),
        }
    }).unwrap();
}
