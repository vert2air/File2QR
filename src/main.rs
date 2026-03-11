fn main() -> eframe::Result<()> {
    // 起動時の詳細ログを抑制（エラー/警告は表示）
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter_module("wgpu_hal::vulkan::instance", log::LevelFilter::Off)
        .filter_module("wgpu_hal::gles", log::LevelFilter::Error)
        .filter_module("wgpu_hal::dx12", log::LevelFilter::Warn)
        .filter_module("egui_wgpu", log::LevelFilter::Warn)
        .init();

    // 環境変数でレンダラーを明示的に指定可能
    let renderer_env = std::env::var("FILE2QR_RENDERER").ok();

    // 優先順位: 1. wgpu（広い互換性）、2. glow（OpenGL 2.0+必須）
    let renderers_to_try = match renderer_env.as_deref() {
        Some("glow") => vec![eframe::Renderer::Glow], // glowのみ試行
        Some("wgpu") => vec![eframe::Renderer::Wgpu], // wgpuのみ試行
        _ => vec![eframe::Renderer::Wgpu, eframe::Renderer::Glow], // 両方試行
    };

    let mut last_error = None;

    for renderer in renderers_to_try {
        log::info!("レンダラーを試行: {:?}", renderer);

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1024.0, 768.0])
                .with_min_inner_size([800.0, 600.0]),
            renderer,
            ..Default::default()
        };

        let result = eframe::run_native(
            "File2QR Copilot",
            options,
            Box::new(|cc| Ok(Box::new(file2qr::App::new(cc)))),
        );

        match result {
            Ok(_) => return Ok(()),
            Err(e) => {
                log::warn!("{:?}レンダラーでの起動に失敗: {}", renderer, e);
                last_error = Some(e);
                // 次のレンダラーを試行
            }
        }
    }

    // すべて失敗
    if let Some(err) = last_error {
        Err(err)
    } else {
        panic!("利用可能なレンダラーがありません");
    }
}
