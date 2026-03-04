//! Screen capture module for MemFlow Core
//!
//! Provides cross-platform screen capture functionality

use anyhow::Result;
use image::DynamicImage;

/// Capture result containing the image and metadata
pub struct CaptureResult {
    /// The captured image
    pub image: DynamicImage,
    /// Width of the capture
    pub width: u32,
    /// Height of the capture
    pub height: u32,
    /// Number of monitors captured
    pub monitor_count: usize,
}

/// Capture all screens and return a panorama image
pub fn capture_screen() -> Result<CaptureResult> {
    #[cfg(windows)]
    {
        capture_windows()
    }
    
    #[cfg(not(windows))]
    {
        capture_crossplatform()
    }
}

#[cfg(windows)]
fn capture_windows() -> Result<CaptureResult> {
    use xcap::Monitor;
    
    let monitors = Monitor::all()?;
    if monitors.is_empty() {
        return Err(anyhow::anyhow!("No monitors found"));
    }
    
    // If only one monitor, return directly
    if monitors.len() == 1 {
        let monitor = &monitors[0];
        let image_buffer = monitor.capture_image()?;
        let width = image_buffer.width();
        let height = image_buffer.height();
        
        let mut raw_pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
        for p in image_buffer.pixels() {
            raw_pixels.extend_from_slice(&p.0);
        }
        
        let rgba_image = image::RgbaImage::from_raw(width, height, raw_pixels)
            .ok_or_else(|| anyhow::anyhow!("Failed to create image"))?;
        
        return Ok(CaptureResult {
            image: DynamicImage::ImageRgba8(rgba_image),
            width,
            height,
            monitor_count: monitors.len(),
        });
    }
    
    // Multiple monitors - create panorama
    capture_panorama(monitors)
}

#[cfg(not(windows))]
fn capture_crossplatform() -> Result<CaptureResult> {
    capture_panorama(xcap::Monitor::all()?)
}

fn capture_panorama(monitors: Vec<xcap::Monitor>) -> Result<CaptureResult> {
    if monitors.is_empty() {
        return Err(anyhow::anyhow!("No monitors found"));
    }
    
    let min_x = monitors.iter().map(|m| m.x()).min().unwrap_or(0);
    let min_y = monitors.iter().map(|m| m.y()).min().unwrap_or(0);
    let max_x = monitors.iter().map(|m| m.x() + m.width() as i32).max().unwrap_or(1920);
    let max_y = monitors.iter().map(|m| m.y() + m.height() as i32).max().unwrap_or(1080);
    
    let canvas_width = (max_x - min_x) as u32;
    let canvas_height = (max_y - min_y) as u32;
    
    tracing::info!("Capturing panorama: {} monitors, {}x{}", 
        monitors.len(), canvas_width, canvas_height);
    
    // Capture each monitor in parallel
    let num_monitors = monitors.len();
    drop(monitors); // Release the monitors
    
    let captures: Vec<_> = (0..num_monitors)
        .map(|idx| {
            std::thread::spawn(move || {
                let monitors = xcap::Monitor::all()?;
                if idx >= monitors.len() {
                    return Err(anyhow::anyhow!("Monitor index out of bounds"));
                }
                
                let monitor = &monitors[idx];
                let x_offset = (monitor.x() - min_x) as u32;
                let y_offset = (monitor.y() - min_y) as u32;
                
                let image_buffer = monitor.capture_image()?;
                let width = image_buffer.width();
                let height = image_buffer.height();
                
                let mut raw_pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
                for p in image_buffer.pixels() {
                    raw_pixels.extend_from_slice(&p.0);
                }
                
                let rgba_image = image::RgbaImage::from_raw(width, height, raw_pixels)
                    .ok_or_else(|| anyhow::anyhow!("Failed to create image"))?;
                
                Ok((rgba_image, x_offset, y_offset))
            })
        })
        .collect();
    
    // Create blank canvas
    let mut panorama = image::RgbaImage::new(canvas_width, canvas_height);
    
    // Composite each monitor
    for handle in captures {
        match handle.join().unwrap_or_else(|e| Err(anyhow::anyhow!("Thread panic: {:?}", e))) {
            Ok((monitor_img, x_offset, y_offset)) => {
                image::imageops::overlay(&mut panorama, &monitor_img, x_offset as i64, y_offset as i64);
            }
            Err(e) => {
                tracing::warn!("Failed to capture monitor: {}", e);
            }
        }
    }
    
    Ok(CaptureResult {
        image: DynamicImage::ImageRgba8(panorama),
        width: canvas_width,
        height: canvas_height,
        monitor_count: num_monitors,
    })
}

/// Encode image to WebP format
pub fn encode_webp(image: &DynamicImage, quality: f32) -> Result<Vec<u8>> {
    let rgba_image = image.to_rgba8();
    let encoder = webp::Encoder::from_rgba(&rgba_image, rgba_image.width(), rgba_image.height());
    let webp_memory = encoder.encode(quality);
    Ok(webp_memory.to_vec())
}

/// Calculate perceptual hash (dHash) for deduplication
pub fn calculate_phash(image: &DynamicImage) -> u64 {
    let gray = image::imageops::grayscale(image);
    let resized = image::imageops::resize(&gray, 9, 8, image::imageops::FilterType::Lanczos3);
    
    let mut hash: u64 = 0;
    let mut bit_index = 0;
    
    for y in 0..8 {
        for x in 0..8 {
            let left = resized.get_pixel(x, y).0[0];
            let right = resized.get_pixel(x + 1, y).0[0];
            if left > right {
                hash |= 1 << bit_index;
            }
            bit_index += 1;
        }
    }
    
    hash
}

/// Calculate Hamming distance between two hashes
pub fn hamming_distance(hash1: u64, hash2: u64) -> u32 {
    (hash1 ^ hash2).count_ones()
}

/// Check if two images are similar based on Hamming distance threshold
pub fn is_similar(hash1: u64, hash2: u64, threshold: u32) -> bool {
    hamming_distance(hash1, hash2) <= threshold
}
