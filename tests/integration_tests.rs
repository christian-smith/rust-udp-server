extern crate udp;
extern crate byteorder;

use byteorder::{BigEndian, WriteBytesExt};
use std::io::prelude::*;
use std::io::Cursor;
use std::net::UdpSocket; 
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use udp::udpserver::UdpServer;
use udp::data::{Message, Packet};

#[test]
fn udpserver_processes_packets() {
    let (sender, receiver): (Sender<Packet>, Receiver<Packet>) = mpsc::channel();

    let server_address: &str = "127.0.0.1:8888";
    let udpserver = UdpServer::new(server_address, 1, sender);
    udpserver.spawn_threads();

    let client_address: &str = "127.0.0.1:6788";
    let socket = UdpSocket::bind(client_address).unwrap();

    let packet1 = create_packet_data(0, 10, 0, 1);
    let packet2 = create_packet_data(0, 10, 10, 1);
    let packet3 = create_packet_data(0x8000, 10, 20, 1);
    let packets = vec![packet1, packet2, packet3];

    for packet in packets.iter() {
        let buf: &[u8] = &packet;
        let _ = socket.send_to(buf, server_address);
    }

    let mut message = Message::new();

    for _ in 0..packets.len() {
        let packet = receiver.recv().unwrap();
        message.add_packet(packet);
    }

    assert_eq!(true, message.complete);
}

fn create_packet_data(flags: u16, data_size: u16, offset: u32, packet_id: u32) -> Vec<u8> {
        let buf = vec![];
        let mut writer = Cursor::new(buf);
        writer.write_u16::<BigEndian>(flags).unwrap(); //flags
        writer.write_u16::<BigEndian>(data_size).unwrap(); //data_size
        writer.write_u32::<BigEndian>(offset).unwrap(); //offset
        writer.write_u32::<BigEndian>(packet_id).unwrap(); //trans_Id
        let sized = data_size as usize;
        let data = vec![2; sized];
        let data: &[u8] = &data;
        writer.write(data).unwrap();
        writer.into_inner()
}
