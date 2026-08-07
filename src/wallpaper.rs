use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct Wallpaper {
    pub name: String,
    pub path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub thumb_cached: bool, // false => thumbnail_path is just the original image (placeholder)
}

fn thumb_path_for(wallpaper_path: &Path, thumbnail_dir: &Path) -> Option<PathBuf> {
    let stem = wallpaper_path.file_stem()?.to_str()?;
    Some(thumbnail_dir.join(format!("thumb_{}.png", stem)))
}

/// Fast scan: no image decoding/resizing here. Uses cached thumb if present,
/// otherwise falls back to the original path so the GUI has *something* to show.
pub fn scan_wallpapers(target_dir: &Path, thumbnail_dir: &Path) -> Vec<Wallpaper> {
    fs::create_dir_all(thumbnail_dir)
        .unwrap_or_else(|_| panic!("Could not create folder {}", thumbnail_dir.display()));

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

        let Some(stem_os) = path.file_stem() else {
            continue;
        };
        let Some(stem_str) = stem_os.to_str() else {
            continue;
        };
        let name = stem_str.to_string();

        let (thumbnail_path, thumb_cached) = match thumb_path_for(path, thumbnail_dir) {
            Some(thumb) if thumb.exists() => (thumb, true),
            _ => (path.to_path_buf(), false),
        };

        wallpapers.push(Wallpaper {
            name,
            path: path.to_path_buf(),
            thumbnail_path,
            thumb_cached,
        });
    }
    wallpapers
}

/// Slow part, meant to run off the UI thread. Generates a missing thumbnail
/// and returns its path on success.
pub fn generate_thumbnail(wallpaper_path: &Path, thumbnail_dir: &Path) -> Option<PathBuf> {
    let thumb_path = thumb_path_for(wallpaper_path, thumbnail_dir)?;

    if thumb_path.exists() {
        return Some(thumb_path);
    }

    let img = image::open(wallpaper_path).ok()?;
    let thumb = img.thumbnail(640, 360);
    thumb.save(&thumb_path).ok()?;

    println!("Thumbnail generated on {}", thumb_path.display());

    Some(thumb_path)
}
