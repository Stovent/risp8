#[cfg(feature = "gui")]
mod gui;
#[cfg(feature = "tui")]
mod tui;

use risp8::Chip8;

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

    // gui::gui_main(chip8, chip8_in, chip8_out);
    let mut app = tui::TuiApp::new(); app.run(chip8, chip8_in, chip8_out).unwrap();
}
