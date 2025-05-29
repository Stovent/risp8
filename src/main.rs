use risp8::Chip8;

fn main() {
    let mut chip8 = match Chip8::new("ROM/MAZE.ch8", 20000000) {
        Ok(c) => c,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    // for _i in 0..1000 {
    //     chip8.interpreter();
    // }

    chip8.jit();
}
