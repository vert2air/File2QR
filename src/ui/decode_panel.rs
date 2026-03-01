use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::decode::{self, HashEntry};

/// 出力先ディレクトリの選択肢
#[derive(PartialEq, Clone)]
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

pub struct DecodePanel {
    /// 指定された入力ファイルパスのリスト
    pub input_files: Vec<String>,
    pub file_path_input: String,

    /// 解析済みのHashEntryマップ
    pub entries: HashMap<String, HashEntry>,

    /// デコード対象として選択されたhash値のSet
    pub selected_hashes: std::collections::HashSet<String>,

    /// 出力先ディレクトリ
    pub output_dir: OutputDir,
    pub custom_dir: String,

    /// 直接テキスト出力領域
    pub decoded_text: Option<String>,

    /// メッセージ
    pub status_msg: Option<String>,
    pub error_msg: Option<String>,
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
        }
    }
}

impl DecodePanel {
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // ── 入力ファイル指定 ──
        ui.label("📄 入力ファイル (複数指定可):");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.file_path_input)
                    .hint_text("ファイルパス（Enterで追加）")
                    .desired_width(480.0),
            );
            if ui.button("追加").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.add_file(self.file_path_input.clone());
                self.file_path_input.clear();
            }
            if ui.button("📂 選択...").clicked() {
                if let Some(paths) = rfd::FileDialog::new().pick_files() {
                    for p in paths {
                        self.add_file(p.to_string_lossy().to_string());
                    }
                }
            }
        });

        // D&D対応
        ctx.input(|i| {
            for dropped in &i.raw.dropped_files {
                if let Some(p) = &dropped.path {
                    self.add_file(p.to_string_lossy().to_string());
                }
            }
        });

        // 現在の入力ファイル一覧
        if !self.input_files.is_empty() {
            ui.group(|ui| {
                ui.label("入力ファイル:");
                let mut to_remove = None;
                for (i, f) in self.input_files.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.small_button("✕").clicked() {
                            to_remove = Some(i);
                        }
                        ui.label(f);
                    });
                }
                if let Some(idx) = to_remove {
                    self.input_files.remove(idx);
                    self.reparse_all();
                }
            });
        }

        ui.separator();

        // ── 解析結果一覧 ──
        if self.entries.is_empty() {
            ui.label("（入力ファイルを指定するとここに解析結果が表示されます）");
        } else {
            ui.label("🔍 解析結果:");
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    let hashes: Vec<String> = self.entries.keys().cloned().collect();
                    for hash in &hashes {
                        let entry = &self.entries[hash];
                        let complete = entry.is_complete();
                        let missing = entry.missing_indices();

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let mut selected = self.selected_hashes.contains(hash);
                                let enabled = complete;
                                ui.add_enabled(
                                    enabled,
                                    egui::Checkbox::new(&mut selected, ""),
                                );
                                if enabled {
                                    if selected {
                                        self.selected_hashes.insert(hash.clone());
                                    } else {
                                        self.selected_hashes.remove(hash);
                                    }
                                }

                                ui.label(format!("hash: {}", hash));
                            });

                            // ファイル名
                            let fname = entry
                                .filename
                                .clone()
                                .unwrap_or("(unknown)".to_string());
                            ui.label(format!("  ファイル名: {}", fname));

                            // 圧縮フラグ
                            let compress_str = match entry.compressed {
                                Some(true) => "ON",
                                Some(false) => "OFF",
                                None => "--",
                            };
                            ui.label(format!("  xz圧縮: {}", compress_str));

                            // 不足フラグメント
                            if missing.is_empty() {
                                ui.colored_label(egui::Color32::GREEN, "  ✅ 全フラグメント揃っています");
                            } else {
                                ui.colored_label(
                                    egui::Color32::YELLOW,
                                    format!(
                                        "  ⚠ 不足フラグメント: {:?}",
                                        missing
                                    ),
                                );
                            }
                        });
                    }
                });
        }

        ui.separator();

        // ── 出力先ディレクトリ ──
        ui.label("📁 出力先:");
        ui.horizontal(|ui| {
            for opt in [
                OutputDir::SameAsInput,
                OutputDir::Downloads,
                OutputDir::CurrentDir,
                OutputDir::Custom(self.custom_dir.clone()),
            ] {
                let label = opt.label();
                let selected = matches!(
                    (&self.output_dir, &opt),
                    (OutputDir::SameAsInput, OutputDir::SameAsInput)
                        | (OutputDir::Downloads, OutputDir::Downloads)
                        | (OutputDir::CurrentDir, OutputDir::CurrentDir)
                        | (OutputDir::Custom(_), OutputDir::Custom(_))
                );
                if ui.radio(selected, label).clicked() {
                    self.output_dir = opt;
                }
            }
        });

        if matches!(self.output_dir, OutputDir::Custom(_)) {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.custom_dir)
                        .hint_text("出力ディレクトリのパス")
                        .desired_width(400.0),
                );
                if ui.button("📂 選択...").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        self.custom_dir = dir.to_string_lossy().to_string();
                        self.output_dir = OutputDir::Custom(self.custom_dir.clone());
                    }
                }
            });
        }

        ui.separator();

        // ── デコード実行 ──
        let can_decode = !self.selected_hashes.is_empty();
        if ui
            .add_enabled(
                can_decode,
                egui::Button::new("💾 選択したデータを復元").min_size(egui::vec2(200.0, 36.0)),
            )
            .clicked()
        {
            self.decode_selected();
        }

        // ── ステータス/エラー ──
        if let Some(ref msg) = self.status_msg {
            ui.colored_label(egui::Color32::GREEN, format!("✅ {}", msg));
        }
        if let Some(ref err) = self.error_msg {
            ui.colored_label(egui::Color32::RED, format!("❌ {}", err));
        }

        // ── 直接テキスト表示 ──
        if let Some(ref text) = self.decoded_text.clone() {
            ui.separator();
            ui.label("📝 復元されたテキスト:");
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(self.decoded_text.as_mut().unwrap_or(&mut String::new()))
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
            let _ = text;
        }
    }

    /// ファイルを追加して即座に解析
    fn add_file(&mut self, path: String) {
        if path.is_empty() || self.input_files.contains(&path) {
            return;
        }
        self.input_files.push(path);
        self.reparse_all();
    }

    /// 全入力ファイルを再解析
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
                    self.error_msg =
                        Some(format!("ファイル読み込みエラー ({}): {}", path, e));
                }
            }
        }

        let refs: Vec<&str> = all_lines.iter().map(|s| s.as_str()).collect();
        self.entries = decode::parse_lines(&refs);
    }

    /// 選択されたhashのデータを復元・出力
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
                        .unwrap_or("output".to_string());

                    if filename == "(direct_text)" {
                        // テキスト領域に表示
                        match String::from_utf8(data) {
                            Ok(text) => self.decoded_text = Some(text),
                            Err(e) => {
                                self.error_msg = Some(format!("UTF-8変換エラー: {}", e));
                            }
                        }
                    } else {
                        // ファイルに出力
                        let out_path = self.resolve_output_path(&filename);
                        if let Err(e) = std::fs::write(&out_path, &data) {
                            self.error_msg =
                                Some(format!("ファイル書き込みエラー ({}): {}", out_path.display(), e));
                        } else {
                            success_count += 1;
                        }
                    }
                }
                Err(e) => {
                    self.error_msg = Some(format!("復元エラー ({}): {}", hash, e));
                }
            }
        }

        if self.error_msg.is_none() {
            if self.decoded_text.is_some() {
                self.status_msg = Some("テキストを復元しました".to_string());
            } else {
                self.status_msg = Some(format!("{}件のファイルを復元しました", success_count));
            }
        }
    }

    fn resolve_output_path(&self, filename: &str) -> PathBuf {
        match &self.output_dir {
            OutputDir::SameAsInput => {
                // 最初の入力ファイルのディレクトリを使う
                if let Some(first) = self.input_files.first() {
                    if let Some(parent) = std::path::Path::new(first).parent() {
                        return parent.join(filename);
                    }
                }
                PathBuf::from(filename)
            }
            OutputDir::Downloads => {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join("Downloads").join(filename)
            }
            OutputDir::CurrentDir => PathBuf::from(filename),
            OutputDir::Custom(dir) => PathBuf::from(dir).join(filename),
        }
    }
}
