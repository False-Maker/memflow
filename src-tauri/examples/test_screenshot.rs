use std::time::Instant;
use xcap::Monitor;

fn main() {
    let start = Instant::now();
    let monitors = Monitor::all().unwrap();
    println!("Found {} monitors", monitors.len());

    for (i, monitor) in monitors.iter().enumerate() {
        println!("Monitor {}: {:?}", i, monitor);

        let should_capture = true; // Set to false to test monitor listing only

        if should_capture {
            match monitor.capture_image() {
                Ok(image) => {
                    let filename = format!("monitor_{}_screenshot.png", i);
                    match image.save(&filename) {
                        Ok(_) => println!("Saved screenshot to {}", filename),
                        Err(e) => println!("Failed to save screenshot: {}", e),
                    }
                }
                Err(e) => println!("Failed to capture image: {}", e),
            }
        }
    }

    println!("Total time: {:?}", start.elapsed());
}
