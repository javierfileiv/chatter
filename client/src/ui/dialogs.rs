use cursive::{
    traits::*,
    views::{Dialog, EditView, LinearLayout, TextView},
    Cursive,
};

use crate::Context;

pub fn show_connect_dialog(siv: &mut Cursive, ctx: &Context) {
    let user = ctx.username.lock().unwrap();
    let user_field = EditView::new()
        .content(user.clone())
        .with_name("connect_user");
    let pass_field = EditView::new().secret().with_name("connect_pass");
    let pass_form = LinearLayout::horizontal().child(pass_field);
    let room_field = EditView::new().with_name("connect_room");
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
        .button("Connect", |s| {
            let _user = s
                .call_on_name("connect_user", |v: &mut EditView| v.get_content())
                .unwrap();
            let _pass = s
                .call_on_name("connect_pass", |v: &mut EditView| v.get_content())
                .unwrap();
            let _room = s
                .call_on_name("connect_room", |v: &mut EditView| v.get_content())
                .unwrap();

            // TODO: WebSocket connect + auth + join room
            s.pop_layer();
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}
