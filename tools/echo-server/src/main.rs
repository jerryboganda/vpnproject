use vpnbridge_test_support::{TcpEchoServer, UdpEchoServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (_tcp_server, tcp_addr) = TcpEchoServer::start().await?;
    let (_udp_server, udp_addr) = UdpEchoServer::start().await?;

    println!("VPNBridge Test Echo Server running:");
    println!("  TCP Echo: {tcp_addr}");
    println!("  UDP Echo: {udp_addr}");

    tokio::signal::ctrl_c().await?;
    println!("Shutting down echo server...");
    Ok(())
}
