#![feature(rustc_private)]
// mac os does this

extern crate libc;
use libc::{
    write,
    c_void,
    c_char,
    size_t,
    close,
    mmap,
    MAP_SHARED,
};

fn main() {
    unsafe {

    }
}
