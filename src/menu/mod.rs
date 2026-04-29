pub mod menu_bar;

use iced::widget::{Space, container, row};
use iced::{Element, Length, Padding};
use iced_longbridge::components::button::{Variant, button_ex};
use iced_longbridge::theme::{AppTheme, Size};

use crate::app::Message;
use crate::menu::menu_bar::MenuId;

struct FnKeyDef {
    key: &'static str,
    label: &'static str,
    message: Option<Message>,
}

pub fn fn_key_bar<'a>(theme: &AppTheme) -> Element<'a, Message> {
    let t = *theme;
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
        let label = format!("F{} {}", def.key, def.label);
        let disabled = def.message.is_none();
        let btn = button_ex(
            theme,
            label,
            Variant::Ghost,
            Size::Xs,
            def.message.clone(),
            false,
            disabled,
        );
        items.push(btn);
        items.push(Space::new().width(Length::Fixed(2.0)).into());
    }

    container(row(items).align_y(iced::Alignment::Center))
        .width(Length::Fill)
        .padding(Padding::from([4, 4]))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(t.background)),
            ..Default::default()
        })
        .into()
}
