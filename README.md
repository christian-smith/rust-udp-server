# Rust UDP server (custom UDP reassembler)
## Description
I wrote this code to handle a case where custom udp packets needed to be reassembled and checked against a sha256 hash.  The server runs on multiple threads

- The server spawns a UDP listening socket along with 4 processing threads.
- Each processing thread processes the incoming packet and then send the resulting Packet over a channel to the receiver.
- A run loop receives each packet coming in over the channel and adds the Packet to the corresponding Message by associated packet_id.
- When the message is complete the sha256 hash is printed.

## Run
```
$ cargo run
```

## Testing
```
$ cargo test
```
