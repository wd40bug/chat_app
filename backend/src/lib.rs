mod message;

use std::{
    net::{TcpListener, ToSocketAddrs},
    sync::Arc,
};

use async_std::{
    io::{prelude::BufReadExt, BufReader, WriteExt},
    net::TcpStream,
    prelude::StreamExt,
    task::{self, JoinHandle},
};
use futures::{channel::mpsc, sink::SinkExt, Future};

use crate::message::Message;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Reciever<T> = mpsc::UnboundedReceiver<T>;
type Sender<T> = mpsc::UnboundedSender<T>;

enum Event {
    NewPeer {
        name: String,
        stream: TcpStream,
    },
    Message {
        from: String,
        to: Vec<String>,
        msg: String,
    },
}

pub fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    let mut incoming = listener.incoming();
    while let Some(Ok(result)) = incoming.next() {
        let _handle = spawn_and_log_error(handle_client(TcpStream::from(result)));
    }
    #[allow(clippy::try_err)]
    Err("incoming.next() returned a None(documentation says is impossible)")?
}
async fn handle_client(stream: TcpStream) -> Result<()> {
    let mut writer = &stream;
    let reader = BufReader::new(&stream);
    let mut messages = reader.split(0x17);
    println!("accepted {}", stream.peer_addr()?);
    while let Some(line) = messages.next().await {
        let line = line?.iter().map(|c| *c as char).collect::<String>();
        let message = Message::from(line);
        println!("{}", message);
        writer.write_all(message.msg.as_bytes()).await?;
        writer.write_all(&[0x17]).await?;
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

async fn writer_loop(mut messages: Reciever<String>, stream: Arc<TcpStream>) -> Result<()> {
    let mut stream = &*stream;
    while let Some(message) = messages.next().await {
        stream.write_all(message.as_bytes()).await?;
    }
    Ok(())
}
