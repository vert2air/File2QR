use crate::encode::EcLevel;
use crate::encode::fragment::{generate_qr_image, to_egui_image};
use eframe::egui;
use egui::TextureHandle;
use std::sync::{Arc, Mutex};
use std::thread;

/// QRコード生成の中間データ（画像とテクスチャ）
struct QrData {
    scale: u32, // この画像のscale
    texture: TextureHandle,
}

pub struct QrWindow {
    /// 全フラグメント文字列
    pub fragments: Vec<String>,
    pub ec_level: EcLevel,

    /// 表示設定
    pub cols: usize, // 横の個数
    pub rows: usize, // 縦の個数
    pub scale: u32,  // 拡大率（環境変数で固定）

    /// 現在のページ (0-indexed)
    pub page: usize,

    /// QRコードデータ（scale別にキャッシュ）
    qr_data: Vec<Option<QrData>>,

    /// バックグラウンド生成の進捗状況
    generation_progress: Arc<Mutex<usize>>,

    /// 全画面表示モード
    pub fullscreen: bool,

    /// 現在ページの生成が完了したか
    current_page_ready: bool,

    pub open: bool,
}

impl QrWindow {
    pub fn new(
        ctx: &egui::Context,
        fragments: Vec<String>,
        ec_level: EcLevel,
    ) -> Self {
        let n = fragments.len();
        let progress = Arc::new(Mutex::new(0));

        // DPIスケーリングを取得
        let ppp = ctx.pixels_per_point();

        // 環境変数FILE2QR_SCALEからscaleを取得、デフォルトは2
        let scale = std::env::var("FILE2QR_SCALE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(2)
            .max(1);

        eprintln!(
            "QRコード拡大率: {}x (環境変数FILE2QR_SCALEで変更可、DPI={:.2})",
            scale, ppp
        );

        let mut win = Self {
            fragments,
            ec_level,
            cols: 3,
            rows: 2,
            scale,
            page: 0,
            qr_data: (0..n).map(|_| None).collect(),
            generation_progress: progress.clone(),
            fullscreen: false,
            current_page_ready: false,
            open: true,
        };

        // 最初のページだけ即座に生成
        win.start_background_generation(ctx);
        win.load_page_qr(ctx);
        win
    }

    /// テスト用: Contextなしで初期化
    /// 
    /// 注: 環境変数FILE2QR_SCALEは読まず、常にscale=2で初期化されます。
    /// QRコード生成やバックグラウンド処理も行いません。
    #[cfg(test)]
    pub fn new_for_test(fragments: Vec<String>, ec_level: EcLevel) -> Self {
        let n = fragments.len();
        let progress = Arc::new(Mutex::new(0));
        let scale = 2;

        Self {
            fragments,
            ec_level,
            cols: 3,
            rows: 2,
            scale,
            page: 0,
            qr_data: (0..n).map(|_| None).collect(),
            generation_progress: progress,
            fullscreen: false,
            current_page_ready: false,
            open: true,
        }
    }

    pub fn per_page(&self) -> usize {
        self.cols * self.rows
    }

    pub fn total_pages(&self) -> usize {
        self.fragments.len().div_ceil(self.per_page())
    }

    /// 現在ページのフラグメントインデックス範囲
    pub fn page_range(&self) -> std::ops::Range<usize> {
        let start = self.page * self.per_page();
        let end = (start + self.per_page()).min(self.fragments.len());
        start..end
    }

    /// 現在ページのQRコードを生成
    fn load_page_qr(&mut self, ctx: &egui::Context) {
        self.current_page_ready = false;

        for i in self.page_range() {
            self.ensure_qr_at(i, ctx);
        }

        // 全て生成完了したかチェック
        self.check_page_ready();
    }

    /// 現在ページの全QRコードが生成完了したかチェック
    fn check_page_ready(&mut self) {
        let all_ready = self.page_range().all(|i| self.qr_data[i].is_some());
        if all_ready {
            self.current_page_ready = true;
        }
    }

    /// 指定インデックスのQRコードを現在のscaleで生成（キャッシュ確認）
    fn ensure_qr_at(&mut self, i: usize, ctx: &egui::Context) {
        let need_regen = if let Some(ref data) = self.qr_data[i] {
            data.scale != self.scale
        } else {
            true
        };

        if need_regen {
            self.generate_qr_at(i, ctx);
        }
    }

    /// 指定インデックスのQRコードを生成
    fn generate_qr_at(&mut self, i: usize, ctx: &egui::Context) {
        match generate_qr_image(&self.fragments[i], self.ec_level, self.scale)
        {
            Ok(img) => {
                let color_img = to_egui_image(&img);
                let tex = ctx.load_texture(
                    format!("qr_{}_s{}", i, self.scale),
                    color_img,
                    egui::TextureOptions::NEAREST,
                );
                self.qr_data[i] =
                    Some(QrData { scale: self.scale, texture: tex });
            }
            Err(e) => {
                eprintln!("QRコード生成エラー: index={}, error={}", i, e);

                // エラー時はダミーの1x1白画像を作成して進める
                use image::{GrayImage, Luma};
                let dummy_img = GrayImage::from_pixel(1, 1, Luma([255u8]));
                let color_img = to_egui_image(&dummy_img);
                let tex = ctx.load_texture(
                    format!("qr_{}_error", i),
                    color_img,
                    egui::TextureOptions::NEAREST,
                );
                self.qr_data[i] =
                    Some(QrData { scale: self.scale, texture: tex });
            }
        }
    }

    /// バックグラウンドで全QRコードを生成準備
    fn start_background_generation(&self, ctx: &egui::Context) {
        let fragments = self.fragments.clone();
        let ec_level = self.ec_level;
        let progress = self.generation_progress.clone();
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            for i in 0..fragments.len() {
                // 検証のみ（実際の生成はメインスレッドで）
                if generate_qr_image(&fragments[i], ec_level, 1).is_ok() {
                    if let Ok(mut p) = progress.lock() {
                        *p = i + 1;
                    }
                    // 最後のフレームだけrepaint要求
                    if i == fragments.len() - 1 {
                        ctx_clone.request_repaint();
                    }
                }
            }
        });
    }

    /// scale変更時に全QRコードを再生成
    pub fn reload_all_for_new_scale(&mut self, ctx: &egui::Context) {
        // キャッシュをクリア
        for data in self.qr_data.iter_mut() {
            *data = None;
        }
        // 現在ページを再生成
        self.load_page_qr(ctx);
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        // キーボード入力
        ctx.input(|i| {
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.prev_page(ctx);
            }
            if i.key_pressed(egui::Key::ArrowRight)
                || i.key_pressed(egui::Key::Space)
            {
                self.next_page(ctx);
            }
            if i.key_pressed(egui::Key::Escape) {
                // ESCで全画面解除、もう一度押すと閉じる
                if self.fullscreen {
                    self.fullscreen = false;
                } else {
                    self.open = false;
                }
            }
            // F11キーで全画面トグル
            if i.key_pressed(egui::Key::F11) {
                self.fullscreen = !self.fullscreen;
            }
        });

        // バックグラウンド生成の進捗確認
        self.check_background_progress(ctx);

        if self.fullscreen {
            // 全画面モード: CentralPanelを使って画面全体に表示
            egui::CentralPanel::default().show(ctx, |ui| {
                // 閉じるボタンと元に戻すボタン
                ui.horizontal(|ui| {
                    if ui.button("🗙 閉じる").clicked() {
                        self.open = false;
                    }
                    if ui.button("🗗 元に戻す").clicked() {
                        self.fullscreen = false;
                    }
                    ui.separator();
                    ui.label("ヒント: ESCキーで元に戻す、F11で全画面切替");
                });
                ui.separator();

                egui::ScrollArea::both().auto_shrink([false, false]).show(
                    ui,
                    |ui| {
                        self.show_controls(ui, ctx);
                        ui.separator();
                        self.show_qr_grid(ui);
                    },
                );
            });
        } else {
            // 通常モード: Window表示
            let mut open = self.open;

            egui::Window::new("QRコード表示")
                .id(egui::Id::new("qr_display_window"))
                .open(&mut open)
                .resizable(true)
                .default_size([800.0, 600.0])
                .max_size([f32::INFINITY, f32::INFINITY])
                .scroll([true, true])
                .show(ctx, |ui| {
                    // 全画面表示ボタン
                    ui.horizontal(|ui| {
                        if ui.button("🗖 全画面表示").clicked() {
                            self.fullscreen = true;
                        }
                        ui.label("(F11キーでも切替可)");
                    });
                    ui.separator();

                    self.show_controls(ui, ctx);
                    ui.separator();
                    self.show_qr_grid(ui);
                });

            self.open = open;
        }
    }

    fn check_background_progress(&mut self, ctx: &egui::Context) {
        let progress =
            if let Ok(p) = self.generation_progress.lock() { *p } else { 0 };

        // 現在ページの範囲のみ生成（バックグラウンドで全検証完了していても）
        for i in self.page_range() {
            if i < progress && self.qr_data[i].is_none() {
                self.ensure_qr_at(i, ctx);
            }
        }

        // 生成完了チェック
        self.check_page_ready();
    }

    fn show_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let ppp = ctx.pixels_per_point();

        ui.horizontal(|ui| {
            // ページナビ
            let can_prev = self.page > 0;
            let can_next = self.page + 1 < self.total_pages();

            if ui.add_enabled(can_prev, egui::Button::new("◀ 前")).clicked()
            {
                self.prev_page(ctx);
            }

            ui.label(format!(
                "ページ {}/{} (全{}枚)",
                self.page + 1,
                self.total_pages(),
                self.fragments.len()
            ));

            if ui.add_enabled(can_next, egui::Button::new("次 ▶")).clicked()
            {
                self.next_page(ctx);
            }

            ui.separator();

            // 表示グリッド設定（即座に変更、再生成なし）
            ui.label("縦:");
            if ui.small_button("-").clicked() && self.rows > 1 {
                self.rows -= 1;
            }
            ui.label(format!("{}", self.rows));
            if ui.small_button("+").clicked() {
                self.rows += 1;
            }

            ui.label("横:");
            if ui.small_button("-").clicked() && self.cols > 1 {
                self.cols -= 1;
            }
            ui.label(format!("{}", self.cols));
            if ui.small_button("+").clicked() {
                self.cols += 1;
            }

            ui.separator();

            // 拡大率表示（環境変数で固定）
            ui.label(format!("拡大率: {}x", self.scale));

            ui.separator();
            ui.label(format!("DPI: {:.1}x", ppp));
        });
    }

    fn show_qr_grid(&mut self, ui: &mut egui::Ui) {
        if !self.current_page_ready {
            // 現在ページ生成中
            ui.centered_and_justified(|ui| {
                ui.label("QRコード生成中...");
            });
            return;
        }

        let range = self.page_range();
        let indices: Vec<usize> = range.collect();

        for row_idx in 0..self.rows {
            ui.horizontal_top(|ui| {
                for col_idx in 0..self.cols {
                    let flat = row_idx * self.cols + col_idx;
                    if flat >= indices.len() {
                        break;
                    }
                    let qr_idx = indices[flat];

                    ui.vertical(|ui| {
                        // QRコード画像
                        if let Some(ref data) = self.qr_data[qr_idx] {
                            let texture_size = data.texture.size_vec2();
                            let pixels_per_point = ui.ctx().pixels_per_point();

                            // 表示サイズを計算し、整数ピクセルに丸める
                            let display_size_raw =
                                texture_size / pixels_per_point;
                            let display_size = egui::vec2(
                                display_size_raw.x.round(),
                                display_size_raw.y.round(),
                            );

                            ui.image(egui::load::SizedTexture::new(
                                data.texture.id(),
                                display_size,
                            ));
                        } else {
                            ui.label("エラー: 生成失敗");
                        }

                        // ラベル: "N / 総数"
                        ui.label(format!(
                            "{} / {}",
                            qr_idx + 1,
                            self.fragments.len()
                        ));
                    });
                }
            });
        }
    }

    fn prev_page(&mut self, ctx: &egui::Context) {
        if self.page > 0 {
            self.page -= 1;
            self.current_page_ready = false;
            self.load_page_qr(ctx);
        }
    }

    fn next_page(&mut self, ctx: &egui::Context) {
        if self.page + 1 < self.total_pages() {
            self.page += 1;
            self.current_page_ready = false;
            self.load_page_qr(ctx);
        }
    }
}

#[cfg(test)]
#[path = "qr_window_tests.rs"]
mod tests;
