use crate::{file::TorrentMeta, peer::Peer};


#[derive(Debug, Clone, PartialEq)]
pub struct Torrent {
    pub peers: Vec<Peer>,
    pub peer_id: [u8; 20],
    pub info_hash: [u8; 20],
    pub piece_hashes: Vec<[u8; 20]>,
    pub piece_length: i64,
    pub length: i64,
}

impl Torrent {
    pub fn new(torrent_meta: &TorrentMeta, peers: Vec<Peer>, peer_id: [u8; 20]) -> Torrent {
        Torrent {
            peers,
            peer_id,
            info_hash: torrent_meta.info_hash,
            piece_hashes: torrent_meta.piece_hashes.clone(),
            piece_length: torrent_meta.torrent_file.info.piece_length,
            length: torrent_meta.torrent_file.info.length.unwrap(),
        }
    }
}
