use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "Chat client", author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'u', long = "user", help = "User name", default_value = "user")]
    username: String,
    #[arg(
        short = 'p',
        long = "pass",
        help = "User password",
        default_value = "1234"
    )]
    password: String,
    #[arg(
        short = 'i',
        long,
        help = "Server IP address",
        default_value = "127.0.0.1"
    )]
    server_ip: String,
    #[arg(short = 'x', long, help = "Server port number", default_value = "7878")]
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

fn main() -> std::io::Result<()> {
    let ctx = Args::parse();
    let ctx = Context::new(ctx.username, ctx.password, ctx.server_ip, ctx.server_port);

    let ip_addr = ctx.server_ip.parse::<IpAddr>().expect("Invalid IP address");
    let socket_addr = SocketAddr::new(ip_addr, ctx.server_port);

    println!(
        "user {}, pass {}, addr {socket_addr:?}",
        ctx.username, ctx.password
    );
    let mut stream = std::net::TcpStream::connect(socket_addr)?;
    let mut buffer = [0; 512];

    stream.write_all(b"Hello from client!")?;

    let bytes_read = stream.read(&mut buffer)?;
    println!(
        "Server says: {}",
        String::from_utf8_lossy(&buffer[..bytes_read])
    );
    Ok(())
}
