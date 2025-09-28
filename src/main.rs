use std::process;

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌿 Twiggy - Lightning-fast Git Visualization Tool");
    println!("Version: 0.1.0");
    println!("Built with Rust for maximum performance");
    println!();
    println!("Welcome to the future of Git visualization!");
    println!("Phase 1: Project Bootstrap - Complete ✓");

    Ok(())
}
