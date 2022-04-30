use async_std::{
    io::{stdin, BufReader},
    net::TcpStream,
    prelude::*,
    task,
};
use futures::{select, FutureExt};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
fn main() {
    task::block_on(try_run()).unwrap()
}
async fn try_run() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (reader, mut writer) = (&stream, &stream);
    let mut lines_from_server = BufReader::new(reader).split(0x17).fuse();
    let mut lines_from_stdin = BufReader::new(stdin()).lines().fuse();
    let name = match lines_from_stdin.next().await {
        Some(name) => name?,
        None => Err("Something about name entry is invalid")?,
    };
    let to = match lines_from_stdin.next().await {
        Some(name) => name?,
        None => Err("Something about to entry is invalid")?,
    };
    writer.write_all(name.as_bytes()).await?;
    writer.write_all(&[0x17]).await?;
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
                    let line = format!("{0}{1}{2}{1}{3}",name,0x03 as char,to,line);
                    writer.write_all(line.as_bytes()).await?;
                    writer.write_all(&[0x17]).await?;
                }
                None=>break,
            }
        }
    }
    Ok(())
}
