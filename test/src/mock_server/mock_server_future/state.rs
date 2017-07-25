use std::fmt::Display;
use std::hash::Hash;
use std::mem;
use std::sync::{Arc, Mutex};

use futures::Poll;
use tokio_core::net::{TcpListener, TcpStream};
use tokio_proto::pipeline::ServerProto;

use super::server_ready::ServerReady;
use super::wait_for_parameters::WaitForParameters;
use super::super::errors::Error;
use super::super::super::mock_service::MockServiceFactory;

pub enum State<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + Eq,
    P::Response: Clone,
{
    WaitingForParameters(WaitForParameters<P>),
    ServerReady(ServerReady<P>),
    Processing,
}

impl<P> State<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + Eq + Hash,
    P::Response: Clone,
{
    pub fn start_with(
        listener: TcpListener,
        service_factory: MockServiceFactory<P::Request, P::Response>,
        protocol: Arc<Mutex<P>>,
    ) -> Self {
        let wait_for_parameters =
            WaitForParameters::new(listener, service_factory, protocol);

        State::WaitingForParameters(wait_for_parameters)
    }

    pub fn advance(&mut self) -> Poll<(), Error> {
        let state = mem::replace(self, State::Processing);

        let (poll_result, new_state) = state.advance_to_new_state();

        mem::replace(self, new_state);

        poll_result
    }

    fn advance_to_new_state(self) -> (Poll<(), Error>, Self) {
        match self {
            State::WaitingForParameters(handler) => handler.advance(),
            State::ServerReady(handler) => handler.advance(),
            State::Processing => panic!("State has more than one owner"),
        }
    }
}
