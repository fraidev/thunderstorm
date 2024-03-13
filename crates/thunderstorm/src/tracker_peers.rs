use std::sync::Arc;

use dashmap::DashMap;
use serde_bencode::de;

use crate::{
    client::Client,
    file::{self, TorrentMeta},
    peer::{BencodeResponse, Peer},
};

#[derive(Debug, Clone)]
pub struct TrackerPeers {
    torrent_meta: TorrentMeta,
    peer_id: [u8; 20],
    sender: flume::Sender<Client>,
    pub receiver: flume::Receiver<Client>,
    pub peers: Arc<DashMap<Peer, Client>>,
}

impl TrackerPeers {
    pub fn new(torrent_meta: TorrentMeta, max_size: usize, peer_id: [u8; 20]) -> TrackerPeers {
        let (sender, receiver) = flume::bounded(max_size);
        TrackerPeers {
            torrent_meta,
            peer_id,
            sender,
            receiver,
            peers: Arc::new(DashMap::new()),
        }
    }

    pub async fn connect(&self) {
        let info_hash = self.torrent_meta.info_hash;
        let peer_id = self.peer_id;
        let peers = self.peers.clone();
        let sender = self.sender.clone();
        let trackers = self.all_trackers().clone();
        let torrent_meta = self.torrent_meta.clone();

        tokio::spawn(async move {
            loop {
                //TODO: support udp trackers
                let tcp_trackers = trackers.iter().filter(|t| !t.starts_with("udp://"));
                for tracker in tcp_trackers {
                    let url = file::build_tracker_url(&torrent_meta, &peer_id, 6881, tracker);
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
                            //TODO: create peer client with interface that can disconnect it
                            let client =
                                Client::connect(peer.clone(), info_hash, peer_id, true).await;
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
                    tokio::time::sleep(std::time::Duration::from_millis(request_peers_res.interval))
                        .await
                }
            }
        });
    }

    fn all_trackers(&self) -> Vec<String> {
        match (
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
        }
    }
}

pub async fn request_peers(uri: &str) -> anyhow::Result<BencodeResponse> {
    let client = reqwest::Client::new();
    let response = client.get(uri).send().await?;
    let body_bytes = response.bytes().await?;

    let tracker_bencode_decode = de::from_bytes::<BencodeResponse>(&body_bytes)?;
    Ok(tracker_bencode_decode)
}
