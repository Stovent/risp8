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

pub trait Address {
    fn address_u32(&mut self, offset: isize) -> u32;
}

impl<T, const N: usize> Address for [T; N] {
    fn address_u32(&mut self, offset: isize) -> u32 {
        unsafe {
            let ptr = self.as_ptr();
            ptr.offset(offset as isize) as u32
        }
    }
}
