use crate::common::{DeserialiseError, Serialise};

macro_rules! ethertype {
    ($($proto:ident: $et:literal),*$(,)?) => {
        #[derive(Eq, PartialEq, Debug, Copy, Clone)]
        pub enum EtherType {
            PayloadLength(u16),
            Unknown(u16),
            $($proto),*
        }

        impl From<u16> for EtherType {
            fn from(value: u16) -> Self {
                match value {
                    ..=1500 => Self::PayloadLength(value),
                    $($et => Self::$proto,)*
                    _ => Self::Unknown(value),
                }
            }
        }

        impl From<[u8; 2]> for EtherType {
            fn from(value: [u8; 2]) -> Self {
                Self::from(u16::from_be_bytes(value))
            }
        }

        impl From<&EtherType> for u16 {
            fn from(value: &EtherType) -> Self {
                match value {
                    EtherType::PayloadLength(v) => *v,
                    $(EtherType::$proto => $et,)*
                    EtherType::Unknown(v) => *v,
                }
            }
        }

        impl From<&EtherType> for [u8; 2] {
            fn from(value: &EtherType) -> Self {
                u16::from(value).to_be_bytes()
            }
        }

        impl core::fmt::Display for EtherType {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Self::PayloadLength(v) => write!(f, "PayloadLength({v})"),
                    $(Self::$proto => write!(f, stringify!($proto)),)*
                    Self::Unknown(v) => write!(f, "Unknown({v:04x})"),
                }
            }
        }
    };
}

ethertype! {
    Ipv4:               0x0800,
    Arp:                0x0806,
    WakeOnLan:          0x0842,
    VlanTaggedFrame:    0x8100,
    Ipv6:               0x86dd,
    ServiceVlanTag:     0x88a8,
}

impl Serialise for EtherType {
    #[inline]
    fn byte_length(&self) -> usize {
        2
    }

    fn serialise(&self, buf: &mut [u8]) -> usize {
        let bytes: [u8; 2] = self.into();
        buf[..self.byte_length()].copy_from_slice(&bytes);
        self.byte_length()
    }

    fn deserialise(buf: &[u8]) -> Result<Self, DeserialiseError> {
        if buf.len() < 2 {
            Err(DeserialiseError::BufferTooSmall(file!(), line!(), column!(), 2, buf.len()))
        } else {
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&buf[..2]);
            Ok(Self::from(u16::from_be_bytes(bytes)))
        }
    }
}

#[test]
fn test_display() {
    let et = EtherType::from(100);
    assert_eq!(et, EtherType::PayloadLength(100));

    println!("{et}");

    let et = EtherType::from(1501);
    assert_eq!(et, EtherType::Unknown(1501));

    println!("{et}");

    println!("{} {} {}", EtherType::Arp, EtherType::Ipv4, EtherType::Ipv6)
}