#![allow(dead_code)]
#![allow(unused_must_use)]

pub fn log(msg: String) {
    println!("{}", msg);
}

pub fn log_str(msg: &str) {
    log(String::from(msg));
}

pub fn breakpoint() {
    let mut str = String::from("");
    std::io::stdin().read_line(&mut str);
}

pub fn arr8_to_u32(arr: &[u8; 16], offset: usize) -> u32 {
    unsafe {
        let ptr = arr.as_ptr();
        ptr.offset(offset as isize) as u32
    }
}
