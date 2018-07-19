//! This module just contains some dirty code to make the examples work
//! Just ignore it :-)

#![allow(dead_code)]

use std::env::Args;
use std::io::Read;

pub fn run_debug_server_udp(host: String, timeout_in_s: u64) {
    let socket = ::std::net::UdpSocket::bind(host.as_str())
        .expect("Failed to create debug server UDP socket");

    socket
        .set_read_timeout(Some(::std::time::Duration::new(timeout_in_s, 0)))
        .expect("Failed to set read timeout on UDP socket");

    loop {
        let mut buf: [u8; 10000] = [0; 10000 /* should fit almost any message */];
        let num_bytes = match socket.recv_from(&mut buf) {
            Ok((num_bytes, _)) => num_bytes,
            Err(_) => return,
        };

        if buf[0] == 0x1e && buf[1] == 0x0f {
            let pos = buf[10];
            let total = buf[11];
            let id: u64 = buf[2..10].iter().fold(0, |x, &i| x << 8 | i as u64);

            println!(
                "Received message chunk ({}/{}) for message '{:?}': {}",
                pos,
                total,
                id,
                String::from_utf8_lossy(&buf[12..num_bytes])
            );
        } else {
            println!(
                "Received message: {}",
                String::from_utf8_lossy(&buf[0..num_bytes])
            );
        }
    }
}

pub fn run_debug_server_tcp(host: String, num_messages: u8) {
    let socket = ::std::net::TcpListener::bind(host.as_str())
        .expect("Failed to create debug server TCP socket");

    let (mut conn, _) = socket.accept().expect("Failed to accept connection");

    let mut buf: [u8; 20] = [0; 20];
    let mut msg_counter = 0;
    let mut keep: Vec<u8> = Vec::new();

    loop {
        let num_bytes = conn.read(&mut buf).expect("Failed to read on connection");
        let mut counter = 0;
        let mut last_msg = 0;
        for byte in buf[0..num_bytes].iter() {
            counter += 1;
            if *byte == 0x00 {
                println!(
                    "Received message: {}{}",
                    String::from_utf8_lossy(keep.as_slice()),
                    String::from_utf8_lossy(&buf[last_msg..(last_msg + counter)])
                );
                keep.clear();
                last_msg = counter;
                counter = 0;
                msg_counter += 1;
            }
        }

        if last_msg != num_bytes {
            keep.extend(buf[last_msg..num_bytes].iter());
        }

        if msg_counter == num_messages {
            break;
        }
    }
}

pub struct Options {
    pub gelf_host: String,
    pub run_debug_server: bool,
}

impl Options {
    pub fn populate(&mut self, args: Args) {
        let args = args.collect::<Vec<String>>();
        for i in 0..args.len() {
            if args[i] == "--no-server" {
                self.run_debug_server = false;
                continue;
            }

            if args[i] == "--gelf-host" {
                self.gelf_host = args[i + 1].clone();
                continue;
            }
        }
    }
}
