use super::{Serialise, DeserialiseError};

pub trait Address: Serialise {
    const BYTE_LENGTH: usize;

    fn bytes(&self) -> &[u8];
}

macro_rules! addr_type {
    (
        $vis:vis
        $addr_type:ident
        ($byte_len:literal $(,$num_type:ty)?)
    ) => {
        #[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
        #[repr(transparent)]
        $vis struct $addr_type {
            bytes: [u8; $byte_len]
        }

        impl $crate::common::address::Address for $addr_type {
            const BYTE_LENGTH: usize = $byte_len;

            fn bytes(&self) -> &[u8] {
                &self.bytes
            }
        }

        impl $crate::common::serialise::Serialise for $addr_type {
            #[inline]
            fn byte_length(&self) -> usize {
                <Self as $crate::common::address::Address>::BYTE_LENGTH
            }

            fn serialise(&self, buf: &mut [u8]) -> usize {
                buf[..self.byte_length()].copy_from_slice(&self.bytes);
                self.byte_length()
            }

            fn deserialise(buf: &[u8]) -> Result<Self, $crate::common::DeserialiseError> {
                if buf.len() < $byte_len {
                    Err($crate::common::DeserialiseError::BufferTooSmall(file!(), line!(), column!(), $byte_len, buf.len()))
                } else {
                    let mut new_addr = [0u8; $byte_len];
                    new_addr.copy_from_slice(&buf[..$byte_len]);

                    Ok(Self {
                        bytes: new_addr
                    })
                }
            }
        }

        impl From<[u8; $byte_len]> for $addr_type {
            fn from(value: [u8; $byte_len]) -> Self {
                Self {
                    bytes: value
                }
            }
        }

        impl From<$addr_type> for [u8; $byte_len] {
            fn from(value: $addr_type) -> Self {
                value.bytes
            }
        }

        $(
            impl From<$num_type> for $addr_type {
                fn from(value: $num_type) -> Self {
                    let bytes = value.to_be_bytes();
                    let start_idx = (bytes.len() - <Self as $crate::common::address::Address>::BYTE_LENGTH);

                    match <Self as $crate::common::serialise::Serialise>::deserialise(&bytes[start_idx..]) {
                        Ok(a) => a,
                        Err(_) => unreachable!()
                    }
                }
            }

            impl From<$addr_type> for $num_type {
                fn from(value: $addr_type) -> Self {
                    const TBYTES: usize = core::mem::size_of::<$num_type>();
                    let mut bytes = [0u8; TBYTES];
                    let start_idx = (TBYTES - <$addr_type as $crate::common::address::Address>::BYTE_LENGTH);

                    bytes[start_idx..].copy_from_slice(&value.bytes);

                    <$num_type>::from_be_bytes(bytes)
                }
            }
        )?
    };
}

mod mac;
pub use mac::MacAddress;

mod ip;
pub use ip::{Ipv4Address, Ipv6Address};