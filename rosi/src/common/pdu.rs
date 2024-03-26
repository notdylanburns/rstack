use std::sync::Arc;

use super::Serialise;

pub trait Pdu: Serialise {
    fn log(&self, action: &str);
}

impl Pdu for Arc<[u8]> {
    fn log(&self, _: &str) {
        unimplemented!()
    }
}

pub trait Wrapper {
    fn unwrap_data(&self) -> Arc<[u8]>;
}