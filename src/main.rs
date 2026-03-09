fn main() -> eframe::Result<()> {
    // 起動時のVulkan関連警告を抑制（本当のエラーは表示される）
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info) // デフォルトはInfo以上
        .filter_module("wgpu_hal::vulkan::instance", log::LevelFilter::Off) // Vulkan初期化のみ非表示
        .filter_module("egui_wgpu", log::LevelFilter::Warn) // egui_wgpuの詳細情報を抑制
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "File2QR Copilot",
        options,
        Box::new(|cc| Ok(Box::new(file2qr::App::new(cc)))),
    )
}
