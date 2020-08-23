use std::net::SocketAddr;

use tokio::{
    io::{Error as TokioIoError},
    net::{
        TcpListener,
        TcpStream,
        ToSocketAddrs,
    }
};

/// TELNET listener. Currently stub that **does** not implement TELNET at all!
/// 
/// ## Protocols
/// 
/// Telnet: https://tools.ietf.org/html/rfc854
/// Telnet over UTF-8: https://tools.ietf.org/html/rfc5198
/// Telnet Window Size Options: https://tools.ietf.org/html/rfc1073
/// Telnet End of Record Option: https://tools.ietf.org/html/rfc885
/// Telnet Echo Option: https://tools.ietf.org/html/rfc857
pub struct TelnetListener {
    tcp: TcpListener
}

pub type TelnetStream = TcpStream;

impl TelnetListener {
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> Result<TelnetListener, TokioIoError> {
        TcpListener::bind(addr).await.map(|tcp| TelnetListener { tcp })
    }

    pub async fn accept(&mut self) -> Result<(TelnetStream, SocketAddr), TokioIoError> {
        let res = self.tcp.accept().await;

        println!("New connection");

        res
    }
}