use std::{
    net::TcpListener,
    sync::{Arc, OnceLock},
    thread,
    time::Duration,
};

use eframe::egui;
use viewwitness::{
    DEFAULT_EGUI_CAPTURE_ADDR, DEFAULT_EGUI_PAINT_ADDR, EguiFrameProbe, EguiPaintAnnotator,
    EguiPaintObjectDescriptor, EguiPaintReporter, run_egui_capture_server, run_egui_paint_server,
};

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    let paint_listener = bind_viewwitness_listener(DEFAULT_EGUI_PAINT_ADDR, "continuous paint");
    let capture_listener = bind_viewwitness_listener(DEFAULT_EGUI_CAPTURE_ADDR, "exact capture");
    let annotator_slot = Arc::new(OnceLock::new());
    let canvas_annotator = Arc::clone(&annotator_slot);
    let install_annotator = Arc::clone(&annotator_slot);

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
    let mut misplaced_canvas_handle = false;

    let ui_fun = move |ui: &mut egui::Ui, _frame: &mut eframe::Frame| {
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
                ui.checkbox(&mut misplaced_canvas_handle, "Misplaced canvas handle");

                ui.separator();
                ui.small("ViewWitness paint and exact-capture services run on worker threads. Set EGUI_INSPECTION=1 to additionally expose eframe's upstream local inspection endpoint.");
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
            ShowcasePage::Canvas => {
                canvas_page(ui, canvas_annotator.get(), misplaced_canvas_handle)
            }
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
    };

    eframe::run_native(
        "ViewWitness Showcase",
        native_options,
        Box::new(move |cc| {
            if let Some(annotator) =
                install_viewwitness_services(&cc.egui_ctx, paint_listener, capture_listener)
            {
                let _ = install_annotator.set(annotator);
            }
            Ok(Box::new(ShowcaseClosure { ui_fun }))
        }),
    )
}

struct ShowcaseClosure<F> {
    ui_fun: F,
}

impl<F> eframe::App for ShowcaseClosure<F>
where
    F: FnMut(&mut egui::Ui, &mut eframe::Frame) + 'static,
{
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        (self.ui_fun)(ui, frame);
    }
}

fn bind_viewwitness_listener(addr: &str, service: &str) -> Option<TcpListener> {
    match TcpListener::bind(addr) {
        Ok(listener) => {
            eprintln!("ViewWitness showcase {service} endpoint: {addr}");
            Some(listener)
        }
        Err(error) => {
            eprintln!("ViewWitness showcase could not bind {service} endpoint {addr}: {error}");
            None
        }
    }
}

fn install_viewwitness_services(
    ctx: &egui::Context,
    paint_listener: Option<TcpListener>,
    capture_listener: Option<TcpListener>,
) -> Option<EguiPaintAnnotator> {
    if let Some(listener) = paint_listener {
        let (reporter, receiver) = EguiPaintReporter::channel(2);
        ctx.add_plugin(reporter);
        if let Err(error) = thread::Builder::new()
            .name("viewwitness-paint-server".into())
            .spawn(move || {
                if let Err(error) = run_egui_paint_server(listener, receiver) {
                    eprintln!("ViewWitness showcase paint server stopped: {error}");
                }
            })
        {
            eprintln!("ViewWitness showcase could not start paint worker: {error}");
        }
    }

    if let Some(listener) = capture_listener {
        let probe = EguiFrameProbe::install(ctx, 1);
        let annotator = probe.annotator();
        if let Err(error) = thread::Builder::new()
            .name("viewwitness-capture-server".into())
            .spawn(move || {
                if let Err(error) = run_egui_capture_server(listener, probe, Duration::from_secs(2))
                {
                    eprintln!("ViewWitness showcase exact capture server stopped: {error}");
                }
            })
        {
            eprintln!("ViewWitness showcase could not start exact capture worker: {error}");
        }
        return Some(annotator);
    }

    None
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

fn canvas_page(
    ui: &mut egui::Ui,
    annotator: Option<&EguiPaintAnnotator>,
    misplaced_canvas_handle: bool,
) {
    use egui::epaint::{CircleShape, RectShape};

    ui.heading("Custom-painted canvas — explicit multi-shape identity pressure test");
    ui.label("Each visible canvas object below is application-authored identity bound to multiple real egui paint handles during exact capture. Labels remain unannotated generic paint.");
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
    let rectangle_handle_center = if misplaced_canvas_handle {
        first.right_center() + egui::vec2(60.0, 0.0)
    } else {
        first.right_center()
    };
    let rectangle_handle =
        egui::Rect::from_center_size(rectangle_handle_center, egui::vec2(10.0, 10.0));

    if let Some(annotator) = annotator {
        annotator.paint_object(
            EguiPaintObjectDescriptor::new("showcase:painted-rectangle", "diagram_node")
                .with_name("Painted rectangle"),
            |object| {
                object.add_shape_with_id(
                    &painter,
                    "outline",
                    RectShape::stroke(
                        first,
                        8.0,
                        egui::Stroke::new(2.0, ui.visuals().widgets.active.fg_stroke.color),
                        egui::StrokeKind::Middle,
                    ),
                );
                object.add_shape_with_id(
                    &painter,
                    "handle",
                    RectShape::filled(
                        rectangle_handle,
                        2.0,
                        ui.visuals().widgets.active.fg_stroke.color,
                    ),
                );
            },
        );
        annotator.paint_object(
            EguiPaintObjectDescriptor::new("showcase:painted-circle", "diagram_node")
                .with_name("Painted circle"),
            |object| {
                object.add_shape_with_id(
                    &painter,
                    "ring",
                    CircleShape {
                        center: second.center(),
                        radius: 52.0,
                        fill: egui::Color32::TRANSPARENT,
                        stroke: egui::Stroke::new(
                            2.0,
                            ui.visuals().widgets.hovered.fg_stroke.color,
                        ),
                    },
                );
                object.add_shape_with_id(
                    &painter,
                    "center",
                    CircleShape {
                        center: second.center(),
                        radius: 4.0,
                        fill: ui.visuals().widgets.hovered.fg_stroke.color,
                        stroke: egui::Stroke::NONE,
                    },
                );
            },
        );
    } else {
        painter.add(RectShape::stroke(
            first,
            8.0,
            egui::Stroke::new(2.0, ui.visuals().widgets.active.fg_stroke.color),
            egui::StrokeKind::Middle,
        ));
        painter.add(RectShape::filled(
            rectangle_handle,
            2.0,
            ui.visuals().widgets.active.fg_stroke.color,
        ));
        painter.add(CircleShape {
            center: second.center(),
            radius: 52.0,
            fill: egui::Color32::TRANSPARENT,
            stroke: egui::Stroke::new(2.0, ui.visuals().widgets.hovered.fg_stroke.color),
        });
        painter.add(CircleShape {
            center: second.center(),
            radius: 4.0,
            fill: ui.visuals().widgets.hovered.fg_stroke.color,
            stroke: egui::Stroke::NONE,
        });
    }

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

    if misplaced_canvas_handle {
        ui.strong("Pressure defect active: the keyed rectangle handle is intentionally displaced 60 px to the right.");
    }
    ui.small("Expected exact capture: two authored logical objects with four verified, keyed paint bindings (`outline`, `handle`, `ring`, `center`). The canvas background and text labels remain generic renderer evidence, so ViewWitness still does not infer identity for unannotated submissions.");
}
