#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unsafe_op_in_unsafe_fn, unused_must_use, static_mut_refs)]

#[cfg(target_os = "windows")]
mod shell;

#[cfg(target_os = "windows")]
fn main() {
    unsafe {
        shell::run();
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("Luma Shell is a native Windows 11 shell. Build this project on Windows to run it.");
}
