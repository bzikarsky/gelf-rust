use std::net;
use std::io;
use std::io::Read;

use rand;
use libflate::gzip;

use backends::Backend;
use message::WireMessage;
use errors::CreateBackendError;

pub const CHUNK_SIZE_LAN: u16 = 8154;
pub const CHUNK_SIZE_WAN: u16 = 1420;

static MAGIC_BYTES: &'static [u8; 2] = b"\x1e\x0f";

pub struct UdpBackend {
    socket: net::UdpSocket,
    destination: net::SocketAddr,
    chunk_size: u16,
}

impl UdpBackend {
    pub fn new<T: net::ToSocketAddrs>(local: T,
                                      destination: T,
                                      chunk_size: u16)
                                      -> Result<UdpBackend, CreateBackendError> {

        let socket = try!(net::UdpSocket::bind(local));
        let destination_addr = try!(try!(destination.to_socket_addrs())
            .nth(0)
            .ok_or(CreateBackendError("Invalid destination server address")));

        Ok(UdpBackend {
            socket: socket,
            destination: destination_addr,
            chunk_size: chunk_size,
        })
    }
}

impl Backend for UdpBackend {
    fn log(&self, msg: WireMessage) {

        let msg_json = msg.to_json_string().unwrap();

        // encode as gzip
        let mut encoder = gzip::Encoder::new(Vec::new()).unwrap();
        io::copy(&mut io::Cursor::new(msg_json), &mut encoder).unwrap();
        let msg_gzipped = encoder.finish().into_result().unwrap();

        let chunked_msg = ChunkedMessage::new(self.chunk_size, msg_gzipped);

        chunked_msg.into_iter()
            .map(|chunk| self.socket.send_to(&chunk, self.destination))
            .collect::<Vec<Result<usize, io::Error>>>();
    }
}

struct ChunkedMessage {
    chunk_size: u16,
    message: Vec<u8>,
    chunk_num: u8,
    num_chunks: u8,
    message_id: [u8; 8],
}

impl ChunkedMessage {
    fn new(chunk_size: u16, message: Vec<u8>) -> ChunkedMessage {
        let num_chunks: u64 = (message.len() as f64 / chunk_size as f64).ceil() as u64;

        if num_chunks > 128 {
            panic!("Message size exceeds maximum number of chunks")
        }

        ChunkedMessage {
            chunk_size: chunk_size,
            message: message,
            chunk_num: 0,
            message_id: Self::gen_msg_id(),
            num_chunks: num_chunks as u8,
        }
    }

    fn gen_msg_id() -> [u8; 8] {
        let mut raw_id: [u8; 8] = [0; 8];

        for i in 0..8 {
            raw_id[i] = rand::random();
        }

        raw_id
    }
}

impl Iterator for ChunkedMessage {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        if self.chunk_num >= self.num_chunks {
            return None;
        }

        let mut chunk = Vec::new();
        let slice_start = (self.chunk_num as u16 * self.chunk_size) as usize;
        let slice_end = slice_start + self.chunk_size as usize;

        chunk.extend(MAGIC_BYTES.iter().cloned());
        chunk.extend(self.message_id.iter().cloned());
        chunk.push(self.chunk_num);
        chunk.push(self.num_chunks);
        chunk.extend(self.message[slice_start..slice_end].iter().cloned());

        Some(chunk)
    }
}
