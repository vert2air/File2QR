use crate::encode::EcLevel;
use crate::encode::fragment::{generate_qr_image, to_iced_handle};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    button, column, container, horizontal_rule, image as img_widget, row,
    scrollable, text,
};
use iced::{ContentFit, Element, Length};

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
}

struct QrData {
    scale: u32,
    handle: iced::widget::image::Handle,
}

pub struct QrWindow {
    pub fragments: Vec<String>,
    pub ec_level: EcLevel,
    pub cols: usize,
    pub rows: usize,
    pub scale: u32,
    pub page: usize,
    pub open: bool,
    pub fullscreen: bool,
    qr_cache: Vec<Option<QrData>>,
}

impl QrWindow {
    pub fn new(fragments: Vec<String>, ec_level: EcLevel) -> Self {
        let n = fragments.len();
        let scale = std::env::var("FILE2QR_SCALE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(2)
            .max(1);

        let mut win = Self {
            fragments,
            ec_level,
            cols: 3,
            rows: 2,
            scale,
            page: 0,
            open: true,
            fullscreen: false,
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

        // 縦横両方向スクロール可能なグリッド領域
        let scroll_area = scrollable(container(grid).padding(8))
            .direction(Direction::Both {
                vertical: Scrollbar::new(),
                horizontal: Scrollbar::new(),
            })
            .width(Length::Fill)
            .height(Length::Fill);

        let body = column![
            header,
            horizontal_rule(1),
            controls,
            horizontal_rule(1),
            scroll_area,
        ]
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Fill);

        container(body)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
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
                        column![
                            img_widget(data.handle.clone())
                                .content_fit(ContentFit::None)
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
        for i in self.page_range() {
            let need_regen = match self.qr_cache[i] {
                Some(ref d) => d.scale != self.scale,
                None => true,
            };
            if need_regen {
                self.generate_qr_at(i);
            }
        }
    }

    fn generate_qr_at(&mut self, i: usize) {
        match generate_qr_image(&self.fragments[i], self.ec_level, self.scale)
        {
            Ok(img) => {
                let handle = to_iced_handle(&img);
                self.qr_cache[i] = Some(QrData { scale: self.scale, handle });
            }
            Err(e) => {
                eprintln!("QRコード生成エラー: index={}, error={}", i, e);
                let handle = iced::widget::image::Handle::from_rgba(
                    1,
                    1,
                    vec![255u8, 255, 255, 255],
                );
                self.qr_cache[i] = Some(QrData { scale: self.scale, handle });
            }
        }
    }
}

#[cfg(test)]
#[path = "qr_window_tests.rs"]
mod tests;
