use cursive::{views::TextView, CbSink};

pub fn notify_connection_status(cb_sink: &CbSink, connected: bool) {
    match connected {
        true => {
            cb_sink
                .send(Box::new(move |s| {
                    s.call_on_name("notification", |view: &mut TextView| {
                        view.set_content("Connected")
                    });
                }))
                .ok();
        }
        false => {
            cb_sink
                .send(Box::new(move |s| {
                    s.call_on_name("notification", |view: &mut TextView| {
                        view.set_content("Disconnected")
                    });
                }))
                .ok();
        }
    }
}
