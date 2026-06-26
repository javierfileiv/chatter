use crate::Context;
use cursive::{views::TextView, CbSink};
use std::sync::Arc;

pub fn set_connection_status(ctx: Arc<Context>, cb_sink: &CbSink, connected: bool) {
    match connected {
        true => {
            let msg = "Connected";
            *ctx.connected.lock().unwrap() = true;
            cb_sink
                .send(Box::new(move |s| {
                    s.call_on_name("status", |view: &mut TextView| view.set_content(msg));
                }))
                .ok();
        }
        false => {
            let msg = "Disconnected";
            *ctx.connected.lock().unwrap() = false;
            cb_sink
                .send(Box::new(move |s| {
                    s.call_on_name("status", |view: &mut TextView| view.set_content(msg));
                }))
                .ok();
        }
    }
}
