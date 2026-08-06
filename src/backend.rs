use crate::wallpaper::Wallpaper;
use std::process::Command;

pub fn process_selection(wallpaper: &Wallpaper, print_only: bool, custom_cmd: Option<&str>) {
    let path_str = wallpaper.path.to_string_lossy();

    if print_only {
        println!("{}", path_str);
        return;
    }

    if let Some(cmd) = custom_cmd {
        if let Err(e) = Command::new(cmd).arg(path_str.as_ref()).spawn() {
            eprintln!("Error executing '{}': {}", cmd, e);
        }
    } else {
        // Default fallback to awww
        let _ = Command::new("awww")
            .arg("img")
            .arg(path_str.as_ref())
            .arg("--transition-type")
            .arg("fade")
            .arg("--transition-step")
            .arg("60")
            .arg("--transition-fps")
            .arg("120")
            .spawn();
    }
}
