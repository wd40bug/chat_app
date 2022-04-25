use std::net::{TcpListener, ToSocketAddrs};

use async_std::{
    io::{prelude::BufReadExt, BufReader, WriteExt},
    net::TcpStream,
    prelude::StreamExt,
    task::{self, JoinHandle},
};
use futures::Future;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() {
    accept_loop("127.0.0.1:8080").unwrap();
}

fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    let mut incoming = listener.incoming();
    while let Some(Ok(result)) = incoming.next() {
        let _handle = spawn_and_log_error(handle_client(TcpStream::from(result)));
    }
    Err("incoming.next returned a None(documentation says is impossible)")?
}

async fn handle_client(stream: TcpStream) -> Result<()> {
    let mut writer = &stream;
    let reader = BufReader::new(&stream);
    let mut messages = reader.split(0x04);
    println!("accepted {}", stream.peer_addr()?);
    while let Some(line) = messages.next().await {
        let line = line?;
        println!("{}", &line.iter().map(|c| *c as char).collect::<String>());
        writer.write_all(&line).await?;
        writer.write_all(&[0x04]).await?;
    }
    Ok(())
}

fn spawn_and_log_error<F>(fut: F) -> JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e);
        }
    })
}

async fn writer_loop(mut writer: &TcpStream) -> Result<()> {
    todo!()
}
