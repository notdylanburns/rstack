pub mod common;
pub mod protocols;

#[macro_use]
pub(crate) mod util {
    macro_rules! getter {
        ($ident:ident: $t:ty) => {
            #[allow(dead_code)]
            pub fn $ident(&self) -> $t {
                self.$ident
            }
        };
        ($ident:ident($($v:ident).+): $t:ty) => {
            #[allow(dead_code)]
            pub fn $ident(&self) -> $t {
                self.$($v).+
            }
        };
    }

    macro_rules! serialise_enum {
        ($v:vis $name:ident($t:ty, $w:literal) { $($hty:ident: $n:literal),*$(,)? }) => {
            #[derive(Eq, PartialEq, Debug, Copy, Clone)]
            $v enum $name {
                Unknown($t),
                $($hty),*
            }

            impl From<$t> for $name {
                fn from(value: $t) -> Self {
                    match value {
                        $($n => Self::$hty,)*
                        hty => Self::Unknown(hty),
                    }
                }
            }

            impl From<[u8; $w]> for $name {
                fn from(value: [u8; $w]) -> Self {
                    Self::from(<$t>::from_be_bytes(value))
                }
            }

            impl From<$name> for $t {
                fn from(value: $name) -> Self {
                    match value {
                        $($name::$hty => $n,)*
                        $name::Unknown(v) => v,
                    }
                }
            }

            impl From<$name> for [u8; $w] {
                fn from(value: $name) -> Self {
                    <$t>::from(value).to_be_bytes()
                }
            }

            impl $crate::common::Serialise for $name {
                #[inline]
                fn byte_length(&self) -> usize {
                    $w
                }

                fn serialise(&self, buf: &mut [u8]) -> usize {
                    let bytes: [u8; $w] = (*self).into();
                    buf[..self.byte_length()].copy_from_slice(&bytes);
                    self.byte_length()
                }

                fn deserialise(buf: &[u8]) -> Result<Self, $crate::common::DeserialiseError> {
                    if buf.len() < $w {
                        Err($crate::common::DeserialiseError::BufferTooSmall(file!(), line!(), column!(), $w, buf.len()))
                    } else {
                        let mut bytes = [0u8; $w];
                        bytes.copy_from_slice(&buf[..$w]);
                        Ok(Self::from(<$t>::from_be_bytes(bytes)))
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

    pub(super) use {getter, serialise_enum};
}