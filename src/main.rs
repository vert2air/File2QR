fn main() -> eframe::Result<()> {
    // 起動時の詳細ログを抑制（エラー/警告は表示）
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter_module("wgpu_hal::vulkan::instance", log::LevelFilter::Off)
        .filter_module("wgpu_hal::gles", log::LevelFilter::Error)
        .filter_module("wgpu_hal::dx12", log::LevelFilter::Warn)
        .filter_module("egui_wgpu", log::LevelFilter::Warn)
        .init();

    // 環境変数でレンダラーを切り替え可能
    // FILE2QR_RENDERER=glow でglowを使用（軽量、仮想環境推奨）
    // FILE2QR_RENDERER=wgpu でwgpuを使用（高性能、物理マシン推奨）
    let renderer = std::env::var("FILE2QR_RENDERER")
        .ok()
        .and_then(|s| match s.to_lowercase().as_str() {
            "glow" => Some(eframe::Renderer::Glow),
            "wgpu" => Some(eframe::Renderer::Wgpu),
            _ => None,
        })
        .unwrap_or(eframe::Renderer::Glow); // デフォルトはglow（軽量）

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0]),
        renderer,
        ..Default::default()
    };

    eframe::run_native(
        "File2QR Copilot",
        options,
        Box::new(|cc| Ok(Box::new(file2qr::App::new(cc)))),
    )
}
