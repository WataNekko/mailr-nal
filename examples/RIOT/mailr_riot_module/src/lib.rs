#![no_std]

use riot_wrappers::println;

#[no_mangle]
pub extern "C" fn smtp_hello_world() {
    println!("Hello World!");
}
