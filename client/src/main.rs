use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;

use std::error::Error;

mod commands;
mod theme;
mod ui;

#[derive(Parser, Debug)]
#[command(name = "Chat client", author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'u', long = "user", help = "User name (optional)")]
    username: Option<String>,
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
    fn new(cli_user: Option<String>, server_ip: String, server_port: u16) -> Self {
        Context {
            username: cli_user.unwrap_or_default(),
            password: String::new(),
            server_ip,
            server_port,
        }
    }

    pub fn get_user(&self) -> &str {
        if self.username.is_empty() {
            "Not defined"
        } else {
            &self.username
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let ctx = Arc::new(Context::new(
        args.username,
        args.server_ip,
        args.server_port,
    ));

    let ip_addr = ctx.server_ip.parse::<IpAddr>().expect("Invalid IP address");
    let socket_addr = SocketAddr::new(ip_addr, ctx.server_port);

    println!(
        "user {}, pass {}, addr {socket_addr:?}",
        ctx.username, ctx.password
    );

    let mut siv = cursive::default();
    siv.set_theme(theme::create_retro_theme());

    ui::make_ui(&mut siv, &ctx);

    siv.run();

    Ok(())
}
