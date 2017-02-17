use std::cmp;
use rand;

use errors::{Result, ErrorKind};

/// The overhead per chunk is 12 bytes: magic(2) + id(8) + pos(1) + total (1)
const CHUNK_OVERHEAD: u8 = 12;
const CHUNK_SIZE_LAN: u16 = 8154;
const CHUNK_SIZE_WAN: u16 = 1420;
static MAGIC_BYTES: &'static [u8; 2] = b"\x1e\x0f";

#[derive(Clone, Copy, Debug)]
pub enum ChunkSize {
    LAN,
    WAN,
    Custom(u16),
}

impl ChunkSize {
    pub fn size(&self) -> u16 {
        match *self {
            ChunkSize::LAN => CHUNK_SIZE_LAN,
            ChunkSize::WAN => CHUNK_SIZE_WAN,
            ChunkSize::Custom(size) => size,
        }
    }
}

pub struct ChunkedMessage {
    chunk_size: ChunkSize,
    payload: Vec<u8>,
    num_chunks: u8,
    id: ChunkedMessageId,
}

impl ChunkedMessage {
    pub fn new(chunk_size: ChunkSize, message: Vec<u8>) -> Result<ChunkedMessage> {

        if chunk_size.size() == 0 {
            bail!(ErrorKind::IllegalChunkSize(chunk_size.size()));
        }

        // Ceiled integer diviosn with (a + b - 1) / b
        // Calculate with 64bit integers to avoid overflow - maybe replace with checked_*?
        let size = chunk_size.size() as u64;
        let num_chunks = (message.len() as u64 + size as u64 - 1) / size;

        if num_chunks > 128 {
            bail!(ErrorKind::ChunkMessageFailed("Number of chunks exceeds 128, which the the \
                                                 maximum number of chunks in GELF. Check your \
                                                 chunk_size"))
        }

        Ok(ChunkedMessage {
            chunk_size: chunk_size,
            payload: message,
            id: ChunkedMessageId::random(),
            num_chunks: num_chunks as u8,
        })
    }

    pub fn len(&self) -> u64 {
        if self.num_chunks > 1 {
            self.payload.len() as u64 + self.num_chunks as u64 * CHUNK_OVERHEAD as u64
        } else {
            self.payload.len() as u64
        }
    }

    pub fn iter(&self) -> ChunkedMessageIterator {
        ChunkedMessageIterator::new(self)
    }
}

pub struct ChunkedMessageIterator<'a> {
    chunk_num: u8,
    message: &'a ChunkedMessage,
}

impl<'a> ChunkedMessageIterator<'a> {
    pub fn new(msg: &'a ChunkedMessage) -> ChunkedMessageIterator {
        ChunkedMessageIterator {
            message: msg,
            chunk_num: 0,
        }
    }
}

impl<'a> Iterator for ChunkedMessageIterator<'a> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        if self.chunk_num >= self.message.num_chunks {
            return None;
        }

        let mut chunk = Vec::new();

        // Set the chunks boundaries
        let chunk_size = self.message.chunk_size.size();
        let slice_start = (self.chunk_num as u16 * chunk_size) as usize;
        let slice_end = cmp::min(slice_start + chunk_size as usize,
                                 self.message.payload.len());

        // The chunk header is only required when the message size exceeds one chunk
        if self.message.num_chunks > 1 {
            chunk.extend(MAGIC_BYTES.iter());
            chunk.extend(self.message.id.as_bytes());
            chunk.push(self.chunk_num);
            chunk.push(self.message.num_chunks);
        }

        chunk.extend(self.message.payload[slice_start..slice_end].iter());

        self.chunk_num += 1;

        Some(chunk)
    }
}

struct ChunkedMessageId([u8; 8]);


impl<'a> ChunkedMessageId {
    fn random() -> ChunkedMessageId {
        let mut bytes = [0; 8];

        for b in 0..8 {
            bytes[b] = rand::random();
        }

        return ChunkedMessageId::from_bytes(bytes);
    }

    #[allow(dead_code)]
    fn from_int(mut id: u64) -> ChunkedMessageId {
        let mut bytes = [0; 8];
        for i in 0..8 {
            bytes[7 - i] = (id & 0xff) as u8;
            id >>= 8;
        }

        ChunkedMessageId(bytes)
    }

    fn from_bytes(bytes: [u8; 8]) -> ChunkedMessageId {
        ChunkedMessageId(bytes)
    }

    fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }

    #[allow(dead_code)]
    fn to_int(&self) -> u64 {
        self.0.iter().fold(0_u64, |id, &i| id << 8 | i as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunked_message_id_from_and_to_bytes() {
        let raw_ids = vec![b"\xff\xff\xff\xff\xff\xff\xff\xff",
                           b"\x00\x00\x00\x00\x00\x00\x00\x00",
                           b"\xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa",
                           b"\x55\x55\x55\x55\x55\x55\x55\x55",
                           b"\x00\x01\x02\x03\x04\x05\x06\x07",
                           b"\x07\x06\x05\x04\x03\x02\x01\x00",
                           b"\x00\x10\x20\x30\x40\x50\x60\x70",
                           b"\x70\x60\x50\x40\x30\x20\x10\x00"];

        for raw_id in raw_ids {
            let id = ChunkedMessageId::from_bytes(raw_id.clone());
            assert_eq!(id.as_bytes(), raw_id);
        }
    }

    #[test]
    fn chunked_message_id_from_and_to_int() {
        let raw_ids = vec![0xffffffffffffffff,
                           0x0000000000000000,
                           0xaaaaaaaaaaaaaaaa,
                           0x5555555555555555,
                           0x0001020304050607,
                           0x0706050403020100,
                           0x0010203040506070,
                           0x7060504030201000];

        for raw_id in raw_ids {
            let id = ChunkedMessageId::from_int(raw_id);
            assert_eq!(id.to_int(), raw_id);
        }
    }

    #[test]
    #[should_panic(expected = "Number of chunks")]
    fn fail_too_many_chunks() {
        ChunkedMessage::new(ChunkSize::Custom(1), get_data(129)).unwrap();
    }

    #[test]
    fn chunk_message_len() {
        let msg_1_chunk = ChunkedMessage::new(ChunkSize::Custom(1), get_data(1)).unwrap();
        let msg_2_chunks = ChunkedMessage::new(ChunkSize::Custom(1), get_data(2)).unwrap();
        let msg_128_chunks = ChunkedMessage::new(ChunkSize::Custom(1), get_data(128)).unwrap();

        assert_eq!(msg_1_chunk.len(), 1);
        assert_eq!(msg_2_chunks.len() as u32, 2 + 2 * CHUNK_OVERHEAD as u32);
        assert_eq!(msg_128_chunks.len() as u64,
                   128 + 128 * (CHUNK_OVERHEAD as u64));
    }

    #[test]
    fn chunk_message_id_random() {
        let msg1 = ChunkedMessage::new(ChunkSize::Custom(1), get_data(1)).unwrap();
        let msg2 = ChunkedMessage::new(ChunkSize::Custom(1), get_data(1)).unwrap();
        let msg3 = ChunkedMessage::new(ChunkSize::Custom(1), get_data(1)).unwrap();

        assert!(msg1.id.to_int() != msg2.id.to_int());
        assert!(msg3.id.to_int() != msg2.id.to_int());
        assert!(msg1.id.to_int() != msg3.id.to_int());
    }

    #[test]
    fn chunk_message_correct_math() {
        let msg1 = ChunkedMessage::new(ChunkSize::Custom(3), get_data(1)).unwrap();
        let msg2 = ChunkedMessage::new(ChunkSize::Custom(3), get_data(2)).unwrap();
        let msg3 = ChunkedMessage::new(ChunkSize::Custom(3), get_data(3)).unwrap();
        let msg4 = ChunkedMessage::new(ChunkSize::Custom(3), get_data(4)).unwrap();
        let msg5 = ChunkedMessage::new(ChunkSize::Custom(3), get_data(5)).unwrap();
        let msg6 = ChunkedMessage::new(ChunkSize::Custom(3), get_data(6)).unwrap();
        let msg7 = ChunkedMessage::new(ChunkSize::Custom(3), get_data(7)).unwrap();

        assert_eq!(msg1.num_chunks, 1);
        assert_eq!(msg2.num_chunks, 1);
        assert_eq!(msg3.num_chunks, 1);
        assert_eq!(msg4.num_chunks, 2);
        assert_eq!(msg5.num_chunks, 2);
        assert_eq!(msg6.num_chunks, 2);
        assert_eq!(msg7.num_chunks, 3);
    }

    #[test]
    fn chunk_message_chunking() {
        // 10 exact chunks
        check_chunks(10, 100, 10);

        // 4 inexact chunks
        check_chunks(33, 100, 4);

        // test no chunks
        let msg = ChunkedMessage::new(ChunkSize::Custom(100), get_data(100)).unwrap();
        let mut iter = msg.iter();
        let chunk = iter.next().unwrap();
        assert_eq!(iter.next(), None);
        assert_eq!(chunk.len(), 100);
        assert_eq!(chunk[0], 0);
        assert_eq!(chunk[99], 99);
    }

    #[test]
    #[should_panic]
    fn test_illegal_chunk_size() {
        ChunkedMessage::new(ChunkSize::Custom(0), get_data(1)).unwrap();
    }

    fn get_data(len: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(len);
        for i in 0..len {
            data.push(i as u8);
        }

        data
    }

    fn check_chunks(chunk_size: u8, msg_size: u8, expected_chunk_count: u8) {
        let msg = ChunkedMessage::new(ChunkSize::Custom(chunk_size as u16),
                                      get_data(msg_size as usize))
            .unwrap();
        let mut counter: u8 = 0;
        for chunk in msg.iter() {
            println!("{:?}", chunk);

            // length is in budget

            assert!(chunk.len() as u8 <= chunk_size + 12);
            // magic bytes
            assert_eq!(chunk[0], MAGIC_BYTES[0]);
            assert_eq!(chunk[1], MAGIC_BYTES[1]);

            // pos/counter section
            assert_eq!(chunk[10], counter);
            assert_eq!(chunk[11], expected_chunk_count);

            // first and last byte
            assert_eq!(chunk[12], counter * chunk_size);
            assert_eq!(*chunk.last().unwrap(),
                       ::std::cmp::min((counter + 1) * chunk_size, 100) - 1);

            counter += 1;
        }

        assert_eq!(counter, expected_chunk_count);
    }

}
