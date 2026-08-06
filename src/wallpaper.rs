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
}

pub fn scan_wallpapers(target_dir: &Path, thumbnail_dir: &Path) -> Vec<Wallpaper> {
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

        wallpapers.push(Wallpaper {
            name,
            path: path.to_path_buf(),
            thumbnail_path,
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
    let thumb = img.thumbnail(640, 360);
    thumb.save(&thumb_path).ok()?;

    println!("Thumbnail generated on {}", thumb_path.display());

    Some(thumb_path)
}
