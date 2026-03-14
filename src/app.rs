use crate::ui::{decode_panel::DecodePanel, encode_panel::EncodePanel};
use iced::widget::{button, column, container, horizontal_rule, row, text};
use iced::{Element, Length, Subscription};

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Encode,
    Decode,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    Encode(crate::ui::encode_panel::EncodeMessage),
    Decode(crate::ui::decode_panel::DecodeMessage),
    FileDropped(Vec<std::path::PathBuf>),
}

pub struct App {
    pub current_tab: Tab,
    pub encode_panel: EncodePanel,
    pub decode_panel: DecodePanel,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_tab: Tab::Encode,
            encode_panel: EncodePanel::default(),
            decode_panel: DecodePanel::default(),
        }
    }
}

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::TabSelected(tab) => {
                self.current_tab = tab;
            }
            Message::Encode(msg) => {
                self.encode_panel.update(msg);
            }
            Message::Decode(msg) => {
                self.decode_panel.update(msg);
            }
            Message::FileDropped(paths) => {
                for path in paths {
                    let s = path.to_string_lossy().to_string();
                    match self.current_tab {
                        Tab::Encode => {
                            self.encode_panel.update(
                                crate::ui::encode_panel::EncodeMessage::FileDropped(s),
                            );
                        }
                        Tab::Decode => {
                            self.decode_panel.update(
                                crate::ui::decode_panel::DecodeMessage::FileDropped(s),
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, _status, _id| match event {
            iced::Event::Window(iced::window::Event::FileDropped(path)) => {
                Some(Message::FileDropped(vec![path]))
            }
            _ => None,
        })
    }

    pub fn view(&self) -> Element<Message> {
        // QRコード表示中は画面全体をQRビューアに明け渡す
        if let Some(ref w) = self.encode_panel.qr_window {
            return container(
                w.view()
                    .map(crate::ui::encode_panel::EncodeMessage::QrWindow)
                    .map(Message::Encode),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        // 通常表示: タブバー + コンテンツ
        let tab_bar = row![
            tab_button("QRコード生成", Tab::Encode, &self.current_tab),
            tab_button("データ復元", Tab::Decode, &self.current_tab),
        ]
        .spacing(4)
        .padding(8);

        let content: Element<Message> = match self.current_tab {
            Tab::Encode => self.encode_panel.view().map(Message::Encode),
            Tab::Decode => self.decode_panel.view().map(Message::Decode),
        };

        let body = column![
            tab_bar,
            horizontal_rule(1),
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(12),
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        container(body).width(Length::Fill).height(Length::Fill).into()
    }
}

fn tab_button<'a>(
    label: &'a str,
    tab: Tab,
    current: &Tab,
) -> Element<'a, Message> {
    let is_active = &tab == current;
    let btn = button(text(label).size(14));
    if is_active {
        btn.style(button::primary)
    } else {
        btn.style(button::secondary)
    }
    .on_press(Message::TabSelected(tab))
    .into()
}
