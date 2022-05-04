use bytecodec::{bytes::Utf8Decoder, DecodeExt, Result};
use httpcodec::{
    BodyDecoder, HeaderField, HttpVersion, ReasonPhrase, Request, RequestDecoder, Response,
    StatusCode,
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream};

#[derive(Serialize, Deserialize, Default, Debug)]
struct ApiResult {
    code: i32,
    data: String,
}

fn handle_http(req: Request<String>) -> Result<Response<String>> {
    // Api 结果结构体
    // Api result struct
    let api_result = ApiResult {
        code: 0,
        data: req.body().to_string(),
    };

    // 构造响应
    // Build response 
    let mut resp = Response::new(
        HttpVersion::V1_0,
        StatusCode::new(200)?,
        ReasonPhrase::new("")?,
        serde_json::to_string(&api_result).unwrap_or_else(|_| "Serde serialize error".to_string()),
    );

    // 设置响应头
    // Set response header
    let header = HeaderField::new("Content-Type", "application/json");
    resp.header_mut().add_field(header?);
    
    Ok(resp)
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
    let result = match decoder.decode_from_bytes(data.as_slice()) {
        Ok(req) => {
            println!("method: {}", req.method());
            println!("request target: {}", req.request_target());
            println!("http_version: {}", req.http_version());
            println!("header: {}", req.header());
            handle_http(req)
        }
        Err(e) => Err(e),
    };
    // 构造应答
    // Build a response
    let resp = match result {
        Ok(resp) => resp,
        Err(e) => {
            let err = format!("{:?}", e);

            let api_result = ApiResult {
                code: -1,
                data: err.clone(),
            };
            Response::new(
                HttpVersion::V1_0,
                StatusCode::new(500).unwrap(),
                ReasonPhrase::new(err.as_str()).unwrap(),
                serde_json::to_string(&api_result)?,
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
    println!(
        "Server is listening on: http://{}:{}",
        listener.address.ip(),
        listener.port
    );

    loop {
        let stream = listener.accept(false)?.0;
        let _ = handle_client(stream);
    }
}
