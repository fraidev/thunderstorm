use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use rand::Rng;
use std::{cmp::min, collections::HashSet, fmt::Write};
use tokio::{fs::File, io::AsyncWriteExt};

use thunderstorm::{
    download,
    file::{self, TorrentMeta},
    peer,
    torrent::Torrent,
    utils,
};

#[tokio::main]
async fn main() {
    let filename = std::env::args().nth(1).expect("No torrent path given");
    let output = std::env::args().nth(2);

    let torrent_meta = file::from_filename(&filename).unwrap();

    download_file(&torrent_meta, output).await
}

pub async fn download_file(torrent_meata: &TorrentMeta, out_file: Option<String>) {
    let mut rng = rand::prelude::ThreadRng::default();
    let random_peers: [u8; 20] = (0..20)
        .map(|_| rng.gen())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    let trackers = match (
        &torrent_meata.torrent_file.announce,
        &torrent_meata.torrent_file.announce_list,
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

    let url = file::build_tracker_url(torrent_meata, &random_peers, 6881, tcp_tracker.unwrap());
    println!("Requesting peers from: {}", url);
    let peers = peer::request_peers(&url).await.unwrap();

    let torrent = Torrent::new(torrent_meata, peers, random_peers);
    let mut final_buf = vec![0u8; torrent.length as usize];
    let recv = download::download_torrent(torrent.clone()).await;

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
        let pr = recv.recv_async().await.unwrap();

        let new = min((done_pieces.len() * pr.buf.len()) as u64, total_size);
        pb.set_position(new);
        let (start, end) = utils::calculate_bounds_for_piece(&torrent, pr.index as usize);
        final_buf[start..end].copy_from_slice(pr.buf.as_slice());

        done_pieces.insert(pr.index as usize);
    }

    let out_filename = match out_file {
        Some(name) => name,
        None => torrent_meata.torrent_file.info.name.clone(),
    };
    let mut file = File::create(out_filename).await.unwrap();
    file.write_all(final_buf.as_slice()).await.unwrap();
    file.sync_all().await.unwrap()
}
