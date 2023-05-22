use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{Context, Error, Result};
use std::io;
use log::{debug, info, trace};

const K_MAX_MSG: usize = 4096 + 1;

async fn read_bytes(stream: &mut TcpStream, bytes: usize) -> Result<Vec<u8>> {
    debug!("reading {} bytes from stream", bytes);
    let mut buf = vec![0u8; bytes];
    let mut bytes_read: usize = 0;
    while bytes_read < bytes {
        trace!("bytes_read: {}", bytes_read);
        let bytes = stream
            .read(&mut buf[bytes_read..])
            .await
            .context("failed to read bytes from tcpstream")?;
        if bytes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Reached end of stream prematurely",
            )
            .into());
        }
        bytes_read += bytes;
    }
    Ok(buf)
}

async fn write_bytes<T: AsRef<[u8]>>(stream: &mut TcpStream, buf: &T) -> Result<()> {
    let slice: &[u8] = buf.as_ref();
    debug!("writing {} bytes to stream", slice.len());
    let mut bytes_written: usize = 0;
    while bytes_written < slice.len() {
        trace!("bytes_written: {}", bytes_written);
        let bytes = stream
            .write(&slice[bytes_written..])
            .await
            .context("failed to write bytes to the stream")?;
        bytes_written += bytes;
    }
    Ok(())
}

async fn handle_client(mut stream: TcpStream) -> Result<()> {
    let header = read_bytes(&mut stream, 4).await.context("read header")?;
    let len = u32::from_le_bytes(header.try_into().unwrap());
    debug!("new message of len {}", len);
    if len > K_MAX_MSG as u32 {
        return Err(Error::msg(
            "received req with len header greater than max size",
        ));
    }
    let mut bytes = read_bytes(&mut stream, len.try_into().unwrap()).await.context("read body")?;
    let message = std::str::from_utf8_mut(&mut bytes).context("request not valid utf8")?;
    let response = format!("ECHO: {}", message);
    let response_header = (response.len() as u32).to_le_bytes();
    debug!("writing response of len {}", response.len());
    write_bytes(&mut stream, &response_header).await.context("write header")?;
    write_bytes(&mut stream, &response).await.context("write body")?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().is_test(true).init();
    let listener = TcpListener::bind("0.0.0.0:6379").await.context("failed to bind address")?;
    loop {
        let (stream, _) = listener.accept().await.context("failed to accept incoming connection")?;
        info!("New connection: {}", stream.peer_addr().unwrap());
        tokio::spawn(async move {
            let r = handle_client(stream).await;
            if let Err(e) = r {
                eprintln!("Error handling client: {:?}", e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;
    use std::io::{Read, Write};

    #[test]
    fn test_echo_server() {
        println!("starting test");
        //std::env::set_var("RUST_LOG", "trace");
        thread::spawn(move || {
            super::main().unwrap();
        });
        thread::sleep(Duration::from_secs(1));
        {
            let mut stream = TcpStream::connect("localhost:6379").unwrap();
            
            let request = "Hello, server!";
            let request_len = request.len() as u32;

            let header = request_len.to_le_bytes();
            stream.write_all(&header).unwrap();
            stream.write_all(request.as_bytes()).unwrap();

            let mut header = [0; 4];
            stream.read_exact(&mut header).unwrap();
            let response_len = u32::from_le_bytes(header);
            let mut response = vec![0; response_len as usize];
            stream.read_exact(&mut response).unwrap();

            let response = String::from_utf8(response).unwrap();

            assert_eq!(response, format!("ECHO: {}", request));
        }
    }
}
