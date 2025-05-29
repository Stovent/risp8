use std::thread;
use std::time::Duration;

use pixels::{Pixels, SurfaceTexture};

use risp8::{Chip8, ExecutionMethod, Receiver, Risp8Answer, Risp8Command, Sender, State};

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

const BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];
const WHITE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

/// The context used to run the app.
struct App {
    pub send: Sender<Risp8Command>,
    pub recv: Receiver<Risp8Answer>,
    pub is_playing: bool,
    pub execution_method: ExecutionMethod,

    pub update_title: bool,
    window: Option<Window>,
    pixels: Option<Pixels>,
}

impl App {
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

    fn handle_keyboard(&mut self, event: KeyEvent) {
        let pressed = event.state == ElementState::Pressed;

        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };

        match code {
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
                self.update_title = true;
            },
            KeyCode::KeyK => {
                self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter)).unwrap();
                self.execution_method = ExecutionMethod::CachedInterpreter;
                self.update_title = true;
            },
            KeyCode::KeyL => {
                self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter2)).unwrap();
                self.execution_method = ExecutionMethod::CachedInterpreter2;
                self.update_title = true;
            },
            KeyCode::KeyM => {
                self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter3)).unwrap();
                self.execution_method = ExecutionMethod::CachedInterpreter3;
                self.update_title = true;
            },
            KeyCode::KeyJ => {
                self.send.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Jit)).unwrap();
                self.execution_method = ExecutionMethod::Jit;
                self.update_title = true;
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
                    self.update_title = true;
                }
            },
            _ => (),
        }
    }

    fn pixels<'a>(&'a self) -> &'a Pixels {
        self.pixels.as_ref().unwrap()
    }

    fn pixels_mut<'a>(&'a mut self) -> &'a mut Pixels {
        self.pixels.as_mut().unwrap()
    }

    fn window<'a>(&'a self) -> &'a Window {
        self.window.as_ref().unwrap()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("risp8")
            .with_inner_size(PhysicalSize::new(640, 320));

        let window = event_loop.create_window(window_attributes).unwrap();
        let pixels = new_pixels(&window);

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
        // println!("{event_loop:?} {window_id:?} {event:?}");

        match event {
            WindowEvent::Resized(size) => {
                let _ = self.pixels_mut().resize_surface(size.width, size.height);
            },
            WindowEvent::CloseRequested => {
                self.send.send(Risp8Command::Exit).unwrap();
                event_loop.exit();
            },
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard(event);
            },
            WindowEvent::RedrawRequested => {
                self.window().pre_present_notify();
                self.pixels().render().unwrap();
            },
            _ => (),
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::ResumeTimeReached { .. } => {
                // This is either a timer resumed or any other kind of event, poll the screen anyway.
                if self.is_playing && self.send.send(Risp8Command::GetScreen).is_err() {
                    event_loop.exit();
                }
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        while !self.recv.is_empty() {
            let Ok(answer) = self.recv.recv() else {
                event_loop.exit();
                return;
            };

            match answer {
                Risp8Answer::Screen(screen) => {
                    chip8_screen_to_rgba(&screen, self.pixels_mut().frame_mut());
                    self.window().request_redraw();
                },
                _ => (), // TODO: sound.
            }
        }

        if self.update_title {
            self.window().set_title(&self.generate_window_title());
            self.update_title = false;
        }

        if self.is_playing {
            event_loop.set_control_flow(ControlFlow::wait_duration(Duration::from_millis(16)));
        }
    }
}

/// Creates a new Pixels renderer.
fn new_pixels(window: &Window) -> Pixels {
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    Pixels::new(State::SCREEN_WIDTH as u32, State::SCREEN_HEIGHT as u32, surface_texture).unwrap()
}

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

pub fn gui_main(mut chip8: Chip8, chip8_in: Sender<Risp8Command>, chip8_out: Receiver<Risp8Answer>) {
    let event_loop = EventLoop::new().unwrap();

    let mut app = App {
        send: chip8_in,
        recv: chip8_out,
        is_playing: false,
        execution_method: ExecutionMethod::Interpreter,

        update_title: true, // To set the window title at the first event loop.
        window: None,
        pixels: None,
    };

    thread::spawn(move || {
        chip8.run();
    });

    event_loop.run_app(&mut app).expect("");
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

    let rom_file = args.next().unwrap();
    let (chip8, chip8_in, chip8_out) = Chip8::new(&rom_file)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

    gui_main(chip8, chip8_in, chip8_out);
}
