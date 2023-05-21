use anyhow::Context;
use anyhow::Result;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;

fn main() -> Result<()> {
    let mut stream = TcpStream::connect("0.0.0.0:1234").context("failed to open connection")?;

    // Spawn a thread to handle the server's responses
    let reader_stream = stream.try_clone().context("failed to clone stream")?;
    let handle = thread::spawn(move || -> Result<()> {
        let mut reader = BufReader::new(reader_stream);
        let mut buffer = String::new();
        while reader
            .read_line(&mut buffer)
            .context("failed to read line from buf")?
            > 0
        {
            print!("{}", buffer);
            buffer.clear();
        }
        Ok(())
    });

    // Read lines from stdin and send them to the server
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.context("failed to read line from stdin")?;
        let mut buffer = line.into_bytes();
        buffer.push('\n' as u8);
        stream
            .write_all(&buffer)
            .context("failed to write buf to stream")?;
    }

    // Wait for the response handling thread to finish
    handle
        .join()
        .expect("The response handling thread panicked")?;

    Ok(())
}
