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

use super::active_mock_server::ActiveMockServer;
use super::bound_connection_future::BoundConnectionFuture;
use super::errors::{Error, NormalizeError, Result};
use super::super::mock_service::MockService;
use super::super::mock_service::MockServiceFactory;

type ServerParametersFuture<P: ServerProto<TcpStream>> = Join<
    BoundConnectionFuture<P>,
    FutureResult<MockService<P::Request, P::Response>, Error>,
>;

pub struct MockServerFuture<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    address: SocketAddr,
    service_factory: MockServiceFactory<P::Request, P::Response>,
    protocol: Arc<Mutex<P>>,
    handle: Handle,
    server_parameters: Option<Result<ServerParametersFuture<P>>>,
    server: Option<ActiveMockServer<P::Transport>>,
}

impl<P> MockServerFuture<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    pub fn new(
        address: SocketAddr,
        service_factory: MockServiceFactory<P::Request, P::Response>,
        protocol: Arc<Mutex<P>>,
        handle: Handle,
    ) -> MockServerFuture<P> {
        Self {
            address,
            service_factory,
            protocol,
            handle,
            server_parameters: None,
            server: None,
        }
    }

    fn start_listening(&mut self) -> Poll<(), Error> {
        let bind_result = TcpListener::bind(&self.address, &self.handle);

        self.server_parameters = match bind_result {
            Ok(listener) => Some(Ok(self.create_server_parameters(listener))),
            Err(error) => Some(Err(error.into())),
        };

        self.poll_server_parameters()
    }

    fn create_server_parameters(
        &mut self,
        listener: TcpListener,
    ) -> ServerParametersFuture<P> {
        let service = self.service_factory.new_service();
        let protocol = self.protocol.clone();
        let connection = BoundConnectionFuture::from(listener, protocol);

        connection.join(service.normalize_error())
    }

    fn poll_server_parameters(&mut self) -> Poll<(), Error> {
        let parameters = mem::replace(&mut self.server_parameters, None);

        let (poll_result, maybe_parameters) = match parameters {
            Some(Ok(mut parameters)) => {
                (parameters.poll(), Some(Ok(parameters)))
            }
            Some(Err(error)) => (Err(error), None),
            None => {
                panic!(
                    "Attempt to poll server parameters future before it is \
                     created"
                )
            }
        };

        mem::replace(&mut self.server_parameters, maybe_parameters);

        match poll_result {
            Ok(Async::Ready(parameters)) => self.start_server(parameters),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(error) => Err(error),
        }
    }

    fn start_server(
        &mut self,
        parameters: (P::Transport, MockService<P::Request, P::Response>),
    ) -> Poll<(), Error> {
        self.server = Some(ActiveMockServer::from_tuple(parameters));

        self.poll_server()
    }

    fn poll_server(&mut self) -> Poll<(), Error> {
        match self.server {
            Some(ref mut server) => server.poll(),
            None => {
                panic!("Attempt to poll server future before it is created")
            }
        }
    }

    fn state(&self) -> State {
        if self.server.is_some() {
            State::ServerReady
        } else if self.server_parameters.is_some() {
            State::WaitingForParameters
        } else {
            State::NoParameters
        }
    }
}

impl<P> Future for MockServerFuture<P>
where
    P: ServerProto<TcpStream>,
    P::Request: Clone + Display + PartialEq,
    P::Response: Clone,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.state() {
            State::NoParameters => self.start_listening(),
            State::WaitingForParameters => self.poll_server_parameters(),
            State::ServerReady => self.poll_server(),
        }
    }
}

enum State {
    NoParameters,
    WaitingForParameters,
    ServerReady,
}
