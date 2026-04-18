pub mod menu_bar;

use iced::widget::{Space, button, container, row, text};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::Message;
use crate::menu::menu_bar::MenuId;

struct FnKeyDef {
    key: &'static str,
    label: &'static str,
    message: Option<Message>,
}

pub fn fn_key_bar<'a>() -> Element<'a, Message> {
    let keys = [
        FnKeyDef {
            key: "1",
            label: "Help",
            message: None,
        },
        FnKeyDef {
            key: "2",
            label: "Menu",
            message: None,
        },
        FnKeyDef {
            key: "3",
            label: "View",
            message: Some(Message::ViewFile),
        },
        FnKeyDef {
            key: "4",
            label: "Edit",
            message: Some(Message::EditFile),
        },
        FnKeyDef {
            key: "5",
            label: "Copy",
            message: Some(Message::CopySelected),
        },
        FnKeyDef {
            key: "6",
            label: "Move",
            message: Some(Message::MoveSelected),
        },
        FnKeyDef {
            key: "7",
            label: "Mkdir",
            message: Some(Message::Mkdir),
        },
        FnKeyDef {
            key: "8",
            label: "Delete",
            message: Some(Message::DeleteSelected),
        },
        FnKeyDef {
            key: "9",
            label: "Menu",
            message: Some(Message::MenuOpen(MenuId::Left)),
        },
        FnKeyDef {
            key: "10",
            label: "Quit",
            message: Some(Message::Quit),
        },
    ];

    let mut items: Vec<Element<'a, Message>> = Vec::new();

    for def in &keys {
        let key_label = text(format!("F{}", def.key))
            .size(12)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.6, 0.6, 0.65));

        let action_label = text(def.label)
            .size(12)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.85, 0.85, 0.9));

        let content = row![key_label, action_label].spacing(2);

        let btn = if let Some(ref msg) = def.message {
            button(content)
                .padding(Padding::from([2, 6]))
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.2))),
                    text_color: Color::WHITE,
                    border: iced::Border {
                        color: Color::from_rgb(0.25, 0.25, 0.3),
                        width: 1.0,
                        radius: 3.0.into(),
                    },
                    ..Default::default()
                })
                .on_press(msg.clone())
        } else {
            button(content)
                .padding(Padding::from([2, 6]))
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.16))),
                    text_color: Color::from_rgb(0.4, 0.4, 0.45),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.25),
                        width: 1.0,
                        radius: 3.0.into(),
                    },
                    ..Default::default()
                })
        };

        items.push(btn.into());
        items.push(Space::with_width(Length::Fixed(2.0)).into());
    }

    container(iced::widget::Row::with_children(items).align_y(iced::Alignment::Center))
        .width(Length::Fill)
        .padding(Padding::from([4, 4]))
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.1))),
            ..Default::default()
        })
        .into()
}
