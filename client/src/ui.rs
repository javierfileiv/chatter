use crate::Context;
use cursive::{views, Cursive};
use std::sync::Arc;

pub mod dialogs;
pub mod layout;
pub mod status;

pub fn make_ui(siv: &mut Cursive, ctx: &Arc<Context>) {
    let header = layout::build_header(ctx);
    let messages = layout::build_messages();
    let input = layout::build_input(ctx);
    let help_text = layout::build_help();
    let footer = layout::build_footer();
    let logger_panel = layout::build_logger_view();

    let layout = layout::assemble_layout(header, messages, input, help_text, footer, logger_panel);
    siv.add_fullscreen_layer(layout);
    siv.add_global_callback(cursive::event::Key::Esc, |s| s.quit());

    // Update clock on global header each 1 second.
    siv.set_fps(1);
    siv.add_global_callback(cursive::event::Event::Refresh, {
        let username = ctx.get_user().to_string();
        move |s| {
            s.call_on_name("global_header", |view: &mut views::TextView| {
                view.set_content(format!(
                    r#"╔═ RUST CERTIFICATION CHAT ═╗ User: {} ╔═ {} ═╗"#,
                    username,
                    chrono::Local::now().format("%H:%M:%S")
                ));
            });
        }
    });
}
