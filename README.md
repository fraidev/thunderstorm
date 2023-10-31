# Thunderstorm

Thunderstorm is a BitTorrent client written entirely in Rust.
This Project it's a rewrite of [tornado](https://github.com/fraidev/tornado) in Rust.

## Setup

Exemple of how to download a debian iso:

```bash
cargo run --release --bin cli debian-12.2.0-amd64-netinst.iso.torrent
```

Tests:

```bash
cargo test
```

Check sha256sum of the debian iso:

```bash
openssl dgst -sha256 debian-12.2.0-amd64-netinst.iso
```

Should be [23ab444503069d9ef681e3028016250289a33cc7bab079259b73100daee0af66](https://cdimage.debian.org/debian-cd/current/amd64/bt-cd/SHA256SUMS) for this example.

