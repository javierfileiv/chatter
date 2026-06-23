use cursive::{
    align::HAlign,
    theme::{BaseColor, Color},
    traits::*,
    views::{Dialog, DummyView, EditView, LinearLayout, Panel, ScrollView, TextView},
    View,
};

use crate::Context;

pub fn build_header(ctx: &Context) -> Box<dyn View> {
    Box::new(
        Panel::new(
            TextView::new(format!(
                r#"╔═ RUST CERTIFICATION CHAT ═╗ User: {} ╔═ {} ═╗"#,
                ctx.get_user(),
                chrono::Local::now().format("%H:%M:%S")
            ))
            .style(Color::Light(BaseColor::Green))
            .h_align(HAlign::Center)
            .with_name("global_header"),
        )
        .full_width(),
    )
}

pub fn build_messages() -> Box<dyn View> {
    let messages = TextView::new("")
        .with_name("messages")
        .min_height(20)
        .scrollable();

    Box::new(
        Dialog::around(
            ScrollView::new(messages)
                .scroll_strategy(cursive::view::ScrollStrategy::StickToBottom)
                .min_width(60)
                .full_width(),
        )
        .title("Messages")
        .title_position(HAlign::Center)
        .full_width(),
    )
}

pub fn build_input() -> Box<dyn View> {
    Box::new(
        Dialog::around(
            EditView::new()
                .on_submit(|s, text| crate::commands::handle_send(s, text.to_string()))
                .with_name("input")
                .min_width(50)
                .max_height(3)
                .full_width(),
        )
        .title("Message")
        .title_position(HAlign::Center)
        .full_width(),
    )
}

pub fn build_help() -> Box<dyn View> {
    Box::new(
        Panel::new(
            TextView::new("ESC:quit | Enter:send | Commands: /help, /clear, /connect, /quit")
                .style(Color::Dark(BaseColor::White)),
        )
        .full_width(),
    )
}

pub fn assemble_layout(
    header: Box<dyn View>,
    messages: Box<dyn View>,
    input: Box<dyn View>,
    help_text: Box<dyn View>,
) -> Box<dyn View> {
    let layout = LinearLayout::vertical()
        .child(header)
        .child(messages)
        .child(input)
        .child(help_text);

    Box::new(
        LinearLayout::horizontal()
            .child(DummyView.full_width())
            .child(layout)
            .child(DummyView.full_width()),
    )
}
