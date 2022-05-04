use std::io::Write;
use wasmedge_wasi_socket::{Shutdown, TcpStream};

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| 1234.to_string());
    println!("connect to 127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
    println!("sending hello message...");
    let written_amount = stream.write(b"hello")?;
    println!("Written amount bytes: {}", written_amount);
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}
