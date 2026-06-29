use std::sync::Arc;

use common::ws_messages::{ClientMessage, Logout, SendMessage};

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
            let logout = ClientMessage::Logout(Logout {
                message: String::new(),
            });
            if let Ok(json) = serde_json::to_string(&logout) {
                let guard = ctx.tx_msg.lock().unwrap();
                if let Some(tx) = guard.as_ref() {
                    tx.send(json).ok();
                }
            }
            siv.quit();
        }
        _ => {
            let guard = ctx.tx_msg.lock().unwrap();
            if let Some(tx) = guard.as_ref() {
                let username = ctx.username.lock().unwrap().clone();
                let msg_struct = ClientMessage::Broadcast(SendMessage {
                    username: username.clone(),
                    message: msg.clone(),
                });
                if let Ok(json) = serde_json::to_string(&msg_struct) {
                    tx.send(json).ok();
                }
            } else {
                ui::dialogs::set_notification(&cb_sink, "Not connected — use /connect first");
            }
        }
    }
    ui::dialogs::clear_input_view(&cb_sink);
}
