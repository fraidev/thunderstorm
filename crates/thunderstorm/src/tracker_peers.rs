use dashmap::DashMap;
use serde_bencode::de;
use std::sync::Arc;

use crate::{
    client::Client,
    file::{self, TorrentMeta},
    peer::{BencodeResponse, Peer},
};

#[derive(Debug, Clone)]
pub struct TrackerPeers {
    torrent_meta: TorrentMeta,
    peer_id: [u8; 20],
    pub sender: flume::Sender<Client>,
    pub receiver: flume::Receiver<Client>,
    pub peers: Arc<DashMap<Peer, String>>,
}

impl TrackerPeers {
    pub fn new(torrent_meta: TorrentMeta, _max_size: usize, peer_id: [u8; 20]) -> TrackerPeers {
        let (sender, receiver) = flume::unbounded();
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

        let tcp_trackers = all_trackers(&self.torrent_meta.clone())
            .into_iter()
            // .filter(|t| !t.starts_with("udp://"))
            .find(|t| !t.starts_with("udp://"));

        //TODO: support udp trackers
        // for tracker in tcp_trackers {
        let tracker = tcp_trackers.unwrap();
        let sender = self.sender.clone();
        let peers = self.peers.clone();
        let torrent_meta = self.torrent_meta.clone();
        tokio::spawn(async move {
            loop {
                let url = file::build_tracker_url(&torrent_meta, &peer_id, 6881, &tracker);
                let sender = sender.clone();

                let request_peers_res_fut = request_peers(&url).await;
                if request_peers_res_fut.is_err() {
                    // return;
                    continue;
                }

                let request_peers_res = request_peers_res_fut.unwrap();

                let new_peers = request_peers_res.clone().get_peers();

                if new_peers.is_err() {
                    // return;
                    continue;
                }

                for peer in new_peers.unwrap() {
                    let sender = sender.clone();

                    if peers.contains_key(&peer) {
                        continue;
                    }

                    let peers = peers.clone();
                    let peer = peer.clone();

                    tokio::spawn(async move {
                        //TODO: create peer client with interface that can disconnect it
                        let client_future = Client::connect(peer.clone(), info_hash, peer_id, true);

                        let client =
                            tokio::time::timeout(std::time::Duration::from_secs(5), client_future)
                                .await;
                        if let Ok(Ok(client)) = client {
                            let s = sender.send_async(client).await;
                            if s.is_ok() {
                                peers.insert(peer, String::from("connected"));
                            }
                        }
                    });
                }
                //sleep interval
                tokio::time::sleep(std::time::Duration::from_millis(request_peers_res.interval))
                    .await
            }
        });
    }
}

fn all_trackers(torrent_meta: &TorrentMeta) -> Vec<String> {
    match (
        &torrent_meta.torrent_file.announce,
        &torrent_meta.torrent_file.announce_list,
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

pub async fn request_peers(uri: &str) -> anyhow::Result<BencodeResponse> {
    let client = reqwest::Client::new();
    let response = client.get(uri).send().await?;
    let body_bytes = response.bytes().await?;

    let tracker_bencode_decode = de::from_bytes::<BencodeResponse>(&body_bytes)?;
    Ok(tracker_bencode_decode)
}
