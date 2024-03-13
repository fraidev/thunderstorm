use std::sync::Arc;

use dashmap::DashMap;
use flume::Receiver;
use serde_bencode::de;

use crate::{
    client::Client,
    file::{self, TorrentMeta},
    peer::{BencodeResponse, Peer},
};

pub struct TrackerStream {
    torrent_meta: TorrentMeta,
    max_size: usize,
    peer_id: [u8; 20],
    sender: flume::Sender<Client>,
    receiver: flume::Receiver<Client>,
    peers: DashMap<Peer, Client>,
}

impl TrackerStream {
    pub fn new(torrent_meta: TorrentMeta, max_size: usize, peer_id: [u8; 20]) -> TrackerStream {
        let (sender, receiver) = flume::bounded(max_size);

        TrackerStream {
            torrent_meta,
            max_size,
            peer_id,
            sender,
            receiver,
            peers: DashMap::new(),
        }
    }

    pub async fn connect(&self) -> Receiver<Client> {
        let trackers = match (
            &self.torrent_meta.torrent_file.announce,
            &self.torrent_meta.torrent_file.announce_list,
        ) {
            (Some(announce), None) => vec![announce.clone()],
            (Some(announce), Some(announce_list)) => {
                let mut h = Vec::<String>::from_iter(announce_list.iter().flatten().cloned());
                if !h.contains(announce) {
                    h.push(announce.clone());
                }
                h.into_iter().collect()
            }
            (None, Some(announce_list)) => announce_list.clone().into_iter().flatten().collect(),
            (None, None) => vec![],
        };
        let tcp_tracker = trackers.iter().find(|t| !t.starts_with("udp://"));
        let url = file::build_tracker_url(
            &self.torrent_meta,
            &self.peer_id,
            6881,
            tcp_tracker.unwrap(),
        );

        let info_hash = self.torrent_meta.info_hash;
        let peer_id = self.peer_id;
        let peers = Arc::from(self.peers.clone());
        let sender = self.sender.clone();
        tokio::spawn(async move {
            loop {
                let request_peers_res = request_peers(&url).await.unwrap();
                let new_peers = request_peers_res.clone().get_peers().unwrap();

                for peer in new_peers {
                    let sender = sender.clone();

                    if peers.contains_key(&peer) {
                        continue;
                    }

                    let peers = peers.clone();
                    let peer = peer.clone();
                    tokio::spawn(async move {
                        let client = Client::connect(peer.clone(), info_hash, peer_id, true).await;
                        match client {
                            Ok(client) => {
                                peers.insert(peer, client.clone());
                                sender.send(client).unwrap();
                            }
                            Err(_e) => {}
                        };
                    });
                }

                //sleep interval
                tokio::time::sleep(std::time::Duration::from_millis(request_peers_res.interval)).await
            }
        });
        self.receiver.clone()
    }
}

pub async fn request_peers(uri: &str) -> anyhow::Result<BencodeResponse> {
    let client = reqwest::Client::new();
    let response = client.get(uri).send().await?;
    let body_bytes = response.bytes().await?;

    let tracker_bencode_decode = de::from_bytes::<BencodeResponse>(&body_bytes)?;
    Ok(tracker_bencode_decode)
}
