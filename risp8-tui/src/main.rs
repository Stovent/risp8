use std::io::stdout;

use crossterm::ExecutableCommand;
use crossterm::event::{self, KeyCode::Char, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, window_size};

use risp8::{Chip8, ExecutionMethod, Receiver, Risp8Answer, Risp8Command, Screen, Sender, State, DEFAULT_SCREEN};

use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
use ratatui::backend::CrosstermBackend;
use ratatui::style::Color;
use ratatui::symbols::Marker;
use ratatui::widgets::{Block, canvas::{Canvas, Rectangle}};

pub struct TuiApp {
    marker: Marker,
    screen: Screen,
    is_playing: bool,
    execution_method: ExecutionMethod,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            marker: Marker::Block,
            screen: DEFAULT_SCREEN,
            is_playing: false,
            execution_method: ExecutionMethod::Interpreter,
        }
    }

    pub fn run(&mut self, mut chip8: Chip8, chip8_in: Sender<Risp8Command>, chip8_out: Receiver<Risp8Answer>) -> std::io::Result<()> {
        if let Ok(size) = window_size() { // Not supported on Windows
            if size.columns < risp8::State::SCREEN_WIDTH as u16 || size.rows < risp8::State::SCREEN_HEIGHT as u16 {
                println!("Warning: terminal is smaller than Chip8 screen");
            }
        }

        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(stdout());
        let terminal_options = TerminalOptions {
            viewport: Viewport::Fullscreen,
        };
        let mut terminal = Terminal::with_options(backend, terminal_options).unwrap();
        terminal.clear()?;

        std::thread::spawn(move || {
            chip8.run();
        });

        loop {
            while let Ok(Some(answer)) = chip8_out.try_recv() {
                match answer {
                    Risp8Answer::Screen(s) => self.screen = s,
                    Risp8Answer::PlaySound => (),
                    Risp8Answer::StopSound => (),
                }
            }
            chip8_in.send(Risp8Command::GetScreen).unwrap();

            terminal.draw(|frame| self.ui(frame)).unwrap();

            if self.handle_keyboard(&chip8_in)? { // Exit requested
                break;
            };
        }

        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn ui(&self, frame: &mut Frame) {
        let playing = if self.is_playing { "Running" } else { "Paused" };
        let exec = match self.execution_method {
            ExecutionMethod::Interpreter => "Interpreter",
            ExecutionMethod::CachedInterpreter => "Cached Interpreter",
            ExecutionMethod::CachedInterpreter2 => "Cached Interpreter 2",
            ExecutionMethod::CachedInterpreter3 => "Cached Interpreter 3",
            ExecutionMethod::Jit => "JIT",
        };
        let screen_title = format!("{playing} | {exec}");

        let screen_block = Block::bordered()
            .title(screen_title);

        let canvas: Canvas<'_, _> = Canvas::default()
            .block(screen_block)
            .x_bounds([0.0, State::SCREEN_WIDTH as f64])
            .y_bounds([0.0, State::SCREEN_HEIGHT as f64])
            .marker(self.marker)
            .paint(|ctx| {
                for y in 0..State::SCREEN_HEIGHT {
                    for x in 0..State::SCREEN_WIDTH {
                        let color = if self.screen[y][x] { Color::White } else { Color::Black };

                        let rect = Rectangle {
                            x: x as f64,
                            y: (State::SCREEN_HEIGHT - 1 - y) as f64,
                            width: 1.0,
                            height: 1.0,
                            color,
                        };
                        ctx.draw(&rect);
                    }
                }
            });

        let size = frame.size();
        frame.render_widget(canvas, size);
    }

    fn change_marker(&mut self) {
        self.marker = match self.marker {
            Marker::Dot => Marker::Braille,
            Marker::Braille => Marker::Block,
            Marker::Block => Marker::HalfBlock,
            Marker::HalfBlock => Marker::Bar,
            Marker::Bar => Marker::Dot,
        };
    }

    /// Returns `Ok(true)` when exit is requested.
    fn handle_keyboard(&mut self, chip8_in: &Sender<Risp8Command>) -> Result<bool, std::io::Error> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Repeat {
                    let pressed = key.kind == KeyEventKind::Press;

                    // println!("{:?} {pressed}", key.code);
                    match key.code {
                        // " ' ( - are the keys for azerty keyboards.
                        Char('v') => chip8_in.send(Risp8Command::SetKey(0x0, pressed)).unwrap(),
                        Char('3') | Char('"') => chip8_in.send(Risp8Command::SetKey(0x1, pressed)).unwrap(),
                        Char('4') | Char('\'') => chip8_in.send(Risp8Command::SetKey(0x2, pressed)).unwrap(),
                        Char('5') | Char('(') => chip8_in.send(Risp8Command::SetKey(0x3, pressed)).unwrap(),
                        Char('e') => chip8_in.send(Risp8Command::SetKey(0x4, pressed)).unwrap(),
                        Char('r') => chip8_in.send(Risp8Command::SetKey(0x5, pressed)).unwrap(),
                        Char('t') => chip8_in.send(Risp8Command::SetKey(0x6, pressed)).unwrap(),
                        Char('d') => chip8_in.send(Risp8Command::SetKey(0x7, pressed)).unwrap(),
                        Char('f') => chip8_in.send(Risp8Command::SetKey(0x8, pressed)).unwrap(),
                        Char('g') => chip8_in.send(Risp8Command::SetKey(0x9, pressed)).unwrap(),
                        Char('c') => chip8_in.send(Risp8Command::SetKey(0xA, pressed)).unwrap(),
                        Char('b') => chip8_in.send(Risp8Command::SetKey(0xB, pressed)).unwrap(),
                        Char('6') | Char('-') => chip8_in.send(Risp8Command::SetKey(0xC, pressed)).unwrap(),
                        Char('y') => chip8_in.send(Risp8Command::SetKey(0xD, pressed)).unwrap(),
                        Char('h') => chip8_in.send(Risp8Command::SetKey(0xE, pressed)).unwrap(),
                        Char('n') => chip8_in.send(Risp8Command::SetKey(0xF, pressed)).unwrap(),
                        _ => (),
                    }

                    if pressed {
                        // Control keys are treated when pressed, not released.
                        match key.code {
                            Char('q') => return Ok(true),
                            Char('p') => if self.is_playing {
                                chip8_in.send(Risp8Command::Pause).unwrap();
                                self.is_playing = false;
                            } else {
                                chip8_in.send(Risp8Command::Play).unwrap();
                                self.is_playing = true;
                            },
                            Char('s') => chip8_in.send(Risp8Command::SingleStep).unwrap(),
                            Char('i') => {
                                chip8_in.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Interpreter)).unwrap();
                                self.execution_method = ExecutionMethod::Interpreter;
                            },
                            Char('k') => {
                                chip8_in.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter)).unwrap();
                                self.execution_method = ExecutionMethod::CachedInterpreter;
                            },
                            Char('l') => {
                                chip8_in.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter2)).unwrap();
                                self.execution_method = ExecutionMethod::CachedInterpreter2;
                            },
                            Char('m') => {
                                chip8_in.send(Risp8Command::SetExecutionMethod(ExecutionMethod::CachedInterpreter3)).unwrap();
                                self.execution_method = ExecutionMethod::CachedInterpreter3;
                            },
                            Char('j') => {
                                chip8_in.send(Risp8Command::SetExecutionMethod(ExecutionMethod::Jit)).unwrap();
                                self.execution_method = ExecutionMethod::Jit;
                            },
                            Char('a') => self.change_marker(),
                            _ => (),
                        }
                    }
                }
            }
        }

        Ok(false)
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

    let rom_file = args.next().unwrap();
    let (chip8, chip8_in, chip8_out) = Chip8::new(&rom_file)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

    let mut app = TuiApp::new();
    app.run(chip8, chip8_in, chip8_out).unwrap();
}
