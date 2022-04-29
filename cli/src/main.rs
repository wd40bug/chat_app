use async_std::{
    io::{stdin, BufReader},
    net::TcpStream,
    prelude::*,
    task,
};
use futures::{select, FutureExt};
use std::io;

fn main() -> io::Result<()> {
    task::block_on(try_run())
}
async fn try_run() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (reader, mut writer) = (&stream, &stream);
    let mut lines_from_server = BufReader::new(reader).split(0x04).fuse();
    let mut lines_from_stdin = BufReader::new(stdin()).lines().fuse();
    loop {
        select! {
            line = lines_from_server.next().fuse()=> match line{
                Some(line)=>{
                    let line = line?.iter().map(|b| *b as char).collect::<String>();
                    println!("{}",line);
                }
                None=>break,
            },
            line = lines_from_stdin.next().fuse() => match line{
                Some(line)=>{
                    let line = line?;
                    let line = format!("{0}{1}{2}{1}{3}","William",0x03 as char,"None",line);
                    writer.write_all(line.as_bytes()).await?;
                    writer.write_all(&[0x04]).await?;
                }
                None=>break,
            }
        }
    }
    Ok(())
}
