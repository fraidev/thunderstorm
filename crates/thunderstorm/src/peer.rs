use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_bencode::de;
use serde_bytes::ByteBuf;

#[derive(Debug)]
pub enum ErrorPeers {
    ReicevedEmptyPeers,
    GetRequestError(Error),
    EncodingError(serde_bencode::Error),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Peer {
    pub ip: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct BencodeResponse {
    pub peers: ByteBuf,
}

pub async fn request_peers(uri: &str) -> Result<Vec<Peer>, ErrorPeers> {
    let client = Client::new();
    let response = client
        .get(uri)
        .send()
        .await
        .map_err(ErrorPeers::GetRequestError)?;
    let body_bytes = response
        .bytes()
        .await
        .map_err(ErrorPeers::GetRequestError)?;
    let tracker_bencode_decode =
        de::from_bytes::<BencodeResponse>(&body_bytes).map_err(ErrorPeers::EncodingError)?;
    let peers_bytes = tracker_bencode_decode.peers.to_vec();
    let peers = from_bytes(&peers_bytes)?;
    Ok(peers)
}

fn from_bytes(peers_bin: &Vec<u8>) -> Result<Vec<Peer>, ErrorPeers> {
    let peer_size = 6;
    let peers_bin_length = peers_bin.len();
    let num_peers = peers_bin_length / peer_size;
    if peers_bin_length % peer_size != 0 {
        return Err(ErrorPeers::ReicevedEmptyPeers);
    }

    let mut peers = Vec::new();
    for i in 0..num_peers {
        let ip_size = 4;
        let offset = i * peer_size;
        let ip_bin = &peers_bin[offset..offset + ip_size];
        let port =
            u16::from_be_bytes([peers_bin[offset + ip_size], peers_bin[offset + ip_size + 1]]);
        let ip_array = [ip_bin[0], ip_bin[1], ip_bin[2], ip_bin[3]];
        let ip = std::net::Ipv4Addr::new(ip_array[0], ip_array[1], ip_array[2], ip_array[3]);
        let peer = Peer {
            ip: ip.to_string(),
            port,
        };
        peers.push(peer);
    }
    Ok(peers)
}
