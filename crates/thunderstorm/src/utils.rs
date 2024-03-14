use crate::torrent::Torrent;
use anyhow::bail;
use rand::Rng;
use tokio_util::sync::CancellationToken;
use tracing::{error, trace, Instrument};

const BLOCK_SIZE: u32 = 16384;

pub fn calculate_bounds_for_piece(torrent: &Torrent, index: usize) -> (usize, usize) {
    let start = index * torrent.piece_length as usize;
    let end = start + torrent.piece_length as usize;
    let torrent_length = torrent.length as usize;

    if end > torrent_length {
        (start, torrent_length)
    } else {
        (start, end)
    }
}

pub fn calculate_piece_size(torrent: &Torrent, index: usize) -> usize {
    let (start, end) = calculate_bounds_for_piece(torrent, index);
    end - start
}

pub fn calculate_block_size(piece_length: u32, requested: u32) -> u32 {
    if piece_length - requested < BLOCK_SIZE {
        return piece_length - requested;
    };
    BLOCK_SIZE
}

pub fn check_integrity(hash: &[u8], buf: &[u8]) -> bool {
    let mut hasher = sha1_smol::Sha1::new();
    hasher.update(buf);
    let result = hasher.digest().bytes();
    result == hash
}

pub fn generate_peer_id() -> [u8; 20] {
    let mut rng = rand::prelude::ThreadRng::default();
    (0..20)
        .map(|_| rng.gen())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap()
}

/// Spawns a future with tracing instrumentation.
pub fn spawn(
    fut: impl std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
) -> tokio::task::JoinHandle<()> {
    let fut = async move {
        trace!("started");
        tokio::pin!(fut);
        let mut trace_interval = tokio::time::interval(std::time::Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = trace_interval.tick() => {
                    trace!("still running");
                },
                r = &mut fut => {
                    match r {
                        Ok(_) => {
                            trace!("finished");
                        }
                        Err(e) => {
                            error!("finished with error: {:#}", e)
                        }
                    }
                    return;
                }
            }
        }
    };
    // .instrument(span);
    tokio::task::spawn(fut)
}

// pub fn spawn_with_cancel(
//     span: tracing::Span,
//     cancellation_token: CancellationToken,
//     fut: impl std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
// ) -> tokio::task::JoinHandle<()> {
//     spawn(span, async move {
//         tokio::select! {
//             _ = cancellation_token.cancelled() => {
//                 bail!("cancelled");
//             },
//             r = fut => r
//         }
//     })
// }
