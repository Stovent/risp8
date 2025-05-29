// use risp8::Chip8;
use risp8::jit::Jit;

fn main() {
    // let mut core = match Chip8::new("ROM/MAZE.ch8", 20000000) {
    //     Ok(c) => c,
    //     Err(e) => {
    //         println!("{}", e);
    //         return;
    //     }
    // };

    // for _i in 0..1000 {
    //     core.interpreter();
    // }

    let mut jit = match Jit::new("ROM/MAZE.ch8", 20000000) {
        Ok(c) => c,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    jit.run();
}
