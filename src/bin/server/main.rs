use anyhow::Result;
use anyhow::Context;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut data = [0 as u8, 64];
    while match stream.read(&mut data) {
        Ok(len) => {
            let w = &data[0..len];
            let str = String::from_utf8_lossy(w);
            println!("echo: {}", str);
            stream.write(w).context("failed to write to stream")?;
            true
        },
        Err(err) => {
            println!("err: {}", err);
            stream.shutdown(std::net::Shutdown::Both).context("failed to end stream")?;
            false
        }
    } {};
    Ok(())
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:1234").context("failed to bind address")?;
    for stream in listener.incoming() {
        let stream = stream.context("failed to open stream")?;
        println!("New connection: {}", stream.peer_addr().unwrap());
        thread::spawn(move|| {
            handle_client(stream)
        });
    };
    Ok(())
}
