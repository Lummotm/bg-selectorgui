use image::{imageops::FilterType, GenericImageView};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct Wallpaper {
    pub name: String,
    pub path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub hex_color: String,
}

pub fn scan_wallpapers(
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
                println!("New wallpaper detected. Extracting color for: {}", name);
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
        .unwrap_or_else(|_| panic!("Could not create folder {}", thumbnail_dir.display()));

    let stem = wallpaper_path.file_stem()?.to_str()?;
    let thumb_path = thumbnail_dir.join(format!("thumb_{}.png", stem));

    if thumb_path.exists() {
        return Some(thumb_path);
    };

    let img = image::open(wallpaper_path).ok()?;

    // Adjusted to the ROADMAP resolution to optimize memory in the UI
    let thumb = img.thumbnail(640, 360);
    thumb.save(&thumb_path).ok()?;

    println!("Thumbnail generated on {}", thumb_path.display());

    Some(thumb_path)
}

pub fn read_colors_file(colors_file_path: &Path) -> HashMap<String, String> {
    println!("Extracting colors from {}", colors_file_path.display());
    let mut names = HashMap::new();
    if let Ok(content) = fs::read_to_string(colors_file_path) {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(" :: ").collect();
            if parts.len() >= 2 {
                let name = parts[0];
                let hex = parts[1];
                names.insert(name.to_string(), hex.to_string());
            }
        }
    } else {
        println!("File doesn't exist. Will generate later.");
    }
    names
}

fn extract_color_from_image(picture: &Path) -> String {
    let Ok(img) = image::open(picture) else {
        return "#888888".to_string();
    };
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

    if let Err(e) = writeln!(file, "{} :: {}", name, hex) {
        eprintln!("Error writing to {}\n[ERROR]{}", colors_file.display(), e);
    }
}

pub fn cache_uncached(
    colors_file: &Path,
    wallpapers: &[Wallpaper],
    cached_colors: &HashMap<String, String>,
) {
    for wallpaper in wallpapers {
        if !cached_colors.contains_key(&wallpaper.name) {
            append_colors_to_file(colors_file, &wallpaper.name, &wallpaper.hex_color);
        }
    }
}
