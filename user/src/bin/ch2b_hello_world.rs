#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

/// 正确输出：
/// Hello world from user mode program!

#[no_mangle]
fn main() -> i32 {
    println!("Hello, world 1 from user mode program!");
    println!("Hello, world 2 from user mode program!");
    println!("Hello, world 3 from user mode program!");
    0
}