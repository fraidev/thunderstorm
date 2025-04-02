use std::time::Duration;

use tokio::net::TcpStream;

use crate::protocol::{Protocol, ProtocolError};
use crate::{bitfield::Bitfield, peer::Peer};

#[derive(Debug)]
pub struct Client {
    pub choked: bool,
    pub bitfield: Bitfield,
    pub protocol: Protocol,
    pub stream: TcpStream,
}

impl Client {
    pub async fn connect(
        peer: Peer,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
        send_interested: bool,
    ) -> Result<Self, ProtocolError> {
        let func = async {
            let addr = format!("{}:{}", peer.ip, peer.port);
            TcpStream::connect(addr).await.map_err(ProtocolError::Io)
        };
        let mut stream = match tokio::time::timeout(Duration::from_secs(6), func).await {
            Ok(Ok(b)) => Ok(b),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(ProtocolError::Timeout(e)),
        }?;

        let protocol = Protocol::connect(peer, info_hash, peer_id).await?;
        let _handshake = protocol.complete_handshake(&mut stream).await?;
        let bitfield_bytes = protocol.recv_bitfield(&mut stream).await?;

        if send_interested {
            protocol.send_unchoke(&mut stream).await?;
            protocol.send_interested(&mut stream).await?;
        }

        Ok(Self {
            choked: true,
            bitfield: Bitfield::new(bitfield_bytes),
            protocol,
            stream,
        })
    }
}
