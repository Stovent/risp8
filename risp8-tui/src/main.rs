use std::io::stdout;

use crossterm::ExecutableCommand;
use crossterm::event::{self, KeyCode::Char, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, window_size};

use risp8::{Chip8, ExecutionMethod, Receiver, Risp8Answer, Risp8Command, Screen, Sender, State, DEFAULT_SCREEN};

use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::Text;
use ratatui::widgets::{Block, Widget};

pub struct TuiApp {
    is_playing: bool,
    execution_method: ExecutionMethod,

    screen_widget: ScreenWidget,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            execution_method: ExecutionMethod::Interpreter,

            screen_widget: ScreenWidget::default(),
        }
    }

    pub fn run(&mut self, mut chip8: Chip8, chip8_in: Sender<Risp8Command>, chip8_out: Receiver<Risp8Answer>) -> std::io::Result<()> {
        if let Ok(size) = window_size() { // Not supported on Windows
            if size.columns < State::SCREEN_WIDTH as u16 || size.rows < State::SCREEN_HEIGHT as u16 {
                println!("Warning: terminal is smaller than Chip8 screen");
            }
        }

        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(stdout());
        let terminal_options = TerminalOptions {
            viewport: Viewport::Fullscreen,
        };
        let mut terminal = Terminal::with_options(backend, terminal_options)?;
        terminal.clear()?;

        std::thread::spawn(move || {
            chip8.run();
        });

        loop {
            while let Ok(Some(answer)) = chip8_out.try_recv() {
                match answer {
                    Risp8Answer::Screen(s) => self.screen_widget.screen = s,
                    Risp8Answer::PlaySound => (),
                    Risp8Answer::StopSound => (),
                }
            }
            chip8_in.send(Risp8Command::GetScreen).unwrap();

            terminal.draw(|frame| self.ui(frame))?;

            if self.handle_keyboard(&chip8_in)? { // Exit requested
                break;
            };
        }

        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn ui(&self, frame: &mut Frame) {
        use Constraint::{Length, Min};

        let frame_area = frame.area();
        let [title_area, screen_block_area] = Layout::vertical([Length(1), Min(0)]).areas(frame_area);
        let screen_block = Block::bordered();
        let screen_area = screen_block.inner(screen_block_area);

        let screen_title = self.get_title(screen_area);
        let screen_block = screen_block.title(screen_title);

        let frame_title = format!("<q> Quit | <p> Play | <iklmj> Execution | {}x{}", frame_area.width, frame_area.height);
        let frame_title = if frame_area.width > frame_title.len() as u16 { // Always show the important information.
            Text::from(frame_title).centered()
        } else {
            Text::from(frame_title).left_aligned()
        };

        frame.render_widget(frame_title, title_area);
        frame.render_widget(screen_block, screen_block_area);
        self.screen_widget.render(screen_area, frame.buffer_mut());
    }

    fn get_title(&self, screen_area: Rect) -> String {
        let playing = if self.is_playing { "Running" } else { "Paused" };
        let exec = match self.execution_method {
            ExecutionMethod::Interpreter => "Interpreter",
            ExecutionMethod::CachedInterpreter => "Cached Interpreter 1",
            ExecutionMethod::CachedInterpreter2 => "Cached Interpreter 2",
            ExecutionMethod::CachedInterpreter3 => "Cached Interpreter 3",
            ExecutionMethod::Jit => "JIT",
        };
        format!("{playing} | {exec} | {}x{}", screen_area.width, screen_area.height)
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
                            _ => (),
                        }
                    }
                }
            }
        }

        Ok(false)
    }
}

#[derive(Copy, Clone, Debug)]
struct ScreenWidget {
    screen: Screen,
}

impl Default for ScreenWidget {
    fn default() -> Self {
        Self {
            screen: DEFAULT_SCREEN,
        }
    }
}

impl Widget for &ScreenWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        let width_ratio = State::SCREEN_WIDTH as f32 / area.width as f32;
        let height_ratio = State::SCREEN_HEIGHT as f32 / area.height as f32;

        for y in 0..area.height {
            for x in 0..area.width {
                let yy = (y as f32 * height_ratio) as usize;
                let xx = (x as f32 * width_ratio) as usize;
                let color = if self.screen[yy][xx] { Color::White } else { Color::Black };

                let pos = (area.x + x, area.y + y);
                buf[pos].set_fg(color).set_bg(color);
            }
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

    let rom_file = args.next().unwrap();
    let (chip8, chip8_in, chip8_out) = Chip8::new(&rom_file)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

    let mut app = TuiApp::new();
    app.run(chip8, chip8_in, chip8_out).unwrap();
}
