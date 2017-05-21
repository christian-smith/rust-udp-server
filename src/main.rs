extern crate udp;

use udp::server;

const ADDRESS: &str = "127.0.0.1:8888";
const THREAD_COUNT: u32 = 4;

fn main() {
    server::start(ADDRESS, THREAD_COUNT);
}
