# BitRev

BitRev is a BitTorrent client written entirely in Rust.
This Project it's a rewrite of [tornado](https://github.com/fraidev/tornado) in Rust.

## Setup

Exemple of how to download a debian iso:

```bash
cargo run --release --bin cli debian-12.5.0-amd64-netinst.iso.torrent
```

Tests:

```bash
cargo test
```

Check sha256sum of the debian iso:

```bash
openssl dgst -sha256 debian-12.5.0-amd64-netinst.iso
```

Should be [013f5b44670d81280b5b1bc02455842b250df2f0c6763398feb69af1a805a14f](https://cdimage.debian.org/debian-cd/current/amd64/bt-cd/SHA256SUMS) for this example.

