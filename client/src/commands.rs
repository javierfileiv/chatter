use std::sync::Arc;

use crate::Context;
use cursive::{
    views::{EditView, HideableView, ResizedView, ScrollView, TextView},
    Cursive,
};
use cursive_flexi_logger_view::FlexiLoggerView;

pub fn handle_send(siv: &mut Cursive, ctx: &Arc<Context>, msg: String) {
    if msg.is_empty() {
        return;
    }

    match msg.as_str() {
        "/help" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.append(
                    "\n=== Commands ===\n/help - Show this help\n/clear - Clear messages\n/connect - Connect to server\n/debug - Toggle debug log view\n/quit - Exit chat\n\n"
                );
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content("");
            });
            return;
        }
        "/clear" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.set_content("");
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content("");
            });
            return;
        }
        "/connect" => {
            crate::ui::dialogs::show_connect_dialog(siv, ctx);
            return;
        }
        "/debug" => {
            siv.call_on_name(
                "logger_panel",
                |v: &mut HideableView<ResizedView<ScrollView<FlexiLoggerView>>>| {
                    v.set_visible(!v.is_visible());
                },
            );
            return;
        }
        "/quit" => {
            siv.quit();
            return;
        }
        _ => {}
    }

    siv.call_on_name("input", |view: &mut EditView| {
        view.set_content("");
    });
}
