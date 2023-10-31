use crate::download::Torrent;

const BLOCK_SIZE: u32 = 16384;

pub fn calculate_bounds_for_piece(torrent: Torrent, index: usize) -> (usize, usize) {
    let start = index * torrent.piece_length as usize;
    let end = start + torrent.piece_length as usize;
    let torrent_length = torrent.length as usize;

    if end > torrent_length {
        (start, torrent_length)
    } else {
        (start, end)
    }
}

pub fn calculate_piece_size(torrent: Torrent, index: usize) -> usize {
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
