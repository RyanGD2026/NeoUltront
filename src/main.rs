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

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutlineTool { Line, Quadratic, Cubic }

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

fn glyph_handles_to_svg_path(glyph: &Glyph) -> String {
    let mut path = String::new();
    let mut current = GlyphPoint { x: 0.0, y: 0.0 };
    for command in &glyph.outline {
        match command {
            GlyphCommand::MoveTo(p) | GlyphCommand::LineTo(p) => current = *p,
            GlyphCommand::QuadTo(c, end) => {
                path.push_str(&format!("M {} {} L {} {} ", current.x, 1000.0-current.y, c.x, 1000.0-c.y));
                current = *end;
            }
            GlyphCommand::CubicTo(c1, c2, end) => {
                path.push_str(&format!("M {} {} L {} {} M {} {} L {} {} ", current.x, 1000.0-current.y, c1.x, 1000.0-c1.y, end.x, 1000.0-end.y, c2.x, 1000.0-c2.y));
                current = *end;
            }
            GlyphCommand::Close => {}
        }
    }
    path
}


fn command_point(command: &GlyphCommand, slot: u8) -> Option<GlyphPoint> {
    match (command, slot) {
        (GlyphCommand::MoveTo(p) | GlyphCommand::LineTo(p), 0) => Some(*p),
        (GlyphCommand::QuadTo(control, end), 0) => Some(*control),
        (GlyphCommand::QuadTo(_, end), 1) => Some(*end),
        (GlyphCommand::CubicTo(c1, _, _), 0) => Some(*c1),
        (GlyphCommand::CubicTo(_, c2, _), 1) => Some(*c2),
        (GlyphCommand::CubicTo(_, _, end), 2) => Some(*end),
        _ => None,
    }
}

fn set_command_point(command: &mut GlyphCommand, slot: u8, point: GlyphPoint) -> bool {
    match (command, slot) {
        (GlyphCommand::MoveTo(p) | GlyphCommand::LineTo(p), 0) => *p = point,
        (GlyphCommand::QuadTo(p, _), 0) => *p = point,
        (GlyphCommand::QuadTo(_, p), 1) => *p = point,
        (GlyphCommand::CubicTo(p, _, _), 0) => *p = point,
        (GlyphCommand::CubicTo(_, p, _), 1) => *p = point,
        (GlyphCommand::CubicTo(_, _, p), 2) => *p = point,
        _ => return false,
    }
    true
}

fn nearest_glyph_point(glyph: &Glyph, point: GlyphPoint, tolerance: f32) -> Option<(usize, u8)> {
    let mut nearest = None;
    let mut best_distance = tolerance * tolerance;
    for (command_index, command) in glyph.outline.iter().enumerate() {
        let slots = match command {
            GlyphCommand::MoveTo(_) | GlyphCommand::LineTo(_) => 1,
            GlyphCommand::QuadTo(_, _) => 2,
            GlyphCommand::CubicTo(_, _, _) => 3,
            GlyphCommand::Close => 0,
        };
        for slot in 0..slots {
            if let Some(candidate) = command_point(command, slot) {
                let dx = candidate.x - point.x;
                let dy = candidate.y - point.y;
                let distance = dx * dx + dy * dy;
                if distance <= best_distance {
                    best_distance = distance;
                    nearest = Some((command_index, slot));
                }
            }
        }
    }
    nearest
}

fn refresh_selected_glyph(ui: &MainWindow, project: &NeoFontProject) {
    let selected = ui.get_selected_glyph().to_string();
    let path = project
        .glyphs
        .get(&selected)
        .map(glyph_to_svg_path)
        .unwrap_or_default();
    ui.set_glyph_path(path.into());
    let handles = project.glyphs.get(&selected).map(glyph_handles_to_svg_path).unwrap_or_default();
    ui.set_glyph_handles_path(handles.into());
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
    let outline_tool = Rc::new(RefCell::new(OutlineTool::Line));
    let pending_curve_points = Rc::new(RefCell::new(Vec::<GlyphPoint>::new()));
    let selected_node = Rc::new(RefCell::new(None::<(usize, u8)>));
    let node_drag_active = Rc::new(RefCell::new(false));
    let undo_history = Rc::new(RefCell::new(Vec::<(String, Vec<GlyphCommand>)>::new()));
    let redo_history = Rc::new(RefCell::new(Vec::<(String, Vec<GlyphCommand>)>::new()));
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
        let tool_state = Rc::clone(&outline_tool);
        let pending = Rc::clone(&pending_curve_points);
        ui.on_outline_tool_selected(move |name| {
            *tool_state.borrow_mut() = match name.as_str() {
                "quadratic" => OutlineTool::Quadratic,
                "cubic" => OutlineTool::Cubic,
                _ => OutlineTool::Line,
            };
            pending.borrow_mut().clear();
            if let Some(ui) = weak_ui.upgrade() { ui.set_status_message(format!("Outline tool: {name}").into()); }
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        let selected_node = Rc::clone(&selected_node);
        let dragging = Rc::clone(&node_drag_active);
        let undo = Rc::clone(&undo_history);
        let redo = Rc::clone(&redo_history);
        ui.on_canvas_pointer_pressed(move |x, y| {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let point = GlyphPoint { x: x.clamp(0.0, 1000.0), y: y.clamp(0.0, 1000.0) };
            let project = project.borrow();
            let hit = project.glyphs.get(&selected)
                .and_then(|glyph| nearest_glyph_point(glyph, point, 32.0));
            *selected_node.borrow_mut() = hit;
            *dragging.borrow_mut() = hit.is_some();
            if hit.is_some() {
                if let Some(glyph) = project.glyphs.get(&selected) {
                    undo.borrow_mut().push((selected.clone(), glyph.outline.clone()));
                    redo.borrow_mut().clear();
                }
            }
            if let Some((command_index, slot)) = hit {
                if let Some(glyph) = project.glyphs.get(&selected) {
                    if let Some(node) = glyph.outline.get(command_index).and_then(|c| command_point(c, slot)) {
                        ui.set_selected_node_x(node.x);
                        ui.set_selected_node_y(node.y);
                        ui.set_node_selected(true);
                        ui.set_status_message(format!("Selected node {}:{}", command_index + 1, slot + 1).into());
                    }
                }
            } else {
                ui.set_node_selected(false);
            }
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        let selected_node = Rc::clone(&selected_node);
        let dragging = Rc::clone(&node_drag_active);
        ui.on_canvas_pointer_moved(move |x, y| {
            if !*dragging.borrow() { return; }
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let Some((command_index, slot)) = *selected_node.borrow() else { return };
            let point = GlyphPoint { x: x.clamp(0.0, 1000.0).round(), y: y.clamp(0.0, 1000.0).round() };
            let mut project = project.borrow_mut();
            let Some(glyph) = project.glyphs.get_mut(&selected) else { return };
            let Some(command) = glyph.outline.get_mut(command_index) else { return };
            if set_command_point(command, slot, point) {
                ui.set_glyph_path(glyph_to_svg_path(glyph).into());
                ui.set_glyph_handles_path(glyph_handles_to_svg_path(glyph).into());
                ui.set_selected_node_x(point.x);
                ui.set_selected_node_y(point.y);
            }
        });
    }

    {
        let dragging = Rc::clone(&node_drag_active);
        ui.on_canvas_pointer_released(move || { *dragging.borrow_mut() = false; });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        let tool_state = Rc::clone(&outline_tool);
        let pending = Rc::clone(&pending_curve_points);
        let undo = Rc::clone(&undo_history);
        let redo = Rc::clone(&redo_history);
        ui.on_canvas_point_added(move |x, y| {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let mut project = project.borrow_mut();
            let Some(glyph) = project.glyphs.get_mut(&selected) else { return };

            let point = GlyphPoint {
                x: x.clamp(0.0, 1000.0).round(),
                y: y.clamp(0.0, 1000.0).round(),
            };
            let tool = *tool_state.borrow();
            if tool != OutlineTool::Line {
                let mut points = pending.borrow_mut();
                points.push(point);
                let needed = if tool == OutlineTool::Quadratic { 2 } else { 3 };
                if points.len() == needed {
                    let has_start = glyph.outline.iter().any(|c| matches!(c, GlyphCommand::MoveTo(_) | GlyphCommand::LineTo(_) | GlyphCommand::QuadTo(_, _) | GlyphCommand::CubicTo(_, _, _)));
                    if has_start {
                        undo.borrow_mut().push((selected.clone(), glyph.outline.clone()));
                        redo.borrow_mut().clear();
                        if tool == OutlineTool::Quadratic {
                            glyph.outline.push(GlyphCommand::QuadTo(points[0], points[1]));
                        } else {
                            glyph.outline.push(GlyphCommand::CubicTo(points[0], points[1], points[2]));
                        }
                    } else {
                        ui.set_status_message(SharedString::from("Add a starting point before a Bézier segment"));
                    }
                    points.clear();
                } else {
                    ui.set_status_message(format!("Bézier control point {}/{}", points.len(), needed).into());
                }
                ui.set_glyph_path(glyph_to_svg_path(glyph).into());
                ui.set_glyph_handles_path(glyph_handles_to_svg_path(glyph).into());
                return;
            }
            undo.borrow_mut().push((selected.clone(), glyph.outline.clone()));
            redo.borrow_mut().clear();
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
        let undo = Rc::clone(&undo_history);
        let redo = Rc::clone(&redo_history);
        ui.on_close_contour_requested(move || {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let mut project = project.borrow_mut();
            let Some(glyph) = project.glyphs.get_mut(&selected) else { return };
            let point_count = glyph
                .outline
                .iter()
                .rev()
                .take_while(|command| !matches!(command, GlyphCommand::Close))
                .count();

            if point_count < 3 {
                ui.set_status_message(SharedString::from("A contour needs at least three points"));
                return;
            }
            if !matches!(glyph.outline.last(), Some(GlyphCommand::Close)) {
                undo.borrow_mut().push((selected.clone(), glyph.outline.clone()));
                redo.borrow_mut().clear();
                glyph.outline.push(GlyphCommand::Close);
            }
            ui.set_glyph_path(glyph_to_svg_path(glyph).into());
            ui.set_status_message(SharedString::from("Contour closed"));
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        let undo = Rc::clone(&undo_history);
        let redo = Rc::clone(&redo_history);
        ui.on_undo_requested(move || {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let Some((glyph_name, previous)) = undo.borrow_mut().pop() else {
                ui.set_status_message("Nothing to undo".into());
                return;
            };
            let mut project = project.borrow_mut();
            if let Some(glyph) = project.glyphs.get_mut(&glyph_name) {
                redo.borrow_mut().push((glyph_name.clone(), glyph.outline.clone()));
                glyph.outline = previous;
            }
            if glyph_name == selected {
                if let Some(glyph) = project.glyphs.get(&selected) {
                    ui.set_glyph_path(glyph_to_svg_path(glyph).into());
                    ui.set_glyph_handles_path(glyph_handles_to_svg_path(glyph).into());
                }
                ui.set_node_selected(false);
            }
            ui.set_status_message("Undo completed".into());
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        let undo = Rc::clone(&undo_history);
        let redo = Rc::clone(&redo_history);
        ui.on_redo_requested(move || {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let Some((glyph_name, next)) = redo.borrow_mut().pop() else {
                ui.set_status_message("Nothing to redo".into());
                return;
            };
            let mut project = project.borrow_mut();
            if let Some(glyph) = project.glyphs.get_mut(&glyph_name) {
                undo.borrow_mut().push((glyph_name.clone(), glyph.outline.clone()));
                glyph.outline = next;
            }
            if glyph_name == selected {
                if let Some(glyph) = project.glyphs.get(&selected) {
                    ui.set_glyph_path(glyph_to_svg_path(glyph).into());
                    ui.set_glyph_handles_path(glyph_handles_to_svg_path(glyph).into());
                }
                ui.set_node_selected(false);
            }
            ui.set_status_message("Redo completed".into());
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        let selected_node = Rc::clone(&selected_node);
        let undo = Rc::clone(&undo_history);
        let redo = Rc::clone(&redo_history);
        ui.on_delete_node_requested(move || {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let Some((command_index, slot)) = *selected_node.borrow() else {
                ui.set_status_message("Select a node first".into());
                return;
            };
            let mut project = project.borrow_mut();
            let Some(glyph) = project.glyphs.get_mut(&selected) else { return };
            let Some(command) = glyph.outline.get(command_index).cloned() else { return };
            if matches!(command, GlyphCommand::Close) {
                ui.set_status_message("Contour-close markers cannot be deleted as nodes".into());
                return;
            }
            undo.borrow_mut().push((selected.clone(), glyph.outline.clone()));
            redo.borrow_mut().clear();
            match command {
                GlyphCommand::MoveTo(_) => {
                    let end = glyph.outline[command_index + 1..].iter()
                        .position(|c| matches!(c, GlyphCommand::Close | GlyphCommand::MoveTo(_)))
                        .map(|offset| command_index + 1 + offset)
                        .unwrap_or(glyph.outline.len());
                    glyph.outline.drain(command_index..end);
                }
                GlyphCommand::LineTo(_) => { glyph.outline.remove(command_index); }
                GlyphCommand::QuadTo(_, endpoint) if slot == 0 => {
                    glyph.outline[command_index] = GlyphCommand::LineTo(endpoint);
                }
                GlyphCommand::QuadTo(_, _) => { glyph.outline.remove(command_index); }
                GlyphCommand::CubicTo(_, _, endpoint) if slot < 2 => {
                    glyph.outline[command_index] = GlyphCommand::LineTo(endpoint);
                }
                GlyphCommand::CubicTo(_, _, _) => { glyph.outline.remove(command_index); }
                GlyphCommand::Close => unreachable!(),
            }
            *selected_node.borrow_mut() = None;
            ui.set_node_selected(false);
            ui.set_glyph_path(glyph_to_svg_path(glyph).into());
            ui.set_glyph_handles_path(glyph_handles_to_svg_path(glyph).into());
            ui.set_status_message("Selected node deleted".into());
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        let undo = Rc::clone(&undo_history);
        let redo = Rc::clone(&redo_history);
        ui.on_undo_point_requested(move || {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let mut project = project.borrow_mut();
            let Some(glyph) = project.glyphs.get_mut(&selected) else { return };
            if !glyph.outline.is_empty() {
                undo.borrow_mut().push((selected.clone(), glyph.outline.clone()));
                redo.borrow_mut().clear();
                glyph.outline.pop();
                ui.set_glyph_path(glyph_to_svg_path(glyph).into());
                ui.set_glyph_handles_path(glyph_handles_to_svg_path(glyph).into());
                ui.set_status_message("Removed last outline command".into());
            } else {
                ui.set_status_message("No outline points to remove".into());
            }
        });
    }

    {
        let weak_ui = ui.as_weak();
        let project = Rc::clone(&project);
        ui.on_export_svg_requested(move || {
            let Some(ui) = weak_ui.upgrade() else { return };
            let selected = ui.get_selected_glyph().to_string();
            let Some(glyph) = project.borrow().glyphs.get(&selected).cloned() else {
                ui.set_status_message(format!("Glyph '{selected}' was not found").into());
                return;
            };
            let Some(path) = rfd::FileDialog::new()
                .add_filter("SVG vector image", &["svg"])
                .set_file_name(format!("{selected}.svg"))
                .save_file()
            else {
                return;
            };

            let svg_path = glyph_to_svg_path(&glyph);
            let svg = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 1000">
  <title>{}</title>
  <path d="{}" fill="#000000" fill-rule="evenodd"/>
</svg>
"##,
                selected, svg_path
            );
            match std::fs::write(&path, svg) {
                Ok(()) => ui.set_status_message(format!("Exported {} as SVG", path.display()).into()),
                Err(error) => ui.set_status_message(format!("SVG export failed: {error}").into()),
            }
        });
    }

    println!("Starting NeoUltront {VERSION} desktop editor...");
    ui.run()
}
