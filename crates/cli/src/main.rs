mod download;
mod file;
mod pool;
mod utils;

#[tokio::main]
async fn main() {
    let filename = "debian-12.2.0-amd64-netinst.iso.torrent".to_string();
    // let filename = std::env::args().nth(1).expect("No torrent path given");
    let output = std::env::args().nth(2);

    let torrent_meta = file::from_filename(&filename).unwrap();
    download::download_file(&torrent_meta, output).await
}
