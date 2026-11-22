use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
};

use neoultront::{Glyph, NeoFontProject, PROJECT_EXTENSION, VERSION};
use slint::{SharedString, VecModel};

slint::include_modules!();

fn update_project_summary(
    ui: &MainWindow,
    project: &NeoFontProject,
    glyph_model: &VecModel<SharedString>,
) {
    ui.set_family_name(project.metadata.family_name.clone().into());
    ui.set_glyph_count(project.glyphs.len() as i32);
    ui.set_units_per_em(project.metadata.units_per_em as i32);
    ui.set_ascender(project.metadata.ascender as i32);
    ui.set_descender(project.metadata.descender as i32);
    glyph_model.set_vec(
        project
            .glyphs
            .keys()
            .map(|name| SharedString::from(name.as_str()))
            .collect::<Vec<_>>(),
    );
}

fn with_nufont_extension(mut path: PathBuf) -> PathBuf {
    if path.extension().and_then(|extension| extension.to_str()) != Some(PROJECT_EXTENSION) {
        path.set_extension(PROJECT_EXTENSION);
    }
    path
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let project = Rc::new(RefCell::new(NeoFontProject::new("Untitled Font")));
    let glyph_model = Rc::new(VecModel::from(vec![
        SharedString::from(".notdef"),
        SharedString::from("A"),
    ]));
    ui.set_glyph_list(glyph_model.clone().into());
    update_project_summary(&ui, &project.borrow(), &glyph_model);

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        let glyph_model = Rc::clone(&glyph_model);
        ui.on_new_requested(move || {
            let Some(ui) = weak_ui.upgrade() else { return };
            let family_name = ui.get_family_name().to_string();
            let family_name = if family_name.trim().is_empty() {
                "Untitled Font".to_owned()
            } else {
                family_name
            };
            let new_project = NeoFontProject::new(family_name);
            update_project_summary(&ui, &new_project, &glyph_model);
            *project.borrow_mut() = new_project;
            ui.set_selected_glyph(SharedString::from("A"));
            ui.set_glyph_name(SharedString::from("A"));
            ui.set_status_message(SharedString::from("New font project created"));
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        let glyph_model = Rc::clone(&glyph_model);
        ui.on_open_requested(move || {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("NeoUltront project", &[PROJECT_EXTENSION])
                .pick_file()
            else {
                return;
            };

            match NeoFontProject::load(&path) {
                Ok(loaded) => {
                    if let Some(ui) = weak_ui.upgrade() {
                        update_project_summary(&ui, &loaded, &glyph_model);
                        let selected = if loaded.glyphs.contains_key("A") {
                            "A".to_owned()
                        } else {
                            loaded.glyphs.keys().next().cloned().unwrap_or_else(|| ".notdef".to_owned())
                        };
                        ui.set_selected_glyph(selected.clone().into());
                        ui.set_glyph_name(selected.into());
                        ui.set_status_message(format!("Opened {}", path.display()).into());
                    }
                    *project.borrow_mut() = loaded;
                }
                Err(error) => {
                    if let Some(ui) = weak_ui.upgrade() {
                        ui.set_status_message(format!("Could not open project: {error}").into());
                    }
                }
            }
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        ui.on_save_requested(move || {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("NeoUltront project", &[PROJECT_EXTENSION])
                .set_file_name("Untitled.nufont")
                .save_file()
            else {
                return;
            };
            let path = with_nufont_extension(path);

            if let Some(ui) = weak_ui.upgrade() {
                let family_name = ui.get_family_name().to_string();
                if !family_name.trim().is_empty() {
                    project.borrow_mut().metadata.family_name = family_name;
                }
            }

            match project.borrow().save(&path) {
                Ok(()) => {
                    if let Some(ui) = weak_ui.upgrade() {
                        ui.set_status_message(format!("Saved {}", path.display()).into());
                    }
                }
                Err(error) => {
                    if let Some(ui) = weak_ui.upgrade() {
                        ui.set_status_message(format!("Could not save project: {error}").into());
                    }
                }
            }
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        let glyph_model = Rc::clone(&glyph_model);
        ui.on_add_glyph_requested(move || {
            let Some(ui) = weak_ui.upgrade() else { return };
            let name = ui.get_glyph_name().to_string();
            let name = name.trim().to_owned();

            if name.is_empty() {
                ui.set_status_message(SharedString::from("Enter a glyph name first"));
                return;
            }

            let glyph = Glyph {
                codepoint: if name.chars().count() == 1 { name.chars().next() } else { None },
                advance_width: 600.0,
                left_side_bearing: 50.0,
                outline: Vec::new(),
            };

            let result = project.borrow_mut().add_glyph(name.clone(), glyph);
            match result {
                Ok(()) => {
                    let current_project = project.borrow();
                    update_project_summary(&ui, &current_project, &glyph_model);
                    ui.set_selected_glyph(name.clone().into());
                    ui.set_glyph_name(name.clone().into());
                    ui.set_status_message(format!("Added glyph '{name}' (empty outline)").into());
                }
                Err(error) => ui.set_status_message(error.to_string().into()),
            }
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        ui.on_glyph_selected(move |glyph_name| {
            let name = glyph_name.to_string();
            if !project.borrow().glyphs.contains_key(&name) {
                if let Some(ui) = weak_ui.upgrade() {
                    ui.set_status_message(
                        format!("Glyph '{name}' is not in this project yet").into(),
                    );
                }
                return;
            }
            if let Some(ui) = weak_ui.upgrade() {
                ui.set_selected_glyph(name.clone().into());
                ui.set_glyph_name(name.clone().into());
                ui.set_status_message(format!("Selected glyph '{name}'").into());
            }
        });
    }

    println!("Starting NeoUltront {VERSION} desktop editor...");
    ui.run()
}
