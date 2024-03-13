use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rand::Rng;
use std::{cmp::min, collections::HashSet, fmt::Write};
use tokio::{fs::File, io::AsyncWriteExt};

use thunderstorm::{
    download::Download,
    file::{self, TorrentMeta},
    torrent::Torrent,
    tracker_peers::TrackerPeers,
    utils,
};

#[tokio::main]
async fn main() {
    let filename = std::env::args().nth(1).expect("No torrent path given");
    let output = std::env::args().nth(2);

    let torrent_meta = file::from_filename(&filename).unwrap();

    download_file(torrent_meta, output).await
}

pub async fn download_file(torrent_meta: TorrentMeta, out_file: Option<String>) {
    let mut rng = rand::prelude::ThreadRng::default();
    let random_peers: [u8; 20] = (0..20)
        .map(|_| rng.gen())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    let torrent = Torrent::new(&torrent_meta);

    //TODO: move it to a download manager state
    let tracker_stream = TrackerPeers::new(torrent_meta, 15, random_peers);

    //TODO: I think this is really bad
    let mut final_buf = vec![0u8; torrent.length as usize];

    //TODO: return more than just the buffer
    let downloader = Download::download_torrent(torrent.clone(), tracker_stream.clone()).await;

    let total_size = torrent.length as u64;
    let pb = ProgressBar::new(total_size);

    // let len_peers = tracker_stream.peers.len().clone();
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}][{msg}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec},{eta})"
            ).unwrap().with_key(
            "eta",
            |state: &ProgressState, 
                w: &mut dyn Write | write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        ).progress_chars("#>-")
    );

    let mut done_pieces = HashSet::<usize>::new();
    while done_pieces.len() < torrent.piece_hashes.len() {
        let pr = downloader.pr_rx.recv_async().await.unwrap();

        let new = min((done_pieces.len() * pr.buf.len()) as u64, total_size);
        pb.set_position(new);

        let peer_len = tracker_stream.peers.len();
        pb.set_message(peer_len.to_string());

        let (start, end) = utils::calculate_bounds_for_piece(&torrent, pr.index as usize);
        final_buf[start..end].copy_from_slice(pr.buf.as_slice());

        done_pieces.insert(pr.index as usize);
    }

    let out_filename = match out_file {
        Some(name) => name,
        // None => torrent_meta.torrent_file.info.name.clone(),
        None => "output".to_string(),
    };
    let mut file = File::create(out_filename).await.unwrap();
    file.write_all(final_buf.as_slice()).await.unwrap();
    file.sync_all().await.unwrap()
}
