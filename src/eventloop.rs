use std::{thread, time};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use time::PreciseTime;
use time::Duration;
use data::{Message, Packet};

pub fn run(receiver: Receiver<Packet>) {
    let messages = Arc::new(Mutex::new(HashMap::new()));
    let cloned_messages = messages.clone();

    thread::spawn(move || {
        loop {
            let packet = receiver.recv().unwrap();
            let mut thread_messages = cloned_messages.lock().unwrap();
            let message = thread_messages.entry(packet.packet_id).or_insert(Message::new());

            message.add_packet(packet);

            if message.complete {
                message.sort();
                println!("{} - {}", message.packets_packet_id().unwrap_or(0), message.sha256());
            }
        }
    });

    loop {
        thread::sleep(time::Duration::from_secs(10));

        let messages = messages.lock().unwrap();

        if messages.iter().count() == 0 {
            continue;
        }

        let outdated = messages.values().filter(|m| m.start.to(PreciseTime::now()) > Duration::seconds(30)).count();

        if outdated == messages.iter().count() {
            break;
        }
    }

    let mut messages = messages.lock().unwrap();

    for (_, message) in messages.iter_mut() {
        message.sort();
    }

    let incomplete: Vec<_> = messages.iter()
        .filter(|&(_, v)| v.complete == false)
        .map(|(_, v)| v)
        .collect();

    if incomplete.iter().count() == 0 {
        println!("All messages complete")
    } else {
        for message in incomplete.iter() {
            if message.start.to(PreciseTime::now()) > Duration::minutes(30) {
                let gaps = message.gaps();
                for (previous, gap) in gaps {
                    println!("{} - Data Gap: {}-{}", message.packets_packet_id().unwrap_or(0), previous, gap);
                }
            }
        }
    }
}
