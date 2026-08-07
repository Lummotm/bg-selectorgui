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

pub fn scan_wallpapers(
    target_dir: &Path,
    thumbnail_dir: &Path,
    thumb_width: u32,
    thumb_height: u32,
) -> Vec<Wallpaper> {
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

        let Some(thumbnail_path) =
            generate_thumbnail(path, thumbnail_dir, thumb_width, thumb_height)
        else {
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

pub fn generate_thumbnail(
    wallpaper_path: &Path,
    thumbnail_dir: &Path,
    width: u32,
    height: u32,
) -> Option<PathBuf> {
    let thumb_path = thumb_path_for(wallpaper_path, thumbnail_dir)?;

    if thumb_path.exists() {
        return Some(thumb_path);
    }

    let img = image::open(wallpaper_path).ok()?;
    let thumb = img.thumbnail(width, height);
    thumb.save(&thumb_path).ok()?;
    println!(
        "Generated thumbnail of dimensions {}x{} at {}",
        width,
        height,
        thumb_path.display()
    );

    Some(thumb_path)
}

fn send_notification(summary: &str, body: &str) {
    let result = Command::new("notify-send")
        .arg("-a")
        .arg("bgselector")
        .arg("-i")
        .arg("image-loading")
        .arg(summary)
        .arg(body)
        .status();

    if result.is_err() || !result.unwrap().success() {
        eprintln!("[bgselector] {}: {}", summary, body);
    }
}
