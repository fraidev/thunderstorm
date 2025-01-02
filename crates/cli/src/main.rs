use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::{
    fmt::Write,
    io::SeekFrom,
    sync::{atomic::AtomicU64, Arc},
};
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};
use tracing::{trace, warn};

use thunderstorm::{
    file::{self, TorrentMeta},
    session::Session,
    torrent::Torrent,
    tracker_peers::TrackerPeers,
    utils,
};

#[tokio::main]
async fn main() {
    #[cfg(not(feature = "tokio-console"))]
    tracing_subscriber::fmt::init();

    #[cfg(feature = "tokio-console")]
    console_subscriber::init();

    let filename = std::env::args().nth(1).expect("No torrent path given");
    let output = std::env::args().nth(2);

    let torrent_meta = file::from_filename(&filename).unwrap();

    download_file(torrent_meta, output).await
}

pub async fn download_file(torrent_meta: TorrentMeta, out_file: Option<String>) {
    let random_peers = utils::generate_peer_id();

    let torrent = Torrent::new(&torrent_meta.clone());

    let peer_states = Arc::new(thunderstorm::peer_state::PeerStates::new());
    let (have_broadcast, _) = tokio::sync::broadcast::channel(128);
    let have_broadcast = Arc::new(have_broadcast);

    //TODO: move it to a download manager state
    let tracker_stream = TrackerPeers::new(
        torrent_meta.clone(),
        15,
        random_peers,
        peer_states,
        have_broadcast.clone(),
    );

    //TODO: I think this is really bad

    //TODO: return more than just the buffer
    let downloader =
        Session::download_torrent(torrent.clone(), tracker_stream.clone(), have_broadcast).await;

    let total_size = torrent.length as u64;
    let pb = ProgressBar::new(total_size);

    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}][{msg}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec},{eta})"
            ).unwrap().with_key(
            "eta",
            | state: &ProgressState, w: &mut dyn Write | write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        ).progress_chars("#>-")
    );

    let out_filename = match out_file {
        Some(name) => name,
        None => torrent_meta.clone().torrent_file.info.name.clone(),
    };
    let mut file = File::create(out_filename).await.unwrap();
    warn!("Total size: {}", total_size);

    // File
    let total_downloaded = Arc::new(AtomicU64::new(0));
    let total_downloaded_clone = total_downloaded.clone();

    tokio::spawn(async move {
        loop {
            let new = total_downloaded_clone.load(std::sync::atomic::Ordering::Relaxed);
            pb.set_position(new);
            pb.set_message("Downloading");
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    let mut hashset = std::collections::HashSet::new();
    while hashset.len() < torrent.piece_hashes.len() {
        let pr = downloader.pr_rx.recv_async().await.unwrap();

        hashset.insert(pr.index);
        let (start, end) = utils::calculate_bounds_for_piece(&torrent, pr.index as usize);
        trace!(
            "index: {}, start: {}, end: {} len {}",
            pr.index,
            start,
            end,
            pr.length
        );
        file.seek(SeekFrom::Start(start as u64)).await.unwrap();
        file.write_all(pr.buf.as_slice()).await.unwrap();

        total_downloaded.fetch_add(pr.length as u64, std::sync::atomic::Ordering::Relaxed);
    }

    file.sync_all().await.unwrap()
}
