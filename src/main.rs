mod app;
mod encode;
mod decode;
mod ui;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("File2QR Copilot")
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([700.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "File2QR Copilot",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
