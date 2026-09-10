use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();

    let mut page = ShowcasePage::Controls;
    let mut name = String::from("Cube");
    let mut enabled = true;
    let mut autosave = true;
    let mut amount = 4.0_f32;
    let mut mode = Mode::Normal;
    let mut selected_row = 1_usize;
    let mut show_inspector = true;
    let mut show_modal = false;
    let mut show_tooltip = true;
    let mut pathological_overlap = false;
    let mut long_labels = false;
    let mut busy = false;

    eframe::run_ui_native("ViewWitness Showcase", native_options, move |ui, _frame| {
        egui::Panel::top("showcase_top").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("ViewWitness Showcase");
                ui.separator();
                ui.label("A living corpus for GUI witness capture");
            });
        });

        egui::Panel::left("showcase_controls")
            .resizable(true)
            .default_size(230.0)
            .show(ui, |ui| {
                ui.heading("Scenario");
                ui.selectable_value(&mut page, ShowcasePage::Controls, "Controls");
                ui.selectable_value(&mut page, ShowcasePage::Table, "Table + selection");
                ui.selectable_value(&mut page, ShowcasePage::Scrolling, "Scrolling");
                ui.selectable_value(&mut page, ShowcasePage::Canvas, "Custom canvas");
                ui.separator();

                ui.heading("Pressure switches");
                ui.checkbox(&mut show_inspector, "Floating inspector");
                ui.checkbox(&mut pathological_overlap, "Pathological overlap");
                ui.checkbox(&mut show_modal, "Modal-like window");
                ui.checkbox(&mut show_tooltip, "Tooltip affordance");
                ui.checkbox(&mut long_labels, "Long labels");
                ui.checkbox(&mut busy, "Busy / disabled state");

                ui.separator();
                ui.small("Run with EGUI_INSPECTION=1 when compiled with the showcase feature to expose eframe's local inspection endpoint.");
            });

        egui::CentralPanel::default().show(ui, |ui| match page {
            ShowcasePage::Controls => controls_page(
                ui,
                &mut name,
                &mut enabled,
                &mut autosave,
                &mut amount,
                &mut mode,
                show_tooltip,
                long_labels,
                busy,
            ),
            ShowcasePage::Table => table_page(ui, &mut selected_row, long_labels),
            ShowcasePage::Scrolling => scrolling_page(ui, long_labels),
            ShowcasePage::Canvas => canvas_page(ui),
        });

        if show_inspector {
            let position = if pathological_overlap {
                egui::pos2(360.0, 90.0)
            } else {
                egui::pos2(760.0, 90.0)
            };

            egui::Window::new("Inspector")
                .id(egui::Id::new("showcase_inspector"))
                .fixed_pos(position)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    ui.label("Observed properties");
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut name);
                    });
                    ui.add(egui::Slider::new(&mut amount, 0.0..=10.0).text("X"));
                    ui.checkbox(&mut enabled, "Enabled");
                });
        }

        if show_modal {
            egui::Window::new("Destructive confirmation")
                .id(egui::Id::new("showcase_modal"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.label("Delete the selected object?");
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            show_modal = false;
                        }
                        let delete = ui.add_enabled(!busy, egui::Button::new("Delete"));
                        if delete.clicked() {
                            show_modal = false;
                        }
                    });
                });
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShowcasePage {
    Controls,
    Table,
    Scrolling,
    Canvas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Precise,
    Experimental,
}

fn controls_page(
    ui: &mut egui::Ui,
    name: &mut String,
    enabled: &mut bool,
    autosave: &mut bool,
    amount: &mut f32,
    mode: &mut Mode,
    show_tooltip: bool,
    long_labels: bool,
    busy: bool,
) {
    ui.heading("Conventional controls");
    ui.label("This page checks how much useful semantics egui gives ViewWitness without custom instrumentation.");
    ui.separator();

    egui::Grid::new("controls_grid")
        .num_columns(2)
        .spacing([18.0, 10.0])
        .show(ui, |ui| {
            let name_label = ui.label(if long_labels {
                "Object name with an intentionally excessive localized-length-style label"
            } else {
                "Object name"
            });
            ui.text_edit_singleline(name).labelled_by(name_label.id);
            ui.end_row();

            ui.label("Amount");
            ui.add(egui::Slider::new(amount, 0.0..=10.0));
            ui.end_row();

            ui.label("Enabled");
            ui.checkbox(enabled, "Allow editing");
            ui.end_row();

            ui.label("Autosave");
            ui.checkbox(autosave, "Save automatically");
            ui.end_row();
        });

    ui.separator();
    ui.label("Mode");
    ui.horizontal(|ui| {
        ui.radio_value(mode, Mode::Normal, "Normal");
        ui.radio_value(mode, Mode::Precise, "Precise");
        ui.radio_value(mode, Mode::Experimental, "Experimental");
    });

    ui.separator();
    ui.horizontal(|ui| {
        let apply = ui.add_enabled(!busy, egui::Button::new("Apply"));
        let reset = ui.button("Reset");
        if show_tooltip {
            apply.on_hover_text("Apply the current inspector values");
            reset.on_hover_text("Restore showcase defaults");
        }
        if busy {
            ui.spinner();
            ui.label("Applying changes…");
        }
    });
}

fn table_page(ui: &mut egui::Ui, selected_row: &mut usize, long_labels: bool) {
    ui.heading("Table + selection");
    ui.label("Dense repeated structure should remain legible without exploding into useless geometry relations.");
    ui.separator();

    egui::Grid::new("object_table")
        .striped(true)
        .num_columns(4)
        .show(ui, |ui| {
            ui.strong("Selected");
            ui.strong("Object");
            ui.strong("Type");
            ui.strong("Visible");
            ui.end_row();

            for row in 0..8 {
                ui.radio_value(selected_row, row, "");
                let object_name = if long_labels && row == 5 {
                    "Extremely long object name intended to force table pressure"
                } else {
                    match row {
                        0 => "Camera",
                        1 => "Cube",
                        2 => "Key light",
                        3 => "Fill light",
                        4 => "Ground",
                        5 => "Character",
                        6 => "Backdrop",
                        _ => "Marker",
                    }
                };
                ui.label(object_name);
                ui.label(if row < 2 { "Geometry" } else { "Scene node" });
                let mut visible = row != 6;
                ui.checkbox(&mut visible, "");
                ui.end_row();
            }
        });
}

fn scrolling_page(ui: &mut egui::Ui, long_labels: bool) {
    ui.heading("Scrolling + partial visibility");
    ui.label(
        "This page exists to pressure clip-space, viewport, and virtualized-content assumptions.",
    );
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("showcase_scroll")
        .max_height(340.0)
        .show(ui, |ui| {
            for index in 0..60 {
                ui.horizontal(|ui| {
                    ui.label(format!("Row {index:02}"));
                    if long_labels && index % 7 == 0 {
                        ui.label("A deliberately long row label that should extend the layout pressure surface");
                    } else {
                        ui.label("Scrollable content");
                    }
                    let _ = ui.button("Action");
                });
            }
        });
}

fn canvas_page(ui: &mut egui::Ui) {
    ui.heading("Custom-painted canvas — negative control");
    ui.label("The visible shapes below intentionally test what AccessKit does NOT know without extra semantic instrumentation.");
    ui.separator();

    let desired_size = egui::vec2(ui.available_width().min(620.0), 360.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 6.0, ui.visuals().extreme_bg_color);

    let first =
        egui::Rect::from_min_size(rect.min + egui::vec2(70.0, 70.0), egui::vec2(150.0, 100.0));
    let second = egui::Rect::from_min_size(
        rect.min + egui::vec2(310.0, 170.0),
        egui::vec2(190.0, 110.0),
    );
    painter.rect_stroke(
        first,
        8.0,
        egui::Stroke::new(2.0, ui.visuals().widgets.active.fg_stroke.color),
        egui::StrokeKind::Middle,
    );
    painter.circle_stroke(
        second.center(),
        52.0,
        egui::Stroke::new(2.0, ui.visuals().widgets.hovered.fg_stroke.color),
    );
    painter.text(
        first.center(),
        egui::Align2::CENTER_CENTER,
        "Painted rectangle",
        egui::FontId::proportional(16.0),
        ui.visuals().text_color(),
    );
    painter.text(
        second.center(),
        egui::Align2::CENTER_CENTER,
        "Painted circle",
        egui::FontId::proportional(16.0),
        ui.visuals().text_color(),
    );

    if response.dragged() {
        ui.ctx().request_repaint();
    }

    ui.small("Expected result: the canvas interaction surface may be visible semantically, but the two painted objects require additional ViewWitness instrumentation if agents are to reason about them as objects.");
}
