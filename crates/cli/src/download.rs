use crate::{
    file::{self, TorrentMeta},
    pool::ConnectionPool,
    utils,
};
use flume::{SendError, Sender};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rand::Rng;
use std::{cmp::min, collections::HashSet, fmt::Write, usize};
use thunderstorm::{
    client::Client,
    message::{self, MessageError, MessageId},
    peer::{self, Peer},
    protocol::ProtocolError,
};
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug, Clone)]
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

enum DownloadError {
    SendPieceResult(SendError<PieceResult>),
    ProtocolError(ProtocolError),
    MessageError(MessageError),
    IntegrityError,
    ClientDoesNotHavePiece,
}

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

pub async fn download_file(torrent_meata: &TorrentMeta, out_file: Option<String>) {
    let mut rng = rand::prelude::ThreadRng::default();
    let random_peers: [u8; 20] = (0..20)
        .map(|_| rng.gen())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    let url = file::build_tracker_url(torrent_meata, &random_peers, 6881);
    let peers = peer::request_peers(&url).await.unwrap();
    let torrent = Torrent::new(torrent_meata, peers, random_peers);
    let final_buf = download_torrent(torrent).await;

    let out_filename = match out_file {
        Some(name) => name,
        None => torrent_meata.torrent_file.info.name.clone(),
    };
    let mut file = File::create(out_filename).await.unwrap();
    file.write_all(final_buf.as_slice()).await.unwrap();
    file.sync_all().await.unwrap()
}

async fn download_torrent(torrent: Torrent) -> Vec<u8> {
    let mut final_buf = vec![0u8; torrent.length as usize];

    let pool = ConnectionPool::new(torrent.clone(), torrent.peers.len());
    let (pool_tx, pool_rx) = pool.connect().await;
    let (pw_tx, pw_rx) = flume::unbounded::<PieceWork>();
    let (pr_tx, pr_rx) = flume::bounded::<PieceResult>(torrent.piece_hashes.len());

    let pieces_of_work = (0..(torrent.piece_hashes.len()) as u64)
        .map(|index| {
            let length = utils::calculate_piece_size(torrent.clone(), index as usize);
            PieceWork {
                index: index as u32,
                length: length as u32,
                hash: torrent.piece_hashes[index as usize],
            }
        })
        .collect::<Vec<PieceWork>>();

    for pw in pieces_of_work {
        pw_tx.send(pw).unwrap();
    }

    tokio::spawn(async move {
        loop {
            let mut client = pool_rx.recv_async().await.unwrap();
            let pw = pw_rx.recv_async().await.unwrap();
            let pw_tx = pw_tx.clone();
            let pr_tx = pr_tx.clone();
            let pool_tx = pool_tx.clone();
            tokio::spawn(async move {
                let task = download_piece(pw, &mut client, &pr_tx);
                let timeout = tokio::time::timeout(std::time::Duration::from_secs(10), task).await;
                match timeout {
                    Ok(Ok(_)) => {}
                    Ok(Err(_e)) => {
                        pw_tx.send(pw).unwrap();
                    }
                    Err(_e) => {
                        pw_tx.send(pw).unwrap();
                    }
                }
                pool_tx.send(client.clone()).unwrap();
            });
        }
    });

    let total_size = torrent.length as u64;
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec},{eta})"
            ).unwrap().with_key(
            "eta", 
            |state: &ProgressState, w: &mut dyn Write
            | write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        ).progress_chars("#>-")
    );

    let mut done_pieces = HashSet::<usize>::new();
    while done_pieces.len() < torrent.piece_hashes.len() {
        let pr = pr_rx.recv_async().await.unwrap();

        let new = min((done_pieces.len() * pr.buf.len()) as u64, total_size);
        pb.set_position(new);
        let (start, end) = utils::calculate_bounds_for_piece(torrent.clone(), pr.index as usize);
        final_buf[start..end].copy_from_slice(pr.buf.as_slice());

        done_pieces.insert(pr.index as usize);
    }

    final_buf
}

async fn download_piece(
    pw: PieceWork,
    client: &mut Client,
    pr_tx: &Sender<PieceResult>,
) -> Result<(), DownloadError> {
    let mut state = State {
        requested: 0,
        downloaded: 0,
        buf: vec![0u8; pw.length as usize],
    };
    if !client.bitfield.has_piece(pw.index as usize) {
        return Err(DownloadError::ClientDoesNotHavePiece);
    }

    while state.downloaded < pw.length {
        if !client.choked {
            let block_size = utils::calculate_block_size(pw.length, state.requested);
            while state.requested < pw.length {
                client
                    .protocol
                    .send_request(pw.index, state.requested, block_size)
                    .await
                    .map_err(DownloadError::ProtocolError)?;
                state.requested += block_size;
            }
        }

        let msg_opt = client
            .protocol
            .read()
            .await
            .map_err(DownloadError::ProtocolError)?;

        if let Some(msg) = msg_opt {
            match msg.id {
                MessageId::MsgChoke => {
                    client.choked = true;
                }
                MessageId::MsgUnchoke => {
                    client.choked = false;
                }
                MessageId::MsgHave => {
                    let index = message::parse_have(msg).map_err(DownloadError::MessageError)?;
                    client.bitfield.set_piece(index as usize);
                }
                MessageId::MsgPiece => {
                    let buf_len = message::parse_piece(pw.index, &mut state.buf, msg)
                        .map_err(DownloadError::MessageError)?;
                    state.downloaded += buf_len as u32;
                }
                _ => {}
            }
        };
    }

    if !utils::check_integrity(&pw.hash, state.buf.as_slice()) {
        return Err(DownloadError::IntegrityError);
    }

    client
        .protocol
        .send_have(pw.index)
        .await
        .map_err(DownloadError::ProtocolError)?;
    pr_tx
        .send(PieceResult {
            index: pw.index,
            length: pw.length,
            buf: state.buf.clone(),
        })
        .map_err(DownloadError::SendPieceResult)
}
