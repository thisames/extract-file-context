mod config;
mod extractor;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Project Context Extractor"),
        ..Default::default()
    };

    eframe::run_native(
        "Project Context Extractor",
        options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::dark();
            visuals.override_text_color = Some(egui::Color32::from_rgb(212, 212, 212));
            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(ui::App::new()))
        }),
    )
}
