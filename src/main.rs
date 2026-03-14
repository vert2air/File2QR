fn main() -> iced::Result {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let (font_data, font_name) = find_japanese_font();

    let mut builder = iced::application(
        "File2QR Copilot",
        file2qr::App::update,
        file2qr::App::view,
    )
    .window_size((1024.0, 768.0))
    .subscription(file2qr::App::subscription);

    if let Some(data) = font_data {
        builder = builder.font(data);
        if let Some(name) = font_name {
            builder = builder.default_font(iced::Font {
                family: iced::font::Family::Name(name),
                weight: iced::font::Weight::Normal,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            });
        }
    }

    builder.run()
}

fn find_japanese_font() -> (Option<&'static [u8]>, Option<&'static str>) {
    let candidates: &[(&str, &str)] = &[
        (r"C:\Windows\Fonts\YuGothR.ttc", "Yu Gothic"),
        (r"C:\Windows\Fonts\YuGothM.ttc", "Yu Gothic"),
        (r"C:\Windows\Fonts\meiryo.ttc", "Meiryo"),
        (r"C:\Windows\Fonts\msgothic.ttc", "MS Gothic"),
        ("/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc", "Hiragino Sans"),
        (
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "Noto Sans CJK JP",
        ),
        (
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "Noto Sans CJK JP",
        ),
    ];

    for (path, family_name) in candidates {
        if let Ok(data) = std::fs::read(path) {
            eprintln!(
                "日本語フォントを読み込みました: {} (family: {})",
                path, family_name
            );
            let leaked: &'static [u8] = Box::leak(data.into_boxed_slice());
            return (Some(leaked), Some(family_name));
        }
    }

    eprintln!("警告: 日本語フォントが見つかりませんでした。");
    (None, None)
}
