use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvError, Sender, SendError};

use rosi::common::{DeserialiseError, Pdu};

pub type ByteReceiver = Receiver<Arc<[u8]>>;
pub type ByteSender = Sender<Arc<[u8]>>;

// pub type PduFilter<S, T> = &'static dyn Fn(&S, &T) -> bool;
// pub type Action<S, T> = (PduFilter<S, T>, FilterAction);

macro_rules! net_service_error {
    (
        $($ident:ident($t:ty)),*
        $(,)?
    ) => {
        pub enum NetServiceError<T> {
            $($ident($t)),*
        }

        $(
            impl<T> From<$t> for NetServiceError<T> {
                fn from(value: $t) -> Self {
                    Self::$ident(value)
                }
            }
        )*
    };
}

net_service_error! {
    SendError(SendError<T>),
    RecvError(RecvError),
    DeserialiseError(DeserialiseError),
    IoError(std::io::Error),
}


struct Send {
    send_down: ByteSender,
    receive_from_above: ByteReceiver,
}

struct Recv {
    send_up: ByteSender,
    receive_from_below: ByteReceiver,
}

pub enum ActionType {
    Drop,
    Process,
    ForwardTo(ByteSender)
}

impl From<&ActionType> for &str {
    fn from(value: &ActionType) -> Self {
        match value {
            ActionType::Allow => "ALLOW",
            ActionType::Drop => "DROP"
        }
    }
}

pub type PduFilter<S: NetService> = &'static dyn Fn(&S, &S::Pdu) -> bool;
pub type PduProcessor<S: NetService> = &'static dyn FnMut(&mut S, S::Pdu) -> Arc<[u8]>;

struct Action<S: NetService + 'static> {
    action: ActionType,
    filter: &'static dyn Fn(&S, &S::Pdu) -> bool,
    log: bool,
}

impl<S: NetService + 'static> Action<S> {
    pub fn new(action: ActionType, filter: &'static dyn Fn(&S, &S::Pdu) -> bool, log: bool) -> Self {
        Self { action, filter, log }
    }

    pub fn should_process(&self, service: &S, pdu: S::Pdu) -> bool {
        if self.filter(service, &pdu) {
            if self.log {
                pdu.log(self.action.into())
            }

            true
        } else {
            false
        }
    }
}

pub enum FilterAction {
    LogAndBlock,
    Block,
    LogAndUnwrap(ByteSender),
    Unwrap(ByteSender),
    LogAndProcess,
    Process,
}

impl FilterAction {
    pub fn unwrap<L: NetService>(layer: L) -> Self {
        Self::Unwrap(layer.get_send_up())
    }

    pub fn log(self) -> Self {
        match self {
            Self::Block => Self::LogAndBlock,
            Self::Process => Self::LogAndProcess,
            Self::Unwrap(v) => Self::LogAndUnwrap(v),
            a => a,
        }
    }
}

pub trait NetService {
    type Pdu: rosi::common::Pdu;

    fn actions(&self) -> &[(Option<PduFilter<Self>>, Option<PduProcessor<Self>>)];
    fn add_action(&mut self, filter: Option<PduFilter<Self>>, processor: Option<PduProcessor<Self>>, action: ActionType);

    fn add_encapsulated_service(&mut self, send_up: ByteSender, filter: Option<PduFilter<Self>>, processor: Option<PduProcessor<Self>>) {
        self.add_action(
            filter,
            processor,
            move |service: &mut Self, packet: Self::Pdu| send_up.send(processor(service, packet))
        )
    }

    fn get_send_up(&self) -> ByteSender;
    fn set_send_down(&mut self, sender: ByteSender);
    fn get_send_from_above(&self) -> ByteReceiver;

    fn process(&mut self) -> Result<(), NetServiceError<Self::Pdu>> {
        let pdu = self.recv().map_err(|e| e.into())?;
        for (filter, processor) in self.actions().iter() {
            if filter.is_some_and(|f| f(self, &pdu)) {
                processor()
            }
        };

        Ok(())
    }

    fn process_pdu(&mut self, pdu: Self::Pdu) -> Result<(), NetServiceError<Self::Pdu>>;

    fn recv(&self) -> Result<Self::Pdu, NetServiceError<Self::Pdu>>;
}

pub trait Stack<S: NetService>: NetService {
    fn processor(service: &mut Self, packet: Self::Pdu, send: &'static dyn FnOnce(ByteSender) -> Result<(), RecvError>) -> Result<(), NetServiceError<Self::Pdu>>;

    fn stack(&self, service: &mut S, filter: Option<PduFilter<Self>>) {
        service.set_send_down(self.get_send_from_above());

        self.add_encapsulated_service(
            service.get_send_up(),
            filter,
            &Self::processor
        );
    }
}
