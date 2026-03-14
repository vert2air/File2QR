use crate::encode::{self, EcLevel, EncodeInput};
use crate::ui::qr_window::{QrWindow, QrWindowMessage};
use iced::widget::{
    button, checkbox, column, combo_box, container, horizontal_rule, row,
    scrollable, text, text_input,
};
use iced::{Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    File,
    DirectText,
}

#[derive(Debug, Clone)]
pub enum EncodeMessage {
    InputModeChanged(InputMode),
    FilePathChanged(String),
    FilePickRequested,
    FileDropped(String),
    DirectTextChanged(String),
    CompressToggled(bool),
    EcLevelSelected(EcLevel),
    GeneratePressed,
    QrWindow(QrWindowMessage),
}

pub struct EncodePanel {
    pub input_mode: InputMode,
    pub file_path: String,
    pub direct_text: String,
    pub compress: bool,
    pub ec_level: EcLevel,
    pub qr_window: Option<QrWindow>,
    pub error_msg: Option<String>,
    ec_combo_state: combo_box::State<EcLevel>,
    /// QrWindow 生成時に渡す DPI スケール
    /// FILE2QR_DPI_SCALE 環境変数で設定（例: 1.25 = 125%）
    dpi_scale: f32,
}

impl Default for EncodePanel {
    fn default() -> Self {
        let levels: Vec<EcLevel> = EcLevel::all().to_vec();
        let dpi_scale = std::env::var("FILE2QR_DPI_SCALE")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
        Self {
            input_mode: InputMode::File,
            file_path: String::new(),
            direct_text: String::new(),
            compress: false,
            ec_level: EcLevel::L,
            qr_window: None,
            error_msg: None,
            ec_combo_state: combo_box::State::new(levels),
            dpi_scale,
        }
    }
}

impl EncodePanel {
    pub fn update(&mut self, msg: EncodeMessage) {
        match msg {
            EncodeMessage::InputModeChanged(mode) => {
                self.input_mode = mode;
            }
            EncodeMessage::FilePathChanged(p) => {
                self.file_path = p;
                self.error_msg = None;
            }
            EncodeMessage::FilePickRequested => {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.file_path = path.to_string_lossy().to_string();
                    self.error_msg = None;
                }
            }
            EncodeMessage::FileDropped(p) => {
                self.file_path = p;
                self.error_msg = None;
            }
            EncodeMessage::DirectTextChanged(t) => {
                self.direct_text = t;
                self.error_msg = None;
            }
            EncodeMessage::CompressToggled(v) => {
                self.compress = v;
            }
            EncodeMessage::EcLevelSelected(level) => {
                self.ec_level = level;
            }
            EncodeMessage::GeneratePressed => {
                self.generate();
            }
            EncodeMessage::QrWindow(qr_msg) => {
                if let Some(ref mut w) = self.qr_window {
                    w.update(qr_msg);
                    if !w.open {
                        self.qr_window = None;
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<EncodeMessage> {
        let mode_row = row![
            text("入力モード:").size(14),
            mode_radio("ファイル", InputMode::File, &self.input_mode),
            mode_radio(
                "テキスト直接入力",
                InputMode::DirectText,
                &self.input_mode
            ),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        let input_area: Element<EncodeMessage> = match self.input_mode {
            InputMode::File => self.view_file_input(),
            InputMode::DirectText => self.view_text_input(),
        };

        let ec_label = format!(
            "{} (最大 {} byte / QR)",
            self.ec_level.label(),
            self.ec_level.max_bytes()
        );
        let options = column![
            checkbox("xz圧縮を有効にする", self.compress)
                .on_toggle(EncodeMessage::CompressToggled),
            row![
                text("エラー訂正レベル:").size(14),
                combo_box(
                    &self.ec_combo_state,
                    &ec_label,
                    Some(&self.ec_level),
                    EncodeMessage::EcLevelSelected,
                )
                .width(300),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(8);

        let gen_btn = button(text("🔲 QRコードを生成").size(14))
            .on_press(EncodeMessage::GeneratePressed)
            .padding([8, 24]);

        let error_view: Element<EncodeMessage> =
            if let Some(ref err) = self.error_msg {
                text(format!("❌ {}", err))
                    .size(13)
                    .color(iced::Color::from_rgb(0.85, 0.1, 0.1))
                    .into()
            } else {
                text("").into()
            };

        let qr_area: Element<EncodeMessage> =
            if let Some(ref w) = self.qr_window {
                w.view().map(EncodeMessage::QrWindow)
            } else {
                text("").into()
            };

        let content = column![
            mode_row,
            horizontal_rule(1),
            input_area,
            horizontal_rule(1),
            options,
            horizontal_rule(1),
            gen_btn,
            error_view,
            qr_area,
        ]
        .spacing(10)
        .width(Length::Fill);

        scrollable(container(content).padding(8).width(Length::Fill)).into()
    }

    fn view_file_input(&self) -> Element<EncodeMessage> {
        let path_row = row![
            text_input(
                "絶対パス、相対パス、またはドラッグ&ドロップ",
                &self.file_path,
            )
            .on_input(EncodeMessage::FilePathChanged)
            .width(500),
            button(text("📂 ファイル選択...").size(13))
                .on_press(EncodeMessage::FilePickRequested)
                .padding([6, 12]),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        column![text("ファイルパス:").size(14), path_row].spacing(6).into()
    }

    fn view_text_input(&self) -> Element<EncodeMessage> {
        column![
            text("テキスト入力:").size(14),
            text_input("ここにテキストを入力してください", &self.direct_text,)
                .on_input(EncodeMessage::DirectTextChanged)
                .width(Length::Fill),
        ]
        .spacing(6)
        .into()
    }

    fn generate(&mut self) {
        self.error_msg = None;

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

        let result = encode::encode(EncodeInput {
            data,
            filename,
            compress: self.compress,
            ec_level: self.ec_level,
        });

        match result {
            Ok(res) => {
                self.qr_window = Some(QrWindow::new(
                    res.fragments,
                    self.ec_level,
                    self.dpi_scale,
                ));
            }
            Err(e) => {
                self.error_msg = Some(e);
            }
        }
    }
}

fn mode_radio<'a>(
    label: &'a str,
    value: InputMode,
    current: &InputMode,
) -> Element<'a, EncodeMessage> {
    iced::widget::radio(
        label,
        value,
        Some(*current),
        EncodeMessage::InputModeChanged,
    )
    .size(16)
    .into()
}

impl std::fmt::Display for EcLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (最大 {} byte / QR)", self.label(), self.max_bytes())
    }
}

#[cfg(test)]
#[path = "encode_panel_tests.rs"]
mod tests;
