#![warn(rust_2018_idioms)]

use futures::{Sink, SinkExt};
use std::error::Error;
use std::net::SocketAddr;
use std::{env, io};
use tokio::net::UdpSocket;
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use tokio_util::{codec, udp};

use fishfinder::adsb::gdl90;

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server { socket, mut buf } = self;

        tokio::task::spawn(async move || {});

        loop {
            let (len, addr) = socket.recv_from(&mut buf).await?;
            println!(
                "{}[recv:{}] => {}",
                addr,
                len,
                std::str::from_utf8(&buf).unwrap()
            );
        }
    }
}

async fn handle_foreflight_client(
    addr: String,
    rx: broadcast::Receiver<gdl90::Message>,
) -> Result<(), io::Error> {
    let sock = UdpSocket::bind("0.0.0.0:0").await?;

    let dest: SocketAddr = addr.parse().expect("Unable to parse socket address");

    let mut framed_encoder = udp::UdpFramed::new(sock, gdl90::Encoder::new());
    let mut msg_stream = BroadcastStream::new(rx)
        .filter(Result::is_ok)
        .map(|m| Ok((m.unwrap(), dest)));

    framed_encoder.send_all(&mut msg_stream).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:63093".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening on: {}", socket.local_addr()?);

    let server = Server {
        socket,
        buf: vec![0; 1024],
    };

    // This starts the server task.
    server.run().await?;

    Ok(())
}
