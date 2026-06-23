// Initial sample code from https://github.com/BekBrace/rust-retro-chat
use chrono::Local;
use clap::Parser;
use cursive::CursiveRunnable;
use cursive::{
    align::HAlign, // Horizontal alignment utilities
    event::Key,    // Handling key press events
    theme::{BaseColor, BorderStyle, Color, Palette, PaletteColor, Theme}, // Styling components
    traits::*,     // Additional traits for UI components
    views::{Dialog, DummyView, EditView, LinearLayout, Panel, ScrollView, TextView}, // UI elements
    Cursive,       // Main Cursive application object
};
use std::net::IpAddr;
use std::net::SocketAddr;

use std::{error::Error, sync::Arc};

// // Importing Chrono for date and time handling
// use chrono::Local;
#[derive(Parser, Debug)]
#[command(name = "Chat client", author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'u', long = "user", help = "User name")]
    username: String,
    #[arg(
        short = 'a',
        long = "ip",
        help = "Server IP address",
        default_value = "127.0.0.1"
    )]
    server_ip: String,
    #[arg(
        short = 'p',
        long = "port",
        help = "Server port number",
        default_value = "8080"
    )]
    server_port: u16,
}

#[derive(Debug)]
struct Context {
    username: String,
    password: String,
    server_ip: String,
    server_port: u16,
}

impl Context {
    fn new(username: String, password: String, server_ip: String, server_port: u16) -> Self {
        Context {
            username,
            password,
            server_ip,
            server_port,
        }
    }
}

fn make_ui(siv: &mut CursiveRunnable, ctx: &Context) {
    // Creating a header to display chat title and username
    let header = TextView::new(format!(
        r#"╔═ RUST CERTIFICATION CHAT ═╗ User: {} ╔═ {} ═╗"#,
        ctx.username,                    // Insert username
        Local::now().format("%H:%M:%S")  // Insert current time
    ))
    .style(Color::Light(BaseColor::Green)) // Green text for retro look
    .h_align(HAlign::Center); // Center-align the header

    // Creating a message area with a scrollable text view
    let messages = TextView::new("") // Initialize empty text view
        .with_name("messages") // Assign a name for later access
        .min_height(20) // Minimum height for the message area
        .scrollable(); // Enable scrolling

    let messages = ScrollView::new(messages)
        .scroll_strategy(cursive::view::ScrollStrategy::StickToBottom) // Keep the scroll at the bottom
        .min_width(60) // Minimum width
        .full_width(); // Occupy full width of the parent

    // Creating an input area for typing messages
    let input = EditView::new()
        .on_submit(move |s, text| send_message(s, text.to_string())) // Define submit behavior
        .with_name("input") // Assign a name for later access
        .min_width(50) // Minimum width
        .max_height(3) // Limit input height to 3 lines
        .full_width(); // Occupy full width of the parent

    // Creating help text for user commands
    let help_text =
        TextView::new("ESC:quit | Enter:send | Commands: /help, /clear, /con, /room, /quit")
            .style(Color::Dark(BaseColor::White)); // Styled with white text

    // Assembling the main layout
    let layout = LinearLayout::vertical()
        .child(Panel::new(header)) // Header panel
        .child(
            Dialog::around(messages) // Dialog box for messages
                .title("Messages") // Add title
                .title_position(HAlign::Center) // Center-align title
                .full_width(),
        )
        .child(
            Dialog::around(input) // Dialog box for input
                .title("Message") // Add title
                .title_position(HAlign::Center) // Center-align title
                .full_width(),
        )
        .child(Panel::new(help_text).full_width()); // Panel for help text

    // Wrapping layout for centering
    let centered_layout = LinearLayout::horizontal()
        .child(DummyView.full_width()) // Dummy views for spacing
        .child(layout)
        .child(DummyView.full_width());

    // Adding the centered layout to the Cursive root
    siv.add_fullscreen_layer(centered_layout);

    // Adding global key bindings
    siv.add_global_callback(Key::Esc, |s| s.quit()); // Quit on ESC
    siv.add_global_callback('/', |s| {
        s.call_on_name("input", |view: &mut EditView| {
            view.set_content("/"); // Insert '/' in input box
        });
    });
}

// Main asynchronous entry point of the application
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ctx = Args::parse();
    let ctx = Context::new(ctx.username, "".to_string(), ctx.server_ip, ctx.server_port);
    let ctx = Arc::new(ctx);

    let ip_addr = ctx.server_ip.parse::<IpAddr>().expect("Invalid IP address");
    let socket_addr = SocketAddr::new(ip_addr, ctx.server_port);

    println!(
        "user {}, pass {}, addr {socket_addr:?}",
        ctx.username, ctx.password
    );

    // Initializing the Cursive UI framework
    let mut siv = cursive::default();
    siv.set_theme(create_retro_theme()); // Applying a custom retro theme

    make_ui(&mut siv, &ctx);

    siv.run(); // Run the Cursive event loop
               // let _ = writer_clone.lock().await.shutdown().await; // Close the writer
    Ok(()) // Exit successfully
}

// Function to handle sending messages
fn send_message(siv: &mut Cursive, msg: String) {
    if msg.is_empty() {
        // Ignore empty messages
        return;
    }

    // Handle specific commands
    match msg.as_str() {
        "/help" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.append(
                    "\n=== Commands ===\n/help - Show this help\n/clear - Clear messages\n/con - Connect to server\n/room - Change room\n/quit - Exit chat\n\n"
                );
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content("");
            });
            return;
        }
        "/clear" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.set_content(""); // Clear messages
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content(""); // Clear input
            });
            return;
        }
        "/quit" => {
            siv.quit(); // Quit the application
            return;
        }
        _ => {}
    }

    // Clear the input field
    siv.call_on_name("input", |view: &mut EditView| {
        view.set_content("");
    });
}

// Function to create a retro-style theme
fn create_retro_theme() -> Theme {
    let mut theme = cursive::theme::Theme {
        shadow: true,
        borders: BorderStyle::Simple,
        ..Default::default()
    };

    let mut palette = Palette::default();
    palette[PaletteColor::Background] = Color::Rgb(0, 0, 20); // Deep blue background
    palette[PaletteColor::View] = Color::Rgb(0, 0, 20); // Deep blue for views
    palette[PaletteColor::Primary] = Color::Rgb(0, 255, 0); // Bright green text
    palette[PaletteColor::TitlePrimary] = Color::Rgb(0, 255, 128); // Green for titles
    palette[PaletteColor::Secondary] = Color::Rgb(255, 191, 0); // Amber secondary elements
    palette[PaletteColor::Highlight] = Color::Rgb(0, 255, 255); // Cyan highlights
    palette[PaletteColor::HighlightInactive] = Color::Rgb(0, 128, 128); // Dark cyan for inactive
    palette[PaletteColor::Shadow] = Color::Rgb(0, 0, 40); // Subtle shadow
    theme.palette = palette; // Apply the palette
    theme
}
