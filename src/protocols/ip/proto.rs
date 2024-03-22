use crate::util::serialise_enum;

serialise_enum! {
    pub IpProtocol(u8, 1) {
        Icmp:       0x01,
        Tcp:        0x06,
        Udp:        0x11,
        Ipv6Icmp:   0x3a,
        Ipv6NoNxt:  0x3b,
        Ipv6Opts:   0x3c,
    }
}