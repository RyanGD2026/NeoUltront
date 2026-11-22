use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
};

use neoultront::{
    Glyph, GlyphCommand, GlyphPoint, NeoFontProject, PROJECT_EXTENSION, VERSION,
};
use slint::{SharedString, VecModel};

slint::include_modules!();

fn glyph_to_svg_path(glyph: &Glyph) -> String {
    let mut path = String::new();

    for command in &glyph.outline {
        match command {
            GlyphCommand::MoveTo(point) => {
                path.push_str(&format!("M {:.2} {:.2} ", point.x, 1000.0 - point.y));
            }
            GlyphCommand::LineTo(point) => {
                path.push_str(&format!("L {:.2} {:.2} ", point.x, 1000.0 - point.y));
            }
            GlyphCommand::QuadTo(control, point) => {
                path.push_str(&format!(
                    "Q {:.2} {:.2} {:.2} {:.2} ",
                    control.x,
                    1000.0 - control.y,
                    point.x,
                    1000.0 - point.y
                ));
            }
            GlyphCommand::CubicTo(control_1, control_2, point) => {
                path.push_str(&format!(
                    "C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} ",
                    control_1.x,
                    1000.0 - control_1.y,
                    control_2.x,
                    1000.0 - control_2.y,
                    point.x,
                    1000.0 - point.y
                ));
            }
            GlyphCommand::Close => path.push_str("Z "),
        }
    }

    path
}

fn refresh_selected_glyph(ui: &MainWindow, project: &NeoFontProject) {
    let selected = ui.get_selected_glyph().to_string();
    let path = project
        .glyphs
        .get(&selected)
        .map(glyph_to_svg_path)
        .unwrap_or_default();
    ui.set_glyph_path(path.into());
}

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
    refresh_selected_glyph(ui, project);
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
            refresh_selected_glyph(&ui, &project.borrow());
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
                        let selected = if loaded.glyphs.contains_key("A") {
                            "A".to_owned()
                        } else {
                            loaded.glyphs.keys().next().cloned().unwrap_or_else(|| ".notdef".to_owned())
                        };
                        ui.set_selected_glyph(selected.clone().into());
                        ui.set_glyph_name(selected.into());
                        update_project_summary(&ui, &loaded, &glyph_model);
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
                    refresh_selected_glyph(&ui, &current_project);
                    ui.set_status_message(format!("Added glyph '{name}'").into());
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
                refresh_selected_glyph(&ui, &project.borrow());
                ui.set_status_message(format!("Selected glyph '{name}'").into());
            }
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        ui.on_canvas_point_added(move |x, y| {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let mut project = project.borrow_mut();
            let Some(glyph) = project.glyphs.get_mut(&selected) else { return };

            let point = GlyphPoint {
                x: x.clamp(0.0, 1000.0).round(),
                y: y.clamp(0.0, 1000.0).round(),
            };
            let starts_contour = glyph
                .outline
                .last()
                .map_or(true, |last| matches!(last, GlyphCommand::Close));
            if starts_contour {
                glyph.outline.push(GlyphCommand::MoveTo(point));
            } else {
                glyph.outline.push(GlyphCommand::LineTo(point));
            }

            ui.set_glyph_path(glyph_to_svg_path(glyph).into());
            ui.set_status_message(
                format!("Added point: ({:.0}, {:.0})", point.x, point.y).into(),
            );
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        ui.on_close_contour_requested(move || {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let mut project = project.borrow_mut();
            let Some(glyph) = project.glyphs.get_mut(&selected) else { return };
            let point_count = glyph
                .outline
                .iter()
                .filter(|command| !matches!(command, GlyphCommand::Close))
                .count();

            if point_count < 3 {
                ui.set_status_message(SharedString::from("A contour needs at least three points"));
                return;
            }
            if !matches!(glyph.outline.last(), Some(GlyphCommand::Close)) {
                glyph.outline.push(GlyphCommand::Close);
            }
            ui.set_glyph_path(glyph_to_svg_path(glyph).into());
            ui.set_status_message(SharedString::from("Contour closed"));
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        ui.on_undo_point_requested(move || {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let mut project = project.borrow_mut();
            let Some(glyph) = project.glyphs.get_mut(&selected) else { return };
            if glyph.outline.pop().is_some() {
                ui.set_glyph_path(glyph_to_svg_path(glyph).into());
                ui.set_status_message(SharedString::from("Removed last outline command"));
            } else {
                ui.set_status_message(SharedString::from("No outline points to undo"));
            }
        });
    }

    println!("Starting NeoUltront {VERSION} desktop editor...");
    ui.run()
}
