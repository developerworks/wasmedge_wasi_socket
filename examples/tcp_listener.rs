use std::io::{Read, Write};
use std::net::SocketAddr;
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream};

fn handle_client((mut stream, addr): (TcpStream, SocketAddr)) -> std::io::Result<()> {
    let local_addr = stream.local_addr()?;
    println!("{} <-> {}", addr, local_addr);
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    println!("get message: {}", buf);
    println!("sendback reversed message...");
    let buf = &buf.chars().rev().collect::<String>().into_bytes();
    let written_amount = stream.write(buf)?;
    println!("Written amount bytes: {}", written_amount);

    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "1234".to_string());
    println!("listening at 127.0.0.1:{}", port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port), false)?;
    handle_client(listener.accept(false).unwrap())
}
