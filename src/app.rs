use crate::ui::{decode_panel::DecodePanel, encode_panel::EncodePanel};
use eframe::egui;

#[derive(PartialEq)]
pub enum Tab {
    Encode,
    Decode,
}

pub struct App {
    pub current_tab: Tab,
    pub encode_panel: EncodePanel,
    pub decode_panel: DecodePanel,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_fonts(&cc.egui_ctx);

        // ID重複警告を無効化
        cc.egui_ctx.options_mut(|o| {
            o.warn_on_id_clash = false;
        });

        Self {
            current_tab: Tab::Encode,
            encode_panel: EncodePanel::default(),
            decode_panel: DecodePanel::default(),
        }
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // システムフォントから日本語対応フォントを探して読み込む
    let candidates = [
        // Windows
        r"C:\Windows\Fonts\msgothic.ttc",
        r"C:\Windows\Fonts\meiryo.ttc",
        r"C:\Windows\Fonts\YuGothM.ttc",
        r"C:\Windows\Fonts\NotoSansCJK-Regular.ttc",
        // macOS
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/Library/Fonts/Arial Unicode MS.ttf",
        // Linux
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJKjp-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    ];

    let mut loaded = false;
    for path in &candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(
                "japanese".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(data)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "japanese".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "japanese".to_owned());
            loaded = true;
            break;
        }
    }

    if !loaded {
        eprintln!("警告: 日本語フォントが見つかりませんでした。");
    }

    ctx.set_fonts(fonts);
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.current_tab,
                    Tab::Encode,
                    "QRコード生成",
                );
                ui.selectable_value(
                    &mut self.current_tab,
                    Tab::Decode,
                    "データ復元",
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.current_tab {
            Tab::Encode => self.encode_panel.show(ctx, ui),
            Tab::Decode => self.decode_panel.show(ctx, ui),
        });

        // QRコード表示ウィンドウの更新（通常のWindow）
        if let Some(ref mut qr_win) = self.encode_panel.qr_window {
            qr_win.show(ctx);
        }
    }
}
