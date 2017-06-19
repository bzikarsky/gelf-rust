use std::cmp;

use rand;

use errors::{Result, ErrorKind};

/// Overhead per chunk is 12 bytes: magic(2) + id(8) + pos(1) + total (1)
const CHUNK_OVERHEAD: u8 = 12;

/// Chunk-size for LANs
const CHUNK_SIZE_LAN: u16 = 8154;

/// Chunk-size for WANs
const CHUNK_SIZE_WAN: u16 = 1420;

/// Magic bytes identifying a GELF message chunk
static MAGIC_BYTES: &'static [u8; 2] = b"\x1e\x0f";

/// ChunkSize is a value type representing the size of a message-chunk
///
/// It provides default sizes for WANs and LANs
#[derive(Clone, Copy, Debug)]
pub enum ChunkSize {
    LAN,
    WAN,
    Custom(u16),
}

impl ChunkSize {
    /// Return the size associated with the chunk-size
    pub fn size(&self) -> u16 {
        match *self {
            ChunkSize::LAN => CHUNK_SIZE_LAN,
            ChunkSize::WAN => CHUNK_SIZE_WAN,
            ChunkSize::Custom(size) => size,
        }
    }
}

/// ChunkedMessage is an internal type for chunking an already serialized `WireMessage`
pub struct ChunkedMessage {
    chunk_size: ChunkSize,
    payload: Vec<u8>,
    num_chunks: u8,
    id: ChunkedMessageId,
}

impl ChunkedMessage {
    /// Construct a new ChunkedMessage
    ///
    /// Several sanity checks are performed on construction:
    /// - chunk_size must be greater than 0
    /// - GELF allows for a maximum of 128 chunks per message
    pub fn new(chunk_size: ChunkSize, message: Vec<u8>) -> Result<ChunkedMessage> {

        if chunk_size.size() == 0 {
            bail!(ErrorKind::IllegalChunkSize(chunk_size.size()));
        }

        // Ceiled integer division with (a + b - 1) / b
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

    /// Return the byte-length of the chunked message inclduing all overhead
    pub fn len(&self) -> u64 {
        if self.num_chunks > 1 {
            self.payload.len() as u64 + self.num_chunks as u64 * CHUNK_OVERHEAD as u64
        } else {
            self.payload.len() as u64
        }
    }

    /// Return an iterator over all chunks of the message
    pub fn iter(&self) -> ChunkedMessageIterator {
        ChunkedMessageIterator::new(self)
    }
}

/// An iterator over all a chunked message's chunks
///
/// This always be constructed by `ChunkedMessage`
pub struct ChunkedMessageIterator<'a> {
    chunk_num: u8,
    message: &'a ChunkedMessage,
}

impl<'a> ChunkedMessageIterator<'a> {
    /// Create a new ChunkedMessageIterator
    fn new(msg: &'a ChunkedMessage) -> ChunkedMessageIterator {
        ChunkedMessageIterator {
            message: msg,
            chunk_num: 0,
        }
    }
}

impl<'a> Iterator for ChunkedMessageIterator<'a> {
    type Item = Vec<u8>;

    /// Returns the next chunk (if existant)
    fn next(&mut self) -> Option<Vec<u8>> {
        if self.chunk_num >= self.message.num_chunks {
            return None;
        }

        let mut chunk = Vec::new();

        // Set the chunks boundaries
        let chunk_size = self.message.chunk_size.size();
        let slice_start = (self.chunk_num as u32 * chunk_size as u32) as usize;
        let slice_end = cmp::min(slice_start + chunk_size as usize,
                                 self.message.payload.len());

        // The chunk header is only required when the message size exceeds one chunk
        if self.message.num_chunks > 1 {
            // Chunk binary layout:
            //  2 bytes (magic bytes)
            //  8 bytes (message id)
            //  1 byte  (chunk number)
            //  1 byte  (total amount of chunks in this message)
            //  n bytes (chunk payload)
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

/// The representation of a chunked message id
///
/// Every chunked message requires an ID which consists of 8 bytes. This is the same
/// as an 64bit integer. This struct provides some convenience functions on this type.
struct ChunkedMessageId([u8; 8]);

#[allow(dead_code)]
impl<'a> ChunkedMessageId {
    /// Create a new, random ChunkedMessageId.
    fn random() -> ChunkedMessageId {
        let mut bytes = [0; 8];

        for b in 0..8 {
            bytes[b] = rand::random();
        }

        return ChunkedMessageId::from_bytes(bytes);
    }

    /// Create a new ChunkedMessageId from a 64 int.
    fn from_int(mut id: u64) -> ChunkedMessageId {
        let mut bytes = [0; 8];
        for i in 0..8 {
            bytes[7 - i] = (id & 0xff) as u8;
            id >>= 8;
        }

        ChunkedMessageId(bytes)
    }

    /// Create a new ChunkedMessageId from a byte-array.
    fn from_bytes(bytes: [u8; 8]) -> ChunkedMessageId {
        ChunkedMessageId(bytes)
    }

    /// Return the message id as a byte-slice.
    fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }

    /// Return the message id as an 64bit uint.
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
    fn chunk_large_message_chunking() {
        // 100k of msg
        chunking(CHUNK_SIZE_WAN, 100000);
    }

    fn chunking(chunk_size: u16, msg_size: u32) {
        check_chunks(chunk_size as u16, msg_size, (msg_size / chunk_size as u32) as u8 + 1);
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

    fn check_chunks(chunk_size: u16, msg_size: u32, expected_chunk_count: u8) {
        let msg_data = get_data(msg_size as usize);
        let msg_data_clone = msg_data.clone();
        let msg = ChunkedMessage::new(ChunkSize::Custom(chunk_size as u16),
                                      msg_data)
            .unwrap();
        let mut counter: u8 = 0;
        for chunk in msg.iter() {
            println!("{:?}", chunk);

            // length is in budget

            assert!(chunk.len() as u16 <= chunk_size + 12);
            // magic bytes
            assert_eq!(chunk[0], MAGIC_BYTES[0]);
            assert_eq!(chunk[1], MAGIC_BYTES[1]);

            // pos/counter section
            assert_eq!(chunk[10], counter);
            assert_eq!(chunk[11], expected_chunk_count);

            // first and last byte
            let first_index = (counter as u32 * chunk_size as u32) as usize;
            let last_index = (::std::cmp::min((counter as u32 + 1) * chunk_size as u32, msg_size) - 1) as usize;
            assert_eq!(chunk[12], msg_data_clone[first_index]);
            assert_eq!(*chunk.last().unwrap(),
                       msg_data_clone[last_index]);

            counter += 1;
        }

        assert_eq!(counter, expected_chunk_count);
    }

}
