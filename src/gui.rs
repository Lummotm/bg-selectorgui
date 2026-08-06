use crate::backend::process_selection;
use crate::wallpaper::Wallpaper;
use std::{cell::RefCell, rc::Rc};

// Slint imports
use slint::{Image, ModelRc, SharedString, VecModel};

// Inject Slint generated code
slint::include_modules!();

struct UiState {
    all_wallpapers: Vec<Wallpaper>,
    print_only: bool,
    custom_cmd: Option<String>,
}

fn step_active_index(state: &UiState, current: i32, direction: i32) -> i32 {
    let len = state.all_wallpapers.len() as i32;
    if len == 0 {
        return current;
    }
    ((current + direction) % len + len) % len
}

pub fn run(
    wallpapers: Vec<Wallpaper>,
    print_only: bool,
    custom_cmd: Option<String>,
) -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;

    let state = Rc::new(RefCell::new(UiState {
        all_wallpapers: wallpapers,
        print_only,
        custom_cmd,
    }));

    let items: Vec<WallpaperData> = state
        .borrow()
        .all_wallpapers
        .iter()
        .map(|wp| {
            let image = Image::load_from_path(&wp.thumbnail_path).unwrap_or_default();
            WallpaperData {
                name: SharedString::from(&wp.name),
                image_path: image,
            }
        })
        .collect();

    let model = Rc::new(VecModel::from(items));
    app.set_wallpapers(ModelRc::from(model));

    {
        let state = state.clone();
        app.on_wallpaper_selected(move |selected_name| {
            let state = state.borrow();
            if let Some(wp) = state
                .all_wallpapers
                .iter()
                .find(|w| w.name == selected_name.as_str())
            {
                process_selection(wp, state.print_only, state.custom_cmd.as_deref());
            }
        });
    }

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
