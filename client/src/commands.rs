use std::sync::Arc;

use crate::{ui, Context};
use cursive::{
    views::{HideableView, ResizedView, ScrollView, TextView},
    Cursive,
};
use cursive_flexi_logger_view::FlexiLoggerView;

pub fn handle_send(siv: &mut Cursive, ctx: &Arc<Context>, msg: String) {
    if msg.is_empty() {
        return;
    }

    let cb_sink = siv.cb_sink().clone();
    ui::dialogs::clear_notification_view(&cb_sink);
    match msg.as_str() {
        "/help" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.append(
                    "\n=== Commands ===\n/help - Show this help\n/clear - Clear messages\n/connect - Connect to server\n/debug - Toggle debug log view\n/quit - Exit chat\n\n"
                );
            });
        }
        "/clear" => {
            ui::dialogs::clear_input_view(&cb_sink);
            ui::dialogs::clear_messages_view(&cb_sink);
        }
        "/connect" => {
            ui::dialogs::show_connect_dialog(siv, ctx);
        }
        "/debug" => {
            siv.call_on_name(
                "logger_panel",
                |v: &mut HideableView<ResizedView<ScrollView<FlexiLoggerView>>>| {
                    v.set_visible(!v.is_visible());
                },
            );
        }
        "/quit" => {
            siv.quit();
        }
        _ => {}
    }
    ui::dialogs::clear_input_view(&cb_sink);
}
