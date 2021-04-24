#![warn(rust_2018_idioms)]

use futures::{Sink, SinkExt};
use std::error::Error;
use std::net::SocketAddr;
use std::time;
use std::{env, io};

use std::collections::HashSet;
use tokio::net::UdpSocket;
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use tokio_util::{codec, udp};

use fishfinder::adsb::gdl90;

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    clients: HashSet<String>,
    tx: broadcast::Sender<gdl90::Message>,
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            socket,
            mut buf,
            mut clients,
            tx,
        } = self;

        loop {
            let (len, addr) = socket.recv_from(&mut buf).await?;
            println!(
                "{}[recv:{}] => {}",
                addr,
                len,
                std::str::from_utf8(&buf).unwrap()
            );

            let to_addr = SocketAddr::new(addr.ip(), 4000);
            let to_addr_string = to_addr.to_string();

            if !clients.contains(&to_addr_string) {
                clients.insert(to_addr_string);
                let rx = tx.subscribe();

                let mut clients_clone = clients.clone();

                tokio::spawn(async move {
                    println!("spawning foreflight pusher: {}", to_addr);
                    handle_foreflight_client(to_addr, rx).await.unwrap();
                    clients_clone.remove(&to_addr.to_string());
                });
            }
        }
    }
}

async fn handle_foreflight_client(
    addr: SocketAddr,
    rx: broadcast::Receiver<gdl90::Message>,
) -> Result<(), io::Error> {
    let sock = UdpSocket::bind("0.0.0.0:0").await?;

    let mut framed_encoder = udp::UdpFramed::new(sock, gdl90::Encoder::new());
    let mut msg_stream = BroadcastStream::new(rx)
        .filter(Result::is_ok)
        .map(|m| Ok((m.unwrap(), addr)));

    framed_encoder.send_all(&mut msg_stream).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::set_max_level(log::LevelFilter::Trace);

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:63093".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening on: {}", socket.local_addr()?);

    let (tx, _) = broadcast::channel(256);

    let server = Server {
        socket,
        buf: vec![0; 1024],
        clients: HashSet::new(),
        tx: tx.clone(),
    };

    tokio::spawn(async move {
        loop {
            let msg = gdl90::Message::Heartbeat(gdl90::Heartbeat::default());
            match tx.send(msg) {
                Ok(ok) => {
                    println!("sent: {:?}", ok);
                }
                Err(err) => {}
            }

            let msg = gdl90::Message::ForeflightIdentify(gdl90::ForeflightIdentify {
                version: 1,
                serial_number: 0xFFFFFFFFFFFFFFFF,
                device_name: "fish".to_string(),
                device_name_long: "fishfinder".to_string(),
                capabilities: 1,
            });

            match tx.send(msg) {
                Ok(ok) => {
                    println!("sent: {:?}", ok);
                }
                Err(err) => {}
            }

            tokio::time::sleep(time::Duration::from_secs(1)).await;
        }
    });

    // This starts the server task.
    server.run().await?;

    Ok(())
}
