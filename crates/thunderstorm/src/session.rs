use std::sync::Arc;

use flume::{Receiver, Sender};

use crate::{client::Client, torrent::Torrent};

#[derive(Clone)]
pub struct Session {
    torrent: Torrent,
    max_size: usize,
}

impl Session {
    pub fn new(torrent: Torrent, max_size: usize) -> Session {
        Session { torrent, max_size }
    }

    pub async fn connect(&self) -> (Sender<Client>, Receiver<Client>) {
        let (sender, receiver) = flume::bounded(self.max_size);
        let mut handles = Vec::with_capacity(self.max_size);

        for i in 0..=(self.max_size - 1) {
            let sender = sender.clone();
            let torrent = Arc::from(self.torrent.clone());
            let h = tokio::spawn(async move {
                let peers = torrent.peers.clone();
                let peer = peers[i].clone();
                let client =
                    Client::connect(peer.clone(), torrent.info_hash, torrent.peer_id, true).await;

                match client {
                    Ok(client) => {
                        sender.send(client).unwrap();
                    }
                    Err(_e) => {}
                };
            });
            handles.push(h);
        }

        for handle in handles {
            match handle.await {
                Ok(_) => {}
                Err(_e) => {}
            }
        }

        (sender, receiver)
    }
}
