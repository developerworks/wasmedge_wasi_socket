use bytecodec::{bytes::Utf8Decoder, DecodeExt, Result};
use httpcodec::{
    BodyDecoder, HttpVersion, ReasonPhrase, Request, RequestDecoder, Response, StatusCode,
};
use std::io::{Read, Write};
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream};

fn handle_http(req: Request<String>) -> Result<Response<String>> {
    Ok(Response::new(
        HttpVersion::V1_0,
        StatusCode::new(200)?,
        ReasonPhrase::new("")?,
        format!("echo: {}", req.body()),
    ))
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    println!("received request from: {:?}", stream.peer_addr()?);
    
    // 缓冲区 1K
    // Create a buffer, size of 1024 bytes for receive bytes stream
    let mut buff = [0u8; 1024];
    
    // 请求数据
    // Create a buffer for store request bytes
    let mut data = Vec::new();
    
    // 循环读取所有请求数据
    // Read all bytes from stream to request data
    loop {
        let n = stream.read(&mut buff)?;
        data.extend_from_slice(&buff[0..n]);
        // 如果当次读取字节数小于缓冲区大小, 说明数据读完了.
        // If read bytes < 1024, bytes in stream read finished
        if n < 1024 {
            break;
        }
    }

    // 创建解码器
    // Create a http decoder to parse bytes data to text request
    let mut decoder = RequestDecoder::<BodyDecoder<Utf8Decoder>>::default();
    
    // HTTP 从字节数组构造 HTTP 请求对象   
    // Parse bytes data to a response (echo)
    let req = match decoder.decode_from_bytes(data.as_slice()) {
        Ok(req) => {
            println!("method: {}", req.method());
            println!("request target: {}", req.request_target());
            println!("http_version: {}", req.http_version());
            println!("header: {}", req.header());
            handle_http(req)
        },
        Err(e) => Err(e),
    };
    // 构造应答
    // Build a response
    let resp = match req {
        Ok(resp) => resp,
        Err(e) => {
            let err = format!("{:?}", e);
            Response::new(
                HttpVersion::V1_0,
                StatusCode::new(500).unwrap(),
                ReasonPhrase::new(err.as_str()).unwrap(),
                err.clone(),
            )
        }
    };

    let write_buf = resp.to_string();
    // 响应流
    // Write response data to peer socket
    let _ = stream.write(write_buf.as_bytes())?;
    let _ = stream.write(b"\n")?;
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    // 默认端口
    // Default listen port
    let port = std::env::var("PORT").unwrap_or_else(|_| "1234".to_string());

    // 系统调用: 通过 FFI 调用底层套接字
    // System call: call libc ffi by syscall macro
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port), false)?;
    println!("Server is listening on: http://{}:{}", listener.address.ip(), listener.port);

    loop {
        let stream = listener.accept(false)?.0;        
        let _ = handle_client(stream);
    }
}
