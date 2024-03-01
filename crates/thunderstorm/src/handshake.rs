#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Handshake {
    pub pstr: String,
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HandshakeError {
    ProtocolLengthCantBeZero,
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            pstr: "BitTorrent protocol".to_string(),
            info_hash,
            peer_id,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut handshake = Vec::new();
        handshake.push(self.pstr.len() as u8);
        handshake.extend(self.pstr.as_bytes());
        handshake.extend(vec![0u8; 8]);
        handshake.extend(self.info_hash);
        handshake.extend(self.peer_id);
        handshake
    }
    pub fn read(
        protocol_str_len: usize,
        handshake_buf: Vec<u8>,
    ) -> Result<Handshake, HandshakeError> {
        if protocol_str_len == 0 {
            return Err(HandshakeError::ProtocolLengthCantBeZero);
        }
        let i = protocol_str_len + 8;
        let info_hash_buffer = handshake_buf[i..(i + 20)].try_into().unwrap();
        let peer_id_buffer = handshake_buf[(i + 20)..].try_into().unwrap();
        Ok(Handshake::new(info_hash_buffer, peer_id_buffer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const HASH_INFO: [u8; 20] = [
        134, 212, 200, 0, 36, 164, 105, 190, 76, 80, 188, 90, 16, 44, 247, 23, 128, 49, 0, 116,
    ];
    const PEER_ID: [u8; 20] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];

    #[test]
    fn serialize_handshake() {
        let expected = vec![
            19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99,
            111, 108, 0, 0, 0, 0, 0, 0, 0, 0, 134, 212, 200, 0, 36, 164, 105, 190, 76, 80, 188, 90,
            16, 44, 247, 23, 128, 49, 0, 116, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20,
        ];
        let handshake = Handshake::new(HASH_INFO, PEER_ID);
        let result = handshake.serialize();

        assert_eq!(result, expected);
    }

    #[test]
    fn sucefull_reading_handshake() {
        let protocol_str_len = 19;
        let handshake_bytes = vec![
            66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99, 111,
            108, 0, 0, 0, 0, 0, 0, 0, 0, 134, 212, 200, 0, 36, 164, 105, 190, 76, 80, 188, 90, 16,
            44, 247, 23, 128, 49, 0, 116, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20,
        ];
        let result = Handshake::read(protocol_str_len, handshake_bytes).unwrap();
        let expected = Handshake::new(HASH_INFO, PEER_ID);

        assert_eq!(result, expected);
    }

    #[test]
    fn failure_reading_handshake_when_pstrlen_is_zero() {
        let protocol_str_len = 0;
        let handshake_bytes = vec![
            66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99, 111,
            108, 0, 0, 0, 0, 0, 0, 0, 0, 134, 212, 200, 0, 36, 164, 105, 190, 76, 80, 188, 90, 16,
            44, 247, 23, 128, 49, 0, 116, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20,
        ];
        let result = Handshake::read(protocol_str_len, handshake_bytes);

        assert_eq!(result, Err(HandshakeError::ProtocolLengthCantBeZero));
    }
}
