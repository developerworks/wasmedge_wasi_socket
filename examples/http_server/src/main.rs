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
    // 缓冲区 1K
    let mut buff = [0u8; 1024];
    // 请求数据
    let mut data = Vec::new();
    // 循环读取所有请求数据
    loop {
        let n = stream.read(&mut buff)?;
        data.extend_from_slice(&buff[0..n]);
        // 如果当次读取字节数小于缓冲区大小, 说明数据读完了
        if n < 1024 {
            break;
        }
    }
    // 创建解码器
    let mut decoder = RequestDecoder::<BodyDecoder<Utf8Decoder>>::default();
    // 解码为请求对象
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
    let r = match req {
        Ok(r) => r,
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

    let write_buf = r.to_string();
    // 响应流
    let _ = stream.write(write_buf.as_bytes())?;
    let _ = stream.write(b"\n")?;
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    // 默认端口
    let port = std::env::var("PORT").unwrap_or_else(|_| "1234".to_string());
    println!("Server listening on {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port), false)?;
    loop {
        let stream = listener.accept(false)?.0;
        let addr = stream.local_addr()?;
        println!("Server is listening on: {:?}", addr);
        let _ = handle_client(stream);
    }
}
