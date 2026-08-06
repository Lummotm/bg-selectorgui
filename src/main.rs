// Based on https://github.com/magetsu002/qs-wallpaper-picker
use std::{
    collections::HashMap,
    env,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::Command,
};

use dirs::cache_dir;
use image::{imageops::FilterType, GenericImageView}; // Needed to main color from the image
use rand::prelude::*;
use std::io::Write;
use walkdir::WalkDir;

fn initialize() {}

fn main() {
    println!("Starting bgselector!!!");
    let home = dirs::home_dir().expect("CRÍTICO: No se encontró HOME");
    let cache_dir = home.join(".cache/bg-selector-gui/");
    let thumbnail_dir = cache_dir.join("thumbnails/");
    let colors_file = cache_dir.join("colors.txt");
    let wallpapers_dir = home.join("Pictures/Wallpapers/00-tmp/");

    let args: Vec<String> = env::args().collect();
    dbg!(&args); // arg[0] is the binary, else is the other stuff

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--reload" => {
                println!("Regenerating all cache.");
                fs::remove_dir_all(&thumbnail_dir).expect("Error removing thumbanail dir.");
            }
            "--cache" => {
                println!("Update thumbnails withouth launchings GUI.")
            }
            _ => {}
        }
    }

    fs::create_dir_all(&thumbnail_dir).expect("CRÍTICO: No se pudo crear la carpeta de caché");

    // Read first cache
    let cached_colors = read_colors_file(&colors_file);

    // Scan using the cache
    let wallpapers = scan_wallpapers(&wallpapers_dir, &thumbnail_dir, &cached_colors);

    if wallpapers.is_empty() {
        eprintln!("No se encontraron imágenes en la carpeta de wallpapers.");
        return;
    }

    let index = get_random_integer(wallpapers.len());
    select_wallpaper(&wallpapers[index]);

    // Cache uncached ones
    cache_uncached(&colors_file, &wallpapers, &cached_colors);
}

struct Wallpaper {
    name: String,
    path: PathBuf,
    thumbnail_path: PathBuf,
    hex_color: String,
}

fn scan_wallpapers(
    target_dir: &Path,
    thumbnail_dir: &Path,
    cached_colors: &HashMap<String, String>,
) -> Vec<Wallpaper> {
    let mut wallpapers = Vec::new();
    let valid_formats = ["jpg", "jpeg", "png", "webp", "gif"];

    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !valid_formats.contains(&ext.to_lowercase().as_str()) {
            continue;
        }

        let stem_option = path.file_stem();
        let Some(stem_os) = stem_option else {
            continue;
        };
        let Some(stem_str) = stem_os.to_str() else {
            continue;
        };
        let name = stem_str.to_string();

        let thumbnail_path = match get_or_create_thumbnail(path, thumbnail_dir) {
            Some(thumb) => thumb,
            None => path.to_path_buf(),
        };

        let hex_color = match cached_colors.get(&name) {
            Some(saved_color) => saved_color.clone(),
            None => {
                println!("Fondo nuevo detectado. Extrayendo color para: {}", name);
                extract_color_from_image(path)
            }
        };

        wallpapers.push(Wallpaper {
            name,
            path: path.to_path_buf(),
            thumbnail_path,
            hex_color,
        });
    }
    wallpapers
}

fn get_or_create_thumbnail(wallpaper_path: &Path, thumbnail_dir: &Path) -> Option<PathBuf> {
    fs::create_dir_all(thumbnail_dir)
        .unwrap_or_else(|_| panic!("No se pudo crear carpeta {}", thumbnail_dir.display()));

    // Extract name without extension (not mistype gifs)
    let stem = wallpaper_path.file_stem()?.to_str()?;

    // Force .png
    let thumb_path = thumbnail_dir.join(format!("tumb_{}.png", stem));

    if thumb_path.exists() {
        return Some(thumb_path);
    };

    let img = image::open(wallpaper_path).ok()?;

    let thumb = img.thumbnail(2000, 420);

    thumb.save(&thumb_path).ok()?;

    println!("Thumbnail generated on {}", thumb_path.display());

    Some(thumb_path)
}

fn select_wallpaper(wallpaper: &Wallpaper) {
    let transition = random_transition();
    let path = &wallpaper.path;

    let cmd = Command::new("awww")
        .arg("img")
        .arg(path)
        .arg("--transition-type")
        .arg(transition)
        .arg("--transition-step")
        .arg("60")
        .arg("--transition-fps")
        .arg("120")
        .spawn();

    match cmd {
        Ok(_) => println!("Wallpaper changed to {}", &wallpaper.name),
        Err(e) => eprintln!("Error when executing awww: {}", e),
    }
}

fn read_colors_file(colors_file_path: &Path) -> HashMap<String, String> {
    println!("Extracting colors from {}", colors_file_path.display());
    let mut names = HashMap::new();
    if let Ok(content) = fs::read_to_string(colors_file_path) {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(" :: ").collect();
            if parts.len() >= 2 {
                // has to be 2 might add more stuff later
                let name = parts[0];
                let hex = parts[1];
                names.insert(name.to_string(), hex.to_string());
            }
        }
    } else {
        println!("File doesn't exist. Will generate later.")
    }
    names
}

fn extract_color_from_image(picture: &Path) -> String {
    let Ok(img) = image::open(picture) else {
        return "#888888".to_string();
    };
    // Get the main color on the image
    let pixel_img = img.resize_exact(1, 1, FilterType::Nearest);
    let pixel = pixel_img.get_pixel(0, 0);
    format!("#{:02X}{:02X}{:02X}", pixel[0], pixel[1], pixel[2])
}

fn append_colors_to_file(colors_file: &Path, name: &str, hex: &str) {
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(colors_file)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening {}.\n[ERROR] {}", colors_file.display(), e);
            return;
        }
    };

    let result = writeln!(file, "{} :: {}", name, hex);

    match result {
        Ok(_) => {}
        Err(e) => eprintln!("Error writing to {}\n[ERROR]{}", colors_file.display(), e),
    }
}

fn cache_uncached(
    colors_file: &Path,
    wallpapers: &[Wallpaper],
    cached_colors: &HashMap<String, String>,
) {
    for wallpaper in wallpapers {
        // If name not in cached the cache it
        if !cached_colors.contains_key(&wallpaper.name) {
            append_colors_to_file(colors_file, &wallpaper.name, &wallpaper.hex_color);
        }
    }
}

fn random_transition() -> String {
    let transitions = ["wipe", "grow", "wave"];
    let n = transitions.len();
    let index = get_random_integer(n);
    transitions[index].to_string()
}

fn get_random_integer(count: usize) -> usize {
    let mut rng = rand::rng();
    rng.random_range(0..count)
}
