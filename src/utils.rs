#![allow(dead_code)]
#![allow(unused_must_use)]

pub fn breakpoint() {
    let mut str = String::from("");
    std::io::stdin().read_line(&mut str);
}

pub trait Address {
    fn address(&self, offset: usize) -> usize;
}

impl<T> Address for T {
    fn address(&self, offset: usize) -> usize {
        self as *const T as usize + offset
    }
}
