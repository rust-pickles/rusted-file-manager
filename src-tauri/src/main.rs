// Windows subsystem is disabled in debug builds so `cargo run` keeps a console
// for logs; release builds use the GUI subsystem and open without a console.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    rusted_shell::run()
}
