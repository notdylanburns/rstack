use rosi::common::Serialise;
use rosi::protocols::ethernet;

use crate::netservice::ByteSender;

use super::netservice::{
    Action, ByteReceiver,
    FilterAction,
    NetService
};

pub struct EthernetService {
    actions: Vec<Action<Self, ethernet::Frame>>,
    send_to_receiver: ByteSender,
    receiver: ByteReceiver,
    sender: ByteSender,
}

impl NetService for EthernetService {
    type Pdu = ethernet::Frame;

    fn actions(&self) -> &[Action<Self, Self::Pdu>] {
        &self.actions
    }

    fn add_action(&mut self, filter: crate::netservice::PduFilter<Self, Self::Pdu>, action: FilterAction) {
        self.actions.push((filter, action))
    }

    fn recv(&self) -> Result<Self::Pdu, rosi::common::DeserialiseError> {
        Self::Pdu::deserialise(&self.receiver.recv().unwrap())
    }

    fn process_pdu(&mut self, pdu: Self::Pdu) -> Result<(), ()> {
        unreachable!()
    }

    fn get_send_up(&self) -> crate::netservice::ByteSender {
        self.send_to_receiver.clone()
    }

    fn set_send_down(&mut self, sender: crate::netservice::ByteSender) {
        self.sender = sender
    }
}