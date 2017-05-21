use data::Packet;
use udpserver::UdpServer;
use eventloop;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

pub fn start(address: &str, threads: u32) {
    let (sender, receiver): (Sender<Packet>, Receiver<Packet>) = mpsc::channel();
    let udpserver = UdpServer::new(address, threads, sender);
    let _ = udpserver.spawn_threads();
    eventloop::run(receiver);
}
