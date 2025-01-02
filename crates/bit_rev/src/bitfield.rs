#[derive(Debug, Clone, PartialEq)]
pub struct Bitfield {
    bytes: Vec<u8>,
}

impl Bitfield {
    pub fn new(bytes: Vec<u8>) -> Bitfield {
        Bitfield { bytes }
    }

    pub fn has_piece(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let offset = index % 8;
        if byte_index >= self.bytes.len() {
            return false;
        }
        (self.bytes[byte_index] >> (7 - offset)) & 1 != 0
    }

    pub fn set_piece(&mut self, index: usize) {
        let byte_index = index / 8;
        let offset = index % 8;
        if byte_index >= self.bytes.len() {
            return;
        }
        let new_char = self.bytes[byte_index] | (1 << (7 - offset));
        self.bytes[byte_index] = new_char;
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.iter().all(|&x| x == 0)
    }
}

#[test]
fn has_piece_test() {
    let bitfield = Bitfield::new(vec![0b01010100, 0b01010100]);
    let outputs = [
        false, true, false, true, false, true, false, false, false, true, false, true, false, true,
        false, false, false, false, false, false,
    ];
    for (index, expected) in outputs.iter().enumerate() {
        assert_eq!(bitfield.has_piece(index), *expected);
    }
}

#[test]
fn set_piece_test() {
    let tests = [
        (
            // Set
            vec![0b01010100, 0b01010100],
            vec![0b01011100, 0b01010100],
            4,
        ),
        (
            // Not Set
            vec![0b01010100, 0b01010100],
            vec![0b01010100, 0b01010100],
            9,
        ),
        (
            // Set
            vec![0b01010100, 0b01010100],
            vec![0b01010100, 0b01010101],
            15,
        ),
        (
            //Not Set
            vec![0b01010100, 0b01010100],
            vec![0b01010100, 0b01010100],
            19,
        ),
    ];

    for (actual, expected, index) in tests.iter() {
        let mut bitfield = Bitfield::new(actual.clone());
        bitfield.set_piece(*index);
        assert_eq!(bitfield.bytes, *expected);
    }
}
