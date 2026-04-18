# Working with iced 0.14

This project uses the [iced](https://github.com/iced-rs/iced) GUI framework (v0.14) with the Elm architecture (Model → View → Update). Below are the key patterns and API conventions you need to know.

## Application bootstrap

```rust
iced::application(App::new, App::update, App::view)
    .title(App::title)
    .subscription(App::subscription)
    .theme(App::theme)
    .default_font(iced::Font::with_name("Font Name"))
    .window_size(iced::Size::new(1200.0, 800.0))
    .run()
```

- `App::new` is the boot function: `fn new() -> (Self, Task<Message>)`.
- There is no `run_with` — the boot function is passed directly to `application()`.
- `title` takes `fn title(&self) -> String` (not passed to `application()`).

## Keyboard subscriptions

```rust
use iced::keyboard;

pub fn subscription(&self) -> Subscription<Message> {
    keyboard::listen().map(|event| {
        match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Message::KeyPressed(key, modifiers)
            }
            _ => Message::Noop,
        }
    })
}
```

- `keyboard::on_key_press` no longer exists. Use `keyboard::listen()` which gives you all keyboard events.

## Tasks (formerly Commands)

- `Command` is now `Task`. Use `Task::none()`, `Task::perform(future, map_fn)`, `Task::batch(vec)`.

## Focus management

```rust
use iced::widget::operation;

// Focus a widget by ID:
operation::focus(iced::widget::text_input::Id::new("my-id"))
```

- `text_input::focus(id)` no longer exists. Use `operation::focus(id)` instead.

## Widget API changes

### Space

```rust
use iced::widget::Space;

// iced 0.14:
Space::new()                          // zero-size spacer
Space::new().width(Length::Fill)       // horizontal fill
Space::new().width(8).height(8)       // fixed size

// NOT valid in 0.14:
// Space::with_width(...)             // removed
// Space::with_height(...)            // removed
// Space::new(width, height)          // takes 0 args
```

### Checkbox

```rust
use iced::widget::checkbox;

// iced 0.14:
checkbox(is_checked)
    .label("Label text")
    .on_toggle(|checked| Message::Toggled(checked))
    .text_size(12)
    .size(16)

// NOT valid in 0.14:
// checkbox("Label", is_checked)      // takes 1 arg, not 2
```

### Progress bar

```rust
use iced::widget::progress_bar;

// iced 0.14:
progress_bar(0.0..=100.0, value).girth(8)

// NOT valid in 0.14:
// progress_bar(...).height(8)        // height() is private, use girth()
```

### Horizontal rule

```rust
use iced::widget::rule;

// iced 0.14:
rule::horizontal(1)

// NOT valid in 0.14:
// horizontal_rule(1)                 // removed from top-level
```

### Button style

Button styles now have a `snap` field. Always use `..Default::default()` to handle it:

```rust
button::Style {
    background: Some(iced::Background::Color(bg)),
    text_color: Color::WHITE,
    border: iced::Border::default(),
    shadow: iced::Shadow::default(),
    ..Default::default()
}
```

### Padding

```rust
use iced::Padding;

Padding::from([vertical, horizontal])   // 2-element array
Padding::from([top, right, bottom, left]) // NOT supported — use the 2-element form or Padding::default().top(x).left(y) etc.
```

## Overlay / stack pattern

For overlays (menus, dialogs), use `stack!`:

```rust
use iced::widget::stack;

let base = /* main content */;
let overlay = /* floating content */;

stack![base, overlay]
    .width(Length::Fill)
    .height(Length::Fill)
```

A click-catcher backdrop can be implemented as an invisible full-size button behind the overlay content.

## Crate source location

The iced widget source is at `~/.cargo/registry/src/*/iced_widget-0.14*/src/`. Check individual widget files there when you need to verify API signatures.
