use cursive::{
    views::{EditView, TextView},
    Cursive,
};

pub fn handle_send(siv: &mut Cursive, msg: String) {
    if msg.is_empty() {
        return;
    }

    match msg.as_str() {
        "/help" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.append(
                    "\n=== Commands ===\n/help - Show this help\n/clear - Clear messages\n/connect - Connect to server\n/quit - Exit chat\n\n",
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
            // TODO: show connect dialog
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
