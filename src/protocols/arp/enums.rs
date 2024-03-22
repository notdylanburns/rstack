use crate::common::{
    Address,
    Serialise, DeserialiseError
};
use crate::common::address::{
    MacAddress,
    Ipv4Address, Ipv6Address
};
use crate::protocols::ethernet::EtherType;

macro_rules! u16_enum {
    ($v:vis $name:ident { $($hty:ident: $n:literal),*$(,)? }) => {
        #[derive(Eq, PartialEq, Debug, Copy, Clone)]
        $v enum $name {
            Unknown(u16),
            $($hty),*
        }

        impl From<u16> for $name {
            fn from(value: u16) -> Self {
                match value {
                    $($n => Self::$hty,)*
                    hty => Self::Unknown(hty),
                }
            }
        }

        impl From<[u8; 2]> for $name {
            fn from(value: [u8; 2]) -> Self {
                Self::from(u16::from_be_bytes(value))
            }
        }

        impl From<$name> for u16 {
            fn from(value: $name) -> Self {
                match value {
                    $($name::$hty => $n,)*
                    $name::Unknown(v) => v,
                }
            }
        }

        impl From<$name> for [u8; 2] {
            fn from(value: $name) -> Self {
                u16::from(value).to_be_bytes()
            }
        }

        impl Serialise for $name {
            #[inline]
            fn byte_length(&self) -> usize {
                2
            }
        
            fn serialise(&self, buf: &mut [u8]) -> usize {
                let bytes: [u8; 2] = (*self).into();
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

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Unknown(v) => write!(f, "{v}"),
                    _ => write!(
                        f,
                        "{}",
                        match self {
                            $(Self::$hty => stringify!($hty),)*
                            _ => unreachable!(),
                        }
                    ),
                }
            }
        }
    };
}

u16_enum! {
    pub Htype {
        Ethernet: 1,
        IEEE802: 6,
        Arcnet: 7,
        FrameRelay: 15,
        Atm16: 16,
        Hdlc: 17,
        FibreChannel: 18,
        Atm19: 19,
        SerialLine: 20
    }
}

u16_enum! {
    pub Operation {
        Request:  1,
        Response: 2,
    }
}

macro_rules! address_type {
    (
        $vis:vis $enum_name:ident($dep:ty) {
            $($e:ident::$n:ident => $addr_type:ident),*
            $(,)?
        }
    ) => {
        #[derive(Debug, Copy, Clone)]
        $vis enum $enum_name {
            $($addr_type($addr_type)),*
        }

        impl $enum_name { 
            pub(super) fn from_bytes(addr_type: $dep, bytes: &[u8]) -> Result<Self, DeserialiseError> {
                match addr_type {
                    $($e::$n => $addr_type::deserialise(bytes).and_then(|r| Ok(Self::$addr_type(r))),)*
                    _ => todo!(),
                }
            }

            #[allow(dead_code)]
            pub(super) fn addr_type(&self) -> $dep {
                match self {
                    $(Self::$addr_type(..) => $e::$n),*
                }
            }
        }

        // impl From<&$enum_name> for &[u8] {
        //     fn from(value: &$enum_name) -> &[u8] {
        //         match value {
        //             $($enum_name::$addr_type(v) => v.bytes()),*
        //         }
        //     }
        // }

        $(
            impl From<$addr_type> for $enum_name {
                fn from(value: $addr_type) -> Self {
                    Self::$addr_type(value)
                }
            }
        )+

        impl Serialise for $enum_name {
            fn byte_length(&self) -> usize {
                match self {
                    $(Self::$addr_type(..) => $addr_type::BYTE_LENGTH,)*
                }
            }

            fn serialise(&self, bytes: &mut [u8]) -> usize {
                let len = self.byte_length();
                bytes[..len].copy_from_slice(
                    match self {
                        $($enum_name::$addr_type(v) => v.bytes()),*
                    }
                );
                len
            }

            fn deserialise(_: &[u8]) -> Result<Self, DeserialiseError> {
                unimplemented!()
            }
        }

        impl core::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$addr_type(v) => write!(f, "{v}")),*
                }
            }
        }
    };
}

address_type! {
    pub HardwareAddress(Htype) {
        Htype::Ethernet => MacAddress,
    }
}

address_type! {
    pub ProtocolAddress(EtherType) {
        EtherType::Ipv4 => Ipv4Address,
        EtherType::Ipv6 => Ipv6Address,
    }
}