use crate::network;
use crate::ui;
use crate::Context;
use cursive::{
    traits::*,
    views::{Dialog, EditView, LinearLayout, TextView},
    CbSink, Cursive,
};
use log::info;
use std::sync::Arc;

// Display notification in view.
pub fn set_notification(cb_sink: &CbSink, msg: &str) {
    let msg = msg.to_string();
    cb_sink
        .send(Box::new(move |s| {
            s.call_on_name("notification", |view: &mut TextView| view.set_content(msg));
        }))
        .ok();
}

// Clean notification view.
pub fn clear_notification_view(cb_sink: &CbSink) {
    cb_sink
        .send(Box::new(move |s| {
            s.call_on_name("notification", |view: &mut TextView| view.set_content(""));
        }))
        .ok();
}

// Clean input view.
pub fn clear_messages_view(cb_sink: &CbSink) {
    cb_sink
        .send(Box::new(move |s| {
            s.call_on_name("messages", |view: &mut TextView| view.set_content(""));
        }))
        .ok();
}

// Clean input view.
pub fn clear_input_view(cb_sink: &CbSink) {
    cb_sink
        .send(Box::new(move |s| {
            s.call_on_name("input", |view: &mut EditView| view.set_content(""));
        }))
        .ok();
}

// Display message in view.
pub fn display_message(cb_sink: &CbSink, msg: String) {
    let msg = format!("{msg}\n");
    cb_sink
        .send(Box::new(move |s| {
            s.call_on_name("messages", |view: &mut TextView| view.append(msg));
        }))
        .ok();
}

// Check user input and launch a new connection to server.
fn do_connect(siv: &mut Cursive) {
    let ctx = siv.user_data::<Arc<Context>>().unwrap().clone();
    let cb_sink: CbSink = siv.cb_sink().clone();
    let username = siv.call_on_name("connect_user", |v: &mut EditView| v.get_content());
    let password = siv.call_on_name("connect_pass", |v: &mut EditView| v.get_content());
    let room_to_join = siv.call_on_name("connect_room", |v: &mut EditView| v.get_content());

    let (username, password, room_to_join) = match (username, password, room_to_join) {
        (Some(u), Some(p), Some(r)) if !u.is_empty() && !p.is_empty() && !r.is_empty() => (u, p, r),
        _ => {
            ui::dialogs::set_notification(&cb_sink, "All credentials fields are required");
            siv.pop_layer();
            return;
        }
    };
    let url = format!("ws://{}:{}", ctx.server_ip, ctx.server_port);
    info!("Connecting {} to {}, room: {}", username, url, room_to_join);

    // save user values
    *ctx.username.lock().unwrap() = username.to_string();
    *ctx.password.lock().unwrap() = password.to_string();
    *ctx.room.lock().unwrap() = room_to_join.to_string();
    network::connect_to_server(ctx, cb_sink);
}

// Opens a dialog to entre credentials for connection.
pub fn show_connect_dialog(siv: &mut Cursive, ctx: &Context) {
    let user = ctx.username.lock().unwrap();
    let user_field = EditView::new()
        .content(user.clone())
        .with_name("connect_user");
    let pass_field = EditView::new().secret().with_name("connect_pass");
    let pass_form = LinearLayout::horizontal().child(pass_field);
    let room_field = EditView::new()
        .content(ctx.room.lock().unwrap().clone())
        .with_name("connect_room");
    let form = LinearLayout::vertical()
        .child(TextView::new("Username:"))
        .child(user_field)
        .child(TextView::new("Password:"))
        .child(pass_form)
        .child(TextView::new("Room:"))
        .child(room_field);
    let dialog = Dialog::new()
        .title("Connect to server")
        .content(form)
        .button("Connect", do_connect)
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}
