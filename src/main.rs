// Based on https://github.com/magetsu002/qs-wallpaper-picker
mod backend;
mod gui;
mod wallpaper;

use rand::seq::SliceRandom;
use std::{env, fs, process};
use wallpaper::scan_wallpapers;

fn print_help() {
    println!("bgselector-gui - A fast and lightweight wallpaper picker.\n");
    println!("USAGE:");
    println!("  bgselector-gui [OPTIONS]\n");
    println!("OPTIONS:");
    println!("  -h, --help           Print help information and exit.");
    println!("  -v, --version        Print version information and exit.");
    println!("  -p, --print          Print selected wallpaper path to stdout and exit.");
    println!("  -e, --exec <cmd>     Execute custom command/script passing selected image path.");
    println!("  --dir <path>         Specify custom wallpaper directory.");
    println!("                       (Default: ~/Pictures/Wallpapers/00-tmp/)");
    println!("  --reload             Delete thumbnail cache and regenerate on start.");
    println!("  --cache              Update thumbnails without launching GUI.");
    println!("  --no-shuffle         Disable random wallpaper order (keep alphabetical).");
}

fn main() -> Result<(), slint::PlatformError> {
    let cache_dir = dirs::cache_dir()
        .expect("CRITICAL: Cache directory not found")
        .join("bg-selector-gui/");

    let thumbnail_dir = cache_dir.join("thumbnails/");

    let home = dirs::home_dir().expect("CRITICAL: HOME not found");
    let mut wallpapers_dir = home.join("Pictures/Wallpapers/00-tmp/");

    let mut exit_after_cache = false;
    let mut shuffle_wallpapers = true;
    let mut print_only = false;
    let mut custom_cmd: Option<String> = None;

    let args: Vec<String> = env::args().collect();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("bgselector-gui version {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            "-p" | "--print" => {
                print_only = true;
            }
            "-e" | "--exec" => {
                if let Some(cmd) = args.get(i + 1) {
                    custom_cmd = Some(cmd.clone());
                    i += 1;
                } else {
                    eprintln!("Error: The --exec flag requires a command/script path argument.");
                    process::exit(1);
                }
            }
            "--reload" => {
                println!("Regenerating all cache.");
                let _ = fs::remove_dir_all(&thumbnail_dir);
            }
            "--cache" => {
                println!("Update thumbnails without launching GUI.");
                exit_after_cache = true;
            }
            "--no-shuffle" => {
                shuffle_wallpapers = false;
            }
            "--dir" => {
                if let Some(custom_dir) = args.get(i + 1) {
                    wallpapers_dir = std::path::PathBuf::from(custom_dir);
                    println!("Wallpaper directory set to: {}", wallpapers_dir.display());
                    i += 1;
                } else {
                    eprintln!("Error: The --dir flag requires a path argument.");
                    process::exit(1);
                }
            }
            unknown => {
                eprintln!("Unknown argument: {}", unknown);
                eprintln!("Use --help to see available options.");
                process::exit(1);
            }
        }
        i += 1;
    }

    fs::create_dir_all(&thumbnail_dir).expect("CRITICAL: Could not create cache folder");

    let mut wallpapers = scan_wallpapers(&wallpapers_dir, &thumbnail_dir);

    if wallpapers.is_empty() {
        eprintln!("No images found in the wallpapers folder.");
        return Ok(());
    }

    if exit_after_cache {
        println!("Thumbnails generated successfully. Exiting.");
        process::exit(0);
    }

    if shuffle_wallpapers {
        wallpapers.shuffle(&mut rand::rng());
    }

    // Launch UI
    gui::run(wallpapers, print_only, custom_cmd)
}
