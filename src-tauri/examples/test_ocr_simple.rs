// Simplified OCR test - minimal dependencies version
// This version only tests the core OCR processing logic without Tauri
use image;
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: {} <图片路径>", args[0]);
        eprintln!("示例: {} monitor_0_screenshot.png", args[0]);
        std::process::exit(1);
    }

    let image_path = &args[1];
    if !Path::new(image_path).exists() {
        eprintln!("错误: 找不到文件: {}", image_path);
        std::process::exit(1);
    }

    println!("正在处理图片: {}", image_path);
    println!("加载图片中...");

    // 1. 测试图片加载
    match image::open(image_path) {
        Ok(img) => {
            println!("✓ 图片加载成功");
            println!("  - 尺寸: {}x{}", img.width(), img.height());
            println!("  - 颜色类型: {:?}", img.color());

            // 2. 测试图片预处理
            println!("\n正在进行图像预处理...");
            let gray = image::imageops::grayscale(&img);
            println!("✓ 灰度化完成");

            // 3. 保存预处理后的图片作为测试
            let output_path = "test_ocr_output.png";
            match gray.save(output_path) {
                Ok(_) => println!("✓ 预处理后的图片已保存到: {}", output_path),
                Err(e) => eprintln!("✗ 保存失败: {}", e),
            }

            println!("\n--- OCR 流程测试结果 ---");
            println!("图片加载和预处理: ✓ 成功");
            println!("\n注意: 实际 OCR 识别需要配置 OCR 引擎（如 RapidOCR）");
            println!("当前仅测试了图像处理流程");
            println!("------------------------");
        }
        Err(e) => {
            eprintln!("✗ 图片加载失败: {}", e);
            std::process::exit(1);
        }
    }
}
