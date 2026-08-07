use crate::backend::process_selection;
use crate::wallpaper::{generate_thumbnail, Wallpaper};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::mpsc, thread};

// Slint imports
use slint::{Image, Model, ModelRc, SharedString, VecModel};

// Inject Slint generated code
slint::include_modules!();

struct UiState {
    all_wallpapers: Vec<Wallpaper>,
}

fn step_active_index(state: &UiState, current: i32, direction: i32) -> i32 {
    let len = state.all_wallpapers.len() as i32;
    if len == 0 {
        return current;
    }
    ((current + direction) % len + len) % len
}

pub fn run(wallpapers: Vec<Wallpaper>, thumbnail_dir: PathBuf) -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;

    // Kick off background thumbnail generation for anything not cached yet.
    let pending: Vec<(usize, PathBuf)> = wallpapers
        .iter()
        .enumerate()
        .filter(|(_, wp)| !wp.thumb_cached)
        .map(|(i, wp)| (i, wp.path.clone()))
        .collect();

    let state = Rc::new(RefCell::new(UiState {
        all_wallpapers: wallpapers,
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
    app.set_wallpapers(ModelRc::from(model.clone()));

    if !pending.is_empty() {
        let (tx, rx) = mpsc::channel::<(usize, PathBuf)>();

        // Generate thumbnails one by one on a background thread.
        thread::spawn(move || {
            for (index, path) in pending {
                if let Some(thumb) = generate_thumbnail(&path, &thumbnail_dir) {
                    let _ = tx.send((index, thumb));
                }
            }
        });

        // Poll the channel from the UI thread and patch the model in place.
        let model_weak = Rc::downgrade(&model);
        let state_weak = Rc::downgrade(&state);
        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(150),
            move || {
                let (Some(model), Some(state)) = (model_weak.upgrade(), state_weak.upgrade())
                else {
                    return;
                };
                while let Ok((index, thumb_path)) = rx.try_recv() {
                    if let Some(mut row) = model.row_data(index) {
                        row.image_path = Image::load_from_path(&thumb_path).unwrap_or_default();
                        model.set_row_data(index, row);
                    }
                    if let Some(wp) = state.borrow_mut().all_wallpapers.get_mut(index) {
                        wp.thumbnail_path = thumb_path;
                        wp.thumb_cached = true;
                    }
                }
            },
        );
    }

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
