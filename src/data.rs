use sha2::{Sha256, Digest};
use std::io::prelude::*;
use time::PreciseTime;
use byteorder::{BigEndian, ReadBytesExt};

const EOF: u16 = 0x8000;

pub struct Packet {
    pub packet_id: u32,
    flags: u16,
    data_size: u16,
    offset: u32,
    data: Vec<u8>
}

impl Packet {
    pub fn new<R: BufRead + Seek> (reader: &mut R) -> Packet {
        let flags = reader.read_u16::<BigEndian>().unwrap();
        let data_size = reader.read_u16::<BigEndian>().unwrap();
        let offset = reader.read_u32::<BigEndian>().unwrap();
        let packet_id = reader.read_u32::<BigEndian>().unwrap();
        let mut data = Vec::new();
        let _ = reader.take(data_size as u64).read_to_end(&mut data);

        Packet { flags, data_size, offset, packet_id, data }
    }
}

pub struct Message {
    pub complete: bool,
    pub start: PreciseTime,
    pub packets: Vec<Packet>,
    eof: u32,
    seen: u32
}

impl Message {
    pub fn new() -> Message {
        Message { 
            complete: false,
            start: PreciseTime::now(), 
            packets: vec![], 
            seen: 0, 
            eof: 0, 
        }
    }

    pub fn add_packet(&mut self, packet: Packet) {
        self.seen += packet.data_size as u32;

        if packet.flags == EOF {
            self.eof = packet.offset + packet.data_size as u32;
        }

        self.packets.push(packet);

        if self.eof == self.seen {
            self.complete = true;
        }
    }

    pub fn sha256(&self) -> String {
        let packets: Vec<u8> = self.packets.iter().flat_map(|p| p.data.clone()).collect();
        let data: &[u8] = &packets;
        let mut hasher = Sha256::default();
        hasher.input(&data);
        format!("{:x}", hasher.result())
    }

    pub fn gaps(&self) -> Vec<(u32, u32)> {
        let mut gaps = vec![];
        let mut previous = 0;
        let mut offset = 0;

        for packet in &self.packets {
            if packet.offset != offset {
                gaps.push((previous, offset));
            }

            previous = packet.offset;
            offset = packet.offset + packet.data_size as u32;;
        }

        return gaps;
    }

    pub fn packets_packet_id(&self) -> Option<u32> {
        self.packets.first().and_then(|p| Some(p.packet_id))
    }

    pub fn sort(&mut self) {
        self.packets.sort_by(|a, b| a.offset.cmp(&b.offset));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use byteorder::{BigEndian, WriteBytesExt};

    #[test]
    fn read_buffer() {
        let mut data = Cursor::new(create_packet_data(0, 10, 0, 1));
        let packet = Packet::new(&mut data);
        assert_eq!(10, packet.data_size);
        assert_eq!(1, packet.packet_id);
    }

    #[test]
    fn add_packet() {
        let mut data = Cursor::new(create_packet_data(0, 10, 0, 1));
        let packet = Packet::new(&mut data);
        let mut message = Message::new();
        message.add_packet(packet);
        assert_eq!(1, message.packets.len());
        assert_eq!(10, message.seen);
    }

    #[test]
    fn gaps() {
        let mut message = Message::new();

        for i in 0..10 {
            if i == 4  || i == 8 { continue };

            let mut data = Cursor::new(create_packet_data(0, 1, i, 1));
            let packet = Packet::new(&mut data);
            message.add_packet(packet);
        }

        let gaps = message.gaps();
        assert_eq!(gaps, &[(3,4), (7,8)]);
    }

    #[test]
    fn eof_flag() {
        let mut data = Cursor::new(create_packet_data(EOF, 10, 2000, 1));
        let packet = Packet::new(&mut data);
        let mut message = Message::new();
        message.add_packet(packet);
        assert_eq!(2010, message.eof);
    }

    #[test]
    fn sha256() {
        let mut message = Message::new();
        let mut offset = 0;

        for _ in 0..3 {
            let mut data = Cursor::new(create_packet_data(0, 10, offset, 1));
            let packet = Packet::new(&mut data);
            message.add_packet(packet);
            offset += 10;
        }

        let mut hasher = Sha256::default();
        let data = [2; 10 * 3];
        hasher.input(&data);
        let result = format!("{:x}", hasher.result());

        assert_eq!(message.sha256(), result);
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
}
