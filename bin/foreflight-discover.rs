use std::net::UdpSocket;


fn main() -> std::io::Result<()> {
    {
        let mut socket = UdpSocket::bind("0.0.0.0:63093")?;

        loop {
            // Receives a single datagram message on the socket. If `buf` is too small to hold
            // the message, it will be cut off.
            let mut buf = [0; 1500];
            let (amt, src) = socket.recv_from(&mut buf)?;

            // Redeclare `buf` as slice of the received data and send reverse data back to origin.
            // let buf = &mut buf[..amt];
            // buf.reverse();
            // socket.send_to(buf, &src)?;


            println!("got packet {}", std::str::from_utf8(&buf).unwrap());

        }


    } // the socket is closed here
    Ok(())
}

