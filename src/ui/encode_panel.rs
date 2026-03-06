use crate::encode::{self, EcLevel, EncodeInput};
use crate::ui::qr_window::QrWindow;
use eframe::egui;

/// 入力モード
#[derive(PartialEq)]
pub enum InputMode {
    File,
    DirectText,
}

pub struct EncodePanel {
    pub input_mode: InputMode,
    pub file_path: String,
    pub direct_text: String,
    pub compress: bool,
    pub ec_level: EcLevel,

    // QRウィンドウ状態
    pub qr_window: Option<QrWindow>,

    // エラーメッセージ
    pub error_msg: Option<String>,
}

impl Default for EncodePanel {
    fn default() -> Self {
        Self {
            input_mode: InputMode::File,
            file_path: String::new(),
            direct_text: String::new(),
            compress: false,
            ec_level: EcLevel::L,
            qr_window: None,
            error_msg: None,
        }
    }
}

impl EncodePanel {
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // ── 入力モード選択 ──
        ui.horizontal(|ui| {
            ui.label("入力モード:");
            ui.radio_value(&mut self.input_mode, InputMode::File, "ファイル");
            ui.radio_value(
                &mut self.input_mode,
                InputMode::DirectText,
                "テキスト直接入力",
            );
        });

        ui.separator();

        // ── ファイル指定 or テキスト入力 ──
        match self.input_mode {
            InputMode::File => {
                self.show_file_input(ctx, ui);
            }
            InputMode::DirectText => {
                self.show_text_input(ui);
            }
        }

        ui.separator();

        // ── オプション ──
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.compress, "xz圧縮を有効にする");
        });

        ui.horizontal(|ui| {
            ui.label("エラー訂正レベル:");
            egui::ComboBox::from_label("")
                .selected_text(format!(
                    "{} (最大 {} byte / QR)",
                    self.ec_level.label(),
                    self.ec_level.max_bytes()
                ))
                .show_ui(ui, |ui| {
                    for &level in EcLevel::all() {
                        let label = format!(
                            "{} (最大 {} byte / QR)",
                            level.label(),
                            level.max_bytes()
                        );
                        ui.selectable_value(&mut self.ec_level, level, label);
                    }
                });
        });

        ui.separator();

        // ── 生成ボタン ──
        if ui
            .add_sized([200.0, 36.0], egui::Button::new("🔲 QRコードを生成"))
            .clicked()
        {
            self.generate(ctx);
        }

        // ── エラー表示 ──
        if let Some(ref err) = self.error_msg {
            ui.colored_label(egui::Color32::RED, format!("❌ {}", err));
        }

        // ── QRウィンドウの描画は app.rs で行う ──
    }

    fn show_file_input(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.label("ファイルパス:");
        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.file_path)
                    .hint_text("絶対パス、相対パス、またはドラッグ&ドロップ")
                    .desired_width(500.0),
            );

            // ドラッグ&ドロップ
            if response.hovered() {
                ctx.input(|i| {
                    if let Some(path) = i.raw.dropped_files.first()
                        && let Some(p) = &path.path
                    {
                        self.file_path = p.to_string_lossy().to_string();
                    }
                });
            }

            if ui.button("📂 選択...").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_file()
            {
                self.file_path = path.to_string_lossy().to_string();
            }
        });

        // グローバルなD&Dも受け付ける
        ctx.input(|i| {
            if let Some(path) = i.raw.dropped_files.first()
                && let Some(p) = &path.path
            {
                self.file_path = p.to_string_lossy().to_string();
                self.error_msg = None;
            }
        });
    }

    fn show_text_input(&mut self, ui: &mut egui::Ui) {
        ui.label("テキスト入力:");
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.direct_text)
                    .desired_width(f32::INFINITY)
                    .desired_rows(8)
                    .hint_text("ここにテキストを入力してください"),
            );
        });
    }

    fn generate(&mut self, ctx: &egui::Context) {
        self.error_msg = None;

        // 入力データとファイル名を取得
        let (data, filename) = match self.input_mode {
            InputMode::File => {
                if self.file_path.trim().is_empty() {
                    self.error_msg =
                        Some("ファイルパスを指定してください".to_string());
                    return;
                }
                match std::fs::read(&self.file_path) {
                    Ok(bytes) => {
                        let fname = std::path::Path::new(&self.file_path)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        (bytes, fname)
                    }
                    Err(e) => {
                        self.error_msg =
                            Some(format!("ファイル読み込みエラー: {}", e));
                        return;
                    }
                }
            }
            InputMode::DirectText => {
                if self.direct_text.trim().is_empty() {
                    self.error_msg =
                        Some("テキストを入力してください".to_string());
                    return;
                }
                (
                    self.direct_text.as_bytes().to_vec(),
                    "(direct_text)".to_string(),
                )
            }
        };

        // エンコード実行
        let result = encode::encode(EncodeInput {
            data,
            filename,
            compress: self.compress,
            ec_level: self.ec_level,
        });

        match result {
            Ok(res) => {
                self.qr_window =
                    Some(QrWindow::new(ctx, res.fragments, self.ec_level));
            }
            Err(e) => {
                self.error_msg = Some(e);
            }
        }
    }
}
