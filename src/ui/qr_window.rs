use crate::encode::EcLevel;
use crate::encode::fragment::{generate_qr_image, to_iced_handle};
use iced::widget::{
    button, column, container, horizontal_rule, image as img_widget, row,
    scrollable, text,
};
use iced::{Element, Length};

/// QrWindow が発行するメッセージ
#[derive(Debug, Clone)]
pub enum QrWindowMessage {
    Close,
    PrevPage,
    NextPage,
    RowsInc,
    RowsDec,
    ColsInc,
    ColsDec,
    ScaleInc,
    ScaleDec,
    /// DPIスケール更新（将来の拡張用。現在は FILE2QR_DPI_SCALE 環境変数で設定）
    DpiUpdated(f32),
}

/// 1枚の QRコード画像キャッシュ
struct QrData {
    /// 画像生成時の物理スケール
    physical_scale: u32,
    handle: iced::widget::image::Handle,
    /// 画像の物理ピクセルサイズ
    width: u32,
    height: u32,
}

pub struct QrWindow {
    pub fragments: Vec<String>,
    pub ec_level: EcLevel,
    pub cols: usize,
    pub rows: usize,
    /// ユーザー指定の論理スケール（1x, 2x, ...）
    pub scale: u32,
    pub page: usize,
    pub open: bool,
    /// テスト互換のためフィールドを残す
    pub fullscreen: bool,
    /// 現在のDPIスケール（1.0 = 96dpi = 100%）
    dpi_scale: f32,
    qr_cache: Vec<Option<QrData>>,
}

impl QrWindow {
    pub fn new(
        fragments: Vec<String>,
        ec_level: EcLevel,
        dpi_scale: f32,
    ) -> Self {
        let n = fragments.len();

        let scale = std::env::var("FILE2QR_SCALE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(2)
            .max(1);

        let phys = physical_scale(scale, dpi_scale);
        eprintln!(
            "QRコード: 論理{}x / DPI={:.2} / 物理{}px/module",
            scale, dpi_scale, phys
        );

        let mut win = Self {
            fragments,
            ec_level,
            cols: 3,
            rows: 2,
            scale,
            page: 0,
            open: true,
            fullscreen: false,
            dpi_scale,
            qr_cache: (0..n).map(|_| None).collect(),
        };
        win.load_page_qr();
        win
    }

    #[cfg(test)]
    pub fn new_for_test(fragments: Vec<String>, ec_level: EcLevel) -> Self {
        let n = fragments.len();
        Self {
            fragments,
            ec_level,
            cols: 3,
            rows: 2,
            scale: 2,
            page: 0,
            open: true,
            fullscreen: false,
            dpi_scale: 1.0,
            qr_cache: (0..n).map(|_| None).collect(),
        }
    }

    pub fn update(&mut self, msg: QrWindowMessage) {
        match msg {
            QrWindowMessage::Close => {
                self.open = false;
            }
            QrWindowMessage::PrevPage => {
                if self.page > 0 {
                    self.page -= 1;
                    self.load_page_qr();
                }
            }
            QrWindowMessage::NextPage => {
                if self.page + 1 < self.total_pages() {
                    self.page += 1;
                    self.load_page_qr();
                }
            }
            QrWindowMessage::RowsInc => {
                self.rows += 1;
                self.load_page_qr();
            }
            QrWindowMessage::RowsDec => {
                if self.rows > 1 {
                    self.rows -= 1;
                }
            }
            QrWindowMessage::ColsInc => {
                self.cols += 1;
                self.load_page_qr();
            }
            QrWindowMessage::ColsDec => {
                if self.cols > 1 {
                    self.cols -= 1;
                }
            }
            QrWindowMessage::ScaleInc => {
                self.scale = (self.scale + 1).min(8);
                self.reload_all_cache();
            }
            QrWindowMessage::ScaleDec => {
                if self.scale > 1 {
                    self.scale -= 1;
                    self.reload_all_cache();
                }
            }
            QrWindowMessage::DpiUpdated(dpi) => {
                let old_phys = physical_scale(self.scale, self.dpi_scale);
                let new_phys = physical_scale(self.scale, dpi);
                self.dpi_scale = dpi;
                if old_phys != new_phys {
                    self.reload_all_cache();
                }
            }
        }
    }

    pub fn view(&self) -> Element<QrWindowMessage> {
        let header = row![
            button(text("✕ 閉じる").size(13))
                .on_press(QrWindowMessage::Close)
                .padding([4, 10]),
            text(format!(
                "QRコード表示  ページ {}/{} (全{}枚)",
                self.page + 1,
                self.total_pages(),
                self.fragments.len()
            ))
            .size(14),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        let controls = self.view_controls();
        let grid = self.view_qr_grid();

        let body = column![
            header,
            horizontal_rule(1),
            controls,
            horizontal_rule(1),
            scrollable(grid),
        ]
        .spacing(8)
        .width(Length::Fill);

        container(body)
            .padding(10)
            .width(Length::Fill)
            .style(container::bordered_box)
            .into()
    }

    fn view_controls(&self) -> Element<QrWindowMessage> {
        let can_prev = self.page > 0;
        let can_next = self.page + 1 < self.total_pages();

        let prev_btn = if can_prev {
            button(text("◀ 前").size(13))
                .on_press(QrWindowMessage::PrevPage)
                .padding([4, 10])
        } else {
            button(text("◀ 前").size(13)).padding([4, 10])
        };

        let next_btn = if can_next {
            button(text("次 ▶").size(13))
                .on_press(QrWindowMessage::NextPage)
                .padding([4, 10])
        } else {
            button(text("次 ▶").size(13)).padding([4, 10])
        };

        row![
            prev_btn,
            next_btn,
            text("|").size(14),
            text("縦:").size(13),
            button(text("-").size(13))
                .on_press(QrWindowMessage::RowsDec)
                .padding([2, 8]),
            text(format!("{}", self.rows)).size(13),
            button(text("+").size(13))
                .on_press(QrWindowMessage::RowsInc)
                .padding([2, 8]),
            text("|").size(14),
            text("横:").size(13),
            button(text("-").size(13))
                .on_press(QrWindowMessage::ColsDec)
                .padding([2, 8]),
            text(format!("{}", self.cols)).size(13),
            button(text("+").size(13))
                .on_press(QrWindowMessage::ColsInc)
                .padding([2, 8]),
            text("|").size(14),
            text("拡大率:").size(13),
            button(text("-").size(13))
                .on_press(QrWindowMessage::ScaleDec)
                .padding([2, 8]),
            text(format!("{}x", self.scale)).size(13),
            button(text("+").size(13))
                .on_press(QrWindowMessage::ScaleInc)
                .padding([2, 8]),
            text(format!(
                "(物理{}px/dot, DPI={:.2})",
                physical_scale(self.scale, self.dpi_scale),
                self.dpi_scale
            ))
            .size(11),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
    }

    fn view_qr_grid(&self) -> Element<QrWindowMessage> {
        let range = self.page_range();
        let indices: Vec<usize> = range.collect();

        if indices.is_empty() {
            return text("QRコードがありません").into();
        }

        let mut rows_vec: Vec<Element<QrWindowMessage>> = Vec::new();

        for row_idx in 0..self.rows {
            let mut cols_vec: Vec<Element<QrWindowMessage>> = Vec::new();

            for col_idx in 0..self.cols {
                let flat = row_idx * self.cols + col_idx;
                if flat >= indices.len() {
                    break;
                }
                let qr_idx = indices[flat];

                let cell: Element<QrWindowMessage> =
                    if let Some(ref data) = self.qr_cache[qr_idx] {
                        // 論理サイズ = 物理ピクセル ÷ DPIスケール
                        // iced が再度 dpi_scale を乗算すると physical_pixels に戻る
                        // → 1モジュール = 整数物理ピクセル → シャープな矩形
                        let display_w = data.width as f32 / self.dpi_scale;
                        let display_h = data.height as f32 / self.dpi_scale;

                        column![
                            img_widget(data.handle.clone())
                                .width(display_w)
                                .height(display_h)
                                .filter_method(
                                    iced::widget::image::FilterMethod::Nearest
                                ),
                            text(format!(
                                "{} / {}",
                                qr_idx + 1,
                                self.fragments.len()
                            ))
                            .size(12),
                        ]
                        .spacing(4)
                        .align_x(iced::Alignment::Center)
                        .into()
                    } else {
                        text(format!(
                            "生成中... {}/{}",
                            qr_idx + 1,
                            self.fragments.len()
                        ))
                        .size(13)
                        .into()
                    };

                cols_vec.push(cell);
            }

            rows_vec.push(
                row(cols_vec)
                    .spacing(16)
                    .align_y(iced::Alignment::Start)
                    .into(),
            );
        }

        column(rows_vec).spacing(16).into()
    }

    // ── ヘルパー ──────────────────────────────────────────────

    pub fn per_page(&self) -> usize {
        self.cols * self.rows
    }

    pub fn total_pages(&self) -> usize {
        self.fragments.len().div_ceil(self.per_page())
    }

    pub fn page_range(&self) -> std::ops::Range<usize> {
        let start = self.page * self.per_page();
        let end = (start + self.per_page()).min(self.fragments.len());
        start..end
    }

    fn reload_all_cache(&mut self) {
        for entry in self.qr_cache.iter_mut() {
            *entry = None;
        }
        self.load_page_qr();
    }

    fn load_page_qr(&mut self) {
        let phys = physical_scale(self.scale, self.dpi_scale);
        for i in self.page_range() {
            let need_regen = match self.qr_cache[i] {
                Some(ref d) => d.physical_scale != phys,
                None => true,
            };
            if need_regen {
                self.generate_qr_at(i);
            }
        }
    }

    fn generate_qr_at(&mut self, i: usize) {
        let phys = physical_scale(self.scale, self.dpi_scale);
        match generate_qr_image(&self.fragments[i], self.ec_level, phys) {
            Ok(img) => {
                let (w, h) = img.dimensions();
                let handle = to_iced_handle(&img);
                self.qr_cache[i] = Some(QrData {
                    physical_scale: phys,
                    handle,
                    width: w,
                    height: h,
                });
            }
            Err(e) => {
                eprintln!("QRコード生成エラー: index={}, error={}", i, e);
                let handle = iced::widget::image::Handle::from_rgba(
                    1,
                    1,
                    vec![255u8, 255, 255, 255],
                );
                self.qr_cache[i] = Some(QrData {
                    physical_scale: phys,
                    handle,
                    width: 1,
                    height: 1,
                });
            }
        }
    }
}

/// 論理スケール × DPI → 整数物理スケール
/// 例: scale=2, dpi=1.25 → ceil(2.5) = 3
/// 例: scale=2, dpi=1.5  → ceil(3.0) = 3
/// 例: scale=2, dpi=1.0  → ceil(2.0) = 2
fn physical_scale(scale: u32, dpi: f32) -> u32 {
    ((scale as f32 * dpi).ceil() as u32).max(1)
}

#[cfg(test)]
#[path = "qr_window_tests.rs"]
mod tests;
