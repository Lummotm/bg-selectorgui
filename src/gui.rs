use crate::backend::process_selection;
use crate::wallpaper::Wallpaper;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

// Slint imports
use slint::{Image, Model, ModelRc, SharedString, VecModel};

// Inject Slint generated code
slint::include_modules!();

struct UiState {
    all_wallpapers: Vec<Wallpaper>,
    // Indices currently loaded in the model (have a real image, not a placeholder)
    loaded: std::collections::HashSet<usize>,
}

// How many steps ahead/behind the active item are preloaded.
// Wider than what is visible (7 = active ± 3) so that when scrolling fast
// the upcoming images are already in memory before becoming visible.
const LOAD_RADIUS: i32 = 6;
// Only unloads when an index is FURTHER away than this.
// A margin larger than LOAD_RADIUS avoids the load-unload-load pattern
// if the user moves back and forth over the same indices.
const UNLOAD_RADIUS: i32 = 10;

fn step_active_index(len: i32, current: i32, direction: i32) -> i32 {
    if len == 0 {
        return current;
    }
    ((current + direction) % len + len) % len
}

// Minimum circular distance between two indices, given the total list size.
fn circular_distance(len: i32, a: i32, b: i32) -> i32 {
    if len == 0 {
        return 0;
    }
    let diff = (a - b).abs() % len;
    diff.min(len - diff)
}

// Loads the images within LOAD_RADIUS of the active item (if not already loaded)
// and unloads those that ended up more than UNLOAD_RADIUS away.
fn update_visible_images(
    model: &Rc<VecModel<WallpaperData>>,
    state: &Rc<RefCell<UiState>>,
    active_index: i32,
) {
    let mut state = state.borrow_mut();
    let len = state.all_wallpapers.len() as i32;
    if len == 0 {
        return;
    }

    // Load what comes into range
    for offset in -LOAD_RADIUS..=LOAD_RADIUS {
        let idx = (((active_index + offset) % len) + len) % len;
        let idx_usize = idx as usize;
        if state.loaded.contains(&idx_usize) {
            continue;
        }
        let wp = &state.all_wallpapers[idx_usize];
        let image = Image::load_from_path(&wp.thumbnail_path).unwrap_or_default();
        model.set_row_data(
            idx_usize,
            WallpaperData {
                name: SharedString::from(&wp.name),
                image_path: image,
            },
        );
        state.loaded.insert(idx_usize);
    }

    // Unload what ended up out of range
    let to_unload: Vec<usize> = state
        .loaded
        .iter()
        .copied()
        .filter(|&idx| circular_distance(len, idx as i32, active_index) > UNLOAD_RADIUS)
        .collect();

    for idx_usize in to_unload {
        let wp = &state.all_wallpapers[idx_usize];
        model.set_row_data(
            idx_usize,
            WallpaperData {
                name: SharedString::from(&wp.name),
                image_path: Image::default(),
            },
        );
        state.loaded.remove(&idx_usize);
    }
}

pub fn run(
    wallpapers: Vec<Wallpaper>,
    _thumbnail_dir: PathBuf,
) -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;

    let state = Rc::new(RefCell::new(UiState {
        all_wallpapers: wallpapers,
        loaded: std::collections::HashSet::new(),
    }));

    // Empty placeholders: real images are loaded on demand
    // in update_visible_images, not here.
    let items: Vec<WallpaperData> = state
        .borrow()
        .all_wallpapers
        .iter()
        .map(|wp| WallpaperData {
            name: SharedString::from(&wp.name),
            image_path: Image::default(),
        })
        .collect();

    let model = Rc::new(VecModel::from(items));
    app.set_wallpapers(ModelRc::from(model.clone()));

    // Initial load: active (0) ± LOAD_RADIUS
    update_visible_images(&model, &state, app.get_active_index());

    {
        let state = state.clone();
        app.on_wallpaper_selected(move |selected_name| {
            let state = state.borrow();
            if let Some(wp) = state
                .all_wallpapers
                .iter()
                .find(|w| w.name == selected_name.as_str())
            {
                process_selection(wp);
            }
        });
    }

    {
        let state = state.clone();
        let model = model.clone();
        let app_weak = app.as_weak();
        app.on_step_requested(move |direction| {
            let app = app_weak.upgrade().unwrap();
            let current = app.get_active_index();
            let len = state.borrow().all_wallpapers.len() as i32;
            let next = step_active_index(len, current, direction);
            app.set_active_index(next);
            update_visible_images(&model, &state, next);
        });
    }

    // Close when selected
    {
        let app_weak = app.as_weak();
        app.on_close_requested(move || {
            if let Some(app) = app_weak.upgrade() {
                let _ = app.hide();
                let _ = slint::quit_event_loop();
            }
        });
    }

    app.run()
}
