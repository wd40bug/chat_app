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
use futures::{channel::mpsc, select, sink::SinkExt, Future, FutureExt};
use shared::{message::Message, Result};

type Reciever<T> = mpsc::UnboundedReceiver<T>;
type Sender<T> = mpsc::UnboundedSender<T>;

enum Void {}
enum Event {
    NewPeer {
        name: String,
        stream: Arc<TcpStream>,
        shutdown: Reciever<Void>,
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
    drop(broker_sender);
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
    let (_shutdown_sender, shutdown_reciever) = mpsc::unbounded();
    broker
        .send(Event::NewPeer {
            name,
            stream: Arc::clone(&stream),
            shutdown: shutdown_reciever,
        })
        .await
        .unwrap();
    while let Some(msg) = messages.next().await {
        let message = msg?.iter().map(|c| *c as char).collect::<String>();
        let msg = serde_json::from_str(&message)?;
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

async fn writer_loop(
    messages: &mut Reciever<String>,
    stream: Arc<TcpStream>,
    shutdown: Reciever<Void>,
) -> Result<()> {
    let mut stream = &*stream;
    let mut messages = messages.fuse();
    let mut shutdown = shutdown.fuse();
    // while let Some(message) = messages.next().await {
    //     stream.write_all(message.as_bytes()).await?;
    //     stream.write_all(&[0x17]).await?;
    // }
    loop {
        select! {
            msg = messages.next().fuse() => match msg{
                Some(msg)=>{
                    stream.write_all(msg.as_bytes()).await?;
                    stream.write_all(&[0x17]).await?;
                }
                None=>break,
            },
            void = shutdown.next().fuse() => match void {
                Some(void)=>match void{},
                None=>break,
            }
        }
    }
    Ok(())
}

async fn broker_loop(events: Reciever<Event>) {
    let (disconnect_sender, mut disconnect_receiver) =
        mpsc::unbounded::<(String, Reciever<String>)>();
    let mut peers: HashMap<String, Sender<String>> = HashMap::new();
    let mut events = events.fuse();
    loop {
        let event = select! {
            event = events.next().fuse() => match event {
                None=>break,
                Some(event)=>event,
            },
            disconnect = disconnect_receiver.next().fuse() => {
                let (name, _pending_messages) = disconnect.unwrap();
                assert!(peers.remove(&name).is_some());
                continue;
            }
        };
        match event {
            Event::Message(message) => {
                for addr in &message.to {
                    if let Some(peer) = peers.get_mut(addr) {
                        let msg = serde_json::ser::to_string(&message).unwrap();
                        peer.send(msg).await.unwrap();
                    } else {
                        println!("person unknown {}", addr)
                    }
                }
            }
            Event::NewPeer {
                name,
                stream,
                shutdown,
            } => match peers.entry(name.clone()) {
                Entry::Occupied(..) => (),
                Entry::Vacant(entry) => {
                    let (client_sender, mut client_reciever) = mpsc::unbounded();
                    entry.insert(client_sender);
                    let mut disconnect_sender = disconnect_sender.clone();
                    spawn_and_log_error(async move {
                        let res = writer_loop(&mut client_reciever, stream, shutdown).await;
                        disconnect_sender
                            .send((name, client_reciever))
                            .await
                            .unwrap();
                        res
                    });
                }
            },
        }
    }
    drop(peers);
    drop(disconnect_sender);
    while let Some((_name, _pending_messages)) = disconnect_receiver.next().await {}
}
