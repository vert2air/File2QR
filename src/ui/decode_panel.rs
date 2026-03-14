use crate::decode::{self, HashEntry};
use iced::widget::{
    button, column, container, horizontal_rule, row, scrollable, text,
    text_editor, text_input,
};
use iced::{Element, Length};
use std::collections::HashMap;
use std::path::PathBuf;

/// 出力先ディレクトリの選択肢
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputDir {
    SameAsInput,
    Downloads,
    CurrentDir,
    Custom(String),
}

impl OutputDir {
    pub fn label(&self) -> &str {
        match self {
            OutputDir::SameAsInput => "入力ファイルと同じディレクトリ",
            OutputDir::Downloads => "Downloadsディレクトリ",
            OutputDir::CurrentDir => "カレントディレクトリ",
            OutputDir::Custom(_) => "指定ディレクトリ",
        }
    }
}

/// DecodePanel が発行するメッセージ
#[derive(Debug, Clone)]
pub enum DecodeMessage {
    FilePathInputChanged(String),
    AddFilePressed,
    FilePickRequested,
    FileDropped(String),
    RemoveFile(usize),
    HashToggled(String, bool),
    OutputDirChanged(OutputDir),
    CustomDirChanged(String),
    CustomDirPickRequested,
    DecodePressed,
    OpenFolderPressed,
    TextEditorAction(text_editor::Action),
}

pub struct DecodePanel {
    pub input_files: Vec<String>,
    pub file_path_input: String,
    pub entries: HashMap<String, HashEntry>,
    pub selected_hashes: std::collections::HashSet<String>,
    pub output_dir: OutputDir,
    pub custom_dir: String,
    pub decoded_text: Option<String>,
    pub status_msg: Option<String>,
    pub error_msg: Option<String>,
    /// テキスト選択・コピー用エディタコンテンツ（読み取り専用で使用）
    decoded_content: text_editor::Content,
}

impl Default for DecodePanel {
    fn default() -> Self {
        Self {
            input_files: Vec::new(),
            file_path_input: String::new(),
            entries: HashMap::new(),
            selected_hashes: std::collections::HashSet::new(),
            output_dir: OutputDir::SameAsInput,
            custom_dir: String::new(),
            decoded_text: None,
            status_msg: None,
            error_msg: None,
            decoded_content: text_editor::Content::new(),
        }
    }
}

impl DecodePanel {
    pub fn update(&mut self, msg: DecodeMessage) {
        match msg {
            DecodeMessage::FilePathInputChanged(s) => {
                self.file_path_input = s;
            }
            DecodeMessage::AddFilePressed => {
                let p = self.file_path_input.clone();
                self.add_file(p);
                self.file_path_input.clear();
            }
            DecodeMessage::FilePickRequested => {
                if let Some(paths) = rfd::FileDialog::new().pick_files() {
                    for p in paths {
                        self.add_file(p.to_string_lossy().to_string());
                    }
                }
            }
            DecodeMessage::FileDropped(p) => {
                self.add_file(p);
            }
            DecodeMessage::RemoveFile(idx) => {
                if idx < self.input_files.len() {
                    self.input_files.remove(idx);
                    self.reparse_all();
                }
            }
            DecodeMessage::HashToggled(hash, checked) => {
                if checked {
                    self.selected_hashes.insert(hash);
                } else {
                    self.selected_hashes.remove(&hash);
                }
            }
            DecodeMessage::OutputDirChanged(dir) => {
                self.output_dir = dir;
            }
            DecodeMessage::CustomDirChanged(s) => {
                self.custom_dir = s.clone();
                self.output_dir = OutputDir::Custom(s);
            }
            DecodeMessage::CustomDirPickRequested => {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    self.custom_dir = dir.to_string_lossy().to_string();
                    self.output_dir =
                        OutputDir::Custom(self.custom_dir.clone());
                }
            }
            DecodeMessage::DecodePressed => {
                self.decode_selected();
            }
            DecodeMessage::OpenFolderPressed => {
                self.open_output_folder();
            }
            DecodeMessage::TextEditorAction(action) => {
                // 読み取り専用: 選択・カーソル移動・コピーのみ許可
                // 文字入力・削除・貼り付けは無視
                let is_edit = matches!(action, text_editor::Action::Edit(_));
                if !is_edit {
                    self.decoded_content.perform(action);
                }
            }
        }
    }

    pub fn view(&self) -> Element<DecodeMessage> {
        // ── 入力ファイル指定 ──
        let file_input_row = row![
            text_input(
                "ファイルパス（追加ボタンで追加）",
                &self.file_path_input,
            )
            .on_input(DecodeMessage::FilePathInputChanged)
            .width(480),
            button(text("追加").size(13))
                .on_press(DecodeMessage::AddFilePressed)
                .padding([6, 12]),
            button(text("📂 選択...").size(13))
                .on_press(DecodeMessage::FilePickRequested)
                .padding([6, 12]),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        // ── 入力ファイル一覧 ──
        let file_list: Element<DecodeMessage> = if self.input_files.is_empty()
        {
            text("（ファイルが追加されていません）")
                .size(13)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5))
                .into()
        } else {
            let items = self.input_files.iter().enumerate().map(|(i, f)| {
                row![
                    button(text("✕").size(12))
                        .on_press(DecodeMessage::RemoveFile(i))
                        .padding([2, 6]),
                    text(f.as_str()).size(13),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center)
                .into()
            });
            column(items).spacing(4).into()
        };

        // ── 解析結果 ──
        let entries_view: Element<DecodeMessage> = if self.entries.is_empty() {
            text("（入力ファイルを指定するとここに解析結果が表示されます）")
                .size(13)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5))
                .into()
        } else {
            let mut hashes: Vec<String> =
                self.entries.keys().cloned().collect();
            hashes.sort();

            let cards: Vec<Element<DecodeMessage>> =
                hashes.iter().map(|hash| self.view_entry_card(hash)).collect();

            scrollable(column(cards).spacing(8)).height(220).into()
        };

        // ── 出力先ディレクトリ ──
        let output_radios = column![
            output_radio(
                "入力ファイルと同じディレクトリ",
                OutputDir::SameAsInput,
                &self.output_dir,
            ),
            output_radio(
                "Downloadsディレクトリ",
                OutputDir::Downloads,
                &self.output_dir,
            ),
            output_radio(
                "カレントディレクトリ",
                OutputDir::CurrentDir,
                &self.output_dir,
            ),
            output_radio(
                "指定ディレクトリ",
                OutputDir::Custom(self.custom_dir.clone()),
                &self.output_dir,
            ),
        ]
        .spacing(4);

        let custom_dir_row: Element<DecodeMessage> =
            if matches!(self.output_dir, OutputDir::Custom(_)) {
                row![
                    text_input("出力ディレクトリのパス", &self.custom_dir,)
                        .on_input(DecodeMessage::CustomDirChanged)
                        .width(400),
                    button(text("📂 フォルダ選択...").size(13))
                        .on_press(DecodeMessage::CustomDirPickRequested)
                        .padding([6, 12]),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .into()
            } else {
                text("").into()
            };

        // ── デコードボタン ──
        let can_decode = !self.selected_hashes.is_empty();
        let decode_btn = if can_decode {
            button(text("💾 選択したデータを復元").size(14))
                .on_press(DecodeMessage::DecodePressed)
                .padding([8, 24])
        } else {
            button(text("💾 選択したデータを復元").size(14)).padding([8, 24])
        };

        // ── ステータス/エラー ──
        let status_view: Element<DecodeMessage> =
            if let Some(ref msg) = self.status_msg {
                let open_btn: Element<DecodeMessage> =
                    if msg.contains("ファイルを復元") {
                        button(text("📂 出力フォルダを開く").size(13))
                            .on_press(DecodeMessage::OpenFolderPressed)
                            .padding([4, 10])
                            .into()
                    } else {
                        text("").into()
                    };
                row![
                    text(format!("✅ {}", msg))
                        .size(13)
                        .color(iced::Color::from_rgb(0.1, 0.6, 0.2)),
                    open_btn,
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .into()
            } else {
                text("").into()
            };

        let error_view: Element<DecodeMessage> =
            if let Some(ref err) = self.error_msg {
                text(format!("❌ {}", err))
                    .size(13)
                    .color(iced::Color::from_rgb(0.85, 0.1, 0.1))
                    .into()
            } else {
                text("").into()
            };

        // ── テキスト復元結果 ──
        let decoded_view: Element<DecodeMessage> =
            if self.decoded_text.is_some() {
                column![
                    horizontal_rule(1),
                    text("📝 復元されたテキスト:").size(14),
                    text_editor(&self.decoded_content)
                        .on_action(DecodeMessage::TextEditorAction)
                        .height(200),
                ]
                .spacing(6)
                .into()
            } else {
                text("").into()
            };

        let content = column![
            text("📄 入力ファイル (複数指定可):").size(14),
            file_input_row,
            file_list,
            horizontal_rule(1),
            text("🔍 解析結果:").size(14),
            entries_view,
            horizontal_rule(1),
            text("📁 出力先:").size(14),
            output_radios,
            custom_dir_row,
            horizontal_rule(1),
            decode_btn,
            status_view,
            error_view,
            decoded_view,
        ]
        .spacing(10)
        .width(Length::Fill);

        scrollable(container(content).padding(8).width(Length::Fill)).into()
    }

    fn view_entry_card(&self, hash: &str) -> Element<DecodeMessage> {
        let entry = &self.entries[hash];
        let complete = entry.is_complete();
        let missing = entry.missing_indices();

        let selected = self.selected_hashes.contains(hash);

        let check: Element<DecodeMessage> = if complete {
            iced::widget::checkbox("", selected)
                .on_toggle({
                    let h = hash.to_string();
                    move |v| DecodeMessage::HashToggled(h.clone(), v)
                })
                .into()
        } else {
            // 未完成は無効チェックボックス（操作不可）
            iced::widget::checkbox("", false).into()
        };

        let fname =
            entry.filename.clone().unwrap_or_else(|| "(unknown)".to_string());
        let compress_str = match entry.compressed {
            Some(true) => "ON",
            Some(false) => "OFF",
            None => "--",
        };

        let status: Element<DecodeMessage> = if missing.is_empty() {
            text("✅ 全フラグメント揃っています")
                .size(13)
                .color(iced::Color::from_rgb(0.1, 0.6, 0.2))
                .into()
        } else {
            text(format!("⚠ 不足フラグメント: {:?}", missing))
                .size(13)
                .color(iced::Color::from_rgb(0.8, 0.6, 0.0))
                .into()
        };

        let card_content = column![
            row![check, text(format!("hash: {}", hash)).size(13),]
                .spacing(6)
                .align_y(iced::Alignment::Center),
            text(format!("  ファイル名: {}", fname)).size(13),
            text(format!("  xz圧縮: {}", compress_str)).size(13),
            status,
        ]
        .spacing(3);

        container(card_content)
            .padding(8)
            .width(Length::Fill)
            .style(container::bordered_box)
            .into()
    }

    fn add_file(&mut self, path: String) {
        if path.is_empty() || self.input_files.contains(&path) {
            return;
        }
        self.input_files.push(path);
        self.reparse_all();
    }

    fn reparse_all(&mut self) {
        self.entries.clear();
        self.error_msg = None;

        let mut all_lines: Vec<String> = Vec::new();
        for path in &self.input_files {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    all_lines.extend(content.lines().map(|l| l.to_string()));
                }
                Err(e) => {
                    self.error_msg = Some(format!(
                        "ファイル読み込みエラー ({}): {}",
                        path, e
                    ));
                }
            }
        }

        let refs: Vec<&str> = all_lines.iter().map(|s| s.as_str()).collect();
        self.entries = decode::parse_lines(&refs);
    }

    fn decode_selected(&mut self) {
        self.status_msg = None;
        self.error_msg = None;
        self.decoded_text = None;

        let mut success_count = 0;

        for hash in self.selected_hashes.clone() {
            let Some(entry) = self.entries.get(&hash) else {
                continue;
            };

            match decode::reconstruct(entry) {
                Ok(data) => {
                    let filename = entry
                        .filename
                        .clone()
                        .unwrap_or_else(|| "output".to_string());

                    if filename == "(direct_text)" {
                        match String::from_utf8(data) {
                            Ok(t) => {
                                self.decoded_content =
                                    text_editor::Content::with_text(&t);
                                self.decoded_text = Some(t);
                            }
                            Err(e) => {
                                self.error_msg =
                                    Some(format!("UTF-8変換エラー: {}", e));
                            }
                        }
                    } else {
                        let out_path = self.resolve_output_path(&filename);
                        if let Err(e) = std::fs::write(&out_path, &data) {
                            self.error_msg = Some(format!(
                                "ファイル書き込みエラー ({}): {}",
                                out_path.display(),
                                e
                            ));
                        } else {
                            success_count += 1;
                        }
                    }
                }
                Err(e) => {
                    self.error_msg =
                        Some(format!("復元エラー ({}): {}", hash, e));
                }
            }
        }

        if self.error_msg.is_none() {
            if self.decoded_text.is_some() {
                self.status_msg = Some("テキストを復元しました".to_string());
            } else {
                self.status_msg = Some(format!(
                    "{}件のファイルを復元しました",
                    success_count
                ));
            }
        }
    }

    fn resolve_output_path(&self, filename: &str) -> PathBuf {
        match &self.output_dir {
            OutputDir::SameAsInput => {
                if let Some(first) = self.input_files.first() {
                    if let Some(parent) = std::path::Path::new(first).parent()
                    {
                        return parent.join(filename);
                    }
                }
                PathBuf::from(filename)
            }
            OutputDir::Downloads => {
                let home =
                    std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join("Downloads").join(filename)
            }
            OutputDir::CurrentDir => PathBuf::from(filename),
            OutputDir::Custom(dir) => PathBuf::from(dir).join(filename),
        }
    }

    fn open_output_folder(&self) {
        let folder_path = match &self.output_dir {
            OutputDir::SameAsInput => {
                if let Some(first) = self.input_files.first() {
                    std::path::Path::new(first)
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| PathBuf::from("."))
                } else {
                    PathBuf::from(".")
                }
            }
            OutputDir::Downloads => {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join("Downloads")
            }
            OutputDir::CurrentDir => {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            }
            OutputDir::Custom(dir) => PathBuf::from(dir),
        };

        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(folder_path)
                .spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ =
                std::process::Command::new("open").arg(folder_path).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(folder_path)
                .spawn();
        }
    }
}

fn output_radio<'a>(
    label: &'a str,
    value: OutputDir,
    current: &OutputDir,
) -> Element<'a, DecodeMessage> {
    // OutputDir は Custom(String) を含むため Copy を実装できず
    // iced::widget::radio が使えないので、button で代替する
    let is_selected = matches!(
        (current, &value),
        (OutputDir::SameAsInput, OutputDir::SameAsInput)
            | (OutputDir::Downloads, OutputDir::Downloads)
            | (OutputDir::CurrentDir, OutputDir::CurrentDir)
            | (OutputDir::Custom(_), OutputDir::Custom(_))
    );
    let prefix = if is_selected { "● " } else { "○ " };
    let btn = button(text(format!("{}{}", prefix, label)).size(13))
        .on_press(DecodeMessage::OutputDirChanged(value))
        .padding([3, 10]);
    if is_selected {
        btn.style(button::primary).into()
    } else {
        btn.style(button::secondary).into()
    }
}

#[cfg(test)]
#[path = "decode_panel_tests.rs"]
mod tests;
