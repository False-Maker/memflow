// OCR 批量测试工具
// 读取 image 目录下所有图片，使用 OCR 识别，并将结果写入文档
use memflow::ocr::{process_image, OcrConfig};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("=== OCR 批量测试工具 ===\n");

    // 获取 image 目录路径（相对于项目根目录）
    let image_dir = Path::new("image");
    
    // 如果当前目录在 src-tauri/examples，需要向上两级到项目根目录
    let current_dir = std::env::current_dir()?;
    let image_dir = if image_dir.exists() {
        image_dir.to_path_buf()
    } else {
        // 尝试从 examples 目录向上查找
        let mut search_dir = current_dir.clone();
        loop {
            let test_path = search_dir.join("image");
            if test_path.exists() {
                break test_path;
            }
            if let Some(parent) = search_dir.parent() {
                search_dir = parent.to_path_buf();
            } else {
                break image_dir.to_path_buf();
            }
        }
    };

    if !image_dir.exists() {
        eprintln!("错误: 找不到 image 目录");
        eprintln!("当前目录: {}", current_dir.display());
        eprintln!("请确保 image 目录存在，或从项目根目录运行此程序");
        std::process::exit(1);
    }

    println!("图片目录: {}\n", image_dir.display());

    // 读取所有图片文件
    let image_extensions = ["png", "jpg", "jpeg", "bmp", "gif"];
    let mut image_files: Vec<PathBuf> = Vec::new();

    match fs::read_dir(&image_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if image_extensions.contains(&ext.to_lowercase().as_str()) {
                                image_files.push(path);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("错误: 无法读取目录 {}: {}", image_dir.display(), e);
            std::process::exit(1);
        }
    }

    // 按文件名排序
    image_files.sort();

    if image_files.is_empty() {
        eprintln!("错误: image 目录中没有找到图片文件");
        std::process::exit(1);
    }

    println!("找到 {} 张图片:\n", image_files.len());
    for (i, file) in image_files.iter().enumerate() {
        println!("  {}. {}", i + 1, file.file_name().unwrap().to_string_lossy());
    }
    println!();

    // 配置 OCR
    let ocr_config = OcrConfig::new("rapidocr")
        .with_redaction(false); // 测试时禁用脱敏，查看原始结果

    // 创建输出文档路径
    let output_file = Path::new("ocr_results.md");
    
    // 打开或创建输出文件
    let mut output = fs::File::create(&output_file)?;
    
    // 写入文档头部
    writeln!(output, "# OCR 识别结果\n")?;
    writeln!(output, "生成时间: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(output, "图片目录: `{}`\n", image_dir.display())?;
    writeln!(output, "共识别 {} 张图片\n\n", image_files.len())?;
    writeln!(output, "---\n")?;

    // 处理每张图片
    let mut success_count = 0;
    let mut fail_count = 0;

    for (index, image_path) in image_files.iter().enumerate() {
        let file_name = image_path.file_name().unwrap().to_string_lossy();
        println!("[{}/{}] 正在处理: {}...", index + 1, image_files.len(), file_name);

        match process_image(
            image_path.to_str().unwrap(),
            ocr_config.clone(),
        ).await {
            Ok(text) => {
                success_count += 1;
                println!("  ✓ 识别成功 (长度: {} 字符)", text.len());

                // 写入结果到文档
                writeln!(output, "## {}. {}\n", index + 1, file_name)?;
                writeln!(output, "**文件路径**: `{}`\n", image_path.display())?;
                
                if text.trim().is_empty() {
                    writeln!(output, "**识别结果**: *（未识别到文本）*\n")?;
                } else {
                    writeln!(output, "**识别结果**:\n")?;
                    writeln!(output, "```")?;
                    writeln!(output, "{}", text)?;
                    writeln!(output, "```\n")?;
                }
                
                writeln!(output, "---\n")?;
            }
            Err(e) => {
                fail_count += 1;
                eprintln!("  ✗ 识别失败: {}", e);

                // 写入错误信息到文档
                writeln!(output, "## {}. {}\n", index + 1, file_name)?;
                writeln!(output, "**文件路径**: `{}`\n", image_path.display())?;
                writeln!(output, "**状态**: ❌ **识别失败**\n")?;
                writeln!(output, "**错误信息**:\n")?;
                writeln!(output, "```")?;
                writeln!(output, "{}", e)?;
                writeln!(output, "```\n")?;
                writeln!(output, "---\n")?;
            }
        }
    }

    // 写入统计信息
    writeln!(output, "\n## 统计信息\n")?;
    writeln!(output, "- **总图片数**: {}", image_files.len())?;
    writeln!(output, "- **成功**: {} ✓", success_count)?;
    writeln!(output, "- **失败**: {} ✗\n", fail_count)?;

    println!("\n=== 处理完成 ===");
    println!("成功: {} 张", success_count);
    println!("失败: {} 张", fail_count);
    println!("结果已保存到: {}", output_file.display());

    Ok(())
}

