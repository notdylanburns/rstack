use std::io;

use rosi::common::{Ipv4Address, Layer, Serialise};
use rosi::protocols::{ethernet, arp};

fn main() -> io::Result<()> {
    let tap = tun_tap::Iface::new("tap0", tun_tap::Mode::Tap)?;

    loop {
        let mut buf = [0u8; 1522];
        let len = tap.recv(&mut buf)?;

        let frame = match ethernet::Frame::deserialise(&buf[4..len]) {
            Ok(frame) => frame,
            Err(e) => {
                eprint!("ethernet: {e}");
                continue;
            },
        };

        print!("\n{frame}");

        match frame.ethertype() {
            ethernet::EtherType::Arp => {
                let arp_packet = match arp::Packet::deserialise(frame.data()) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("arp: {e}");
                        continue;
                    }
                };

                let resp_packet = arp::Packet::response(
                    arp_packet.sha(),
                    arp_packet.tpa(),
                    arp_packet.sha(),
                    arp_packet.spa()
                ).unwrap();

                let mut resp_frame = ethernet::Frame::new(
                    frame.source(),
                    frame.source(),
                    ethernet::EtherType::Arp,
                    vec![],
                );

                println!("{arp_packet}");
                println!("{resp_packet}");

                resp_frame.wrap(&resp_packet);

                println!("{resp_frame}");

                let mut buf = vec![0u8; resp_frame.byte_length()];
                resp_frame.serialise(&mut buf);

                tap.send(&buf)?;
            },
            et => {
                eprintln!("ignoring frame with ethertype {et}");
                continue;
            }
        }
    }
}