use crate::wallpaper::Wallpaper;
use rand::prelude::*;
use std::process::Command;

pub fn select_wallpaper(wallpaper: &Wallpaper) {
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

fn get_random_integer(count: usize) -> usize {
    let mut rng = rand::rng();
    rng.random_range(0..count)
}
