use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct Wallpaper {
    pub name: String,
    pub path: PathBuf,
    pub thumbnail_path: PathBuf,
}

fn thumb_path_for(wallpaper_path: &Path, thumbnail_dir: &Path) -> Option<PathBuf> {
    let stem = wallpaper_path.file_stem()?.to_str()?;
    Some(thumbnail_dir.join(format!("thumb_{}.png", stem)))
}

/// Scans the wallpaper folder and ensures every entry has a cached PNG
/// thumbnail before returning. Slower on first run / after --reload,
/// but the GUI always gets a ready-to-display thumbnail.
pub fn scan_wallpapers(target_dir: &Path, thumbnail_dir: &Path) -> Vec<Wallpaper> {
    fs::create_dir_all(thumbnail_dir)
        .unwrap_or_else(|_| panic!("Could not create folder {}", thumbnail_dir.display()));

    let mut wallpapers = Vec::new();
    let valid_formats = ["jpg", "jpeg", "png", "webp", "gif"];

    let mut notification_sent = false;
    let mut new_cache_count = 0;

    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        // No need to asign memory
        if !valid_formats
            .iter()
            .any(|&fmt| ext.eq_ignore_ascii_case(fmt))
        {
            continue;
        }

        let Some(stem_os) = path.file_stem() else {
            continue;
        };
        let Some(stem_str) = stem_os.to_str() else {
            continue;
        };
        let name = stem_str.to_string();

        if let Some(thumb_path) = thumb_path_for(path, thumbnail_dir) {
            if !thumb_path.exists() {
                if !notification_sent {
                    send_notification("bgselector", "Generating thumbnail cache, please wait...");
                    notification_sent = true;
                }
                new_cache_count += 1;
            }
        }

        let Some(thumbnail_path) = generate_thumbnail(path, thumbnail_dir) else {
            eprintln!("Skipping {}: could not generate thumbnail", path.display());
            continue;
        };

        wallpapers.push(Wallpaper {
            name,
            path: path.to_path_buf(),
            thumbnail_path,
        });
    }

    if notification_sent {
        send_notification(
            "bgselector",
            &format!("Cache ready ({} new images processed)", new_cache_count),
        );
    }

    wallpapers
}

/// Generates a missing thumbnail and returns its path. If it already
/// exists, returns the cached path without touching the image crate.
pub fn generate_thumbnail(wallpaper_path: &Path, thumbnail_dir: &Path) -> Option<PathBuf> {
    let thumb_path = thumb_path_for(wallpaper_path, thumbnail_dir)?;

    if thumb_path.exists() {
        return Some(thumb_path);
    }

    let img = image::open(wallpaper_path).ok()?;
    let thumb = img.thumbnail(854, 480);
    thumb.save(&thumb_path).ok()?;

    println!("Thumbnail generated on {}", thumb_path.display());

    Some(thumb_path)
}

fn send_notification(summary: &str, body: &str) {
    // Try to run notify-send
    let result = Command::new("notify-send")
        .arg("-a")
        .arg("bgselector") // App name
        .arg("-i")
        .arg("image-loading") // Optional icon
        .arg(summary)
        .arg(body)
        .status();

    // If notify-send fails or is missing, fall back to printing to the console
    if result.is_err() || !result.unwrap().success() {
        eprintln!("[bgselector] {}: {}", summary, body);
    }
}
