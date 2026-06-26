use cursive::{
    align::HAlign,
    theme::{BaseColor, Color},
    traits::*,
    views::{Dialog, DummyView, EditView, HideableView, LinearLayout, Panel, ScrollView, TextView},
    View,
};
use cursive_flexi_logger_view::FlexiLoggerView;
use std::sync::Arc;

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

pub fn build_input(ctx: &Arc<Context>) -> Box<dyn View> {
    let ctx = ctx.clone();
    Box::new(
        Dialog::around(
            EditView::new()
                .on_submit(move |s, text| crate::commands::handle_send(s, &ctx, text.to_string()))
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

pub fn build_footer() -> Box<dyn View> {
    let status = TextView::new("status: Disconnected")
        .h_align(HAlign::Left)
        .style(Color::Dark(BaseColor::Yellow))
        .with_name("status")
        .min_width(10)
        .max_height(3)
        .full_width();
    let notif = TextView::new("")
        .h_align(HAlign::Right)
        .style(Color::Dark(BaseColor::Blue))
        .with_name("notification")
        .min_width(10)
        .max_height(3)
        .full_width();
    Box::new(Dialog::around(LinearLayout::horizontal().child(status).child(notif)).full_width())
}

pub fn build_logger_view() -> Box<dyn View> {
    let logger = FlexiLoggerView::scrollable().fixed_height(10);
    let mut logger = HideableView::new(logger);
    logger.set_visible(false);
    Box::new(logger.with_name("logger_panel"))
}

pub fn build_help() -> Box<dyn View> {
    Box::new(
        Panel::new(
            TextView::new(
                "ESC:quit | Enter:send | Commands: /help, /clear, /connect, /debug, /quit",
            )
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
    footer: Box<dyn View>,
    logger_panel: Box<dyn View>,
) -> Box<dyn View> {
    let content = LinearLayout::vertical()
        .child(header)
        .child(messages)
        .child(input)
        .child(help_text)
        .child(footer)
        .child(logger_panel);

    Box::new(
        LinearLayout::horizontal()
            .child(DummyView.full_width())
            .child(content)
            .child(DummyView.full_width()),
    )
}
