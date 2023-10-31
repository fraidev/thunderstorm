use crate::protocol::{Protocol, ProtocolError};
use crate::{bitfield::Bitfield, peer::Peer};

#[derive(Debug, Clone)]
pub struct Client {
    pub choked: bool,
    pub bitfield: Bitfield,
    pub protocol: Protocol,
}

impl Client {
    pub async fn connect(
        peer: Peer,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
        send_interested: bool,
    ) -> Result<Self, ProtocolError> {
        let protocol = Protocol::connect(peer, info_hash, peer_id).await?;
        let _handshake = protocol.complete_handshake().await?;
        let bitfield_bytes = protocol.recv_bitfield().await?;

        if send_interested {
            protocol.send_unchoke().await?;
            protocol.send_interested().await?;
        }

        Ok(Self {
            choked: true,
            bitfield: Bitfield::new(bitfield_bytes),
            protocol,
        })
    }
}
