use backends::Backend;
use message::WireMessage;
use errors::CreateBackendError;

use std::net;

pub struct UdpBackend {
    socket: net::UdpSocket,
    destination: net::SocketAddr,
}


impl UdpBackend {
    pub fn new<T: net::ToSocketAddrs>(local: T,
                                      destination: T)
                                      -> Result<UdpBackend, CreateBackendError> {

        let socket = try!(net::UdpSocket::bind(local));
        let destinationAddr = try!(try!(destination.to_socket_addrs())
            .nth(0)
            .ok_or(CreateBackendError("Invalid destination server address")));

        Ok(UdpBackend {
            socket: socket,
            destination: destinationAddr,
        })
    }
}

impl Backend for UdpBackend {
    fn log(&self, msg: WireMessage) {
        self.socket.send_to(b"test", self.destination);
    }
}
