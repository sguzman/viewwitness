use viewwitness::{EguiCaptureContext, Viewport, to_yaml, witness_from_egui_output};

fn main() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();

    let mut enabled = true;
    let mut amount = 4.0_f32;
    let mut name = String::from("Cube");

    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(640.0, 480.0),
            )),
            ..Default::default()
        },
        |ui| {
            ui.heading("ViewWitness headless showcase");
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut name);
            });
            ui.checkbox(&mut enabled, "Enabled");
            ui.add(egui::Slider::new(&mut amount, 0.0..=10.0).text("Amount"));
            ui.button("Apply");
        },
    );

    let witness = witness_from_egui_output(
        &output,
        EguiCaptureContext::new(Viewport {
            width: 640.0,
            height: 480.0,
            scale_factor: 1.0,
        })
        .with_frame(1),
    )
    .expect("AccessKit output should be available after enable_accesskit");

    print!("{}", to_yaml(&witness).expect("witness serializes"));
    output.drop_without_applying_deltas();
}
