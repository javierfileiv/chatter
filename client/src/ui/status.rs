use cursive::{views::TextView, CbSink};

pub fn notify_connection_status(cb_sink: &CbSink, connected: bool) {
    match connected {
        true => {
            notify_message(cb_sink, "Connected");
        }
        false => {
            notify_message(cb_sink, "Not connected");
        }
    }
}

pub fn notify_message(cb_sink: &CbSink, msg: &str) {
    let msg = msg.to_string();
    cb_sink
        .send(Box::new(move |s| {
            s.call_on_name("notification", |view: &mut TextView| view.set_content(msg));
        }))
        .ok();
}
