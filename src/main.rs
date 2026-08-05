use std::{
    fs::{self},
    path::{Path, PathBuf},
    process::Command,
};

use rand::prelude::*;
use walkdir::WalkDir;

fn main() {
    println!("Starting bgselector!!!");
    let wallpapers = scan_wallpapers();
    let index = get_random_integer(wallpapers.len());
    let random_wallpaper = &wallpapers[index];
    select_wallpaper(random_wallpaper);
}

struct Wallpaper {
    name: String,
    path: PathBuf,
    thumbnail_path: PathBuf,
}

fn scan_wallpapers() -> Vec<Wallpaper> {
    let mut wallpapers = Vec::new();
    let valid_formats = ["jpg", "jpeg", "png", "webp", "gif"];

    let home = match dirs::home_dir() {
        Some(path) => path,
        None => {
            println!("Couldnt find HOME.");
            return wallpapers;
        }
    };

    let target_dir = home.join("Pictures/Wallpapers/0-god/");

    if !target_dir.exists() {
        println!("La carpeta {:?} no existe.", target_dir);
        return wallpapers;
    }

    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        };
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !valid_formats.contains(&ext.to_lowercase().as_str()) {
            continue;
        };
        // println!("Found image {:?}", path);
        // No se pq hay que printear con :?, jsjs :/ confused, :? even more confused JSJSJSJS

        let stem_option = path.file_stem();

        // Abrimos la caja del Option<&OsStr> y si es None hacemos continue
        // No entiendo a que se refiere con abrir la caja Option, supongo que sera que probamos a
        // cargar el valor
        let Some(stem_os) = stem_option else {
            continue;
        };
        let Some(stem_str) = stem_os.to_str() else {
            continue;
        };
        let name = stem_str.to_string();

        let thumbnail_path = match get_or_create_thumbnail(path) {
            Some(thumb) => thumb,
            None => path.to_path_buf(), // Si falla la miniatura, usamos la original como fallback
        };

        wallpapers.push(Wallpaper {
            name,
            path: path.to_path_buf(),
            thumbnail_path,
        })
    }
    wallpapers
}

// Pedimos referencia a un path, un id que no vamos a modificar representa una direccion de la que
// no somos dueños podemos mirar pero no tocar wazaaaa
fn get_or_create_thumbnail(path: &Path) -> Option<PathBuf> {
    let home = dirs::home_dir().expect("Error. Home not found.");
    let thumbnail_dir = home.join(".cache/bg-selector-gui/");
    fs::create_dir_all(&thumbnail_dir)
        .unwrap_or_else(|_| panic!("No se pudo crear carpeta {}", thumbnail_dir.display()));

    let filename = &path.file_name()?.to_str()?;
    let thumb_path = thumbnail_dir.join(format!("tumb_{}", filename));

    if thumb_path.exists() {
        return Some(thumb_path);
    };

    let img = image::open(path).ok()?;
    let thumb = img.thumbnail(640, 360);

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

fn random_transition() -> String {
    let transitions = ["wipe", "grow", "wave"];
    let n = transitions.len();
    let index = get_random_integer(n);
    transitions[index].to_string()
}

fn get_random_integer(last: usize) -> usize {
    let mut rng = rand::rng();
    rng.random_range(0..last)
}
