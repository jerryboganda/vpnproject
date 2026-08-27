use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let socket = UdpSocket::bind("127.0.0.1:5353").await?;
    let local_addr = socket.local_addr()?;
    println!("VPNBridge Mock DNS Server listening on {local_addr}");

    let mut buf = vec![0u8; 512];
    loop {
        let (len, peer) = socket.recv_from(&mut buf).await?;
        if len >= 12 {
            let mut response = buf[..len].to_vec();
            // Set QR bit (response)
            response[2] |= 0x80;
            // Echo back response
            let _ = socket.send_to(&response, peer).await;
        }
    }
}
