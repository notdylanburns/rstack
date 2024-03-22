use super::address::addr_type;

addr_type! {
    pub MacAddress(6, u64)
}

#[allow(dead_code)]
impl MacAddress {
    pub fn from_hex(s: &str) -> Option<Self> {
        let chunks = s.split(':');

        let mut bytes = [0u8; 6];
        for (i, b) in chunks.enumerate() {
            if i >= 6 {
                return None;
            }

            if b.len() != 2 {
                return None;
            }

            let mut byte: u8 = 0;
            for c in b.chars() {
                byte <<= 4;

                byte += match c {
                    '0' => 0,
                    '1' => 1,
                    '2' => 2,
                    '3' => 3,
                    '4' => 4,
                    '5' => 5,
                    '6' => 6,
                    '7' => 7,
                    '8' => 8,
                    '9' => 9,
                    'a'|'A' => 10,
                    'b'|'B' => 11,
                    'c'|'C' => 12,
                    'd'|'D' => 13,
                    'e'|'E' => 14,
                    'f'|'F' => 15,
                    _ => return None,
                }
            }

            bytes[i] = byte;
        };

        Some(Self::from(bytes))
    }
}

impl core::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.bytes[0], self.bytes[1], self.bytes[2],
            self.bytes[3], self.bytes[4], self.bytes[5],
        )
    }
}

#[test]
fn test_macaddress() {
    use super::Serialise;

    let mac = MacAddress::from([0x18, 0x1d, 0xfa, 0xf4, 0xe8, 0x22]);

    let mac_u64 = u64::from(mac);
    let new_mac = MacAddress::from(mac_u64);
    assert_eq!(mac, new_mac);

    let mut buf = [0u8; 6];
    assert_eq!(mac.serialise(&mut buf), 6);
    let new_mac = MacAddress::deserialise(&buf).unwrap();
    assert_eq!(mac, new_mac);

    let new_mac = MacAddress::from(buf);
    assert_eq!(mac, new_mac);

    let mac = MacAddress::from_hex("fe:71:4d:96:e5:95").unwrap();
    assert_eq!(mac.to_string(), "fe:71:4d:96:e5:95");
}