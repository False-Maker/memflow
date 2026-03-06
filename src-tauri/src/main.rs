// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

fn main() {
    // 设置 ONNX Runtime DLL 路径 - 指向打包后的资源目录
    // 在开发模式下使用 target/debug/resources，打包后使用 exe 同级目录
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // 优先查找 exe 同级目录的 onnxruntime.dll
    let dll_path = exe_dir.join("onnxruntime.dll");
    if dll_path.exists() {
        std::env::set_var("ORT_DYLIB_PATH", &dll_path);
    } else {
        // 其次查找 resources 目录（开发模式）
        let resources_path = exe_dir.join("resources").join("onnxruntime.dll");
        if resources_path.exists() {
            std::env::set_var("ORT_DYLIB_PATH", &resources_path);
        }
    }

    memflow::run()
}
