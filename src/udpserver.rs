use std::net::{UdpSocket};
use std::sync::mpsc::Sender;
use std::io::Cursor;
use std::thread;
use std::thread::JoinHandle;
use data::{Packet};

pub struct UdpServer {
    socket: UdpSocket,
    sender: Sender<Packet>,
    threads: u32
}

impl UdpServer {
    pub fn new(address: &str, threads: u32, sender: Sender<Packet>) -> UdpServer {
        let socket = Self::bind_server(address);

        UdpServer {
            socket: socket,
            sender: sender,
            threads: threads
        }
    }

    fn bind_server(addr: &str) -> UdpSocket {
        match UdpSocket::bind(&addr) {
            Ok(s) => {
                println!("Server listening: {}", addr);
                return s
            },
            Err(e) => panic!("Bind socket failed: {}", e)
        };
    }

    pub fn spawn_threads(&self) -> Vec<JoinHandle<()>> {
        (0..self.threads).map(|_| {
            let thread_socket = self.socket.try_clone().unwrap();
            let thread_sender = self.sender.clone();

            let mut buf = [0; 512];

            thread::spawn(move || {
                loop {
                    match thread_socket.recv_from(&mut buf) {
                        Ok((_, _)) => {
                            let data: &[u8] = &buf;
                            let packet = Packet::new(&mut Cursor::new(data));
                            thread_sender.send(packet).unwrap();
                        },
                        Err(e) => {
                            println!("Error receiving: {}", e);
                        }
                    }
                }
            })
        }).collect()
    }
}
