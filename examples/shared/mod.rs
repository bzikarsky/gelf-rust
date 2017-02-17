pub fn run_debug_server_udp(host: &str, timeout_in_s: u64) {
    let socket = ::std::net::UdpSocket::bind(host)
        .expect("Failed to create debug server UDP socket");

    socket.set_read_timeout(Some(::std::time::Duration::new(timeout_in_s, 0)))
        .expect("Failed to set read timeout on UDP socket");

    loop {
        let mut buf: [u8; 10000] = [0; 10000 /* should fit alomost any message */];
        let num_bytes = match socket.recv_from(&mut buf) {
            Ok((num_bytes, _)) => num_bytes,
            Err(_) => return,
        };

        if buf[0] == 0x1e && buf[1] == 0x0f {
            let pos = buf[10];
            let total = buf[11];
            let id: u64 = buf[2..10].iter().fold(0, |x, &i| x << 4 | i as u64);

            println!("Received message chunk ({}/{}) for message '{:?}': {}",
                     pos,
                     total,
                     id,
                     String::from_utf8_lossy(&buf[12..num_bytes]));
        } else {
            println!("Received message: {}",
                     String::from_utf8_lossy(&buf[0..num_bytes]));
        }
    }
}
