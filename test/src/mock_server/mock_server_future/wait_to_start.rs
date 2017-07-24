use std::fmt::Display;
use std::hash::Hash;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::Poll;
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ServerProto;

use super::state::State;
use super::wait_for_parameters::WaitForParameters;
use super::super::errors::Error;
use super::super::super::mock_service::MockServiceFactory;

pub struct WaitToStart<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    address: SocketAddr,
    service_factory: MockServiceFactory<P::Request, P::Response>,
    protocol: Arc<Mutex<P>>,
    handle: Handle,
}

impl<P> WaitToStart<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + Eq + Hash,
    P::Response: Clone,
{
    pub fn new(
        address: SocketAddr,
        service_factory: MockServiceFactory<P::Request, P::Response>,
        protocol: Arc<Mutex<P>>,
        handle: Handle,
    ) -> Self {
        Self {
            address,
            service_factory,
            protocol,
            handle,
        }
    }

    pub fn advance(self) -> (Poll<(), Error>, State<P>) {
        let bind_result = TcpListener::bind(&self.address, &self.handle);

        match bind_result {
            Ok(listener) => self.create_server_parameters(listener),
            Err(error) => (Err(error.into()), self.same_state()),
        }
    }

    fn create_server_parameters(
        self,
        listener: TcpListener,
    ) -> (Poll<(), Error>, State<P>) {
        let service_factory = self.service_factory;
        let protocol = self.protocol;

        WaitForParameters::advance_with(listener, service_factory, protocol)
    }

    fn same_state(self) -> State<P> {
        State::WaitingToStart(self)
    }
}
