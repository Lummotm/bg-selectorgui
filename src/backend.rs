use crate::wallpaper::Wallpaper;

pub fn process_selection(wallpaper: &Wallpaper) {
    let path_str = wallpaper.path.to_string_lossy();

    println!("{}", path_str);
}
