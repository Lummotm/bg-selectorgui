// Based on https://github.com/magetsu002/qs-wallpaper-picker
mod backend;
mod wallpaper;

use backend::select_wallpaper;
use std::{cell::RefCell, env, fs, rc::Rc};
use wallpaper::{cache_uncached, read_colors_file, scan_wallpapers, Wallpaper};

// Slint imports
use slint::{Image, Model, ModelRc, SharedString, VecModel};

// Inject Slint generated code
slint::include_modules!();

/// All the reactive UI state that isn't stored directly on the Slint window.
struct UiState {
    all_wallpapers: Vec<Wallpaper>,
    filter: String,
    search: String,
}

impl UiState {
    fn matches(&self, wp: &Wallpaper) -> bool {
        let name_ok = if self.search.is_empty() {
            true
        } else {
            wp.name.to_lowercase().contains(&self.search.to_lowercase())
        };
        if !name_ok {
            return false;
        }
        match self.filter.as_str() {
            "All" => true,
            other => wp.color_bucket == other,
        }
    }

    fn visible_indices(&self) -> Vec<usize> {
        self.all_wallpapers
            .iter()
            .enumerate()
            .filter(|(_, wp)| self.matches(wp))
            .map(|(i, _)| i)
            .collect()
    }
}

fn build_slint_model(wallpapers: &[Wallpaper], state: &UiState) -> Rc<VecModel<WallpaperData>> {
    let mut items = Vec::with_capacity(wallpapers.len());
    for wp in wallpapers {
        let image = Image::load_from_path(&wp.thumbnail_path).unwrap_or_default();
        items.push(WallpaperData {
            name: SharedString::from(&wp.name),
            image_path: image,
            color_bucket: SharedString::from(&wp.color_bucket),
            visible: state.matches(wp),
        });
    }
    Rc::new(VecModel::from(items))
}

fn status_text(state: &UiState, visible_count: usize) -> String {
    if !state.search.is_empty() {
        return format!("Search: \"{}\" ({} results)", state.search, visible_count);
    }
    if state.filter != "All" {
        return format!("{} ({})", state.filter, visible_count);
    }
    String::new()
}

fn refresh_ui(app: &AppWindow, state: &UiState) {
    let model = build_slint_model(&state.all_wallpapers, state);
    let visible = state.visible_indices();

    app.set_wallpapers(model.into());
    app.set_current_filter(SharedString::from(&state.filter));
    app.set_search_text(SharedString::from(&state.search));
    app.set_status_text(SharedString::from(status_text(state, visible.len())));

    // Keep active-index sane: snap to nearest visible item if the current
    // one just got filtered out.
    let current = app.get_active_index();
    if !visible.contains(&(current as usize)) {
        if let Some(&first) = visible.first() {
            app.set_active_index(first as i32);
        }
    }
}

fn step_active_index(state: &UiState, current: i32, direction: i32) -> i32 {
    let visible = state.visible_indices();
    if visible.is_empty() {
        return current;
    }
    let pos = visible
        .iter()
        .position(|&i| i as i32 == current)
        .unwrap_or(0) as i32;
    let len = visible.len() as i32;
    let new_pos = ((pos + direction) % len + len) % len;
    visible[new_pos as usize] as i32
}

const FILTER_ORDER: &[&str] = &[
    "All",
    "Red",
    "Orange",
    "Yellow",
    "Green",
    "Blue",
    "Purple",
    "Pink",
    "Monochrome",
];

fn cycle_filter(current: &str, direction: i32) -> String {
    let idx = FILTER_ORDER
        .iter()
        .position(|f| *f == current)
        .unwrap_or(0) as i32;
    let len = FILTER_ORDER.len() as i32;
    let new_idx = ((idx + direction) % len + len) % len;
    FILTER_ORDER[new_idx as usize].to_string()
}

fn main() -> Result<(), slint::PlatformError> {
    println!("Starting bgselector!!!");

    let cache_dir = dirs::cache_dir()
        .expect("CRITICAL: Cache directory not found")
        .join("bg-selector-gui/");

    let thumbnail_dir = cache_dir.join("thumbnails/");
    let colors_file = cache_dir.join("colors.txt");

    let home = dirs::home_dir().expect("CRITICAL: HOME not found");
    let wallpapers_dir = home.join("Pictures/Wallpapers/00-tmp/");

    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--reload" => {
                println!("Regenerating all cache.");
                let _ = fs::remove_dir_all(&thumbnail_dir);
            }
            "--cache" => {
                println!("Update thumbnails without launching GUI.");
            }
            _ => {}
        }
    }

    fs::create_dir_all(&thumbnail_dir).expect("CRITICAL: Could not create cache folder");

    let cached_colors = read_colors_file(&colors_file);
    let wallpapers = scan_wallpapers(&wallpapers_dir, &thumbnail_dir, &cached_colors);

    if wallpapers.is_empty() {
        eprintln!("No images found in the wallpapers folder.");
        return Ok(());
    }

    cache_uncached(&colors_file, &wallpapers, &cached_colors);

    // ==========================================
    // SLINT UI INITIALIZATION
    // ==========================================
    let app = AppWindow::new()?;

    let state = Rc::new(RefCell::new(UiState {
        all_wallpapers: wallpapers.clone(),
        filter: "All".to_string(),
        search: String::new(),
    }));

    // Initial population.
    refresh_ui(&app, &state.borrow());

    // -- Wallpaper selection (click or Enter) --
    {
        let state = state.clone();
        app.on_wallpaper_selected(move |selected_name| {
            let state = state.borrow();
            if let Some(wp) = state
                .all_wallpapers
                .iter()
                .find(|w| w.name == selected_name.as_str())
            {
                println!("UI selected: {}", wp.name);
                select_wallpaper(wp);
            }
        });
    }

    // -- Left/Right stepping, skipping filtered-out items --
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        app.on_step_requested(move |direction| {
            let app = app_weak.upgrade().unwrap();
            let state = state.borrow();
            let current = app.get_active_index();
            let next = step_active_index(&state, current, direction);
            app.set_active_index(next);
        });
    }

    // -- Filter selection (click on swatch) --
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        app.on_set_filter(move |name| {
            let app = app_weak.upgrade().unwrap();
            let mut state = state.borrow_mut();
            state.filter = name.to_string();
            refresh_ui(&app, &state);
        });
    }

    // -- Cycle filter with Tab / Shift+Tab --
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        app.on_cycle_filter(move |direction| {
            let app = app_weak.upgrade().unwrap();
            let mut state = state.borrow_mut();
            state.filter = cycle_filter(&state.filter, direction);
            refresh_ui(&app, &state);
        });
    }

    // -- Live search as you type --
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        app.on_search_changed(move |text| {
            let app = app_weak.upgrade().unwrap();
            let mut state = state.borrow_mut();
            state.search = text.to_string();
            refresh_ui(&app, &state);
        });
    }

    // -- Toggle search box visibility (the "/" shortcut or clicking it) --
    {
        let app_weak = app.as_weak();
        app.on_toggle_search(move || {
            let app = app_weak.upgrade().unwrap();
            app.set_search_mode(!app.get_search_mode());
        });
    }

    // -- Escape / close --
    {
        let app_weak = app.as_weak();
        app.on_close_requested(move || {
            if let Some(app) = app_weak.upgrade() {
                let _ = app.hide();
            }
        });
    }

    app.run()
}
