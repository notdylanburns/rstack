use crate::common::Serialise;

pub trait Layer: Serialise {
    fn wrap(&mut self, data: &dyn Serialise);
}