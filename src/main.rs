// Based on https://github.com/magetsu002/qs-wallpaper-picker
mod backend;
mod wallpaper;

use backend::select_wallpaper;
use rand::prelude::*;
use std::{env, fs};
use wallpaper::{cache_uncached, read_colors_file, scan_wallpapers};

fn main() {
    println!("Starting bgselector!!!");

    let cache_dir = dirs::cache_dir()
        .expect("CRITICAL: Cache directory not found")
        .join("bg-selector-gui/");

    let thumbnail_dir = cache_dir.join("thumbnails/");
    let colors_file = cache_dir.join("colors.txt");

    // Wallpaper paths
    let home = dirs::home_dir().expect("CRITICAL: HOME not found");
    let wallpapers_dir = home.join("Pictures/Wallpapers/00-tmp/");

    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--reload" => {
                println!("Regenerating all cache.");
                let _ = fs::remove_dir_all(&thumbnail_dir);
            }
            "--cache" => {
                println!("Update thumbnails without launching GUI.");
            }
            _ => {}
        }
    }

    fs::create_dir_all(&thumbnail_dir).expect("CRITICAL: Could not create cache folder");

    // Read cache
    let cached_colors = read_colors_file(&colors_file);

    // Scan
    let wallpapers = scan_wallpapers(&wallpapers_dir, &thumbnail_dir, &cached_colors);

    if wallpapers.is_empty() {
        eprintln!("No images found in the wallpapers folder.");
        return;
    }

    let mut rng = rand::rng();
    let index = rng.random_range(0..wallpapers.len());
    select_wallpaper(&wallpapers[index]);

    cache_uncached(&colors_file, &wallpapers, &cached_colors);
}
