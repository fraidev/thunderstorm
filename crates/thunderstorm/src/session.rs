use std::sync::Arc;

use crate::torrent::Torrent;
use crate::tracker_peers::TrackerPeers;
use crate::utils;
use flume::Receiver;

#[derive(Debug, Clone, Copy)]
pub struct PieceWork {
    pub index: u32,
    pub length: u32,
    pub hash: [u8; 20],
}

#[derive(Debug, Clone)]
pub struct PieceResult {
    pub index: u32,
    pub length: u32,
    pub buf: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct State {
    pub requested: u32,
    pub downloaded: u32,
    pub buf: Vec<u8>,
}

pub struct Session {
    pub tracker_stream: TrackerPeers,
    pub pr_rx: Receiver<PieceResult>,
}

impl Session {
    pub async fn download_torrent(
        torrent: Torrent,
        tracker_stream: TrackerPeers,
        have_broadcast: Arc<tokio::sync::broadcast::Sender<u32>>,
    ) -> Self {
        let piece_rx = tracker_stream.piece_rx.clone();
        let (pr_tx, pr_rx) = flume::bounded::<PieceResult>(torrent.piece_hashes.len());
        //let (pr_tx, pr_rx) = flume::unbounded::<PieceResult>();

        let pieces_of_work = (0..(torrent.piece_hashes.len()) as u64)
            .map(|index| {
                let length = utils::calculate_piece_size(&torrent, index as usize);
                PieceWork {
                    index: index as u32,
                    length: length as u32,
                    hash: torrent.piece_hashes[index as usize],
                }
            })
            .collect::<Vec<PieceWork>>();

        tracker_stream.connect(pieces_of_work).await;

        let have_broadcast = have_broadcast.clone();

        tokio::spawn(async move {
            loop {
                let pr_tx = pr_tx.clone();
                let piece_rx = piece_rx.clone();
                let piece = piece_rx.recv_async().await.unwrap();
                have_broadcast.send(piece.index).unwrap();

                let pr = PieceResult {
                    index: piece.index,
                    length: piece.length,
                    buf: piece.buf,
                };
                pr_tx.send_async(pr).await.unwrap();
            }
        });

        Self {
            tracker_stream,
            pr_rx,
        }
    }
}
