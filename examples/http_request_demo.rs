use learn_browser::url::{Url, request, strip_html_tags};
use std::env;

fn main() -> Result<(), String> {
    println!("🌐 HTTP Request Demo");
    println!("==================\n");

    // Get URL from command line args or use default
    let args: Vec<String> = env::args().collect();
    let url_str = if args.len() > 1 {
        &args[1]
    } else {
        println!("💡 Usage: cargo run --example http_request_demo <url>");
        println!("   Using default URL: http://example.org/\n");
        "http://example.org/"
    };

    // Create a URL and separate socket
    let url = Url::new(url_str)?;

    println!("📋 URL Details:");
    println!("  Host: {}", url.host);
    println!("  Path: {}", url.path);
    println!();

    // Make the HTTP request using the independent request function
    println!(
        "🚀 Making HTTP request to {} using independent request function...\n",
        url.host
    );

    match request(&url) {
        Ok(response) => {
            // Display response details
            println!("📥 Response received:");
            println!("  Version: {}", response.version);
            println!("  Status: {} {}", response.status, response.explanation);
            println!();

            println!("📋 Headers:");
            for (key, value) in &response.headers {
                println!("  {}: {}", key, value);
            }
            println!();

            println!("📄 Raw HTML Body (first 500 characters):");
            println!("------------------------------------------");
            let body_preview = if response.body.len() > 500 {
                format!("{}...", &response.body[..500])
            } else {
                response.body.clone()
            };
            println!("{}", body_preview);
            println!();

            // Strip HTML tags and show clean text
            let clean_text = strip_html_tags(&response.body);
            println!("🧹 Clean Text (HTML tags removed, first 300 characters):");
            println!("--------------------------------------------------------");
            let clean_preview = if clean_text.len() > 300 {
                format!("{}...", clean_text[..300].trim())
            } else {
                clean_text.trim().to_string()
            };
            println!("{}", clean_preview);
            println!();

            // Show some statistics
            println!("📊 Statistics:");
            println!("  Original length: {} characters", response.body.len());
            println!("  Headers count: {}", response.headers.len());
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
            println!("\n💡 Note: This example requires internet connection to work.");
            println!("   Try URLs like: http://example.com or http://httpbin.org/html");
        }
    }

    Ok(())
}
