use clap::Parser;
use flexi_logger::{Duplicate, FileSpec, Logger};
use log::info;
use std::error::Error;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

mod commands;
mod theme;
mod ui;

#[derive(Parser, Debug)]
#[command(
    name = "Chat client",
    author,
    version,
    about = "Terminal chat client — optional values can be set in the connection dialog (/connect).",
    long_about = None
)]
struct Args {
    #[arg(short = 'l', long, default_value = "logs")]
    log_dir: String,
    #[arg(short = 'u', long = "user", help = "User name (optional)")]
    username: Option<String>,
    #[arg(short = 'x', long = "pass", help = "User password (optional)")]
    passwd: Option<String>,
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
    #[arg(
        short = 'r',
        long = "room",
        help = "Room name to connect to (optional)",
        default_value = "waiting_room"
    )]
    room: String,
}

#[derive(Debug)]
struct Context {
    // constant fields
    pub server_ip: String,
    pub server_port: u16,
    // variable field during life app
    pub username: Mutex<String>,
    pub password: Mutex<String>,
    #[expect(dead_code)]
    pub connected: Mutex<bool>,
    #[expect(dead_code)]
    pub room: Mutex<String>,
}

impl Context {
    fn new(
        cli_user: Option<String>,
        password: Option<String>,
        server_ip: String,
        server_port: u16,
        room: String,
    ) -> Self {
        let ctx = Context {
            server_ip,
            server_port,
            username: Mutex::new(cli_user.unwrap_or_default()),
            password: Mutex::new(password.unwrap_or_default()),
            connected: Mutex::new(false),
            room: Mutex::new(room),
        };
        info!(
            "Starting client — user: {}, server: {}:{}",
            ctx.username.lock().unwrap(),
            ctx.server_ip,
            ctx.server_port
        );
        ctx
    }

    pub fn get_user(&self) -> String {
        let guard = self.username.lock().unwrap();
        if guard.is_empty() {
            "Not defined".to_string()
        } else {
            guard.clone()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut siv = cursive::default();
    let args = Args::parse();
    Logger::try_with_str("info")?
        .format_for_files(flexi_logger::detailed_format)
        .format_for_stderr(flexi_logger::detailed_format)
        .log_to_file_and_writer(
            FileSpec::default()
                .directory(args.log_dir)
                .basename("client")
                .suppress_timestamp(),
            cursive_flexi_logger_view::cursive_flexi_logger(&siv),
        )
        .append()
        .duplicate_to_stderr(Duplicate::Warn)
        .start()?;

    let ctx = Arc::new(Context::new(
        args.username,
        args.passwd,
        args.server_ip,
        args.server_port,
        args.room,
    ));

    let ip_addr = ctx.server_ip.parse::<IpAddr>().expect("Invalid IP address");
    let socket_addr = SocketAddr::new(ip_addr, ctx.server_port);

    info!(
        "user {}, pass {}, addr {socket_addr:?}",
        ctx.username.lock().unwrap(),
        ctx.password.lock().unwrap()
    );
    siv.set_theme(theme::create_retro_theme());

    ui::make_ui(&mut siv, &ctx);

    siv.run();

    Ok(())
}
