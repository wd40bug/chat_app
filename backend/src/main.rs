use std::{
    io::{self, BufRead},
    net::{TcpListener, ToSocketAddrs},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    let mut incoming = listener.incoming();
    while let Some(Ok(mut result)) = incoming.next() {
        let mut reader = io::BufReader::new(&mut result);
        let recieved: Vec<u8> = reader.fill_buf()?.to_vec();
        reader.consume(recieved.len());
        String::from_utf8(recieved)
            .map(|msg| println!("{}", msg))
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Couldn't parse received string as utf8",
                )
            })?
    }
    Err("incoming.next returned a None(documentation says is impossible)")?
}

fn main() {
    accept_loop("127.0.0.1::8080").unwrap();
}
