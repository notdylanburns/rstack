use std::sync::Arc;

use crate::netservice::ByteReceiver;

use super::netservice::{ActionType, NetService, NetServiceError, ByteSender, Stack, PduFilter, PduProcessor};
use super::ethernet::EthernetService;

struct Tap {
    tap: tun_tap::Iface,
    actions: Vec<[(Option<PduFilter<Self>>, Option<PduProcessor<Self>>)]>,
    send_from_above: ByteSender
}

impl Tap {
    pub fn new(name: &str) -> std::io::Result<Self> {
        Ok(
            Self {
                tap: tun_tap::Iface::new(name, tun_tap::Mode::Tap)?,
                send_up: None,
            }
        )
    }

    pub fn wrap<S: NetService>(&mut self) {
        self.
    }
}

impl NetService for Tap {
    type Pdu = Arc<[u8]>;

    fn actions(&self) -> &[(Option<PduFilter<Self>>, ActionType, Option<PduProcessor<Self>>)] {
        self.actions
    }

    fn add_action(&mut self, filter: Option<PduFilter<Self>>, processor: Option<PduProcessor<Self>>, action: ActionType) {
        self.actions.push((filter, action, processor))
    }

    fn get_send_from_above(&self) -> crate::netservice::ByteReceiver {
        self.
    }
}
