use crate::tracker_stream::TrackerStream;
use crate::utils;
use crate::{
    client::Client,
    message::{self, MessageError, MessageId},
    protocol::ProtocolError,
    torrent::Torrent,
};
use flume::{Receiver, SendError, Sender};

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

pub async fn download_torrent(
    torrent: Torrent,
    tracker_stream: TrackerStream,
) -> Receiver<PieceResult> {
    let client_rx = tracker_stream.connect().await;
    let (pw_tx, pw_rx) = flume::unbounded::<PieceWork>();
    let (pr_tx, pr_rx) = flume::bounded::<PieceResult>(torrent.piece_hashes.len());

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

    for pw in pieces_of_work {
        pw_tx.send(pw).unwrap();
    }

    tokio::spawn(async move {
        while pr_tx.len() < torrent.piece_hashes.len() {
            let mut client = client_rx.recv_async().await.unwrap();
            let pw_tx = pw_tx.clone();
            let pr_tx = pr_tx.clone();
            let pw_rx = pw_rx.clone();
            tokio::spawn(async move {
                loop {
                    let pw = pw_rx.recv_async().await.unwrap();
                    let task = download_piece(pw, &mut client, &pr_tx);
                    let timeout =
                        tokio::time::timeout(std::time::Duration::from_secs(10), task).await;
                    match timeout {
                        Ok(Ok(_)) => {}
                        Ok(Err(_e)) => {
                            pw_tx.send(pw).unwrap();
                        }
                        Err(_e) => {
                            pw_tx.send(pw).unwrap();
                        }
                    }
                }
            });
        }
    });

    pr_rx
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
