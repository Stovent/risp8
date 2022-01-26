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
    fn address(&self, offset: isize) -> usize;
}

impl<T, const N: usize> Address for [T; N] {
    fn address(&self, offset: isize) -> usize {
        unsafe {
            let ptr = self.as_ptr();
            ptr.offset(offset) as usize
        }
    }
}

impl<T> Address for &T {
    fn address(&self, _: isize) -> usize {
        *self as *const T as usize
    }
}
