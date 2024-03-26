use std::sync::Arc;

#[derive(Debug)]
pub enum DeserialiseError {
    Static(&'static str),
    Heap(String),
    BufferTooSmall(&'static str, u32, u32, usize, usize),
}

impl From<&'static str> for DeserialiseError {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

impl From<String> for DeserialiseError {
    fn from(value: String) -> Self {
        Self::Heap(value)
    }
}

impl core::fmt::Display for DeserialiseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to deserialise: ")?;

        match self {
            Self::Static(s) => write!(f, "{s}."),
            Self::Heap(s) => write!(f, "{s}."),
            Self::BufferTooSmall(file, l, c, required, bufsize) => write!(f, "{file}:{l}:{c} buffer too small (expected {required}, actual {bufsize}).")
        }
    }
}

pub trait Serialise {
    fn byte_length(&self) -> usize;
    fn serialise(&self, buf: &mut [u8]) -> usize;
    fn deserialise(buf: &[u8]) -> Result<Self, DeserialiseError>
    where Self: Sized;
}

macro_rules! serialise_impl {
    ($t:ty) => {
        impl Serialise for $t {
            fn byte_length(&self) -> usize {
                (Self::BITS / 8) as usize
            }

            fn serialise(&self, buf: &mut [u8]) -> usize {
                buf[..self.byte_length()].copy_from_slice(&self.to_be_bytes());
                self.byte_length()
            }

            fn deserialise(buf: &[u8]) -> Result<Self, DeserialiseError> {
                let mut bytes = [0; (Self::BITS / 8) as usize];
                bytes.copy_from_slice(&buf[..(Self::BITS / 8) as usize]);
                Ok(Self::from_be_bytes(bytes))
            }
        }
    };
}

serialise_impl!(u8);
serialise_impl!(u16);
serialise_impl!(u32);
serialise_impl!(u64);
serialise_impl!(u128);

impl Serialise for &[u8] {
    #[inline]
    fn byte_length(&self) -> usize {
        self.len()
    }

    fn serialise(&self, buf: &mut [u8]) -> usize {
        let len = self.byte_length();
        buf[..len].copy_from_slice(self);
        len
    }

    fn deserialise(_: &[u8]) -> Result<Self, DeserialiseError> {
        unimplemented!()
    }
}

impl Serialise for Arc<[u8]> {
    #[inline]
    fn byte_length(&self) -> usize {
        self.len()
    }

    fn serialise(&self, buf: &mut [u8]) -> usize {
        let len = self.byte_length();
        buf[..len].copy_from_slice(self);
        len
    }

    fn deserialise(_: &[u8]) -> Result<Self, DeserialiseError> {
        unimplemented!()
    }
}

macro_rules! serialise_field {
    ($field:expr, $index:expr, $buf:ident) => {
        ($index) + $field.serialise(&mut $buf[($index)..])
    };
}

macro_rules! serialise_fields {
    ($(start=$index:expr,)? buf=$buf:ident, $($field:expr),+ $(,)?) => {
        {
            #[allow(unused_variables)]
            let index = 0;
            $(let index = $index;)?

            $(
                let index = $crate::common::serialise_field!($field, index, $buf);
            )+

            index
        }
    };
}

pub(crate) use {serialise_field, serialise_fields};