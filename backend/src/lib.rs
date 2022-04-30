mod message;

use std::{
    collections::{hash_map::Entry, HashMap},
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
        stream: Arc<TcpStream>,
    },
    Message(Message),
}

pub fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    let mut incoming = listener.incoming();
    let (broker_sender, broker_reciever) = mpsc::unbounded();
    let _broker_handle = task::spawn(broker_loop(broker_reciever));
    while let Some(Ok(result)) = incoming.next() {
        let _handle = spawn_and_log_error(handle_client(
            broker_sender.clone(),
            TcpStream::from(result),
        ));
    }
    Err("incoming.next() returned a None(documentation says is impossible)")?
}
async fn handle_client(mut broker: Sender<Event>, stream: TcpStream) -> Result<()> {
    let stream = Arc::new(stream);
    let reader = BufReader::new(&*stream);
    let mut messages = reader.split(0x17);
    let name = match messages.next().await {
        Some(name) => name?.iter().map(|c| *c as char).collect::<String>(),
        None => Err("Client disconnected immediately")?,
    };
    broker
        .send(Event::NewPeer {
            name,
            stream: Arc::clone(&stream),
        })
        .await
        .unwrap();
    while let Some(msg) = messages.next().await {
        let message = msg?.iter().map(|c| *c as char).collect::<String>();
        let msg = Message::from(message);
        broker.send(Event::Message(msg)).await.unwrap();
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
        stream.write_all(&[0x17]).await?;
    }
    Ok(())
}

async fn broker_loop(mut events: Reciever<Event>) -> Result<()> {
    let mut peers: HashMap<String, Sender<String>> = HashMap::new();
    while let Some(event) = events.next().await {
        match event {
            Event::Message(Message { from, to, msg }) => {
                for addr in to {
                    if let Some(peer) = peers.get_mut(&addr) {
                        let message = format!("{}: {}", from, msg);
                        peer.send(message).await?
                    } else {
                        println!("person unknown {}", addr)
                    }
                }
            }
            Event::NewPeer { name, stream } => match peers.entry(name) {
                Entry::Occupied(..) => (),
                Entry::Vacant(entry) => {
                    let (client_sender, client_reciever) = mpsc::unbounded();
                    entry.insert(client_sender);
                    spawn_and_log_error(writer_loop(client_reciever, stream));
                }
            },
        }
    }
    Ok(())
}
