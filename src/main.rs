mod gui;

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

    let romfile = args.next().unwrap();
    let (chip8, chip8_in, chip8_out) = Chip8::new(&romfile)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

    gui::gui_main(chip8, chip8_in, chip8_out);
}
