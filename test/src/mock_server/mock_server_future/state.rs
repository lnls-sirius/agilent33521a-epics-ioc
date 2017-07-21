use std::fmt::Display;
use std::mem;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::{Async, Future, Poll};
use futures::future::{FutureResult, Join};
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ServerProto;
use tokio_service::NewService;

use super::wait_to_start::WaitToStart;
use super::super::active_mock_server::ActiveMockServer;
use super::super::bound_connection_future::BoundConnectionFuture;
use super::super::errors::{Error, NormalizeError};
use super::super::super::mock_service::MockService;
use super::super::super::mock_service::MockServiceFactory;

pub enum State<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    WaitingToStart(WaitToStart<P>),
    WaitingForParameters(WaitForParameters<P>),
    ServerReady(ServerReady<P>),
    Processing,
}

impl<P> State<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    pub fn start_with(
        address: SocketAddr,
        service_factory: MockServiceFactory<P::Request, P::Response>,
        protocol: Arc<Mutex<P>>,
        handle: Handle,
    ) -> Self {
        let wait_to_start =
            WaitToStart::new(address, service_factory, protocol, handle);

        State::WaitingToStart(wait_to_start)
    }

    pub fn advance(&mut self) -> Poll<(), Error> {
        let state = mem::replace(self, State::Processing);

        let (poll_result, new_state) = state.advance_to_new_state();

        mem::replace(self, new_state);

        poll_result
    }

    fn advance_to_new_state(self) -> (Poll<(), Error>, Self) {
        match self {
            State::WaitingToStart(handler) => handler.advance(),
            State::WaitingForParameters(handler) => handler.advance(),
            State::ServerReady(handler) => handler.advance(),
            State::Processing => panic!("State has more than one owner"),
        }
    }
}

pub struct WaitForParameters<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    parameters: Join<
        BoundConnectionFuture<P>,
        FutureResult<MockService<P::Request, P::Response>, Error>,
    >,
}

impl<P> WaitForParameters<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    pub fn advance_with(
        listener: TcpListener,
        service_factory: MockServiceFactory<P::Request, P::Response>,
        protocol: Arc<Mutex<P>>,
    ) -> (Poll<(), Error>, State<P>) {
        let service = service_factory.new_service();
        let connection = BoundConnectionFuture::from(listener, protocol);
        let parameters = connection.join(service.normalize_error());

        let wait_for_parameters = WaitForParameters { parameters };

        wait_for_parameters.advance()
    }

    fn advance(mut self) -> (Poll<(), Error>, State<P>) {
        match self.parameters.poll() {
            Ok(Async::Ready(parameters)) => self.create_server(parameters),
            Ok(Async::NotReady) => (Ok(Async::NotReady), self.same_state()),
            Err(error) => (Err(error), self.same_state()),
        }
    }

    fn create_server(
        self,
        parameters_tuple: (P::Transport, MockService<P::Request, P::Response>),
    ) -> (Poll<(), Error>, State<P>) {
        ServerReady::advance_with(parameters_tuple)
    }

    fn same_state(self) -> State<P> {
        State::WaitingForParameters(self)
    }
}

pub struct ServerReady<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    server: ActiveMockServer<P::Transport>,
}

impl<P> ServerReady<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    fn advance_with(
        parameters_tuple: (P::Transport, MockService<P::Request, P::Response>),
    ) -> (Poll<(), Error>, State<P>) {
        let server_ready = Self {
            server: ActiveMockServer::from_tuple(parameters_tuple),
        };

        server_ready.advance()
    }

    fn advance(mut self) -> (Poll<(), Error>, State<P>) {
        (self.server.poll(), State::ServerReady(self))
    }
}
