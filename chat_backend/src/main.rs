use async_std::{
    // 3
    prelude::*, // 1
    task,       // 2
};
use std::net::{TcpListener, ToSocketAddrs};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    let mut incoming = listener.incoming();
    while let Some(result) = incoming.next() {
        let stream = result?;
    }
    Err("incoming.next returned a None(documentation says is impossible)")?
}

fn main() {
    accept_loop("127.0.0.1::8080").unwrap();
}
