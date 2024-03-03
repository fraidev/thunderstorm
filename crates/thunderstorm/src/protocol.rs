use crate::handshake::{Handshake, HandshakeError};
use crate::message;
use crate::{message::Message, peer::Peer};
use byteorder::{BigEndian, ByteOrder};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::error::Elapsed;

const HANDSHAKE_TIMEOUT: u64 = 3;

#[derive(Debug)]
pub enum ProtocolError {
    Handshake(HandshakeError),
    Timeout(Elapsed),
    Io(std::io::Error),
    InfoHashIsNotEqual,
    ExpectedBitfieldId,
    MessageIsNone,
}

#[derive(Debug, Clone)]
pub struct Protocol {
    pub stream: Arc<Mutex<TcpStream>>,
    pub peer: Peer,
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Protocol {
    pub async fn connect(
        peer: Peer,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
    ) -> Result<Self, ProtocolError> {
        let func = async {
            let addr = format!("{}:{}", peer.ip, peer.port);
            let stream_raw = TcpStream::connect(addr).await.map_err(ProtocolError::Io)?;
            let stream = Arc::new(Mutex::new(stream_raw));
            Ok(Self {
                stream,
                peer,
                info_hash,
                peer_id,
            })
        };
        match tokio::time::timeout(Duration::from_secs(6), func).await {
            Ok(Ok(b)) => Ok(b),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(ProtocolError::Timeout(e)),
        }
    }

    pub async fn read(&self) -> Result<Option<Message>, ProtocolError> {
        let length_buf = &mut [0u8; 4];
        let mut stream = self.stream.lock().await;
        stream
            .read_exact(length_buf)
            .await
            .map_err(ProtocolError::Io)?;

        let length = BigEndian::read_u32(length_buf) as usize;

        let msg_bytes = &mut vec![0u8; length];
        stream
            .read_exact(msg_bytes)
            .await
            .map_err(ProtocolError::Io)?;
        Ok(message::read(length_buf, msg_bytes))
    }

    pub async fn send_request(
        &self,
        index: u32,
        start: u32,
        length: u32,
    ) -> Result<(), ProtocolError> {
        let mut stream = self.stream.lock().await;
        let msg = message::format_request(index, start, length);
        let msg_bytes = message::serialize(Some(msg));
        stream
            .write_all(&msg_bytes)
            .await
            .map_err(ProtocolError::Io)
    }

    pub async fn send_interested(&self) -> Result<(), ProtocolError> {
        let mut stream = self.stream.lock().await;
        let msg = message::Message {
            id: message::MessageId::MsgInterested,
            payload: vec![],
        };
        let msg_bytes = message::serialize(Some(msg));
        stream
            .write_all(&msg_bytes)
            .await
            .map_err(ProtocolError::Io)
    }

    pub async fn send_not_interested(&self) -> Result<(), ProtocolError> {
        let mut stream = self.stream.lock().await;
        let msg = message::Message {
            id: message::MessageId::MsgNotInterested,
            payload: vec![],
        };
        let msg_bytes = message::serialize(Some(msg));
        stream
            .write_all(&msg_bytes)
            .await
            .map_err(ProtocolError::Io)
    }

    pub async fn send_unchoke(&self) -> Result<(), ProtocolError> {
        let mut stream = self.stream.lock().await;
        let msg = message::Message {
            id: message::MessageId::MsgUnchoke,
            payload: vec![],
        };
        let msg_bytes = message::serialize(Some(msg));
        stream
            .write_all(&msg_bytes)
            .await
            .map_err(ProtocolError::Io)
    }

    pub async fn send_have(&self, index: u32) -> Result<(), ProtocolError> {
        let mut stream = self.stream.lock().await;
        let msg = message::format_have(index);
        let msg_bytes = message::serialize(Some(msg));
        stream
            .write_all(&msg_bytes)
            .await
            .map_err(ProtocolError::Io)
    }

    pub async fn complete_handshake(&self) -> Result<Handshake, ProtocolError> {
        let timeout = tokio::time::timeout(Duration::from_secs(HANDSHAKE_TIMEOUT), async {
            let mut stream = self.stream.lock().await;
            let handshake = Handshake::new(self.info_hash, self.peer_id);
            let handshake_bytes = handshake.serialize();
            stream
                .write_all(&handshake_bytes)
                .await
                .map_err(ProtocolError::Io)?;

            let protocol_str_len_buf = &mut [0u8; 1];
            stream
                .read_exact(protocol_str_len_buf)
                .await
                .map_err(ProtocolError::Io)?;
            let protocol_str_len = protocol_str_len_buf[0] as usize;
            let handshake_bytes = &mut vec![0u8; protocol_str_len + 48];
            stream
                .read_exact(handshake_bytes)
                .await
                .map_err(ProtocolError::Io)?;

            Handshake::read(protocol_str_len, handshake_bytes.to_vec())
                .map_err(ProtocolError::Handshake)
        })
        .await;

        match timeout {
            Ok(Ok(h)) => {
                if h.info_hash != self.info_hash {
                    return Err(ProtocolError::InfoHashIsNotEqual);
                }
                Ok(h)
            }
            Ok(Err(e)) => Err(e),
            Err(e) => Err(ProtocolError::Timeout(e)),
        }
    }

    pub async fn recv_bitfield(&self) -> Result<Vec<u8>, ProtocolError> {
        let func = async {
            match self.read().await? {
                None => Err(ProtocolError::MessageIsNone),
                Some(msg) => {
                    if msg.id != message::MessageId::MsgBitfield {
                        Err(ProtocolError::ExpectedBitfieldId)
                    } else {
                        Ok(msg.payload)
                    }
                }
            }
        };
        match tokio::time::timeout(Duration::from_secs(6), func).await {
            Ok(Ok(b)) => Ok(b),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(ProtocolError::Timeout(e)),
        }
    }
}
