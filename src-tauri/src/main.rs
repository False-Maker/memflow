// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let is_headless = std::env::args().any(|arg| arg == "--headless");
    if is_headless {
        memflow::run_headless()
    } else {
        memflow::run()
    }
}
