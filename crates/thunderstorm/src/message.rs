#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MessageId {
    MsgChoke = 0,
    MsgUnchoke = 1,
    MsgInterested = 2,
    MsgNotInterested = 3,
    MsgHave = 4,
    MsgBitfield = 5,
    MsgRequest = 6,
    MsgPiece = 7,
    MsgCancel = 8,
    MsgReject = 16,
    MsgHashRequest = 21,
    MsgHashes = 22,
    MsgHashReject = 23,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: MessageId,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageError {
    InvalidMessageId(String),
    InvalidPayload(String),
}

pub fn format_request(index: u32, start: u32, length: u32) -> Message {
    let mut payload = Vec::with_capacity(12);
    payload.extend_from_slice(&index.to_be_bytes());
    payload.extend_from_slice(&start.to_be_bytes());
    payload.extend_from_slice(&length.to_be_bytes());
    Message {
        id: MessageId::MsgRequest,
        payload,
    }
}

pub fn format_have(index: u32) -> Message {
    let mut payload = Vec::with_capacity(4);
    payload.extend_from_slice(&index.to_be_bytes());
    Message {
        id: MessageId::MsgHave,
        payload,
    }
}

pub fn parse_piece(index: u32, buf: &mut [u8], msg: Message) -> Result<usize, MessageError> {
    match msg.id {
        MessageId::MsgPiece => {
            if msg.payload.len() < 8 {
                return Err(MessageError::InvalidPayload(format!(
                    "Payload too short. {} < 8",
                    msg.payload.len()
                )));
            }
            if u32::from_be_bytes(msg.payload[0..4].try_into().unwrap()) != index {
                return Err(MessageError::InvalidPayload(format!(
                    "Expected index {}, got {}",
                    index,
                    u32::from_be_bytes(msg.payload[0..4].try_into().unwrap())
                )));
            }
            let start = u32::from_be_bytes(msg.payload[4..8].try_into().unwrap()) as usize;
            if start > (buf.len()) {
                return Err(MessageError::InvalidPayload(format!(
                    "Start offset too high. {} >= {}",
                    start,
                    buf.len()
                )));
            }
            let data = msg.payload[8..].to_vec();
            if start + (data.len()) > (buf.len()) {
                return Err(MessageError::InvalidPayload(format!(
                    "Data too long. {} + {} > {}",
                    start,
                    data.len(),
                    buf.len()
                )));
            }

            buf[start..(start + data.len())].copy_from_slice(data.as_slice());
            Ok(data.len())
        }
        _ => Err(MessageError::InvalidMessageId(format!(
            "Expected PIECE (ID {}), got ID {}",
            MessageId::MsgPiece as u8,
            msg.id as u8
        ))),
    }
}

pub fn parse_have(msg: Message) -> Result<u32, MessageError> {
    match msg.id {
        MessageId::MsgHave => {
            if msg.payload.len() != 4 {
                return Err(MessageError::InvalidPayload(format!(
                    "Expected payload length 4, got length {}",
                    msg.payload.len()
                )));
            }
            let index = u32::from_be_bytes(msg.payload[0..4].try_into().unwrap());
            Ok(index)
        }
        _ => Err(MessageError::InvalidMessageId(format!(
            "Expected HAVE (ID {}), got ID {}",
            MessageId::MsgHave as u8,
            msg.id as u8
        ))),
    }
}

pub fn serialize(msg: Option<Message>) -> Vec<u8> {
    match msg {
        None => Vec::with_capacity(4),
        Some(m) => {
            let length = m.payload.len() + 1;
            let mut buf = Vec::with_capacity(4 + length);
            buf.extend_from_slice(&length.to_be_bytes());
            buf.push(m.id as u8);
            buf.extend_from_slice(&m.payload);
            buf
        }
    }
}

pub fn read(length_buf: &[u8], message_buf: &[u8]) -> Option<Message> {
    let length = u32::from_be_bytes(length_buf.try_into().unwrap());
    match length {
        0 => None,
        _ => {
            let char_code = &message_buf[0..];
            let id = char_code[0];
            let payload = message_buf[1..(length) as usize].into();
            let message_id = match id {
                0 => MessageId::MsgChoke,
                1 => MessageId::MsgUnchoke,
                2 => MessageId::MsgInterested,
                3 => MessageId::MsgNotInterested,
                4 => MessageId::MsgHave,
                5 => MessageId::MsgBitfield,
                6 => MessageId::MsgRequest,
                7 => MessageId::MsgPiece,
                8 => MessageId::MsgCancel,
                _ => {
                    return None;
                }
            };
            Some(Message {
                id: message_id,
                payload,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_request_test() {
        let expected = vec![
            0x00, 0x00, 0x00, 0x04, // Index
            0x00, 0x00, 0x02, 0x37, // Begin
            0x00, 0x00, 0x10, 0xe1, // Length
        ];
        let index = 4;
        let start = 567;
        let length = 4321;
        let msg = format_request(index, start, length);
        assert_eq!(msg.payload, expected);
    }

    #[test]
    fn format_have_test() {
        let expected = vec![
            0x00, 0x00, 0x00, 0x04, // Index
        ];
        let index = 4;
        let msg = format_have(index);
        assert_eq!(msg.payload, expected);
    }

    #[test]
    fn parse_piece_test() {
        let index = 4;
        let buf = &mut [0u8; 10];
        let msg = Message {
            id: MessageId::MsgPiece,
            payload: vec![
                0x00, 0x00, 0x00, 0x04, // Index
                0x00, 0x00, 0x00, 0x02, // Begin
                0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // Length
            ],
        };

        let expected_buf = vec![0x00, 0x00, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x00];

        let expected_result = parse_piece(index, buf, msg);

        assert_eq!(expected_result, Ok(6));
        assert_eq!(buf, expected_buf.as_slice());
    }

    #[test]
    fn parse_have_test() {
        let msg = Message {
            id: MessageId::MsgHave,
            payload: vec![0x00, 0x00, 0x00, 0x04],
        };
        let expected_result = parse_have(msg);
        assert_eq!(expected_result, Ok(4));
    }

    #[test]
    fn serialize_test() {
        let msg = Message {
            id: MessageId::MsgPiece,
            payload: vec![0x00, 0x00, 0x00, 0x04],
        };
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x07, 0x00, 0x00, 0x00, 0x04,
        ];
        let result = serialize(Some(msg));
        assert_eq!(result, expected);
    }

    #[test]
    fn read_test() {
        let length_buf = vec![0x00, 0x00, 0x00, 0x05];
        let message_buf = vec![0x04, 0x00, 0x00, 0x00, 0x04];
        let expected = Message {
            id: MessageId::MsgHave,
            payload: vec![0x00, 0x00, 0x00, 0x04],
        };
        let result = read(&length_buf, &message_buf);
        assert_eq!(result, Some(expected));
    }
}
