mod backend;
mod gui;
mod wallpaper;

use rand::seq::SliceRandom;
use std::{env, fs, process};
use wallpaper::scan_wallpapers;

fn print_help() {
    println!("bgselector - A fast and lightweight wallpaper picker.\n");
    println!("Returns the path of the selected wallpaper.\n");
    println!("USAGE:");
    println!("  bgselector [OPTIONS]\n");
    println!("OPTIONS:");
    println!("  -h, --help               Print help information and exit.");
    println!("  -v, --version            Print version information and exit.");
    println!("  --dir <path>             Specify custom wallpaper directory.");
    println!("                           (Default: ~/Pictures/Wallpapers/)");
    println!("  --thumb <width> <height> Specify custom thumbnail dimensions.");
    println!("                           (Default: 640 360)");
    println!("  --reload                 Delete thumbnail cache and regenerate on start.");
    println!("  --cache                  Update thumbnails without launching GUI.");
    println!("  --no-shuffle             Disable random wallpaper order (keep alphabetical).");
}

fn main() -> Result<(), slint::PlatformError> {
    let cache_dir = dirs::cache_dir()
        .expect("CRITICAL: Cache directory not found")
        .join("bgselector/");
    let thumbnail_dir = cache_dir.join("thumbnails/");
    let home = dirs::home_dir().expect("CRITICAL: HOME not found");

    let mut wallpapers_dir = home.join("Pictures/Wallpapers/");
    let mut exit_after_cache = false;
    let mut shuffle_wallpapers = true;

    let mut thumb_width: u32 = 640;
    let mut thumb_height: u32 = 360;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("bgselector version {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            "--reload" => {
                eprintln!("Regenerating all cache.");
                let _ = fs::remove_dir_all(&thumbnail_dir);
            }
            "--cache" => {
                eprintln!("Update thumbnails without launching GUI.");
                exit_after_cache = true;
            }
            "--no-shuffle" => shuffle_wallpapers = false,
            "--dir" => {
                if let Some(custom_dir) = args.next() {
                    wallpapers_dir = std::path::PathBuf::from(custom_dir);
                    eprintln!("Wallpaper directory set to: {}", wallpapers_dir.display());
                } else {
                    eprintln!("Error: The --dir flag requires a path argument.");
                    process::exit(1);
                }
            }
            "--thumb" => {
                let w = args.next().and_then(|val| val.parse::<u32>().ok());
                let h = args.next().and_then(|val| val.parse::<u32>().ok());

                if let (Some(w), Some(h)) = (w, h) {
                    thumb_width = w;
                    thumb_height = h;
                    eprintln!(
                        "Custom thumbnail size set to: {}x{}",
                        thumb_width, thumb_height
                    );
                } else {
                    eprintln!(
                        "Error: The --thumb flag requires two numeric arguments: <width> <height>"
                    );
                    process::exit(1);
                }
            }
            unknown => {
                eprintln!("Unknown argument: {unknown}");
                eprintln!("Use --help to see available options.");
                process::exit(1);
            }
        }
    }

    fs::create_dir_all(&thumbnail_dir).expect("CRITICAL: Could not create cache folder");

    let mut wallpapers =
        scan_wallpapers(&wallpapers_dir, &thumbnail_dir, thumb_width, thumb_height);

    if wallpapers.is_empty() {
        eprintln!("No images found in the wallpapers folder.");
        return Ok(());
    }

    if exit_after_cache {
        eprintln!("Thumbnails generated successfully. Exiting.");
        process::exit(0);
    }

    if shuffle_wallpapers {
        wallpapers.shuffle(&mut rand::rng());
    }

    gui::run(wallpapers, thumbnail_dir)
}
