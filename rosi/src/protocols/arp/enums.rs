use crate::util::serialise_enum;

use crate::common::{
    Address,
    Serialise, DeserialiseError
};
use crate::common::address::{
    MacAddress,
    Ipv4Address, Ipv6Address
};
use crate::protocols::ethernet::EtherType;

serialise_enum! {
    pub Htype(u16, 2) {
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

serialise_enum! {
    pub Operation(u16, 2) {
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